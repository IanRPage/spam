use super::process::Process;
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
};

#[derive(Debug)]
pub struct System {
    cpu_usage: f64,
    mem_total: u32,
    mem_free: u32,
    swap_total: u32,
    swap_free: u32,
    proc_map: HashMap<u32, Process>,
}

impl System {
    pub fn new() -> System {
        System {
            cpu_usage: 0.0,
            mem_total: 0,
            mem_free: 0,
            swap_total: 0,
            swap_free: 0,
            proc_map: HashMap::new(),
        }
    }

    pub fn update_sys(&mut self) {
        // update cpu usage
        let stat = fs::read_to_string("/proc/stat").unwrap();
        let stat: Vec<String> = stat
            .lines()
            .map(|line| {
                if line.starts_with("cpu") {
                    line.split_whitespace()
                        .skip(1)
                        .collect::<Vec<&str>>()
                        .join(" ")
                } else {
                    line.to_string()
                }
            })
            .collect();
        let cpu = &stat[0];
        let cpu: Vec<u32> = cpu.split(" ").map(|n| n.parse::<u32>().unwrap()).collect();
        let active = &cpu[0..3].iter().sum::<u32>() + &cpu[4..].iter().sum::<u32>();
        let total: u32 = cpu.iter().sum();
        let usage = (active as f64 / total as f64) * 100.0;
        self.cpu_usage = format!("{:.3}", usage).parse::<f64>().unwrap();

        // update memory
        let meminfo = fs::read_to_string("/proc/meminfo").unwrap();
        for line in meminfo.lines() {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            match parts[0] {
                "MemTotal:" => self.mem_total = parts[1].parse::<u32>().unwrap(),
                "MemFree:" => self.mem_free = parts[1].parse::<u32>().unwrap(),
                "SwapTotal:" => self.swap_total = parts[1].parse::<u32>().unwrap(),
                "SwapFree:" => self.swap_free = parts[1].parse::<u32>().unwrap(),
                _ => continue,
            }
        }

        // update proc list
        let entries = if let Ok(contents) = fs::read_dir("/proc") {
            contents
        } else {
            return;
        };
        let mut pids: Vec<u32> = Vec::new();

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(fname) = path.file_name().and_then(|name| name.to_str()) {
                    if let Ok(pid) = fname.parse::<u32>() {
                        pids.push(pid);
                        if let Ok(proc_stats) = fs::read_to_string(format!("/proc/{}/stat", pid)) {
                            let parts: Vec<&str> = proc_stats.trim_end().split(" ").collect();
                            let command = parts[1][1..parts[1].len() - 1].to_string();
                            let state = parts[2].to_string();
                            let vsize = parts[22].parse::<u64>().unwrap();
                            let rss = parts[23].parse::<u64>().unwrap();
                            self.proc_map
                                .entry(pid)
                                .and_modify(|proc| {
                                    proc.pid = pid;
                                    proc.command = command.clone();
                                    proc.state = state.clone();
                                    proc.vsize = vsize;
                                    proc.rss = rss;
                                })
                                .or_insert(Process {
                                    pid,
                                    command,
                                    state,
                                    vsize,
                                    rss,
                                });
                        }
                    }
                }
            }
        }
        self.proc_map.retain(|&pid, _| pids.contains(&pid));
    }

    // pub fn display(&self) {
    //     println!("CPU Usage: {:.2}%", self.cpu_usage);
    //     println!("Memory Total: {} kB", self.mem_total);
    //     println!("Memory Free: {} kB", self.mem_free);
    //     println!("Swap Total: {} kB", self.swap_total);
    //     println!("Swap Free: {} kB", self.swap_free);
    //     println!("PID\tCOMMAND\tSTATE\tVSIZE\tRSS");
    //     for (_, proc) in &self.proc_map {
    //         println!(
    //             "{}\t{}\t{}\t{}\t{}",
    //             proc.pid, proc.command, proc.state, proc.vsize, proc.rss
    //         );
    //     }
    // }
    pub fn display(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "CPU%: {}%", self.cpu_usage).unwrap();
        writeln!(handle, "Mem Total: {} kB\tMem Free: {} kB", self.mem_total, self.mem_free).unwrap();
        writeln!(handle, "Swap Total: {} kB\tSwap Free: {} kB", self.swap_total, self.swap_free).unwrap();
        writeln!(handle, "{:<10} {:<7} {:<15} {:<15} {:<15}", "PID", "STATE", "VSIZE", "RSS", "COMMAND").unwrap();
        for (_, proc) in &self.proc_map {
            writeln!(
                handle,
                "{:<10} {:<7} {:<15} {:<15} {:<15}",
                proc.pid, proc.state, proc.vsize, proc.rss, proc.command
            )
            .unwrap();
        }
    }
}
