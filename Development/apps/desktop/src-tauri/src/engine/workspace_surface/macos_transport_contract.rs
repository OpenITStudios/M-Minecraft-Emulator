//! Synthetic review contract for the distinct-macOS-user transport candidate.
//! Test-only: no IPC, native observation, allocator, capture, input, or owner.
//! The existing retained controller must eventually supply these facts;
//! neither a transport peer nor this equality check authenticates them.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Target {
    slot: u32,
    uid: u32,
    login_session: u64,
    pid: u32,
    process_birth: u64,
    window: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Route {
    target: Target,
    connection_generation: u64,
}

impl Route {
    fn valid(self) -> bool {
        let t = self.target;
        (1..=5).contains(&t.slot) && t.uid != 0 && t.login_session != 0
            && t.pid != 0 && t.process_birth != 0 && t.window != 0
            && self.connection_generation != 0
    }
}

// Synthetic observation facts, not a proposed public wire schema. These
// booleans must come from native verification, not from the caller or peer.
#[derive(Clone, Copy)]
struct Observation {
    route: Route,
    authenticated_peer: bool,
    distinct_virtual_session: bool,
    exact_game_window: bool,
    focused_game: bool,
}

fn route_current(expected: Route, observed: Option<Observation>) -> bool {
    expected.valid() && observed.is_some_and(|o| {
        o.route == expected && o.authenticated_peer
            && o.distinct_virtual_session && o.exact_game_window
    })
}

// One controller-owned slot binding per OS user in this candidate only.
// This is not a decision to reject future supported same-user isolation.
fn distinct_targets(a: Target, b: Target) -> bool {
    a.slot != b.slot && a.uid != b.uid && a.login_session != b.login_session
        && a.pid != b.pid
}

// Frame metadata only: never allocate from a peer-declared byte count.
// Limits here are owner-supplied synthetic budgets, not a production preset.
// A per-slot latest-frame mailbox would replace, not queue, an accepted
// frame. The eventual owner must invalidate it on route loss/replacement.
#[derive(Clone, Copy)]
struct Frame {
    route: Route,
    sequence: u64,
    bytes: usize,
}

