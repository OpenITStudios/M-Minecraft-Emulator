//! Native frame deadline arithmetic. No Windows handles, waits or policy producer.
const HUNDRED_NS_PER_SECOND: i64 = 10_000_000;
const MAX_FREQUENCY: i64 = i64::MAX / 20_000_000;
const INVALID_DEADLINE: i64 = 1;

fn valid_ceilings(active: u32, background: u32) -> bool {
    (1..=1000).contains(&active) && (1..=active).contains(&background)
}

fn valid_frequency(frequency: i64) -> bool {
    (1..=MAX_FREQUENCY).contains(&frequency)
}

// Zero = release now; negative = relative Windows timer due in 100 ns units;
// positive = invalid input. No catch-up: the next deadline starts at release.
fn frame_deadline(frequency: i64, previous: i64, now: i64, ceiling: u32) -> i64 {
    if !valid_frequency(frequency) || !valid_ceilings(ceiling, 1)
        || previous < 0 || now < previous {
        return INVALID_DEADLINE;
    }
    let interval = (frequency + i64::from(ceiling) - 1) / i64::from(ceiling);
    let elapsed = now - previous;
    if previous == 0 || elapsed >= interval {
        return 0;
    }
    let remaining = interval - elapsed;
    -(remaining * HUNDRED_NS_PER_SECOND + frequency - 1) / frequency
}

#[no_mangle]
pub extern "C" fn lazytester_frame_pacing_valid_ceilings(active: u32, background: u32) -> u32 {
    u32::from(valid_ceilings(active, background))
}

#[no_mangle]
pub extern "C" fn lazytester_frame_pacing_valid_frequency(frequency: i64) -> u32 {
    u32::from(valid_frequency(frequency))
}

#[no_mangle]
pub extern "C" fn lazytester_frame_pacing_deadline(frequency: i64, previous: i64, now: i64, ceiling: u32) -> i64 {
    frame_deadline(frequency, previous, now, ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_frame_and_no_catch_up() {
        assert_eq!(frame_deadline(10_000_000, 0, 1, 144), 0);
        assert_eq!(frame_deadline(10_000_000, 1, 10_000_001, 144), 0);
        assert_eq!(frame_deadline(10_000_000, 10_000_001, 10_000_002, 144), -69_444);
    }

    #[test]
    fn promotion_and_demotion_recompute_from_last_release() {
        assert_eq!(frame_deadline(10_000_000, 1, 100_001, 144), 0);
        assert_eq!(frame_deadline(10_000_000, 1, 100_001, 10), -900_000);
        assert_eq!(frame_deadline(3, 1, 1, 2), -6_666_667);
    }

    #[test]
    fn invalid_limits_and_clock_are_refused() {
        for (active, background) in [(0, 1), (1001, 1), (60, 0), (60, 61)] {
            assert!(!valid_ceilings(active, background));
        }
        assert!(valid_ceilings(60, 5));
        assert_eq!(frame_deadline(0, 1, 2, 60), INVALID_DEADLINE);
        assert_eq!(frame_deadline(MAX_FREQUENCY + 1, 1, 2, 60), INVALID_DEADLINE);
        assert_eq!(frame_deadline(1000, 2, 1, 60), INVALID_DEADLINE);
        assert_eq!(frame_deadline(1000, 1, 2, 0), INVALID_DEADLINE);
        assert!(frame_deadline(MAX_FREQUENCY, 1, 1, 1) < 0);
    }
}
