use sysinfo::System;

pub trait DataMonitor {
    fn new() -> Self;
    fn fetch(&mut self, src: &System);
}
