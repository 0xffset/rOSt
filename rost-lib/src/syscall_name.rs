#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SysCallName {
    ThreadExit = 300,
    ThreadYield = 301,
    ThreadSleep = 302,
}
