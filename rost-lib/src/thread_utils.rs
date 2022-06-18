use core::cell::RefCell;

use alloc::rc::Rc;
use kernel::processes::dispatcher::exit_thread;
use kernel::processes::run_next_thread;
use kernel::processes::thread::{Thread, ThreadState};

use crate::syscall_name::SysCallName;

// TODO Add handling for process/thread exit code
// We should probably have thread exit code handlers for non-zero exit codes, and pass them as the process exit code.
pub(crate) extern "C" fn handler_thread_exit(
    _code: u64,
    _: u64,
    caller: Rc<RefCell<Thread>>,
) -> u64 {
    exit_thread(caller).unwrap();
    run_next_thread();
    panic!("No threads to run");
}

pub(crate) extern "C" fn handler_thread_yield(_: u64, _: u64, _caller: Rc<RefCell<Thread>>) -> u64 {
    run_next_thread();
    panic!("No threads to run");
}

pub(crate) extern "C" fn handler_thread_sleep(
    time: u64,
    _: u64,
    caller: Rc<RefCell<Thread>>,
) -> u64 {
    Thread::change_state(caller, ThreadState::Sleeping(time));
    run_next_thread();
    panic!("No threads to run");
}

pub extern "C" fn thread_exit(status: u64) -> ! {
    crate::syscall(SysCallName::ThreadExit, status, 0);
    panic!("Thread exited");
}

pub extern "C" fn thread_yield() {
    crate::syscall(SysCallName::ThreadYield, 0, 0);
}

pub extern "C" fn thread_sleep(time: u64) {
    crate::syscall(SysCallName::ThreadSleep, time, 0);
}
