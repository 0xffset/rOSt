use bootloader::boot_info::{MemoryRegionKind, MemoryRegions};
use internal_utils::{serial_println, FullFrameAllocator};

use crate::memory::frame_allocator::BitmapFrameAllocator;

#[inline(always)]
pub fn log(msg: &str) {
    #[cfg(debug_assertions)]
    serial_println!("[debug] {}", msg);
}

#[inline(always)]
pub fn print_frame_memory(allocator: &BitmapFrameAllocator) {
    #[cfg(debug_assertions)]
    {
        serial_println!("[   ---{:^15}---   ]", "FRAME ALLOCATOR");
        {
            let mut size = allocator.get_total_memory_size();
            let mut size_format = "B";
            if size >= 2 * 1024 {
                if size < 2 * 1024 * 1024 {
                    size /= 1024;
                    size_format = "KiB";
                } else if size < 2 * 1024 * 1024 * 1024 {
                    size /= 1024 * 1024;
                    size_format = "MiB";
                } else {
                    size /= 1024 * 1024 * 1024;
                    size_format = "GiB";
                }
            }
            serial_println!("[debug] Total memory: {:>4}{:>3}", size, size_format);
        }
        {
            let mut size = allocator.get_free_memory_size();
            let mut size_format = "B";
            if size >= 2 * 1024 {
                if size < 2 * 1024 * 1024 {
                    size /= 1024;
                    size_format = "KiB";
                } else if size < 2 * 1024 * 1024 * 1024 {
                    size /= 1024 * 1024;
                    size_format = "MiB";
                } else {
                    size /= 1024 * 1024 * 1024;
                    size_format = "GiB";
                }
            }
            serial_println!("[debug] Free memory: {:>4}{:>3}", size, size_format);
        }
    }
}

#[inline(always)]
pub fn print_memory_map(memory_map: &MemoryRegions) {
    #[cfg(debug_assertions)]
    {
        serial_println!("[   ---{:^15}---   ]", "MEMORY MAP");
        memory_map.iter().for_each(|region| {
            let mut size = region.end - region.start;
            let mut size_format = "B";
            if size >= 2 * 1024 {
                if size < 2 * 1024 * 1024 {
                    size /= 1024;
                    size_format = "KiB";
                } else if size < 2 * 1024 * 1024 * 1024 {
                    size /= 1024 * 1024;
                    size_format = "MiB";
                } else {
                    size /= 1024 * 1024 * 1024;
                    size_format = "GiB";
                }
            }
            serial_println!(
                "{:14}- {:>4}{:>3}  ({:X}) ({:X})",
                decode_memory_kind(region.kind),
                size,
                size_format,
                region.start,
                region.end
            );
        });
    }
}

#[cfg(debug_assertions)]
fn decode_memory_kind(kind: MemoryRegionKind) -> &'static str {
    match kind {
        MemoryRegionKind::Usable => "usable",
        MemoryRegionKind::Bootloader => "bootloader",
        MemoryRegionKind::UnknownBios(kind) => match kind {
            1 => "usable BIOS",
            2 => "Reserved BIOS",
            3 => "ACPI reclaimable",
            4 => "ACPI NVS",
            5 => "Bad memory",
            _ => "unknown BIOS",
        },
        MemoryRegionKind::UnknownUefi(kind) => match kind {
            0 => "EfiReservedMemoryType",
            1 => "EfiLoaderCode",
            2 => "EfiLoaderData",
            3 => "EfiBootServicesCode",
            4 => "EfiBootServicesData",
            5 => "EfiRuntimeServiceCode",
            6 => "EfiRuntimeServicesData",
            7 => "EfiConventionalMemory",
            8 => "EfiUnusableMemory",
            9 => "EfiACPIReclaimMemory",
            10 => "EfiACPIMemoryNVS",
            11 => "EfiMemoryMappedIO",
            12 => "EfiMemoryMappedIOPort Space",
            13 => "EfiPalCode",
            14 => "EfiPersistentMemory",
            _ => "unknown UEFI",
        },
        _ => "unknown",
    }
}
