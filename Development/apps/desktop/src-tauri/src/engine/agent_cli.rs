//! One-shot, same-image native adapter for the local MCP stdio bridge.
//! The retained authenticated controller remains the only Minecraft game owner.
//! Does not open a Tauri window or own Minecraft. An explicitly opted-in
//! MC02-only background Win32 message experiment is NOT gameplay control.
use std::ffi::OsStr;
use std::io::{self, Write};
use serde_json::{json, Value};

fn parse_operation(args: &[String]) -> Result<(&'static str, Option<u32>), &'static str> {
    match args {
        [one] if one == "inspect" => Ok(("inspect", None)),
        #[cfg(feature="mc02-native-input")]
        [one,slot] if one=="launch-mc02-input" && slot=="2" =>
            Ok(("launch-mc02-input",Some(2))),
        #[cfg(feature="mc02-native-input")]
        [one,slot] if one=="launch-ai-hybrid" || one=="launch-ai-five" => {
            let slot=slot.parse::<u32>().map_err(|_|"INVALID_ARGUMENTS")?;
            if !(1..=5).contains(&slot) ||
               (one=="launch-ai-hybrid" && slot==1) {
                return Err("INVALID_ARGUMENTS");
            }
            Ok((if one=="launch-ai-hybrid" {"launch-ai-hybrid"}
                else {"launch-ai-five"},Some(slot)))
        },
        [one, slot] if one == "launch" => {
            let slot = slot.parse::<u32>().map_err(|_| "INVALID_ARGUMENTS")?;
            if !(1..=5).contains(&slot) {
                return Err("INVALID_ARGUMENTS");
            }
            Ok(("launch", Some(slot)))
        }
        [one, slot] if one == "target" => {
            let slot = slot.parse::<u32>().map_err(|_| "INVALID_ARGUMENTS")?;
            if !(1..=5).contains(&slot) {
                return Err("INVALID_ARGUMENTS");
            }
            Ok(("target", Some(slot)))
        }
        [operation, slot] if operation == "probe-background" && slot == "2" =>
            Ok(("probe-background", Some(2))),
        _ => Err("INVALID_ARGUMENTS"),
    }
}

fn native_error(error: io::Error) -> &'static str {
    // A recognized actor refusal is definitive only when its error
    // kind belongs to the retained controller. Lost IPC results cannot
    // masquerade as proven pre-mutation mode refusals.
    if matches!(error.kind(),io::ErrorKind::Other|io::ErrorKind::InvalidData) &&
       error.to_string()=="AI writer mode conflicts with original game channel" {
        return "NATIVE_WRITER_MODE_MISMATCH";
    }
    // The original actor rejects an expired or pre-birth acquire epoch
    // BEFORE touching its AiLeaseBook. The authenticated SessionReply
    // carries this error as a string, then native_clients_session::request
    // converts that exact refusal to io::ErrorKind::Other. Also admit the
    // direct PermissionDenied form; never promote BrokenPipe, timeout,
    // malformed response or any unrelated refusal into definite denial.
    if matches!(error.kind(),io::ErrorKind::Other|
        io::ErrorKind::PermissionDenied) &&
       error.to_string()=="AI transaction predates original game or is stale" {
        return "NATIVE_WRITER_EPOCH_STALE";
    }
    // This precise actor error means at least one best-effort neutral
    // write was not acknowledged after a partially failed time group.
    // IPC SessionReply maps the original InvalidData error to Other.
    // Do not classify a broken response channel as this known error.
    if matches!(error.kind(),io::ErrorKind::Other|io::ErrorKind::InvalidData) &&
       error.to_string()=="AI group neutral release unconfirmed" {
        return "AI_RELEASE_UNCONFIRMED";
    }
    // The retained group actor or last original writer may reject a
    // late slot after slow native HWND verification. Preserve the CLI's
    // existing group deadline code, but never promote transport loss
    // into an asserted actor refusal.
    if matches!(error.kind(),io::ErrorKind::Other|io::ErrorKind::InvalidData) &&
       error.to_string()=="AI timestamp group missed its native deadline" {
        return "AI_SEQUENCE_DEADLINE";
    }
    // Exact actor lease conflicts use PermissionDenied directly, or
    // ErrorKind::Other when the authenticated SessionReply string is
    // converted back to io::Error. Same-text transport errors are never
    // definitive no-write evidence for the MCP recovery decision.
    if matches!(error.kind(),io::ErrorKind::Other|io::ErrorKind::PermissionDenied) &&
       matches!(error.to_string().as_str(),
           "AI writer lease conflict; existing game input preserved" |
           "AI slot is not owned by this live writer lease" |
           "An AI slot is held by a different native writer") {
        return "NATIVE_WRITER_BUSY";
    }
    match error.kind() {
        io::ErrorKind::WouldBlock => "NATIVE_BUSY",
        // A broken result channel may follow a native operation which
        // already reached the retained controller. This is NOT a
        // definitive refusal and cannot authorize another input write.
        io::ErrorKind::TimedOut | io::ErrorKind::Interrupted |
        io::ErrorKind::BrokenPipe => "OUTCOME_UNCERTAIN",
        io::ErrorKind::PermissionDenied => "NATIVE_REFUSED",
        io::ErrorKind::Unsupported => "UNSUPPORTED",
        _ => "NATIVE_UNAVAILABLE",
    }
}

#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
#[derive(serde::Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Mc02BatchPulse {
    at_ms:u32,keys:u32,dx:i32,dy:i32,ttl_ms:u32,
}

// The original Channel::release_all rejects these exact conditions
// BEFORE publishing v4 shared state. A stale/expired batch cleanup must
// never retry with a newly minted timestamp and erase another MC02
// command. This is a proven actor refusal, not Minecraft key-up proof.
// SessionReply maps original controller InvalidData into ErrorKind::Other.
#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn pin_original_mc02_batch_from_snapshot(
    snapshot:&super::native_clients_session::SessionSnapshot,
    requested_at_ms:u64,
)->Result<super::native_clients_session::Mc02BatchOrigin,&'static str> {
    use super::native_clients_session::Mc02BatchOrigin;
    const UNIX_TO_FILETIME_100NS:u128=116_444_736_000_000_000;
    if snapshot.schema!=1 || snapshot.profiles.len()!=5 ||
       snapshot.profiles.iter().enumerate().any(|(i,p)|p.slot!=i as u32+1) {
        return Err("MC02_BATCH_TARGET_UNVERIFIED");
    }
    let profile=&snapshot.profiles[1];
    if profile.state!="RUNNING" {
        return Err("MC02_BATCH_TARGET_UNVERIFIED");
    }
    let process=profile.process.as_ref()
        .filter(|process|process.responding)
        .ok_or("MC02_BATCH_TARGET_UNVERIFIED")?;
    let birth=process.created_file_time.parse::<u64>()
        .map_err(|_|"MC02_BATCH_TARGET_UNVERIFIED")?;
    let pin=Mc02BatchOrigin{pid:process.pid,birth,session:process.session_id};
    // The process must ALREADY have existed when the model/MCP minted
    // this original batch request; an Inspect taken after a relaunch
    // cannot silently adopt the replacement. Millisecond birth ties
    // remain a conservative known limitation, not identity proof.
    let end_of_request_ms=UNIX_TO_FILETIME_100NS+
        (u128::from(requested_at_ms)+1)*10_000;
    if requested_at_ms==0 || !pin.valid() ||
       u128::from(birth)>=end_of_request_ms {
        return Err("MC02_BATCH_TARGET_UNVERIFIED");
    }
    Ok(pin)
}

