use spam::data::{DataMonitor, ProcessMonitor, SystemMonitor};
use std::{
    fmt::Write as _,
    io::{self, Write as _},
    thread,
};
use sysinfo::{self, System};

fn main() {
    let mut sys = SystemMonitor::new();
    let mut procs = ProcessMonitor::new();
    let src_sys = System::new_all();
    loop {
        thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.fetch(&src_sys);
        procs.fetch(&src_sys);
        display(&sys, &procs);
    }
}

fn display(sys_monitor: &SystemMonitor, proc_monitor: &ProcessMonitor) {
    let mut buf = String::new();

    buf.write_str("\x1B[H\x1B[J").unwrap();

    writeln!(buf, "CPU%: {}%", sys_monitor.cpu_usage).unwrap();
    writeln!(buf, "# Processes: {}", proc_monitor.n_procs).unwrap();
    writeln!(
        buf,
        "Mem Total: {} KiB\t\tMem Free: {} KiB",
        sys_monitor.mem_total, sys_monitor.mem_free
    )
    .unwrap();
    writeln!(
        buf,
        "Swap Total: {} KiB\t\tSwap Free: {} KiB",
        sys_monitor.swap_total, sys_monitor.swap_free
    )
    .unwrap();
    writeln!(
        buf,
        "{:<10} {:<15} {:<15} {:<15} {:<15}",
        "PID", "STATE", "VSIZE", "RSS", "COMMAND"
    )
    .unwrap();
    for (pid, data) in &proc_monitor.proc_map {
        writeln!(
            buf,
            "{:<10} {:<15?} {:<15} {:<15} {:<15?}",
            pid, data.1, data.2, data.3, data.0,
        )
        .unwrap();
    }
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write!(handle, "{}", buf).unwrap();
    handle.flush().unwrap();
}
