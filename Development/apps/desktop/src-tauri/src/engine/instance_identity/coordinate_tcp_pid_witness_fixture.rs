use super::ProcessKey;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6};

const MAX_ROWS: usize = 16_384;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Tuple { local: SocketAddr, remote: SocketAddr }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State { Established, Other }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Row { tuple: Tuple, state: State, pid: u32 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResultKind {
    InvalidQuery, Overflow, Missing, Ambiguous, Disconnected,
    DifferentOwnerPid, PidCandidateOnlyUnverified,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContinuityKind { ProcessBirthChanged, SocketChanged, PidOnlyStillUnverified }

// Typed endpoints are already host-order. A future Win32 adapter must
// convert dwLocalPort and dwRemotePort from network byte order for BOTH
// MIB_TCPROW_OWNER_PID and MIB_TCP6ROW_OWNER_PID before comparison.
fn valid(tuple: Tuple) -> bool {
    tuple.local.port() != 0 && tuple.remote.port() != 0
        && tuple.local.ip().is_loopback() && tuple.remote.ip().is_loopback()
        && matches!(
            (tuple.local.ip(), tuple.remote.ip()),
            (IpAddr::V4(_), IpAddr::V4(_)) | (IpAddr::V6(_), IpAddr::V6(_))
        )
}

fn classify(tuple: Tuple, pid: u32, rows: &[Row]) -> ResultKind {
    if pid == 0 || !valid(tuple) { return ResultKind::InvalidQuery; }
    if rows.len() > MAX_ROWS { return ResultKind::Overflow; }
    let mut matches = rows.iter().filter(|r| r.tuple == tuple);
    let Some(found) = matches.next() else { return ResultKind::Missing; };
    if matches.next().is_some() { return ResultKind::Ambiguous; }
    if found.state != State::Established { return ResultKind::Disconnected; }
    if found.pid != pid || found.pid == 0 { return ResultKind::DifferentOwnerPid; }
    ResultKind::PidCandidateOnlyUnverified
}

fn birth(key: &ProcessKey) -> Option<u64> {
    if key.pid == 0 { return None; }
    key.created_file_time.parse::<u64>().ok().filter(|v| *v > 0)
}

// Synthetic ProcessKey values here are NOT independently OS-attested
// handles. A real original-controller caller must pin/recheck both.
fn compare_two(
    expected: &ProcessKey, previous: &ProcessKey, latest: &ProcessKey,
    tuple: Tuple, old_rows: &[Row], new_rows: &[Row],
) -> ContinuityKind {
    let Some(expected_birth) = birth(expected) else { return ContinuityKind::ProcessBirthChanged; };
    if previous.pid != expected.pid || latest.pid != expected.pid
        || birth(previous) != Some(expected_birth) || birth(latest) != Some(expected_birth) {
        return ContinuityKind::ProcessBirthChanged;
    }
    if classify(tuple, expected.pid, old_rows) != ResultKind::PidCandidateOnlyUnverified
        || classify(tuple, expected.pid, new_rows) != ResultKind::PidCandidateOnlyUnverified {
        return ContinuityKind::SocketChanged;
    }
    ContinuityKind::PidOnlyStillUnverified
}

fn v4() -> Tuple {
    Tuple {
        local: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 50123),
        remote: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 19132),
    }
}
fn v6() -> Tuple {
    Tuple {
        local: SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 50123, 0, 0)),
        remote: SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 19132, 0, 0)),
    }
}
fn row(tuple: Tuple, pid: u32) -> Row {
    Row { tuple, pid, state: State::Established }
}
fn key(pid: u32, birth: u64) -> ProcessKey {
    ProcessKey { pid, created_file_time: birth.to_string() }
}

#[test]
fn ipv4_ipv6_matching_pid_never_implies_game_or_world_identity() {
    for tuple in [v4(), v6()] {
        assert_eq!(classify(tuple, 222, &[row(tuple, 222)]),
            ResultKind::PidCandidateOnlyUnverified);
    }
    assert_eq!(classify(v4(), 222, &[row(v6(), 222)]), ResultKind::Missing);
}

#[test]
fn query_scope_and_exact_four_tuple_refuse_external_swapped_and_mixed_families() {
    let q=v4();
    let remote=Tuple {remote: SocketAddr::new(Ipv4Addr::new(192,0,2,4).into(),19132),..q};
    let mixed=Tuple {remote:v6().remote,..q};
    let wildcard=Tuple {local:SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(),50123),..q};
    let zero=Tuple {local:SocketAddr::new(Ipv4Addr::LOCALHOST.into(),0),..q};
    for invalid in [remote,mixed,wildcard,zero] {
        assert_eq!(classify(invalid,222,&[row(invalid,222)]),ResultKind::InvalidQuery);
    }
    assert_eq!(classify(q,0,&[row(q,0)]),ResultKind::InvalidQuery);
    for other in [
        Tuple {remote:SocketAddr::new(Ipv4Addr::LOCALHOST.into(),19133),..q},
        Tuple {local:q.remote,remote:q.local},
        Tuple {local:SocketAddr::new(Ipv4Addr::LOCALHOST.into(),0xcbc3),..q},
    ] {
        assert_eq!(classify(q,222,&[row(other,222)]),ResultKind::Missing);
    }
}

#[test]
fn duplicates_inactive_helper_pid_and_excess_rows_are_never_authorized() {
    let q=v4();
    assert_eq!(classify(q,222,&[]),ResultKind::Missing);
    assert_eq!(classify(q,222,&[row(q,222),row(q,222)]),ResultKind::Ambiguous);
    assert_eq!(classify(q,222,&[row(q,333)]),ResultKind::DifferentOwnerPid);
    let inactive=Row{state:State::Other,..row(q,222)};
    assert_eq!(classify(q,222,&[inactive]),ResultKind::Disconnected);
    assert_eq!(classify(q,222,&vec![row(q,222);MAX_ROWS+1]),ResultKind::Overflow);
    let other=Tuple{local:SocketAddr::new(Ipv4Addr::LOCALHOST.into(),50124),..q};
    assert_eq!(classify(q,222,&[row(other,333),row(q,222)]),
        ResultKind::PidCandidateOnlyUnverified);
}

#[test]
fn pid_reuse_birth_change_and_port_reuse_invalidate_two_snapshots() {
    let q=v4();
    let original=key(222,134000000000000222);
    let same=key(222,134000000000000222);
    let recycled=key(222,134000000000000223);
    assert_eq!(compare_two(&original,&same,&same,q,&[row(q,222)],&[row(q,222)]),
        ContinuityKind::PidOnlyStillUnverified);
    assert_eq!(compare_two(&original,&same,&recycled,q,&[row(q,222)],&[row(q,222)]),
        ContinuityKind::ProcessBirthChanged);
    assert_eq!(compare_two(&original,&same,&same,q,&[row(q,222)],&[row(q,333)]),
        ContinuityKind::SocketChanged);
    assert_eq!(compare_two(&original,&same,&same,q,&[row(q,222)],&[]),
        ContinuityKind::SocketChanged);
    assert_eq!(compare_two(&key(0,0),&same,&same,q,&[row(q,222)],&[row(q,222)]),
        ContinuityKind::ProcessBirthChanged);
}
