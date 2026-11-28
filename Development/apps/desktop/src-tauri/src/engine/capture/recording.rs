//! Per-slot on-demand recording; no game-process lifecycle owner.
//! One supervisor, one bounded progress reader and one FFmpeg child per active slot.
use super::{recording_target::Target,verified_target,video_encoder as video,worker};
use crate::engine::process_identity::OwnedProcess;
use serde::Serialize;
use std::{
    io::{self,Read,Seek,SeekFrom,Write},
    os::windows::{io::{AsRawHandle,FromRawHandle,OwnedHandle},process::CommandExt},
    path::PathBuf,process::{Child,Command,Stdio},
    sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,AtomicU64,AtomicU8,Ordering}},
    thread,time::{Duration,Instant},
};
use windows_sys::Win32::System::JobObjects::*;

const STARTING:u8=0;
const RECORDING:u8=1;
const STOPPING:u8=2;
const SAVED:u8=3;
const FAILED:u8=4;
const MAX_ACTIVE_PIXEL_RATE:u64=4096*2160*30; // conservative initial admission, not measured GPU throughput
const POLL:Duration=Duration::from_millis(200);
const FRAME_TIMEOUT:Duration=Duration::from_secs(15);
const STOP_TIMEOUT:Duration=Duration::from_secs(10);
const READER_TIMEOUT:Duration=Duration::from_secs(2);

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct RecordingReceipt{pub slot:u32,pub state:&'static str,pub frames:u64,pub recovery_file:Option<String>}
struct Run{state:AtomicU8,stop:AtomicBool,terminal_progress:AtomicBool,frames:AtomicU64,pixel_rate:u64,recovery_file:OnceLock<String>}
impl Run{
    fn active(&self)->bool{self.state.load(Ordering::Acquire)<SAVED}
    fn receipt(&self,slot:u32)->RecordingReceipt{
        let observed=self.state.load(Ordering::Acquire);
        let state=match observed{
            STARTING=>"Starting",RECORDING=>"Recording",STOPPING=>"Stopping",SAVED=>"Saved",_=>"Failed"};
        // Progress writes frames before promoting STARTING. A concurrent
        // status read can observe the old state and new count; keep that
        // Starting receipt valid rather than reporting conflicting proof.
        let frames=if observed==STARTING{0}else{self.frames.load(Ordering::Acquire)};
        let recovery_file=if observed==FAILED{self.recovery_file.get().cloned()}else{None};
        RecordingReceipt{slot,state,frames,recovery_file}
    }
}
static SLOTS:Mutex<[Option<Arc<Run>>;5]>=Mutex::new([const{None};5]);
fn slots()->io::Result<std::sync::MutexGuard<'static,[Option<Arc<Run>>;5]>>{
    SLOTS.lock().map_err(|_|io::Error::other("Recording state unavailable"))
}
fn index(slot:u32)->io::Result<usize>{
    if !(1..=5).contains(&slot){return Err(io::Error::other("Unknown Minecraft profile"));}
    Ok((slot-1) as usize)
}
fn admit(slots:&[Option<Arc<Run>>;5],idx:usize,rate:u64)->io::Result<()>{
    if slots[idx].as_ref().is_some_and(|r|r.active()){
        return Err(io::Error::other("Minecraft instance already has an active recording"));
    }
    let used=slots.iter().flatten().filter(|r|r.active()).map(|r|r.pixel_rate).sum::<u64>();
    if rate==0||used.saturating_add(rate)>MAX_ACTIVE_PIXEL_RATE{
        return Err(io::Error::other("Recording GPU budget reached"));
    }
    Ok(())
}
pub(super) fn status()->io::Result<Vec<RecordingReceipt>>{
    Ok(slots()?.iter().enumerate().filter_map(|(i,r)|r.as_ref().map(|r|r.receipt(i as u32+1))).collect())
}
pub(super) fn require_idle(slot:Option<u32>)->io::Result<()>{
    let all=slots()?;
    let active=match slot{Some(s)=>all[index(s)?].as_ref().is_some_and(|r|r.active()),
        None=>all.iter().flatten().any(|r|r.active())};
    if active{return Err(io::Error::other("Stop the active recording before closing Minecraft"));}
    Ok(())
}
pub(super) fn stop(slot:u32)->io::Result<()>{
    if let Some(r)=slots()?[index(slot)?].as_ref(){
        if r.active(){r.stop.store(true,Ordering::Release);}
    }
    Ok(())
}
pub(super) fn start(slot:u32,expected:OwnedProcess)->io::Result<bool>{
    let idx=index(slot)?;
    if !video::available(){return Err(io::Error::new(io::ErrorKind::Unsupported,"Recording backend approval pending"));}
    let target=Target::pin(verified_target(slot,&expected)?)?;
    let rate=target.size.pixels_per_second();
    admit(&*slots()?,idx,rate)?;
    let Some(path)=rfd::FileDialog::new().set_title("Save Minecraft recording")
        .add_filter("Matroska video",&["mkv"]).set_file_name(format!("Minecraft-{slot:02}.mkv")).save_file()
        else{return Ok(false)};
    if !path.extension().is_some_and(|x|x.eq_ignore_ascii_case("mkv"))||path.exists(){
        return Err(io::Error::other("Choose a new .mkv file; existing files are not replaced"));
    }
    verified_target(slot,&expected)?;
    target.verify()?;
    let image=worker::reviewed_image("ffmpeg.exe",video::REVIEWED_FFMPEG_SHA256)?;
    let parent=path.parent().ok_or_else(||io::Error::other("Recording save folder unavailable"))?;
    let temp=tempfile::Builder::new().prefix(".lazy-tester-video-").suffix(".part").tempfile_in(parent)?;
    let run=Arc::new(Run{state:AtomicU8::new(STARTING),stop:AtomicBool::new(false),terminal_progress:AtomicBool::new(false),frames:AtomicU64::new(0),pixel_rate:rate,recovery_file:OnceLock::new()});
    let mut all=slots()?;
    admit(&all,idx,rate)?;
    let active=Arc::clone(&run);
    // Hold only the short admission mutex through thread creation; the
    // recording never owns the manager lock during its active lifetime.
    thread::Builder::new().name(format!("recording-{slot:02}")).spawn(move||{
        let success=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
            record(target,image,temp,path,&active))).is_ok_and(|r|r.is_ok());
        active.state.store(if success{SAVED}else{FAILED},Ordering::Release);
    })?;
    all[idx]=Some(run);
    Ok(true)
}