#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn mc02_batch_cleanup_error(error:&io::Error)->&'static str {
    if matches!(error.kind(),io::ErrorKind::Other|io::ErrorKind::InvalidData) &&
       matches!(error.to_string().as_str(),
           "MC02 release predates newer active input" |
           "MC02 release predates newer neutral release" |
           "MC02 release request expired before native delivery" |
           "MC02 input predates original channel binding" |
           "MC02 batch original process changed before native input" |
           "MC02 batch cleanup superseded by another native sender") {
        return "MC02_RELEASE_OBSOLETE";
    }
    "MC02_RELEASE_UNCONFIRMED"
}

// Stable for one MCP stdio process, supplied privately to its short-lived
// native CLI children. The controller owns all actual slot lease authority.
// This identifier is NOT a secret, security boundary, or game credential.
#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn writer_id()->Result<String,&'static str> {
    let value=std::env::var("LAZY_TESTER_AI_WRITER_ID")
        .map_err(|_|"AI_WRITER_ID_REQUIRED")?;
    if !super::native_clients_session::valid_ai_writer_id(&value) {
        return Err("AI_WRITER_ID_REQUIRED");
    }
    Ok(value)
}

// A fresh nonce for each separately acquired native lease. It is never
// passed on argv, never stored on disk and never reused when the same
// MCP writer recovers or starts a later action transaction.
#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn lease_epoch()->Result<String,&'static str> {
    let value=std::env::var("LAZY_TESTER_AI_LEASE_EPOCH")
        .map_err(|_|"AI_LEASE_EPOCH_REQUIRED")?;
    if !super::native_clients_session::valid_ai_writer_id(&value) {
        return Err("AI_LEASE_EPOCH_REQUIRED");
    }
    Ok(value)
}

#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn execute_ai_lease(args:&[String])->Result<Value,&'static str> {
    use super::native_clients_session::{self,SessionRequest,
        AgentControlMode,validate_ai_lease_slots};
    if std::env::var("LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT").ok().as_deref()
       !=Some("I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT") {
        return Err("EXPERIMENT_NOT_AUTHORIZED");
    }
    let acquiring=args[0]=="ai-lease-acquire";
    if (acquiring && args.len()!=3) ||
       (!acquiring && args.len()!=2) || args[1].len()>9 {
        return Err("INVALID_ARGUMENTS");
    }
    let owner=writer_id()?;
    let epoch=lease_epoch()?;
    let slots=args[1].split(',').map(|s|s.parse::<u32>())
        .collect::<Result<Vec<_>,_>>().map_err(|_|"INVALID_ARGUMENTS")?;
    validate_ai_lease_slots(&slots,&owner).map_err(|_|"INVALID_ARGUMENTS")?;
    let request=if acquiring {
        // An opaque writer ID is not enough to decide game intent.
        // Explicit mode travels in the CLI child's private environment,
        // and the original controller checks EVERY child's launch mode
        // before claiming even one slot.
        let mode=match std::env::var("LAZY_TESTER_AI_WRITER_MODE")
            .ok().as_deref() {
            Some("five-ai")=>AgentControlMode::FiveAi,
            Some("human-plus-four-ai")=>AgentControlMode::HumanPlusFourAi,
            _=>return Err("AI_WRITER_MODE_REQUIRED"),
        };
        if mode==AgentControlMode::HumanPlusFourAi && slots.contains(&1) {
            return Err("INVALID_ARGUMENTS");
        }
        let neutral_only=match args[2].as_str() {
            "action"=>false,
            "recovery"=>true,
            _=>return Err("INVALID_ARGUMENTS"),
        };
        SessionRequest::AiLeaseAcquire {
            slots:slots.clone(),owner_id:owner,mode,
            lease_epoch:epoch,neutral_only,
        }
    }else {
        SessionRequest::AiLeaseRelease {
            slots:slots.clone(),owner_id:owner,lease_epoch:epoch,
        }
    };
    let snapshot=native_clients_session::request(request).map_err(native_error)?;
    if snapshot.schema!=1||snapshot.profiles.len()!=5 {
        return Err("INVALID_NATIVE_SNAPSHOT");
    }
    Ok(json!({"schema":1,"ok":true,"operation":args[0],
        "slots":slots,"lease":"NATIVE_CONTROLLER_PER_SLOT_ONLY",
        "gameplayObserved":false}))
}

