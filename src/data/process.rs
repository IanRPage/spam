use super::monitor::DataMonitor;
use std::collections::{HashMap, HashSet};
use sysinfo::System;

pub struct ProcessMonitor {
    pub proc_map: HashMap<u32, (String, String, u64, u64)>,
    pub n_procs: u64,
}

impl DataMonitor for ProcessMonitor {
    fn new() -> Self {
        Self {
            proc_map: HashMap::new(),
            n_procs: 0,
        }
    }

    fn fetch(&mut self, src: &System) {
        let mut pids = HashSet::new();
        for (pid, process) in src.processes() {
            let pid = pid.as_u32();
            pids.insert(pid);
            let command = process.name().to_str().unwrap().to_string();
            let state = process.status().to_string();
            let vsize = process.virtual_memory();
            let rss = process.memory();
            self.proc_map
                .entry(pid)
                .and_modify(|proc| {
                    proc.0 = command.clone();
                    proc.1 = state.clone();
                    proc.2 = vsize;
                    proc.3 = rss;
                })
                .or_insert((command, state, vsize, rss));
        }
        self.proc_map.retain(|pid, _| pids.contains(pid));
        self.n_procs = self.proc_map.len() as u64;
    }
}