// A kill-on-close job owns only the recorder child, never Minecraft.
// Assign immediately after spawn; the candidate must be reviewed as a
// self-contained encoder that launches no descendants.
struct Recorder{child:Child,_job:OwnedHandle}
impl Recorder{
    fn spawn(image:&worker::Image,target:&Target,temp:&tempfile::NamedTempFile)->io::Result<Self>{
        let job_raw=unsafe{CreateJobObjectW(std::ptr::null(),std::ptr::null())};
        if job_raw.is_null(){return Err(io::Error::last_os_error());}
        let job=unsafe{OwnedHandle::from_raw_handle(job_raw as _)};
        let mut limits:JOBOBJECT_EXTENDED_LIMIT_INFORMATION=unsafe{std::mem::zeroed()};
        limits.BasicLimitInformation.LimitFlags=JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe{SetInformationJobObject(job_raw,JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            std::mem::size_of_val(&limits) as u32)}==0{return Err(io::Error::last_os_error());}
        target.verify()?;
        let child=Command::new(&image.path).args(video::args(target.hwnd(),target.size)?)
            .creation_flags(0x08000000).stdin(Stdio::piped())
            .stdout(Stdio::from(temp.as_file().try_clone()?)).stderr(Stdio::piped()).spawn()?;
        let mut rec=Self{child,_job:job};
        if unsafe{AssignProcessToJobObject(job_raw,rec.child.as_raw_handle() as _)}==0{
            let error=io::Error::last_os_error();
            if rec.child.kill().is_ok(){let _=rec.child.wait();}
            return Err(error);
        }
        Ok(rec)
    }
}
impl Drop for Recorder{
    fn drop(&mut self){
        if !matches!(self.child.try_wait(),Ok(Some(_))){
            if self.child.kill().is_ok(){let _=self.child.wait();}
        }
    }
}

