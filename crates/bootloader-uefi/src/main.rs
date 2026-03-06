#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

#[cfg(target_os = "uefi")]
use sadas_boot_protocol::BootInfo;

#[cfg(target_os = "uefi")]
use core::panic::PanicInfo;

#[cfg(target_os = "uefi")]
const EFI_SUCCESS: usize = 0;
#[cfg(target_os = "uefi")]
const EFI_LOAD_ERROR: usize = 1;

#[cfg(target_os = "uefi")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_log("[sadas][ERROR] panic in UEFI bootloader\n");
    loop {
        halt_cpu();
    }
}

#[cfg(target_os = "uefi")]
#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: usize, _system_table: usize) -> usize {
    serial_log("[sadas][INFO] sadas UEFI bootloader started\n");

    let kernel = match load_from_esp("kernel.elf") {
        Ok(blob) => blob,
        Err(msg) => {
            serial_log(msg);
            return EFI_LOAD_ERROR;
        }
    };

    let initfs = match load_from_esp("initfs.cpio") {
        Ok(blob) => blob,
        Err(msg) => {
            serial_log(msg);
            return EFI_LOAD_ERROR;
        }
    };

    let mut boot_info = BootInfo::empty();
    boot_info.initfs.address = initfs.0;
    boot_info.initfs.size = initfs.1;

    if !collect_uefi_boot_info(&mut boot_info) {
        serial_log("[sadas][ERROR] failed to collect UEFI memory/GOP/ACPI info\n");
        return EFI_LOAD_ERROR;
    }

    if !exit_boot_services() {
        serial_log("[sadas][ERROR] exit boot services failed\n");
        return EFI_LOAD_ERROR;
    }

    serial_log("[sadas][INFO] jumping to kernel entry\n");
    jump_to_kernel(kernel.0, &boot_info)
}

#[cfg(target_os = "uefi")]
fn load_from_esp(_path: &str) -> Result<(u64, u64), &'static str> {
    // Placeholder for real SimpleFileSystem protocol implementation.
    Err("[sadas][ERROR] ESP file loading not wired yet (expected kernel.elf/initfs.cpio)\n")
}

#[cfg(target_os = "uefi")]
fn collect_uefi_boot_info(_info: &mut BootInfo) -> bool {
    // Placeholder for real UEFI memory map + GOP + ACPI retrieval.
    false
}

#[cfg(target_os = "uefi")]
fn exit_boot_services() -> bool {
    // Placeholder for real ExitBootServices call with fresh memory map key.
    false
}

#[cfg(target_os = "uefi")]
fn jump_to_kernel(_kernel_addr: u64, _boot_info: &BootInfo) -> usize {
    serial_log("[sadas][ERROR] kernel jump not wired yet\n");
    EFI_LOAD_ERROR
}

#[cfg(target_os = "uefi")]
fn serial_log(msg: &str) {
    for &byte in msg.as_bytes() {
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") 0x3F8u16,
                in("al") byte,
                options(nostack, nomem, preserves_flags)
            );
        }
    }
}

#[cfg(target_os = "uefi")]
fn halt_cpu() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(not(target_os = "uefi"))]
fn main() {
    // Host-side build sanity binary for workspace checks.
    println!("sadas-bootloader-uefi host stub: build for x86_64-unknown-uefi to produce .efi");
}
