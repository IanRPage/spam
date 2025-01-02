use super::process::Process;
use libc::{self, _SC_PAGESIZE};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write as _,
    fs,
    io::{self, Write as _},
};

#[derive(Debug)]
pub struct System {
    cpu_usage: f64,
    mem_total: u32,
    mem_free: u32,
    swap_total: u32,
    swap_free: u32,
    proc_map: HashMap<u32, Process>,
    page_size: i64,
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
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
            page_size: unsafe { libc::sysconf(_SC_PAGESIZE) },
        }
    }

    pub fn update_sys(&mut self) {
        self.refresh_cpu_usage();
        self.refresh_memory();
        self.refresh_proc_list();
    }

    fn refresh_cpu_usage(&mut self) {
        let stat = fs::read_to_string("/proc/stat").unwrap();
        let cpu_line = stat
            .lines()
            .find(|line| line.starts_with("cpu"))
            .expect("Unable to find cpu line");
        let cpu: Vec<u32> = cpu_line
            .split_whitespace()
            .skip(1)
            .map(|val| val.parse().unwrap_or(0))
            .collect();
        let (active, total) = cpu
            .iter()
            .enumerate()
            .fold((0, 0), |(active, total), (i, &val)| {
                let total = total + val;
                let active = if i != 3 { active + val } else { active };
                (active, total)
            });
        let usage = (active as f64 / total as f64) * 100.0;
        self.cpu_usage = format!("{usage:.3}").parse().unwrap_or(0.0);
    }

    fn refresh_memory(&mut self) {
        let meminfo = fs::read_to_string("/proc/meminfo").unwrap();
        for line in meminfo.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let [data, val, ..] = parts.as_slice() {
                match *data {
                    "MemTotal:" => self.mem_total = val.parse::<u32>().unwrap(),
                    "MemFree:" => self.mem_free = val.parse::<u32>().unwrap(),
                    "SwapTotal:" => self.swap_total = val.parse::<u32>().unwrap(),
                    "SwapFree:" => self.swap_free = val.parse::<u32>().unwrap(),
                    _ => continue,
                }
            }
        }
    }

    fn refresh_proc_list(&mut self) {
        let mut pids = HashSet::new();
        for entry in fs::read_dir("/proc").unwrap() {
            let entry = entry.unwrap();
            if let Some(fname) = entry.file_name().to_str() {
                if let Ok(pid) = fname.parse::<u32>() {
                    pids.insert(pid);
                    let path = entry.path();
                    if let Ok(proc_stats) = fs::read_to_string(path.join("stat")) {
                        let parts: Vec<&str> = proc_stats.trim_end().split(" ").collect();
                        let command = parts[1][1..parts[1].len() - 1].to_string();
                        let state = parts[2].to_string();
                        let vsize = parts[22].parse::<u64>().unwrap() / 1024;
                        let rss = parts[23].parse::<i64>().unwrap() * self.page_size / 1024;
                        self.proc_map
                            .entry(pid)
                            .and_modify(|proc| {
                                proc.command = command.clone();
                                proc.state = state.clone();
                                proc.vsize = vsize;
                                proc.rss = rss;
                            }).or_insert(Process { pid, command, state, vsize, rss });
                    }
                }
            }
        }
        self.proc_map.retain(|&pid, _| pids.contains(&pid));
    }

    pub fn display(&self) {
        let mut buf = String::new();

        buf.write_str("\x1B[H\x1B[J").unwrap();

        writeln!(buf, "CPU%: {}%", self.cpu_usage).unwrap();
        writeln!(
            buf,
            "Mem Total: {} KiB\t\tMem Free: {} KiB",
            self.mem_total, self.mem_free
        )
        .unwrap();
        writeln!(
            buf,
            "Swap Total: {} KiB\t\tSwap Free: {} KiB",
            self.swap_total, self.swap_free
        )
        .unwrap();
        writeln!(
            buf,
            "{:<10} {:<7} {:<15} {:<15} {:<15}",
            "PID", "STATE", "VSIZE", "RSS", "COMMAND"
        )
        .unwrap();
        for proc in self.proc_map.values() {
            writeln!(
                buf,
                "{:<10} {:<7} {:<15} {:<15} {:<15}",
                proc.pid, proc.state, proc.vsize, proc.rss, proc.command
            )
            .unwrap();
        }
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write!(handle, "{}", buf).unwrap();
        handle.flush().unwrap();
    }
}