fn record(target:Target,image:worker::Image,mut temp:tempfile::NamedTempFile,path:PathBuf,run:&Arc<Run>)->io::Result<()>{
    if run.stop.load(Ordering::Acquire){return Err(io::Error::other("Recording cancelled before start"));}
    let mut rec=Recorder::spawn(&image,&target,&temp)?;
    let stderr=rec.child.stderr.take().ok_or_else(||io::Error::other("Recording progress unavailable"))?;
    let reader_run=Arc::clone(run);
    let (tx,rx)=std::sync::mpsc::sync_channel(1);
    // Progress is numeric metadata only. No logs or private media are copied.
    // One bounded pipe reader; the child/job close releases its writer.
    let reader=thread::Builder::new().name("recording-progress".into()).spawn(move||{
        let _=tx.send(read_progress(stderr,&reader_run));
    })?;
    let started=Instant::now();
    let mut stopping=None;
    let mut progress_complete=false;
    let mut last_frame=0;
    let mut last_advance=started;
    // Close the child/job before waiting for EOF, including on ordinary errors.
    let supervised=(|| -> io::Result<()> {
    loop{
        target.verify()?; // fail closed; no replacement window/slot
        if !progress_complete{
            match rx.try_recv(){
                Ok(Ok(true))=>progress_complete=true,
                Ok(Ok(false))=>return Err(io::Error::other("Recording progress ended without a final receipt")),
                Ok(Err(error))=>return Err(error),
                Err(std::sync::mpsc::TryRecvError::Disconnected)=>
                    return Err(io::Error::other("Recording progress reader disconnected")),
                Err(std::sync::mpsc::TryRecvError::Empty)=>{},
            }
        }
        if let Some(status)=rec.child.try_wait()?{
            if !status.success(){return Err(io::Error::other("Hardware recorder failed"));}
            break;
        }
        // A final progress report is not proof that the child exited.
        // Bound the remaining trailer/exit wait even after a natural end.
        if stopping.is_none()&&run.terminal_progress.load(Ordering::Acquire){
            run.state.store(STOPPING,Ordering::Release);
            stopping=Some(Instant::now());
        }
        if stopping.is_none()&&(run.stop.load(Ordering::Acquire)||
            started.elapsed()>=Duration::from_secs(video::MAX_SECONDS)){
            run.state.store(STOPPING,Ordering::Release);
            stopping=Some(Instant::now());
            let mut stdin=rec.child.stdin.take().ok_or_else(||io::Error::other("Recording stop unavailable"))?;
            // Natural exit may close stdin just before the user stops.
            // BrokenPipe is not success; still require exit and trailer proof.
            if let Err(error)=stdin.write_all(b"q\n"){
                if error.kind()!=io::ErrorKind::BrokenPipe{return Err(error);}
            }
        }
        if stopping.is_none()&&frame_stalled(
            &mut last_frame,&mut last_advance,run.frames.load(Ordering::Acquire),Instant::now()){
            return Err(io::Error::new(io::ErrorKind::TimedOut,"Recording frame progress stalled"));
        }
        if stopping.is_some_and(|t:Instant|t.elapsed()>=STOP_TIMEOUT){
            return Err(io::Error::new(io::ErrorKind::TimedOut,"Recording finalization timed out"));
        }
        if temp.as_file().metadata()?.len()>video::MAX_BYTES+1024*1024{
            return Err(io::Error::other("Recording exceeded storage limit"));
        }
        thread::sleep(POLL);
    }
    Ok(())
    })();
    drop(rec); // close job/writer before bounded reader completion
    // A received result (or disconnect) means the reader is returning.
    // Never join a potentially blocked pipe reader after a timeout.
    let completion=if progress_complete {Ok(Ok(true))} else {rx.recv_timeout(READER_TIMEOUT)};
    if !matches!(&completion,Err(std::sync::mpsc::RecvTimeoutError::Timeout)) {
        reader.join().map_err(|_|io::Error::other("Recording progress reader failed"))?;
    }
    supervised?;
    let ended=completion.map_err(|_|io::Error::other("Recording progress completion unavailable"))??;
    if !ended||run.frames.load(Ordering::Acquire)==0{return Err(io::Error::other("Recording has no final frame receipt"));}
    target.verify()?;
    let file=temp.as_file_mut();
    let len=file.metadata()?.len();
    if !(64..=video::MAX_BYTES+1024*1024).contains(&len){return Err(io::Error::other("Recording output size invalid"));}
    file.seek(SeekFrom::Start(0))?;
    let mut header=[0u8;4];file.read_exact(&mut header)?;
    if header!=[0x1a,0x45,0xdf,0xa3]{return Err(io::Error::other("Recording container invalid"));}
    file.sync_all()?;
    publish_finalized(temp,&path,run)
}

