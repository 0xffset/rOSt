use core::cell::RefCell;

use alloc::{rc::Rc, sync::Arc, vec::Vec};
use bootloader::BootInfo;
use internal_utils::serial_println;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    interrupts,
    memory::{self, frame_allocator::BitmapFrameAllocator},
    processes::thread::Thread,
    syscalls::system_call::{register_syscall, setup_syscalls},
};

use internal_utils::structures::{
    driver::{Driver, Registrator},
    kernel_information::KernelInformation,
};

use crate::debug;

lazy_static! {
    static ref REGISTERED_DRIVERS: Mutex<Vec<Registrator>> = Mutex::new(Vec::new());
    static ref INITIALIZED_DRIVERS: Mutex<Vec<Driver>> = Mutex::new(Vec::new());
}

pub(crate) static mut KERNEL_INFORMATION: Option<KernelInformation> = None;

extern "C" fn test_syscall(a: u64, b: u64, caller: Rc<RefCell<Thread>>) -> u64 {
    let thread = caller.borrow();
    serial_println!(
        "Syscall 0 from process {} and thread {}",
        thread.process.as_ref().borrow().id,
        thread.id
    );
    0
}

extern "C" fn test_syscall2(a: u64, b: u64, caller: Rc<RefCell<Thread>>) -> u64 {
    let thread = caller.borrow();
    serial_println!(
        "Syscall 1 from process {} and thread {}",
        thread.process.as_ref().borrow().id,
        thread.id
    );
    1
}

/// Initialises the components of the OS, **must** be called before any other functions.
pub fn init(boot_info: &'static BootInfo) -> KernelInformation {
    debug::print_memory_map(&boot_info.memory_regions);
    memory::save_kernel_memory();
    let mut allocator = BitmapFrameAllocator::init(boot_info);
    memory::init(boot_info, &mut allocator);
    let kernel_info = KernelInformation::new(boot_info, Arc::new(Mutex::new(allocator)));
    interrupts::reload_gdt();
    interrupts::init_idt();
    setup_syscalls();
    interrupts::enable();

    register_syscall(0, test_syscall);
    register_syscall(1, test_syscall2);

    unsafe {
        KERNEL_INFORMATION = Some(kernel_info.clone());
    }

    kernel_info
}

pub fn get_kernel_information() -> KernelInformation {
    unsafe { KERNEL_INFORMATION.clone().unwrap() }
}

/// Reinitializes all the registered drivers
pub fn reload_drivers() {
    let kernel_info = get_kernel_information();
    let mut initialized_drivers = INITIALIZED_DRIVERS.lock();
    initialized_drivers.clear();
    initialized_drivers.extend(
        REGISTERED_DRIVERS
            .lock()
            .iter()
            .map(|registrator| registrator(kernel_info.clone())),
    );
}

/// Registers a driver. After registering drivers call reload_drivers to initialize them.
pub fn register_driver(registrator: Registrator) {
    REGISTERED_DRIVERS.lock().push(registrator);
}

/// Endless loop calling halt continuously.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
