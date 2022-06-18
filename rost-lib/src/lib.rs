#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features)]
#![feature(generic_const_exprs, core_intrinsics, alloc_error_handler)]

use core::arch::asm;

use crate::syscall_name::SysCallName;
extern crate alloc;

pub mod syscall_name;
pub mod thread_utils;

pub fn __initialize_syscalls() {
    use kernel::syscalls::system_call::register_syscall;
    register_syscall(
        SysCallName::ThreadExit as u16,
        thread_utils::handler_thread_exit,
    );
    register_syscall(
        SysCallName::ThreadYield as u16,
        thread_utils::handler_thread_yield,
    );
    register_syscall(
        SysCallName::ThreadSleep as u16,
        thread_utils::handler_thread_sleep,
    );
}

#[inline(always)]
pub(crate) fn syscall(name: SysCallName, arg1: u64, arg2: u64) -> u64 {
    unsafe {
        let result: u64;
        asm!(
            "push r10; push r11; push rcx",
            "syscall",
            "pop rcx; pop r11; pop r10",
            in("rdi")(name as u64),
            in("rsi")(arg1),
            in("rdx")(arg2),
            out("rax")(result)
        );
        result
    }
}