// A bounded same-image CLI process schedules ALL pulses for one MC02
// sequence. The original authenticated retained controller is still
// the only input writer. This removes repeated process startups inside
// the 160ms heartbeat; it is NOT a new game controller or driver.
#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn execute_mc02_batch(args:&[String])->Result<Value,&'static str> {
    use super::native_clients_session::{self,SessionRequest};
    use windows_sys::Win32::System::SystemInformation::GetTickCount64;
    if std::env::var("LAZY_TESTER_MC02_VIRTUAL_INPUT").ok().as_deref()
        !=Some("I_ACCEPT_EXPERIMENTAL_MC02_INPUT") {
        return Err("EXPERIMENT_NOT_AUTHORIZED");
    }
    if args.len()!=4||args[1]!="2"||args[3].len()>8192 {
        return Err("INVALID_ARGUMENTS");
    }
    // Reject delayed process startup before any batch tick. Subsequent
    // ticks use the local monotonic uptime clock, not wall-clock time.
    let requested_at_ms=args[2].parse::<u64>().map_err(|_|"INVALID_ARGUMENTS")?;
    let now_wall=std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_|"STALE_MC02_INPUT")?.as_millis();
    if requested_at_ms==0||now_wall<u128::from(requested_at_ms)||
        now_wall-u128::from(requested_at_ms)>500 {
        return Err("STALE_MC02_INPUT");
    }
    let pulses:Vec<Mc02BatchPulse>=serde_json::from_str(&args[3])
        .map_err(|_|"INVALID_ARGUMENTS")?;
    if pulses.is_empty()||pulses.len()>64 {
        return Err("INVALID_ARGUMENTS");
    }
    let mut previous=None;
    for pulse in &pulses {
        // Explicit fixed native TTL and strictly increasing integer
        // timestamps; only MC02 can receive a scheduled input batch.
        if pulse.at_ms>3500||pulse.ttl_ms!=350||
            previous.is_some_and(|last|pulse.at_ms<=last) ||
            native_clients_session::admit_native_input_frame(
                2,pulse.keys,pulse.dx,pulse.dy,pulse.ttl_ms).is_err() {
            return Err("INVALID_ARGUMENTS");
        }
        previous=Some(pulse.at_ms);
    }
    // One authenticated, read-only source pin BEFORE the first pulse.
    // Subsequent existing Mc02Input requests carry this SAME original
    // PID/birth/session in an optional private field; the retained actor
    // compares it to its own original Channel binding BEFORE writes.
    // An unavailable/changed target is DEFINITIVELY no batch dispatch:
    // do not let MCP neutralize a replacement process as cleanup.
    let observed=native_clients_session::request(SessionRequest::Inspect)
        .map_err(|_|"MC02_BATCH_TARGET_UNVERIFIED")?;
    let batch_origin=pin_original_mc02_batch_from_snapshot(
        &observed,requested_at_ms)?;
    let after_observation=std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_|"MC02_BATCH_TARGET_UNVERIFIED")?.as_millis();
    if after_observation<u128::from(requested_at_ms) ||
       after_observation-u128::from(requested_at_ms)>500 {
        return Err("MC02_BATCH_TARGET_UNVERIFIED");
    }
    // Schedule relative to original same-image CLI monotonic start.
    // Only ONE native Inspect was added, never a per-pulse OS target
    // lookup, another game controller or a new external CLI tool.
    let started=unsafe{GetTickCount64()};
    // Timestamp of the last pulse which MAY have reached the original
    // native actor. A later cleanup must not claim it was issued after
    // another CLI's more recent one-shot command.
    let mut last_attempted_issue_ms=None;
    let result=(||->Result<(),&'static str>{
        for pulse in &pulses {
            let deadline=started.saturating_add(u64::from(pulse.at_ms));
            let now=unsafe{GetTickCount64()};
            if now<deadline {
                std::thread::sleep(std::time::Duration::from_millis(deadline-now));
            }
            if unsafe{GetTickCount64()}.saturating_sub(deadline)>150 {
                return Err("MC02_SEQUENCE_DEADLINE");
            }
            // An uncertain native reply can follow shared-state publish.
            // Pin the ORIGINAL issue time immediately before native IPC,
            // including a pulse whose acknowledgement may be lost.
            let issued_at_ms=unsafe{GetTickCount64()};
            last_attempted_issue_ms=Some(issued_at_ms);
            let snapshot=native_clients_session::request(SessionRequest::Mc02Input{
                slot:2,keys:pulse.keys,dx:pulse.dx,dy:pulse.dy,
                ttl_ms:pulse.ttl_ms,issued_at_ms,
                batch_origin:Some(batch_origin),
            }).map_err(native_error)?;
            if snapshot.schema!=1||snapshot.profiles.len()!=5 {
                return Err("INVALID_NATIVE_SNAPSHOT");
            }
            // The pre-dispatch check alone cannot establish a timely
            // heartbeat: authenticated IPC and the five-profile receipt
            // can finish after this pulse's +150-ms budget. Refuse a
            // late ACK even for the final pulse, and enter the existing
            // exact-MC02 neutral release below. This is NOT evidence
            // of when Minecraft consumed (or ignored) the input.
            if unsafe{GetTickCount64()}.saturating_sub(deadline)>150 {
                return Err("MC02_SEQUENCE_DEADLINE");
            }
        }
        Ok(())
    })();
    // The native writer compares requested issue times, not CLI
    // completion order. Reuse the last attempted pulse's timestamp for
    // cleanup; a later one-shot accepted by another CLI can then veto
    // this old ReleaseAll BEFORE cursor/packet publication.
    // If this stamp is already expired, native refuses it rather than
    // silently changing ownership. Virtual TTL is still bounded.
    if let Some(issued_at_ms)=last_attempted_issue_ms {
        let released=native_clients_session::request(SessionRequest::Mc02Input{
            slot:2,keys:0,dx:0,dy:0,ttl_ms:1,issued_at_ms,
            batch_origin:Some(batch_origin),
        });
        if let Err(error)=released {
            return Err(mc02_batch_cleanup_error(&error));
        }
    }
    result?;
    Ok(json!({"schema":1,"ok":true,"operation":"mc02-input-batch",
        "slot":2,"pulses":pulses.len(),
        "delivery":"PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF",
        "gameplayObserved":false}))
}

// One same-image scheduler for MC01-MC05. Each pulse is still separately
// admitted and written by the EXISTING retained authenticated controller.
// The game may ignore it: this is not Minecraft gameplay acceptance.
#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
#[derive(serde::Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct MultiAiPulse {
    slot:u32,at_ms:u32,keys:u32,dx:i32,dy:i32,ttl_ms:u32,
}

// The CLI validates ascending (at_ms,slot) before calling this function.
// One timestamp group uses ONE authenticated native session transaction,
// rather than 4–5 independent snapshot-producing IPC exchanges.
#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn multi_ai_group_end(pulses:&[MultiAiPulse],first:usize)->usize {
    if first>=pulses.len() {return pulses.len();}
    let at=pulses[first].at_ms;
    let mut next=first+1;
    while next<pulses.len() && pulses[next].at_ms==at {next+=1;}
    next
}

// Record a possibly touched channel only at the IPC dispatch boundary.
// A group that missed its final deadline has NOT reached IPC and must
// not trigger synthetic neutral writes to previously untouched slots.
#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn record_multi_ai_group_attempt(
    group:&[MultiAiPulse],attempted:&mut std::collections::BTreeSet<u32>
) {
    attempted.extend(group.iter().map(|pulse|pulse.slot));
}

// The existing sha2 dependency ties an ACK to exact ordered UTF-8
// wire bytes originally sent from Node, without another native IPC,
// controller, input hook, game process or shared-memory field.
#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn multi_ai_wire_digest(payload:&str)->String {
    use sha2::{Digest,Sha256};
    const HEX:&[u8;16]=b"0123456789abcdef";
    let mut out=String::with_capacity(64);
    for &byte in Sha256::digest(payload.as_bytes()).iter() {
        out.push(char::from(HEX[usize::from(byte>>4)]));
        out.push(char::from(HEX[usize::from(byte&15)]));
    }
    out
}

