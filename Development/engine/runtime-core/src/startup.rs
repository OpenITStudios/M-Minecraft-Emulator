use crate::resources::{current_host_pressure, HostPressure, StartDecision};
use std::{io, thread, time::Duration};

const RECHECK_INTERVAL: Duration = Duration::from_secs(1);
const MAX_WAIT_CHECKS: usize = 8;

pub(crate) fn wait_until_start_is_safe() -> io::Result<HostPressure> {
    let mut last = current_host_pressure();
    for _ in 0..MAX_WAIT_CHECKS {
        match last.start_decision {
            StartDecision::StartNow => return Ok(last),
            StartDecision::Block => {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    format!(
                        "host memory is too constrained to start another virtual client ({} MB available)",
                        last.available_memory_mb
                    ),
                ))
            }
            StartDecision::Wait => {
                thread::sleep(RECHECK_INTERVAL);
                last = current_host_pressure();
            }
        }
    }

    if last.start_decision == StartDecision::Block {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "host resources became critically constrained while waiting to start another virtual client",
        ));
    }

    // CPU pressure can remain high because an already-running VM is still
    // settling. After the bounded wait budget, memory safety remains the hard
    // gate and startup may proceed; target-machine acceptance will calibrate
    // this budget rather than turning it into an unbounded scheduler.
    Ok(last)
}

#[cfg(test)]
mod tests {
    use super::{MAX_WAIT_CHECKS, RECHECK_INTERVAL};

    #[test]
    fn adaptive_start_wait_is_bounded() {
        assert_eq!(MAX_WAIT_CHECKS, 8);
        assert_eq!(RECHECK_INTERVAL.as_secs(), 1);
    }
}
