#[derive(Debug)]
pub struct Process {
    pub(crate) pid: u32,
    pub(crate) command: String,
    pub(crate) state: String,
    pub(crate) vsize: u64,
    pub(crate) rss: u64,
}