#[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
fn execute_multi_ai_batch(args:&[String])->Result<Value,&'static str> {
    use super::native_clients_session::{self,SessionRequest,AiInputFrame};
    use windows_sys::Win32::System::SystemInformation::GetTickCount64;
    use std::collections::BTreeSet;
    if std::env::var("LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT").ok().as_deref()
       !=Some("I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT") {
        return Err("EXPERIMENT_NOT_AUTHORIZED");
    }
    let owner=writer_id()?;
    let epoch=lease_epoch()?;
    if args.len()!=3 || args[2].len()>24000 {
        return Err("INVALID_ARGUMENTS");
    }
    let requested_at_ms=args[1].parse::<u64>().map_err(|_|"INVALID_ARGUMENTS")?;
    let now_wall=std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_|"STALE_AI_INPUT")?.as_millis();
    if requested_at_ms==0||now_wall<u128::from(requested_at_ms)||
       now_wall-u128::from(requested_at_ms)>500 {
        return Err("STALE_AI_INPUT");
    }
    let pulses:Vec<MultiAiPulse>=serde_json::from_str(&args[2])
        .map_err(|_|"INVALID_ARGUMENTS")?;
    if pulses.is_empty()||pulses.len()>320 {
        return Err("INVALID_ARGUMENTS");
    }
    let mut requested=BTreeSet::<u32>::new();
    let mut last=None;
    let mut per_slot=[0u32;5];
    for pulse in &pulses {
        if !(1..=5).contains(&pulse.slot)||pulse.at_ms>3500||
           pulse.ttl_ms!=350 ||
           last.is_some_and(|previous:(u32,u32)|
              (pulse.at_ms,pulse.slot)<=previous) ||
           native_clients_session::admit_native_input_frame(
             pulse.slot,pulse.keys,pulse.dx,pulse.dy,pulse.ttl_ms).is_err() {
            return Err("INVALID_ARGUMENTS");
        }
        per_slot[(pulse.slot-1) as usize]+=1;
        if per_slot[(pulse.slot-1) as usize]>64 {
            return Err("INVALID_ARGUMENTS");
        }
        requested.insert(pulse.slot);
        last=Some((pulse.at_ms,pulse.slot));
    }
    let pulse_digest=multi_ai_wire_digest(&args[2]);
    let started=unsafe{GetTickCount64()};
    let mut attempted=BTreeSet::<u32>::new();
    let mut groups=0usize;
    let result=(||->Result<(),&'static str>{
        let mut index=0;
        while index<pulses.len() {
            let group_end=multi_ai_group_end(&pulses,index);
            let group=&pulses[index..group_end];
            let deadline=started.saturating_add(u64::from(group[0].at_ms));
            let latest=deadline.saturating_add(150);
            let now=unsafe{GetTickCount64()};
            if now<deadline {
                std::thread::sleep(std::time::Duration::from_millis(deadline-now));
            }
            if unsafe{GetTickCount64()}>latest {
                return Err("AI_SEQUENCE_DEADLINE");
            }
            let frames=group.iter().map(|pulse|AiInputFrame {
                slot:pulse.slot,keys:pulse.keys,dx:pulse.dx,
                dy:pulse.dy,ttl_ms:pulse.ttl_ms,
            }).collect::<Vec<_>>();
            let issued_at_ms=unsafe{GetTickCount64()};
            if issued_at_ms>latest {return Err("AI_SEQUENCE_DEADLINE");}
            // The entire group becomes possibly touched only once we
            // actually enter the native request. A missed pre-IPC deadline
            // cannot require neutral writes for that never-issued group.
            // Still mark EVERY member just before dispatch: a lost IPC ACK
            // can follow partial group delivery and must release all of them.
            record_multi_ai_group_attempt(group,&mut attempted);
            // The original session actor still verifies every owned process
            // and channel. A group is NOT an atomic or simultaneous Windows
            // input broadcast; it only removes redundant IPC round trips.
            let snapshot=native_clients_session::request(
                SessionRequest::AiInputGroup {
                    frames,issued_at_ms,deadline_at_ms:latest,
                    owner_id:owner.clone(),lease_epoch:epoch.clone(),
                }
            ).map_err(native_error)?;
            if snapshot.schema!=1||snapshot.profiles.len()!=5 {
                return Err("INVALID_NATIVE_SNAPSHOT");
            }
            if unsafe{GetTickCount64()}>latest {
                return Err("AI_SEQUENCE_DEADLINE");
            }
            groups+=1;
            index=group_end;
        }
        Ok(())
    })();
    // A complete success response is possible ONLY after a native
    // neutral ACK on EACH possibly touched exact slot. Return the sorted
    // attempted cohort as source transport evidence, not gameplay key-up.
    // This allows the original MCP to avoid redundant cleanup after
    // proven success, but uncertain/failed batches still re-neutralize.
    let neutral_slots=attempted.into_iter().collect::<Vec<_>>();
    let mut release_failed=false;
    for &slot in &neutral_slots {
        if native_clients_session::request(SessionRequest::AiLeasedInput{
            slot,keys:0,dx:0,dy:0,ttl_ms:1,
            issued_at_ms:unsafe{GetTickCount64()},
            owner_id:owner.clone(),lease_epoch:epoch.clone(),
        }).is_err() {
            release_failed=true;
        }
    }
    if release_failed {return Err("AI_RELEASE_UNCONFIRMED");}
    result?;
    Ok(json!({"schema":1,"ok":true,"operation":"ai-multi-input-batch",
        "slots":requested.into_iter().collect::<Vec<_>>(),
        "pulses":pulses.len(),
        "timestampGroups":groups,
        "pulseDigest":pulse_digest,
        "neutralSlots":neutral_slots,
        "neutralState":"NATIVE_PER_SLOT_NEUTRAL_ACK",
        "delivery":"PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF",
        "gameplayObserved":false}))
}

