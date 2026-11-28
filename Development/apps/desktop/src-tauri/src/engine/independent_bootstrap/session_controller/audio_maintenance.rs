//! Controller-side audio cadence and exact retained-game roster projection.
//! The existing session_audio owner retains COM policy and restoration.
use super::Sessions;
use super::super::session_audio;
use std::{io, time::{Duration, Instant}};

// Audio COM reads are separate from process exit. Stable audio uses its
// 1-second discovery deadline; a process-local focus hint can wake it early.
// Keep the short deadline when the hint or prior observation is unavailable.
pub(super) const AUDIO_IDLE_RESYNC: Duration = session_audio::DISCOVERY_INTERVAL;
pub(super) type AudioRoster = [Option<(u32, u64)>; 5];
pub(super) fn audio_roster(sessions: &Sessions) -> AudioRoster {
    // Retained handles are the only source of ownership. Cheap metadata
    // sampling neither enumerates PIDs nor authorizes any new process.
    std::array::from_fn(|index| sessions.running[index].as_ref()
        .map(|running| (running.children[0].pid, running.children[0].created)))
}
// All non-owned foreground windows imply the same mute policy.
fn owned_audio_foreground(foreground: Option<u32>, roster: &AudioRoster) -> Option<u32> {
    foreground.filter(|pid| roster.iter().flatten().any(|(owned, _)| owned == pid))
}

#[derive(Default)]
pub(super) struct AudioIdleGate {
    last_success: Option<Instant>,
    foreground: Option<Option<u32>>,
    roster: Option<AudioRoster>,
}
impl AudioIdleGate {
    pub(super) fn due(&self, now: Instant, foreground: Option<u32>, roster: AudioRoster) -> bool {
        self.last_success.is_none_or(|last|
            now.saturating_duration_since(last) >= AUDIO_IDLE_RESYNC)
            || self.foreground != Some(owned_audio_foreground(foreground, &roster))
            || self.roster != Some(roster)
    }
    pub(super) fn success(&mut self, now: Instant, foreground: Option<u32>, roster: AudioRoster) {
        self.last_success = Some(now);
        self.foreground = Some(owned_audio_foreground(foreground, &roster));
        self.roster = Some(roster);
    }
    pub(super) fn invalidate(&mut self) {
        // A failed COM sync is retried on the next existing idle tick, not
        // suppressed by a previously successful cadence checkpoint.
        self.last_success = None;
    }
    pub(super) fn current_success(&self, roster: AudioRoster) -> bool {
        self.last_success.is_some() && self.roster == Some(roster)
    }
    pub(super) fn wait_budget(&self, now: Instant, roster: AudioRoster, focus_ready: bool,
        discovery: Option<Duration>) -> Duration {
        let short=Duration::from_millis(250);
        let Some(last)=self.last_success.filter(|_| self.roster==Some(roster)) else {
            // A missing/failed sync or a changed roster must not turn an
            // overdue discovery into an immediate repeated COM retry.
            return short;
        };
        let remaining=AUDIO_IDLE_RESYNC.saturating_sub(now.saturating_duration_since(last));
        let remaining=discovery.map_or(remaining, |budget| remaining.min(budget));
        if focus_ready {remaining} else {remaining.min(short)}
    }
}
pub(super) fn audio_maintenance_needed(
    mode: crate::engine::manager_settings::BackgroundAudio,
    audio_present: bool, roster: AudioRoster,
) -> bool {
    use crate::engine::manager_settings::BackgroundAudio;
    audio_present || mode == BackgroundAudio::Mute
        && roster.iter().any(Option::is_some)
}
pub(super) fn sync_audio(sessions:&Sessions,audio:&mut Option<session_audio::Audio>,
    mode:crate::engine::manager_settings::BackgroundAudio, foreground:Option<u32>)->io::Result<()> {
    use crate::engine::manager_settings::BackgroundAudio;
    let mut targets=[(0u32,0u64);5];
    let mut count=0;
    if mode!=BackgroundAudio::Keep {
        for identity in sessions.running.iter().flatten().filter_map(|s|s.children[0].identity().ok()) {
            targets[count]=identity;
            count+=1;
        }
    }
    let targets=&targets[..count];
    if targets.is_empty() {
        if let Some(current) = audio.as_mut() { current.restore()?; }
        *audio = None;
        return Ok(());
    }
    if audio.is_none(){*audio=Some(session_audio::Audio::new()?);}
    audio.as_mut().unwrap().sync(targets, foreground, |pid, born| {
        // Cheap retained metadata only narrows the candidate set; the
        // matching child still needs fresh native identity/liveness proof.
        sessions.running.iter().flatten().any(|started| {
            let child = &started.children[0];
            child.pid == pid && child.created == born
                && child.identity().ok() == Some((pid, born))
        })
    })
}
