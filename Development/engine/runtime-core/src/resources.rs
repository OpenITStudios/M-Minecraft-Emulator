use serde::Serialize;
use sysinfo::System;

pub const VIRTUAL_MEMORY_LIMIT_MB: u64 = 4096;

// Provisional admission thresholds. These protect against obvious host
// contention; target-machine acceptance must calibrate them before they are
// described as performance-optimal.
const MEMORY_CRITICAL_PERCENT: u8 = 15;
const MEMORY_PRESSURE_PERCENT: u8 = 25;
const MIN_START_AVAILABLE_MEMORY_MB: u64 = 2048;
const CPU_WAIT_PERCENT: u8 = 80;
const CPU_SATURATED_PERCENT: u8 = 95;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StartDecision {
    StartNow,
    Wait,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostPressure {
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub available_percent: u8,
    pub cpu_usage_percent: u8,
    pub start_decision: StartDecision,
}

pub fn evaluate_pressure(
    total_memory_mb: u64,
    available_memory_mb: u64,
    cpu_usage_percent: u8,
) -> HostPressure {
    let available_percent = if total_memory_mb == 0 {
        0
    } else {
        ((available_memory_mb.saturating_mul(100)) / total_memory_mb).min(100) as u8
    };
    let cpu_usage_percent = cpu_usage_percent.min(100);

    let memory_critical = available_percent < MEMORY_CRITICAL_PERCENT
        || available_memory_mb < MIN_START_AVAILABLE_MEMORY_MB;
    let memory_pressure = available_percent < MEMORY_PRESSURE_PERCENT;
    let cpu_critical = cpu_usage_percent >= CPU_SATURATED_PERCENT;
    let cpu_pressure = cpu_usage_percent >= CPU_WAIT_PERCENT;

    let start_decision = if memory_critical {
        StartDecision::Block
    } else if memory_pressure || cpu_pressure {
        StartDecision::Wait
    } else {
        StartDecision::StartNow
    };

    HostPressure {
        total_memory_mb,
        available_memory_mb,
        available_percent,
        cpu_usage_percent,
        start_decision,
    }
}

pub fn current_host_pressure() -> HostPressure {
    // Snapshot pressure needs RAM and sampled CPU, not the host process table.
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_cpu_all();
    let cpu = system.global_cpu_usage().round().clamp(0.0, 100.0) as u8;
    evaluate_pressure(
        system.total_memory() / 1024 / 1024,
        system.available_memory() / 1024 / 1024,
        cpu,
    )
}

#[cfg(test)]
mod tests {
    use super::{evaluate_pressure, StartDecision};

    #[test]
    fn healthy_host_can_start_immediately() {
        let report = evaluate_pressure(16 * 1024, 6 * 1024, 35);
        assert_eq!(report.start_decision, StartDecision::StartNow);
    }

    #[test]
    fn memory_pressure_waits_instead_of_starting_into_contention() {
        let report = evaluate_pressure(16 * 1024, 3 * 1024, 35);
        assert_eq!(report.start_decision, StartDecision::Wait);
    }

    #[test]
    fn cpu_pressure_waits_without_reclassifying_memory() {
        let report = evaluate_pressure(16 * 1024, 8 * 1024, 85);
        assert_eq!(report.start_decision, StartDecision::Wait);
    }

    #[test]
    fn critical_memory_blocks_new_virtual() {
        let report = evaluate_pressure(16 * 1024, 1536, 20);
        assert_eq!(report.start_decision, StartDecision::Block);
    }

    #[test]
    fn cpu_saturation_delays_but_does_not_kill_existing_clients() {
        let report = evaluate_pressure(16 * 1024, 8 * 1024, 97);
        assert_eq!(report.start_decision, StartDecision::Wait);
    }
}