// This boundary is reached only after target, exit, final progress,
// container and sync checks. If publishing fails, do not destroy the
// already finalized recording or overwrite someone else's destination.
fn publish_finalized(temp:tempfile::NamedTempFile,path:&std::path::Path,run:&Run)->io::Result<()>{
    match temp.persist_noclobber(path){
        Ok(_)=>Ok(()),
        Err(error)=>{
            let failure=error.error;
            match error.file.keep(){
                Ok((_file,retained))=>{
                    if let Some(name)=retained.file_name().and_then(|name|name.to_str()){
                        let _=run.recovery_file.set(name.to_owned());
                    }
                }
                Err(mut keep_error)=>{
                    // The filesystem may be unavailable. Preserve the
                    // existing temporary entry if possible, but do not
                    // claim recovery without a successful keep receipt.
                    keep_error.file.disable_cleanup(true);
                }
            }
            Err(failure)
        }
    }
}

// One deadline for first-frame and mid-stream stalls, no additional timer.
fn frame_stalled(last:&mut u64,advanced:&mut Instant,current:u64,now:Instant)->bool{
    if current>*last{*last=current;*advanced=now;}
    now.saturating_duration_since(*advanced)>=FRAME_TIMEOUT
}

fn read_progress(mut input:impl Read,run:&Run)->io::Result<bool>{
    let mut buf=[0u8;4096];
    let mut line=[0u8;256];
    let mut length=0;
    let mut total=0usize;
    let mut ended=false;
    let mut report_frame=None;
    loop{
        let n=input.read(&mut buf)?;
        if n==0{break;}
        total+=n;
        if total>8*1024*1024{return Err(io::Error::other("Recording progress budget exceeded"));}
        for &b in &buf[..n]{
            if ended{return Err(io::Error::other("Unexpected data after final recording progress"));}
            if b==b'\n'{
                let text=std::str::from_utf8(&line[..length]).map_err(|_|io::Error::other("Invalid recording progress"))?.trim();
                if let Some(value)=text.strip_prefix("frame="){
                    let frames=value.trim().parse::<u64>().map_err(|_|io::Error::other("Invalid frame count"))?;
                    if report_frame.replace(frames).is_some(){
                        return Err(io::Error::other("Duplicate frame count in recording progress"));
                    }
                }else if let Some(value)=text.strip_prefix("progress="){
                    if !matches!(value,"continue"|"end"){
                        return Err(io::Error::other("Invalid recording progress marker"));
                    }
                    let frames=report_frame.take().ok_or_else(||io::Error::other("Recording report has no frame count"))?;
                    if frames<run.frames.load(Ordering::Acquire){
                        return Err(io::Error::other("Recording frame count regressed"));
                    }
                    // Publish only a complete progress report, not an isolated
                    // frame line from a truncated or malformed stream.
                    run.frames.store(frames,Ordering::Release);
                    if frames>0{let _=run.state.compare_exchange(STARTING,RECORDING,Ordering::AcqRel,Ordering::Acquire);}
                    ended=value=="end";
                    if ended{run.terminal_progress.store(true,Ordering::Release);}
                }
                length=0;
            }else{
                if length==line.len(){return Err(io::Error::other("Recording progress line too long"));}
                line[length]=b;length+=1;
            }
        }
    }
    Ok(ended&&length==0)
}

