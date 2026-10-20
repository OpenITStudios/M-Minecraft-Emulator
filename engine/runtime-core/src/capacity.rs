use crate::client::ClientId;
use sysinfo::System;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HostCapacity {
    pub logical_cpus: usize,
    pub total_memory_gb: f64,
    pub recommended_virtual_clients: usize,
}

pub(crate) fn current_host_capacity() -> HostCapacity {
    // Capacity needs only CPU topology and installed memory, not all processes.
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_all();
    let logical_cpus = system.cpus().len();
    let total_memory_gb = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    HostCapacity {
        logical_cpus,
        total_memory_gb,
        recommended_virtual_clients: recommended_virtual_clients(total_memory_gb, logical_cpus),
    }
}

pub(crate) fn recommended_virtual_clients(total_memory_gb: f64, logical_cpus: usize) -> usize {
    recommended_by_memory(total_memory_gb)
        .min(recommended_by_cpu(logical_cpus))
        .min(ClientId::VIRTUAL.len())
}

fn recommended_by_memory(total_gb: f64) -> usize {
    if total_gb >= 24.0 {
        3
    } else if total_gb >= 16.0 {
        2
    } else if total_gb >= 12.0 {
        1
    } else {
        0
    }
}

fn recommended_by_cpu(logical_cpus: usize) -> usize {
    if logical_cpus >= 8 {
        3
    } else if logical_cpus >= 6 {
        2
    } else if logical_cpus >= 4 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::recommended_virtual_clients;

    #[test]
    fn recommendation_is_bounded_by_memory_and_cpu() {
        assert_eq!(recommended_virtual_clients(64.0, 16), 3);
        assert_eq!(recommended_virtual_clients(16.0, 16), 2);
        assert_eq!(recommended_virtual_clients(64.0, 6), 2);
        assert_eq!(recommended_virtual_clients(12.0, 4), 1);
        assert_eq!(recommended_virtual_clients(8.0, 16), 0);
        assert_eq!(recommended_virtual_clients(64.0, 2), 0);
    }
}