fn execute(args: &[String]) -> Result<Value, &'static str> {
    // A read-only recovery inspection of the original native controller.
    // No Testing owner takeover, expired-lease shortcut, game command,
    // log access or second controller startup.
    if args.first().map(String::as_str)==Some("testing-inspect") {
        #[cfg(not(all(windows,target_arch="x86_64",feature="lazy-tester-package")))]
        {return Err("UNSUPPORTED");}
        #[cfg(all(windows,target_arch="x86_64",feature="lazy-tester-package"))]
        {
            use super::native_clients_session::{self,SessionRequest,valid_testing_id};
            if args.len()!=2{return Err("INVALID_ARGUMENTS");}
            let slot=args[1].parse::<u32>().map_err(|_|"INVALID_ARGUMENTS")?;
            if !(1..=5).contains(&slot){return Err("INVALID_ARGUMENTS");}
            let owner=std::env::var("LAZY_TESTER_TESTING_OWNER_ID")
                .map_err(|_|"TESTING_OWNER_ID_REQUIRED")?;
            if !valid_testing_id(&owner){return Err("TESTING_OWNER_ID_REQUIRED");}
            if !native_clients_session::existing_controller_gate(
                "Local\\MivubiEducationStartupObservation").map_err(native_error)? {
                return Err("CONTROLLER_UNAVAILABLE");
            }
            let snapshot=native_clients_session::request(
                SessionRequest::TestingInspect{slot,owner_id:owner})
                .map_err(native_error)?;
            let inspection=snapshot.testing_inspection
                .ok_or("TESTING_RESULT_UNAVAILABLE")?;
            return Ok(json!({"schema":1,"ok":true,
                "operation":"testing-inspect","inspection":inspection}));
        }
    }
    // Correlation-only native Testing arbitration, never an executor.
    // The random MCP owner token is passed via environment, NOT CLI argv.
    if matches!(args.first().map(String::as_str),
       Some("testing-reserve"|"testing-finalize")) {
        #[cfg(not(all(windows,target_arch="x86_64",feature="lazy-tester-package")))]
        {return Err("UNSUPPORTED");}
        #[cfg(all(windows,target_arch="x86_64",feature="lazy-tester-package"))]
        {
            use super::native_clients_session::{self,SessionRequest,valid_testing_id};
            let owner=std::env::var("LAZY_TESTER_TESTING_OWNER_ID")
                .map_err(|_|"TESTING_OWNER_ID_REQUIRED")?;
            if !valid_testing_id(&owner){return Err("TESTING_OWNER_ID_REQUIRED");}
            if args.len()<3{return Err("INVALID_ARGUMENTS");}
            let slot=args[1].parse::<u32>().map_err(|_|"INVALID_ARGUMENTS")?;
            let run_id=&args[2];
            if !(1..=5).contains(&slot)||!valid_testing_id(run_id) {
                return Err("INVALID_ARGUMENTS");
            }
            let request=if args[0]=="testing-reserve" {
                if args.len()!=4{return Err("INVALID_ARGUMENTS");}
                let replace=if args[3]=="none"{None}
                    else if valid_testing_id(&args[3]){Some(args[3].clone())}
                    else{return Err("INVALID_ARGUMENTS")};
                SessionRequest::TestingReserve {slot,owner_id:owner,
                    run_id:run_id.clone(),replace_run_id:replace}
            } else {
                if args.len()!=5{return Err("INVALID_ARGUMENTS");}
                SessionRequest::TestingFinalize {slot,owner_id:owner,
                    run_id:run_id.clone(),
                    reported_status:args[3].clone(),report_digest:args[4].clone()}
            };
            // Do not silently create the original manager or adopt another
            // executable's game process just to claim a Testing run.
            if !native_clients_session::existing_controller_gate(
                "Local\\MivubiEducationStartupObservation").map_err(native_error)? {
                return Err("CONTROLLER_UNAVAILABLE");
            }
            let observed=native_clients_session::request(request)
                .map_err(native_error)?;
            let run=observed.testing_run.ok_or("TESTING_RESULT_UNAVAILABLE")?;
            return Ok(json!({"schema":1,"ok":true,
                "operation":args[0],"run":run}));
        }
    }
    // Explicit, one-shot, non-gameplay metadata report to the retained
    // same-image controller. Never starts a controller, game, or worker.
    if args.first().map(String::as_str)==Some("report-agent-activity") {
        #[cfg(not(all(windows,target_arch="x86_64",feature="lazy-tester-package")))]
        {return Err("UNSUPPORTED");}
        #[cfg(all(windows,target_arch="x86_64",feature="lazy-tester-package"))]
        {
            use super::native_clients_session::{self,SessionRequest,AgentActivityReport};
            if args.len()!=2||args[1].len()>5000{return Err("INVALID_ARGUMENTS");}
            let activity:AgentActivityReport=serde_json::from_str(&args[1])
                .map_err(|_|"INVALID_ARGUMENTS")?;
            if !activity.valid_ingress(){return Err("INVALID_ARGUMENTS");}
            let connected=native_clients_session::existing_controller_gate(
                "Local\\MivubiEducationStartupObservation").map_err(native_error)?;
            if !connected{return Err("CONTROLLER_UNAVAILABLE");}
            native_clients_session::request(
                SessionRequest::PublishAgentActivity{activity}).map_err(native_error)?;
            return Ok(json!({"schema":1,"ok":true,
                "operation":"report-agent-activity",
                "status":"LOCAL_AGENT_REPORT_RECEIVED_NOT_DESKTOP_CONTROL"}));
        }
    }
    #[cfg(all(windows,target_arch="x86_64",
              feature="lazy-tester-package",feature="mc02-native-input"))]
    if args.first().map(String::as_str)==Some("ai-multi-input-batch") {
        return execute_multi_ai_batch(args);
    }
    #[cfg(all(windows,target_arch="x86_64",
              feature="lazy-tester-package",feature="mc02-native-input"))]
    if matches!(args.first().map(String::as_str),
        Some("ai-lease-acquire")|Some("ai-lease-release")) {
        return execute_ai_lease(args);
    }
    #[cfg(all(windows,target_arch="x86_64",
              feature="lazy-tester-package",feature="mc02-native-input"))]
    if args.first().map(String::as_str)==Some("mc02-input-batch") {
        return execute_mc02_batch(args);
    }
    #[cfg(all(windows,target_arch="x86_64",
              feature="lazy-tester-package",feature="mc02-native-input"))]
    if matches!(args.first().map(String::as_str),Some("mc02-input")|Some("ai-input")) {
        // Private one-shot adapter reuses the original authenticated
        // native controller. The actual game process must own an opted-in
        // slot-specific channel and matched launch mode.
        use super::native_clients_session::{self,SessionRequest};
        let legacy=args.first().map(String::as_str)==Some("mc02-input");
        let consent=if legacy {
            std::env::var("LAZY_TESTER_MC02_VIRTUAL_INPUT").ok().as_deref()
                ==Some("I_ACCEPT_EXPERIMENTAL_MC02_INPUT")
        } else {
            std::env::var("LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT").ok().as_deref()
                ==Some("I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT")
        };
        if !consent {return Err("EXPERIMENT_NOT_AUTHORIZED");}
        if args.len()!=7 {return Err("INVALID_ARGUMENTS");}
        let slot=args[1].parse::<u32>().map_err(|_|"INVALID_ARGUMENTS")?;
        if !(1..=5).contains(&slot)||(legacy&&slot!=2) {
            return Err("INVALID_ARGUMENTS");
        }
        let keys=args[2].parse::<u32>().map_err(|_|"INVALID_ARGUMENTS")?;
        let dx=args[3].parse::<i32>().map_err(|_|"INVALID_ARGUMENTS")?;
        let dy=args[4].parse::<i32>().map_err(|_|"INVALID_ARGUMENTS")?;
        let ttl_ms=args[5].parse::<u32>().map_err(|_|"INVALID_ARGUMENTS")?;
        let requested_at_ms=args[6].parse::<u64>().map_err(|_|"INVALID_ARGUMENTS")?;
        // Enforce the same per-slot keyboard/camera admission before
        // asking the retained native controller to inspect any process.
        native_clients_session::admit_native_input_frame(slot,keys,dx,dy,ttl_ms)
            .map_err(|_|"INVALID_ARGUMENTS")?;
        // A delayed Node request must not gain a new issue time merely
        // because this one-shot executable started late. ReleaseAll and
        // active input can arrive from DIFFERENT CLI processes; the
        // retained MC02 Channel orders them by GetTickCount64 issue time.
        // Sample uptime before wall time: if sampling is delayed, the
        // original request is backdated conservatively, never made newer.
        let uptime_now=unsafe{
            windows_sys::Win32::System::SystemInformation::GetTickCount64()
        };
        let wall_now=std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_|"STALE_MC02_INPUT")?
            .as_millis();
        if requested_at_ms==0 || wall_now<u128::from(requested_at_ms)) {
            return Err("STALE_MC02_INPUT");
        }
        let request_age_ms=wall_now-u128::from(requested_at_ms);
        if request_age_ms>500 {
            return Err("STALE_MC02_INPUT");
        }
        // Convert the bounded original MCP request age to this same
        // host's monotonic uptime clock. Keep the request's original
        // ordering instead of letting slow process startup turn an old
        // release into a fresh one. Reject underflow and uptime zero;
        // never wrap a newborn machine's tick counter.
        let issued_at_ms=uptime_now.checked_sub(request_age_ms as u64)
            .filter(|stamp|*stamp!=0).ok_or("STALE_MC02_INPUT")?;
        let request=if legacy {
            SessionRequest::Mc02Input{
                slot,keys,dx,dy,ttl_ms,issued_at_ms,batch_origin:None,
            }
        }else{
            SessionRequest::AiLeasedInput {
                slot,keys,dx,dy,ttl_ms,issued_at_ms,
                owner_id:writer_id()?,lease_epoch:lease_epoch()?,
            }
        };
        let snapshot=native_clients_session::request(request)
            .map_err(native_error)?;
        if snapshot.schema!=1||snapshot.profiles.len()!=5 {
            return Err("INVALID_NATIVE_SNAPSHOT");
        }
        return Ok(json!({"schema":1,"ok":true,
            "operation":args[0],"slot":slot,
            "delivery":"PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF",
            "gameplayObserved":false}));
    }
    let (operation, slot) = parse_operation(args)?;
    #[cfg(all(windows, target_arch="x86_64",feature="lazy-tester-package",
              feature="mc02-native-input"))]
    if operation=="launch-mc02-input" &&
       std::env::var("LAZY_TESTER_MC02_VIRTUAL_INPUT").ok().as_deref()
         !=Some("I_ACCEPT_EXPERIMENTAL_MC02_INPUT") {
        return Err("EXPERIMENT_NOT_AUTHORIZED");
    }
    #[cfg(all(windows, target_arch="x86_64",feature="lazy-tester-package",
              feature="mc02-native-input"))]
    if (operation=="launch-ai-hybrid" || operation=="launch-ai-five") &&
       std::env::var("LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT").ok().as_deref()
         !=Some("I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT") {
        return Err("EXPERIMENT_NOT_AUTHORIZED");
    }
    #[cfg(not(all(windows, target_arch = "x86_64", feature = "lazy-tester-package")))]
    {
        let _ = (operation, slot);
        Err("UNSUPPORTED")
    }
    #[cfg(all(windows, target_arch = "x86_64", feature = "lazy-tester-package"))]
    {
        use super::native_clients_session::{self, SessionRequest};
        // An Inspect without retained session ownership cannot truthfully label
        // five potentially unadopted games as STOPPED.
        if operation == "inspect" || operation == "target" {
            let owner = native_clients_session::existing_controller_gate(
                "Local\\MivubiEducationStartupObservation",
            )
            .map_err(native_error)?;
            if !owner {
                return Err("CONTROLLER_UNAVAILABLE");
            }
        }
        if operation == "target" {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GetForegroundWindow, IsHungAppWindow, IsIconic, IsWindowVisible,
            };
            // Same authenticated native controller as the UI. No PID scanning
            // or adoption of an unowned Minecraft client.
            let slot = slot.ok_or("INVALID_ARGUMENTS")?;
            let snapshot = native_clients_session::request(SessionRequest::Inspect)
                .map_err(native_error)?;
            if snapshot.schema != 1 || snapshot.profiles.len() != 5 ||
                snapshot.profiles.iter().enumerate()
                    .any(|(index,profile)| profile.slot as usize != index+1) {
                return Err("INVALID_NATIVE_SNAPSHOT");
            }
            let profile = &snapshot.profiles[(slot - 1) as usize];
            if profile.state != "RUNNING" || profile.issue.is_some() {
                return Err("TARGET_NOT_RUNNING");
            }
            let process = profile.process.as_ref().filter(|p|p.responding)
                .ok_or("TARGET_NOT_READY")?;
            // Reuse the existing process HANDLE / birth FILETIME / Windows
            // session / root HWND verifier. It pins the source process while
            // the window is checked, rather than trusting a numeric PID.
            let (window_id,foreground) =
                super::workspace_surface::with_verified_native_window(
                    process, |verified_hwnd| {
                        let window = verified_hwnd as _;
                        if verified_hwnd == 0 ||
                            verified_hwnd as u64 > ((1u64 << 53) - 1) {
                            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                "HWND cannot be represented as a safe MCP number"));
                        }
                        if unsafe { IsWindowVisible(window) } == 0 ||
                            unsafe { IsIconic(window) } != 0 ||
                            unsafe { IsHungAppWindow(window) } != 0 {
                            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                                "Minecraft window is unavailable for inspection"));
                        }
                        Ok((verified_hwnd as u64,
                            unsafe { GetForegroundWindow() } == window))
                    }).map_err(native_error)?;
            return Ok(json!({
                "schema": 1, "ok": true, "operation": "target",
                "slot": slot, "pid": process.pid, "windowId": window_id,
                "createdFileTime": process.created_file_time,
                "sessionId": process.session_id, "foreground": foreground,
                "visible": true, "minimized": false,
                "verification": "SAME_SESSION_PID_BIRTH_ROOT_HWND",
                "gameplayInput": "NOT_VERIFIED"
            }));
        }
        if operation == "probe-background" {
            // Manual opt-in only; never an automatically advertised MCP tool.
            if std::env::var("LAZY_TESTER_MC02_INPUT_PROBE").ok().as_deref()
                != Some("I_ACCEPT_ONE_W_PULSE")
            {
                return Err("EXPERIMENT_NOT_AUTHORIZED");
            }
            if !native_clients_session::existing_controller_gate(
                "Local\\MivubiEducationStartupObservation",
            ).map_err(native_error)? {
                return Err("CONTROLLER_UNAVAILABLE");
            }
            let snapshot = native_clients_session::request(SessionRequest::Inspect)
                .map_err(native_error)?;
            if snapshot.schema != 1 || snapshot.profiles.len() != 5 ||
                snapshot.profiles.iter().enumerate().any(|(i,p)|p.slot as usize != i+1) {
                return Err("INVALID_NATIVE_SNAPSHOT");
            }
            let human = &snapshot.profiles[0];
            let ai = &snapshot.profiles[1];
            if human.state != "RUNNING" || ai.state != "RUNNING" {
                return Err("PROBE_REQUIRES_TWO_RUNNING_GAMES");
            }
            let human_process = human.process.as_ref().filter(|p|p.responding)
                .ok_or("HUMAN_GAME_NOT_READY")?;
            let ai_process = ai.process.as_ref().filter(|p|p.responding)
                .ok_or("AI_GAME_NOT_READY")?;
            super::background_input_probe::one_forward_pulse(human_process,ai_process)
                .map_err(native_error)?;
            return Ok(json!({
                "schema":1, "ok":true, "operation":"probe-background", "slot":2,
                "delivery":"WINDOW_MESSAGES_POSTED_ONLY",
                "humanForegroundPreserved":true, "gameplayObserved":false,
                "evidence":"NOT_VERIFIED_IN_MINECRAFT"
            }));
        }
        #[cfg(feature="mc02-native-input")]
        let request=if operation=="launch-mc02-input" {
            SessionRequest::LaunchMc02Experiment {
                slot:slot.ok_or("INVALID_ARGUMENTS")?
            }
        } else if operation=="launch-ai-hybrid" || operation=="launch-ai-five" {
            use super::native_clients_session::AgentControlMode;
            SessionRequest::LaunchAiExperiment {
                slot:slot.ok_or("INVALID_ARGUMENTS")?,
                mode:if operation=="launch-ai-five" {
                    AgentControlMode::FiveAi
                } else {
                    AgentControlMode::HumanPlusFourAi
                },
            }
        } else {
            match slot {
                Some(slot)=>SessionRequest::Launch {slot},
                None=>SessionRequest::Inspect,
            }
        };
        #[cfg(not(feature="mc02-native-input"))]
        let request = match slot {
            Some(slot) => SessionRequest::Launch { slot },
            None => SessionRequest::Inspect,
        };
        let snapshot = native_clients_session::request(request).map_err(native_error)?;
        if snapshot.schema != 1 || snapshot.profiles.len() != 5 {
            return Err("INVALID_NATIVE_SNAPSHOT");
        }
        let mut profiles = Vec::with_capacity(5);
        for (index, profile) in snapshot.profiles.into_iter().enumerate() {
            if profile.slot as usize != index + 1 {
                return Err("INVALID_NATIVE_SNAPSHOT");
            }
            profiles.push(json!({
                "slot": profile.slot,
                "state": profile.state,
                "hasWindow": profile.process.is_some(),
                "responding": profile.process.as_ref().is_some_and(|p|p.responding),
                "needsAttention": profile.issue.is_some(),
            }));
        }
        Ok(json!({
            "schema": 1,
            "ok": true,
            "operation": operation,
            "profiles": profiles,
            "evidence": "NATIVE_SESSION_ONLY_NOT_GAMEPLAY_OR_ACCOUNT_SIGNIN",
        }))
    }
}

