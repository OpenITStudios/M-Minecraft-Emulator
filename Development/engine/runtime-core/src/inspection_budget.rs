//! Platform-independent cooperative time budget for read-only inspection.
//! Copies share one monotonic start time. OS operations are not preempted:
//! callers must mark incomplete results partial rather than claiming a hard
//! wall-clock latency guarantee or treating a partial scan as deletion consent.
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct InspectionBudget {
    started: Instant,
    limit: Duration,
}

impl InspectionBudget {
    pub fn new(limit: Duration) -> Self {
        Self { started: Instant::now(), limit }
    }

    pub fn exhausted(self) -> bool {
        self.started.elapsed() >= self.limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_keep_the_same_deadline_without_restarting_an_inspection() {
        let exhausted = InspectionBudget::new(Duration::ZERO);
        let same = exhausted;
        assert!(exhausted.exhausted());
        assert!(same.exhausted());

        let old = InspectionBudget {
            started: Instant::now() - Duration::from_secs(5),
            limit: Duration::from_secs(2),
        };
        assert!(old.exhausted());
    }
}