#[cfg(test)]
mod tests{
    use super::*;
    fn run(rate:u64)->Arc<Run>{Arc::new(Run{state:AtomicU8::new(STARTING),stop:AtomicBool::new(false),terminal_progress:AtomicBool::new(false),frames:AtomicU64::new(0),pixel_rate:rate,recovery_file:OnceLock::new()})}
    #[test]
    fn admission_prevents_duplicate_and_aggregate_overcommit(){
        let mut slots:[Option<Arc<Run>>;5]=[const{None};5];
        slots[0]=Some(run(MAX_ACTIVE_PIXEL_RATE-100));
        assert!(admit(&slots,0,1).is_err());
        assert!(admit(&slots,1,101).is_err());
        assert!(admit(&slots,1,100).is_ok());
        slots[0].as_ref().unwrap().state.store(SAVED,Ordering::Release);
        assert!(admit(&slots,0,MAX_ACTIVE_PIXEL_RATE).is_ok());
    }
    #[test]
    fn starting_receipt_does_not_mix_an_unpromoted_frame_with_starting(){
        let r=run(1);
        r.frames.store(1,Ordering::Release);
        let receipt=r.receipt(1);
        assert_eq!(receipt.state,"Starting");
        assert_eq!(receipt.frames,0);
        r.state.store(RECORDING,Ordering::Release);
        let receipt=r.receipt(1);
        assert_eq!(receipt.state,"Recording");
        assert_eq!(receipt.frames,1);
    }
    #[test]
    fn frame_watchdog_detects_first_and_later_stalls(){
        let start=Instant::now();
        let mut advanced=start;
        let mut last=0;
        assert!(!frame_stalled(&mut last,&mut advanced,0,start+Duration::from_secs(14)));
        assert!(frame_stalled(&mut last,&mut advanced,0,start+FRAME_TIMEOUT));
        assert!(!frame_stalled(&mut last,&mut advanced,10,start+FRAME_TIMEOUT));
        assert!(!frame_stalled(&mut last,&mut advanced,10,start+FRAME_TIMEOUT+Duration::from_secs(14)));
        assert!(frame_stalled(&mut last,&mut advanced,10,start+FRAME_TIMEOUT*2));
    }
    #[test]
    fn malformed_or_truncated_reports_cannot_claim_more_frames(){
        let r=run(1);
        assert!(!read_progress(&b"frame=5\n"[..],&r).unwrap());
        assert_eq!(r.frames.load(Ordering::Acquire),0);
        for invalid in [&b"progress=end\n"[..],&b"frame=1\nframe=2\nprogress=end\n"[..],
            &b"frame=1\nprogress=unknown\n"[..]]{
            assert!(read_progress(invalid,&r).is_err());
        }
        assert!(read_progress(&b"frame=5\nprogress=end\nframe=6\n"[..],&r).is_err());
        assert!(read_progress(&b"frame=4\nprogress=end\n"[..],&r).is_err());
    }
    #[test]
    fn finalized_publish_conflict_preserves_existing_file_and_recording(){
        let folder=tempfile::tempdir().unwrap();
        let destination=folder.path().join("recording.mkv");
        std::fs::write(&destination,b"existing").unwrap();
        let mut temp=tempfile::Builder::new().prefix(".lazy-tester-video-").suffix(".part")
            .tempfile_in(folder.path()).unwrap();
        temp.write_all(b"finalized fixture").unwrap();
        temp.as_file().sync_all().unwrap();
        let r=run(1);
        r.frames.store(1,Ordering::Release);
        assert!(publish_finalized(temp,&destination,&r).is_err());
        r.state.store(FAILED,Ordering::Release);
        let receipt=r.receipt(1);
        assert_eq!(receipt.state,"Failed");
        let retained=receipt.recovery_file.unwrap();
        assert_eq!(std::fs::read(folder.path().join(retained)).unwrap(),b"finalized fixture");
        assert_eq!(std::fs::read(destination).unwrap(),b"existing");
    }

    #[test]
    fn terminal_marker_visible_before_eof_and_does_not_validate_trailing_bytes(){
        struct AfterMarker<'a>{run:&'a Run,first:bool}
        impl Read for AfterMarker<'_>{
            fn read(&mut self,buf:&mut [u8])->io::Result<usize>{
                let bytes:&[u8]=if self.first{
                    self.first=false;
                    assert!(!self.run.terminal_progress.load(Ordering::Acquire));
                    b"frame=5\nprogress=end\n"
                }else{
                    assert!(self.run.terminal_progress.load(Ordering::Acquire));
                    b"unexpected"
                };
                buf[..bytes.len()].copy_from_slice(bytes);
                Ok(bytes.len())
            }
        }
        let r=run(1);
        assert!(read_progress(AfterMarker{run:&r,first:true},&r).is_err());
        assert!(r.terminal_progress.load(Ordering::Acquire));
    }

    #[test]
    fn progress_requires_terminal_receipt_and_real_frames(){
        let r=run(1);
        assert!(!read_progress(&b"frame=0\nprogress=continue\n"[..],&r).unwrap());
        assert_eq!(r.state.load(Ordering::Acquire),STARTING);
        assert!(read_progress(&b"frame=5\nprogress=end\n"[..],&r).unwrap());
        assert_eq!(r.frames.load(Ordering::Acquire),5);
        assert_eq!(r.state.load(Ordering::Acquire),RECORDING);
        assert!(read_progress(&b"frame=bad\n"[..],&r).is_err());
    }
}