fn frame_admitted(
    expected: Route,
    observed: Option<Observation>,
    frame: Frame,
    last_sequence: u64,
    fresh: bool,
    byte_limit: usize,
) -> bool {
    route_current(expected, observed) && frame.route == expected && fresh
        && frame.sequence > last_sequence && frame.bytes > 0
        && frame.bytes <= byte_limit
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Writer {
    route: Route,
    epoch: u64,
}

// Unknown delivery is not a successful release; disconnect alone never
// proves that keys/buttons were neutralized. The existing controller owns
// any later reconciliation, not a timer or a transport reconnect.
#[derive(Clone, Copy)]
enum PriorWriter {
    None,
    Released(Writer),
    Unsettled(Writer),
}

fn switch_admitted(
    previous: Option<Writer>,
    outcome: PriorWriter,
    next: Writer,
    observed: Option<Observation>,
) -> bool {
    if !route_current(next.route, observed) || next.epoch == 0 {
        return false;
    }
    match (previous, outcome) {
        (None, PriorWriter::None) => true,
        (Some(old), PriorWriter::Released(released)) => {
            old == released && next.epoch > old.epoch
        }
        _ => false,
    }
}

fn input_admitted(
    selected: Route,
    retained_writer: Option<Writer>,
    request_writer: Writer,
    observed: Option<Observation>,
    delivery_settled: bool,
    fresh: bool,
) -> bool {
    delivery_settled && fresh && request_writer.epoch != 0
        && retained_writer == Some(request_writer) && request_writer.route == selected
        && route_current(selected, observed) && observed.is_some_and(|o| o.focused_game)
}

fn route(slot: u32) -> Route {
    Route {
        target: Target {
            slot, uid: 500 + slot, login_session: u64::from(slot) + 10,
            pid: 100 + slot, process_birth: u64::from(slot) + 20,
            window: 200 + slot,
        },
        connection_generation: 1,
    }
}

fn observation(route: Route) -> Observation {
    Observation {
        route, authenticated_peer: true, distinct_virtual_session: true,
        exact_game_window: true, focused_game: true,
    }
}

#[test]
fn connection_is_not_identity_or_independent_desktop_proof() {
    let a = route(1);
    let good = observation(a);
    assert!(route_current(a, Some(good)));
    assert!(!route_current(a, None));
    assert!(!route_current(a, Some(Observation { authenticated_peer: false, ..good })));
    assert!(!route_current(a, Some(Observation { distinct_virtual_session: false, ..good })));
    assert!(!route_current(a, Some(Observation { exact_game_window: false, ..good })));
    assert!(!route_current(a, Some(observation(route(2)))));
    let mut changed = a;
    changed.target.uid += 1;
    assert!(!route_current(a, Some(observation(changed))));
    changed = a;
    changed.target.login_session += 1;
    assert!(!route_current(a, Some(observation(changed))));
    changed = a;
    changed.target.process_birth += 1;
    assert!(!route_current(a, Some(observation(changed))));
    changed = a;
    changed.target.window += 1;
    assert!(!route_current(a, Some(observation(changed))));
    assert!(!route_current(route(6), Some(observation(route(6)))));
}

#[test]
fn two_slots_cannot_share_the_same_user_or_login_context() {
    let a = route(1).target;
    let b = route(2).target;
    assert!(distinct_targets(a, b));
    assert!(!distinct_targets(a, Target { uid: a.uid, ..b }));
    assert!(!distinct_targets(a, Target { login_session: a.login_session, ..b }));
    assert!(!distinct_targets(a, Target { pid: a.pid, ..b }));
    assert!(!distinct_targets(a, Target { slot: a.slot, ..b }));
}

#[test]
fn stale_wrong_route_or_oversize_frame_never_enters_latest_slot() {
    let a = route(1);
    let good = observation(a);
    let frame = Frame { route: a, sequence: 9, bytes: 64 };
    assert!(frame_admitted(a, Some(good), frame, 8, true, 64));
    assert!(!frame_admitted(a, Some(good), frame, 9, true, 64));
    assert!(!frame_admitted(a, Some(good), frame, 8, false, 64));
    assert!(!frame_admitted(a, Some(good), Frame { bytes: 65, ..frame }, 8, true, 64));
    assert!(!frame_admitted(a, Some(good), Frame { bytes: 0, ..frame }, 8, true, 64));
    assert!(!frame_admitted(a, Some(good), Frame { route: route(2), ..frame }, 8, true, 64));
    let replacement = Route { connection_generation: 2, ..a };
    assert!(!frame_admitted(replacement, Some(observation(replacement)), frame, 0, true, 64));
    assert!(!frame_admitted(a, None, frame, 8, true, 64));
}

#[test]
fn switch_requires_exact_neutral_release_not_disconnect_or_stale_receipt() {
    let a = Writer { route: route(1), epoch: 1 };
    let b = Writer { route: route(2), epoch: 2 };
    let unsettled = PriorWriter::Unsettled(a);
    assert!(matches!(unsettled, PriorWriter::Unsettled(w) if w == a));
    assert!(switch_admitted(None, PriorWriter::None, a, Some(observation(a.route))));
    assert!(!switch_admitted(Some(a), PriorWriter::None, b, Some(observation(b.route))));
    assert!(!switch_admitted(Some(a), unsettled, b, Some(observation(b.route))));
    assert!(!switch_admitted(Some(a), PriorWriter::Released(Writer { epoch: 3, ..a }), b, Some(observation(b.route))));
    assert!(switch_admitted(Some(a), PriorWriter::Released(a), b, Some(observation(b.route))));
    assert!(!switch_admitted(Some(a), PriorWriter::Released(a), Writer { epoch: 1, ..b }, Some(observation(b.route))));
    assert!(!switch_admitted(Some(a), PriorWriter::Released(a), b, None));
}

#[test]
fn input_is_fenced_to_selected_writer_fresh_focus_and_connection() {
    let a = Writer { route: route(1), epoch: 1 };
    let b = Writer { route: route(2), epoch: 2 };
    let good = observation(a.route);
    assert!(input_admitted(a.route, Some(a), a, Some(good), true, true));
    assert!(!input_admitted(b.route, Some(a), a, Some(good), true, true));
    assert!(!input_admitted(a.route, Some(a), b, Some(good), true, true));
    assert!(!input_admitted(a.route, None, a, Some(good), true, true));
    assert!(!input_admitted(a.route, Some(a), a, Some(good), false, true));
    assert!(!input_admitted(a.route, Some(a), a, Some(good), true, false));
    assert!(!input_admitted(a.route, Some(a), a, Some(Observation { focused_game: false, ..good }), true, true));
    let replacement = Route { connection_generation: 2, ..a.route };
    assert!(!input_admitted(replacement, Some(a), a, Some(observation(replacement)), true, true));
}