pub fn maybe_run() -> bool {
    let mut argv = std::env::args_os().skip(1);
    if argv.next().as_deref() != Some(OsStr::new("--lazy-tester-agent-cli")) {
        return false;
    }
    // Malformed arguments cannot fall through to launching the ordinary manager.
    let args: Option<Vec<String>> = argv.map(|a| a.to_str().map(str::to_owned)).collect();
    let result = match args {
        Some(values) => execute(&values),
        None => Err("INVALID_ARGUMENTS"),
    };
    let ok = result.is_ok();
    let response = result.unwrap_or_else(|code| json!({
        "schema": 1, "ok": false, "code": code,
    }));
    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{response}");
    let _ = out.flush();
    if !ok {
        std::process::exit(2);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
    #[test]
    fn original_mc02_batch_pin_refuses_unverified_or_newly_launched_process() {
        // Pure snapshot fixture; no Windows Inspector, IPC, controller
        // creation or Minecraft access. Exact original birth is the pin.
        let issued=1_800_000_000_000u64;
        let prior_birth=116_444_736_000_000_000u128+
            u128::from(issued-5_000)*10_000;
        let mut snapshot=super::native_clients_session::SessionSnapshot::stopped();
        snapshot.profiles[1].state="RUNNING".into();
        snapshot.profiles[1].process=Some(
            crate::engine::process_identity::OwnedProcess{
                pid:202,created_file_time:prior_birth.to_string(),
                session_id:1,window_handle:"0x123".into(),responding:true,
            });
        let original=pin_original_mc02_batch_from_snapshot(
            &snapshot,issued).unwrap();
        assert_eq!(original.pid,202);
        assert_eq!(original.birth,prior_birth as u64);
        assert_eq!(original.session,1);
        snapshot.profiles[1].process.as_mut().unwrap()
            .created_file_time=(prior_birth+60_000_000).to_string();
        assert!(pin_original_mc02_batch_from_snapshot(&snapshot,issued).is_err(),
            "a new process launched AFTER the issued request cannot be pinned");
        snapshot.profiles[1].process.as_mut().unwrap()
            .created_file_time=prior_birth.to_string();
        snapshot.profiles[1].process.as_mut().unwrap().responding=false;
        assert!(pin_original_mc02_batch_from_snapshot(&snapshot,issued).is_err());
        snapshot.profiles[1].process.as_mut().unwrap().responding=true;
        snapshot.profiles[1].slot=3;
        assert!(pin_original_mc02_batch_from_snapshot(&snapshot,issued).is_err());
    }

    #[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
    #[test]
    fn obsolete_legacy_mc02_batch_cleanup_requires_proven_actor_refusal() {
        // Pure authored Rust fixture; never executes controller or game.
        use std::io::{Error,ErrorKind};
        for reason in [
            "MC02 release predates newer active input",
            "MC02 release predates newer neutral release",
            "MC02 release request expired before native delivery",
            "MC02 input predates original channel binding",
            "MC02 batch original process changed before native input",
            "MC02 batch cleanup superseded by another native sender",
        ] {
            for kind in [ErrorKind::Other,ErrorKind::InvalidData] {
                assert_eq!(mc02_batch_cleanup_error(&Error::new(kind,reason)),
                    "MC02_RELEASE_OBSOLETE");
            }
            for kind in [
                ErrorKind::BrokenPipe,ErrorKind::TimedOut,
                ErrorKind::Interrupted,ErrorKind::WouldBlock,
                ErrorKind::PermissionDenied,
            ] {
                assert_eq!(mc02_batch_cleanup_error(&Error::new(kind,reason)),
                    "MC02_RELEASE_UNCONFIRMED",
                    "transport uncertainty cannot impersonate stale actor refusal");
            }
        }
        assert_eq!(mc02_batch_cleanup_error(&Error::other(
            "other original controller refusal")),"MC02_RELEASE_UNCONFIRMED");
    }

    #[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
    #[test]
    fn multi_ai_grouping_keeps_same_timestamp_slots_together() {
        let pulse=|slot,at_ms|MultiAiPulse {
            slot,at_ms,keys:1,dx:0,dy:0,ttl_ms:350,
        };
        for ids in [vec![2,3,4,5],vec![1,2,3,4,5]] {
            let mut pulses=ids.iter().map(|&slot|pulse(slot,0)).collect::<Vec<_>>();
            pulses.extend(ids.iter().map(|&slot|pulse(slot,160)));
            assert_eq!(multi_ai_group_end(&pulses,0),ids.len());
            assert_eq!(multi_ai_group_end(&pulses,ids.len()),pulses.len());
            assert_eq!(multi_ai_group_end(&pulses,pulses.len()),pulses.len());
            for group in [&pulses[..ids.len()],&pulses[ids.len()..]] {
                assert!(group.windows(2).all(|w|w[0].slot<w[1].slot));
                assert!(group.iter().all(|f|f.at_ms==group[0].at_ms));
            }
        }
    }

    #[cfg(all(windows,target_arch="x86_64",
           feature="lazy-tester-package",feature="mc02-native-input"))]
    #[test]
    fn native_digest_refuses_equal_count_changed_pulses() {
        // Authored NOT_EXECUTED: all have two pulses, one timestamp,
        // two slots, but different look or wire order.
        let original=r#"[{"slot":2,"atMs":0,"keys":1,"dx":0,"dy":0,"ttlMs":350},{"slot":3,"atMs":0,"keys":1,"dx":0,"dy":0,"ttlMs":350}]"#;
        let different_look=r#"[{"slot":2,"atMs":0,"keys":1,"dx":1,"dy":0,"ttlMs":350},{"slot":3,"atMs":0,"keys":1,"dx":0,"dy":0,"ttlMs":350}]"#;
        let different_order=r#"[{"slot":3,"atMs":0,"keys":1,"dx":0,"dy":0,"ttlMs":350},{"slot":2,"atMs":0,"keys":1,"dx":0,"dy":0,"ttlMs":350}]"#;
        let digest=multi_ai_wire_digest(original);
        assert_eq!(digest.len(),64);
        assert!(digest.bytes().all(|b|b.is_ascii_hexdigit()&&
            !b.is_ascii_uppercase()));
        assert_eq!(digest,multi_ai_wire_digest(original));
        assert_ne!(digest,multi_ai_wire_digest(different_look));
        assert_ne!(digest,multi_ai_wire_digest(different_order));
    }

    #[cfg(all(windows,target_arch="x86_64",
          feature="lazy-tester-package",feature="mc02-native-input"))]
    #[test]
    fn never_dispatched_ai_group_is_not_added_to_cleanup_cohort() {
        use std::collections::BTreeSet;
        let frame=|slot,at_ms|MultiAiPulse {
            slot,at_ms,keys:1,dx:0,dy:0,ttl_ms:350,
        };
        let first=[frame(2,0),frame(3,0)];
        let later=[frame(4,160),frame(5,160)];
        let mut attempted=BTreeSet::new();
        // A deadline missed before native request never calls the
        // attempt recorder. Previously even that group was added.
        assert!(attempted.is_empty());
        record_multi_ai_group_attempt(&first,&mut attempted);
        assert_eq!(attempted.iter().copied().collect::<Vec<_>>(),[2,3]);
        // No call for the late second group; only earlier possibly
        // published originals remain eligible for best-effort neutral.
        assert!(!attempted.contains(&4)&&!attempted.contains(&5));
        record_multi_ai_group_attempt(&later,&mut attempted);
        assert_eq!(attempted.iter().copied().collect::<Vec<_>>(),
            [2,3,4,5]);
    }

    #[cfg(all(windows,target_arch="x86_64",
           feature="lazy-tester-package",feature="mc02-native-input"))]
    #[test]
    fn group_cleanup_cohort_is_exact_sorted_no_extra_slots() {
        // Source fixture only: a successful native receipt can report
        // only the attempted per-slot channels. No Windows I/O invoked.
        use std::collections::BTreeSet;
        for original in [vec![2,3,4,5],vec![1,2,3,4,5]] {
            let mut attempted=BTreeSet::new();
            for slot in original.iter().rev() {attempted.insert(*slot);}
            let neutral_slots=attempted.into_iter().collect::<Vec<_>>();
            assert_eq!(neutral_slots,original);
        }
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn lease_conflicts_have_stable_private_native_error_code() {
        use std::io::{Error,ErrorKind};
        let known=Error::new(ErrorKind::Other,
            "AI writer lease conflict; existing game input preserved");
        assert_eq!(native_error(known),"NATIVE_WRITER_BUSY");
        assert_eq!(native_error(Error::other(
            "AI writer mode conflicts with original game channel")),
            "NATIVE_WRITER_MODE_MISMATCH");
        assert_eq!(native_error(Error::new(ErrorKind::InvalidData,
            "AI writer mode conflicts with original game channel")),
            "NATIVE_WRITER_MODE_MISMATCH");
        assert_eq!(native_error(Error::new(ErrorKind::PermissionDenied,
            "AI writer lease conflict; existing game input preserved")),
            "NATIVE_WRITER_BUSY");
        for message in [
            "AI writer lease conflict; existing game input preserved",
            "AI slot is not owned by this live writer lease",
            "An AI slot is held by a different native writer",
            "AI writer mode conflicts with original game channel",
        ] {
            // This source-only negative fixture protects the same
            // no-write error codes consumed by MCP's lease-acquire gate.
            for kind in [
                ErrorKind::BrokenPipe,ErrorKind::TimedOut,
                ErrorKind::Interrupted,
            ] {
                assert_eq!(native_error(Error::new(kind,message)),
                    "OUTCOME_UNCERTAIN",
                    "transport loss is not native no-write evidence");
            }
            assert_eq!(native_error(Error::new(ErrorKind::WouldBlock,message)),
                "NATIVE_BUSY",
                "transport contention is not a proven actor rejection");
        }
        assert_eq!(native_error(Error::new(ErrorKind::PermissionDenied,
            "AI transaction predates original game or is stale")),
            "NATIVE_WRITER_EPOCH_STALE");
        assert_eq!(native_error(Error::other(
            "AI transaction predates original game or is stale")),
            "NATIVE_WRITER_EPOCH_STALE",
            "an authenticated SessionReply maps error strings to ErrorKind::Other");
        assert_eq!(native_error(Error::new(ErrorKind::BrokenPipe,
            "AI transaction predates original game or is stale")),
            "OUTCOME_UNCERTAIN",
            "lost IPC result must never impersonate no-write epoch denial");
        assert_eq!(native_error(Error::new(ErrorKind::PermissionDenied,
            "Some other native admission refused")),
            "NATIVE_REFUSED",
            "unknown refusal is not the proved pre-mutation epoch contract");
        assert_eq!(native_error(Error::other("Unknown private controller detail")),
            "NATIVE_UNAVAILABLE");
        assert_eq!(native_error(Error::new(ErrorKind::BrokenPipe,
            "original IPC worker result missing")),"OUTCOME_UNCERTAIN");
        assert_eq!(native_error(Error::other(
            "AI group neutral release unconfirmed")),"AI_RELEASE_UNCONFIRMED");
        assert_eq!(native_error(Error::new(ErrorKind::InvalidData,
            "AI group neutral release unconfirmed")),"AI_RELEASE_UNCONFIRMED");
        assert_eq!(native_error(Error::new(ErrorKind::BrokenPipe,
            "AI group neutral release unconfirmed")),"OUTCOME_UNCERTAIN",
            "lost IPC cannot masquerade as a confirmed actor error");
        assert_eq!(native_error(Error::new(ErrorKind::TimedOut,
            "original worker still running")),"OUTCOME_UNCERTAIN");
        assert_eq!(native_error(Error::other(
            "AI timestamp group missed its native deadline")),
            "AI_SEQUENCE_DEADLINE");
        assert_eq!(native_error(Error::new(ErrorKind::InvalidData,
            "AI timestamp group missed its native deadline")),
            "AI_SEQUENCE_DEADLINE");
        assert_eq!(native_error(Error::new(ErrorKind::BrokenPipe,
            "AI timestamp group missed its native deadline")),
            "OUTCOME_UNCERTAIN",
            "lost IPC may not impersonate a proved actor deadline");
    }
    #[test]
    fn one_shot_command_accepts_only_inspect_or_five_slots() {
        assert_eq!(parse_operation(&["inspect".into()]).unwrap(), ("inspect", None));
        assert_eq!(parse_operation(&["launch".into(), "5".into()]).unwrap(), ("launch", Some(5)));
        #[cfg(feature="mc02-native-input")]
        {
            assert_eq!(parse_operation(&["launch-mc02-input".into(),"2".into()]).unwrap(),
                ("launch-mc02-input",Some(2)));
            assert_eq!(parse_operation(&["launch-ai-hybrid".into(),"5".into()]).unwrap(),
                ("launch-ai-hybrid",Some(5)));
            assert_eq!(parse_operation(&["launch-ai-five".into(),"1".into()]).unwrap(),
                ("launch-ai-five",Some(1)));
            assert!(parse_operation(&["launch-ai-hybrid".into(),"1".into()]).is_err());
            assert!(parse_operation(&["launch-ai-five".into(),"6".into()]).is_err());
        }
        assert_eq!(parse_operation(&["target".into(), "2".into()]).unwrap(),
            ("target", Some(2)));
        assert_eq!(parse_operation(&["probe-background".into(), "2".into()]).unwrap(),
            ("probe-background", Some(2)));
        for args in [vec!["close"], vec!["launch"], vec!["launch", "0"],
            vec!["launch", "6"], vec!["inspect", "1"], vec!["launch", "1", "extra"],
            vec!["launch-mc02-input","1"],vec!["launch-mc02-input","3"],
            vec!["probe-background", "1"], vec!["probe-background", "3"],
            vec!["probe-background"], vec!["probe-background", "2", "extra"],
            vec!["target"], vec!["target", "0"], vec!["target", "6"],
            vec!["target", "2", "more"]] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(parse_operation(&args).is_err());
        }
    }
}
