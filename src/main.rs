#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features)]
#![feature(
    custom_test_frameworks,
    abi_x86_interrupt,
    generic_const_exprs,
    core_intrinsics,
    alloc_error_handler
)]
#![test_runner(test_framework::test_runner::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;

use alloc::rc::Rc;
use bootloader::{entry_point, BootInfo};
use core::arch::asm;
use core::cell::RefCell;
use core::panic::PanicInfo;
use internal_utils::structures::kernel_information::KernelInformation;
use internal_utils::{constants::MIB, serial_println};
use rost_lib::syscall_name::SysCallName;
use tinytga::RawTga;
use vga::vga_core::{Clearable, ImageDrawable};

use core::alloc::Layout;

entry_point!(kernel);
pub fn kernel(boot_info: &'static mut BootInfo) -> ! {
    let kernel_info = kernel::init(boot_info);
    bootup_sequence(kernel_info.clone());

    #[cfg(test)]
    kernel_test(kernel_info);
    #[cfg(not(test))]
    kernel_main(kernel_info);

    kernel::hlt_loop();
}

fn bootup_sequence(kernel_info: KernelInformation) {
    rost_lib::__initialize_syscalls();
    kernel::register_driver(vga::driver_init);
    kernel::register_driver(ata::driver_init);
    kernel::reload_drivers();
    let data = include_bytes!("./assets/rost-logo.tga");
    let logo = RawTga::from_slice(data).unwrap();
    let logo_header = logo.header();
    let mut vga_device = vga::vga_device::VGADeviceFactory::from_kernel_info(kernel_info);
    vga_device.clear(vga::vga_color::BLACK);
    vga_device.draw_image(
        (vga_device.width as u16 - logo_header.width) / 2,
        (vga_device.height as u16 - logo_header.height) / 2,
        &logo,
    );
}

#[no_mangle]
extern "C" fn user_mode_check_1() {
    exit(0);
}

#[no_mangle]
extern "C" fn user_mode_check_2() {
    let mut i = 1;
    loop {
        i += 1;
        if i > 100_000_000 {
            break;
        }
    }
    exit(0);
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

fn exit(status: u64) -> ! {
    crate::syscall(SysCallName::ThreadExit, status, 0);
    panic!("Thread exited");
}

fn sleep(time: u64) {
    crate::syscall(SysCallName::ThreadSleep, time, 0);
}

pub fn kernel_main(kernel_info: KernelInformation) {
    use kernel::processes::{
        add_process,
        process::Process,
        run_processes,
        thread::{Thread, ThreadState},
    };
    let process1: Rc<RefCell<Process>>;
    let thread1: Rc<RefCell<Thread>>;
    unsafe {
        process1 = add_process(Process::from_extern(user_mode_check_1, 1));
        thread1 = Thread::new_native(0x1000, 2 * MIB, process1);
    }
    Thread::change_state(thread1, ThreadState::Ready);

    //let process2 = add_process(Process::new(user_mode_check_2, 2));
    //let _thread2 = Thread::new(0x1000, 2 * MIB, process2);

    run_processes();
    serial_println!("Something went wrong");
    /*
        let test = Box::new(4);
        log_println!("New boxed value: {:#?}", test);
        log_println!("im not dying :)");
    */
    /*
        log_println!("Getting all disks...");
        let disks = ata::get_all_disks();
        log_println!("Got {} disks, taking the non-bootable one...", disks.len());
        let mut disk = disks
            .into_iter()
            .map(|mut disk| (disk.has_bootloader(), disk))
            .find(|(boot, _)| !boot.unwrap_or(true))
            .expect("No non-bootable disk found")
            .1;
        log_println!("Got a disk, looking for partitions...");
        let mut partitions = disk.get_partitions().expect("Error getting partitions");
        if partitions.len() == 0 {
            log_println!("No partitions found, creating a new one...");
            let partition_size = disk.descriptor.lba_48_addressable_sectors as u32 / 2;
            disk.create_partition(partition_size, 0xED)
                .expect("Error creating partition");
            log_println!("Partition created, double-checking...");
            partitions = disk.get_partitions().expect("Error getting partitions");
            if partitions.len() == 0 {
                log_println!("No partitions found, giving up.");
                return;
            }
        }
        log_println!("Found {} partitions:", partitions.len());
        for partition in partitions {
            log_println!(
                "{:8} - starting at {:8X}",
                format_size(partition.descriptor.sectors * 512),
                partition.descriptor.start_lba
            )
        }
    */
}

/// This is the main function for tests.
#[cfg(test)]
pub fn kernel_test(_kernel_info: KernelInformation) {
    use test_framework::test_runner::KERNEL_INFO;

    unsafe { KERNEL_INFO = Some(_kernel_info) };
    test_main();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[cfg(not(test))]
#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    use test_framework::ansi_colors;

    serial_println!("{}", ansi_colors::Red("[PANIC]"));
    serial_println!("Error: {}\n", info);
    kernel::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    use test_framework::{
        ansi_colors,
        qemu_exit::{exit_qemu, QemuExitCode},
    };

    serial_println!("{}", ansi_colors::Red("[PANIC]"));
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);

    kernel::hlt_loop();
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use internal_utils::structures::kernel_information::KernelInformation;
    use x86_64::structures::paging::{Size2MiB, Size4KiB};

    #[test_case]
    fn should_allocate_frame(kernel_information: KernelInformation) {
        use x86_64::structures::paging::PhysFrame;
        let mut allocator = kernel_information.allocator.lock();
        let size = allocator.get_free_memory_size();
        let frame: Option<PhysFrame<Size4KiB>> = allocator.allocate_frame();
        assert!(frame.is_some());
        assert_eq!(4096, size - allocator.get_free_memory_size());
    }

    #[test_case]
    fn should_allocate_big_frame(kernel_information: KernelInformation) {
        use x86_64::structures::paging::PhysFrame;
        let mut allocator = kernel_information.allocator.lock();
        let size = allocator.get_free_memory_size();
        let frame: Option<PhysFrame<Size2MiB>> = allocator.allocate_frame();
        assert!(frame.is_some());
        assert_eq!(2 * 1024 * 1024, size - allocator.get_free_memory_size());
    }

    #[test_case]
    fn should_allocate_small_box(_: KernelInformation) {
        let boxed = Box::new(4);
        assert_eq!(4, *boxed);
    }

    #[test_case]
    fn should_allocate_large_box(_: KernelInformation) {
        let boxed = Box::new([13u8; 256]);
        assert_eq!(boxed.len(), 256);
        for i in 0..256 {
            assert_eq!(boxed[i], 13);
        }
    }
}
