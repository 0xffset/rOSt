// This might be reimplemented from scratch in the future.

// TODO: Implement all remaining interrupt handlers for CPU interrupts
// We need to implement all interrupt handlers and add basic handling to them so we don't double fault.
// Better handling for each of them will be added later.

mod cpu_handlers;
mod interrupt_register;
pub use interrupt_register::init_idt;
pub(crate) mod gdt;
mod pic_handlers;
pub use gdt::{reload_gdt, GDT};
mod pic;

use crate::debug;

/// Initializes the PICs and enables interrupts
pub fn enable() {
    unsafe {
        // can cause undefined behaviour if the offsets were not set correctly
        pic::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
    debug::log("Interrupts enabled");
}
