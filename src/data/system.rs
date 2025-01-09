use super::monitor::DataMonitor;
use sysinfo::System;

pub struct SystemMonitor {
    pub cpu_usage: f64,
    pub mem_total: u64,
    pub mem_free: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

impl DataMonitor for SystemMonitor {
    fn new() -> Self {
        Self {
            cpu_usage: 0.0,
            mem_total: 0,
            mem_free: 0,
            swap_total: 0,
            swap_free: 0,
        }
    }

    fn fetch(&mut self, src: &System) {
        self.cpu_usage = 0.0;
        self.mem_total = src.total_memory();
        self.mem_free = src.free_memory();
        self.swap_total = src.total_swap();
        self.swap_free = src.free_swap();
    }
}
