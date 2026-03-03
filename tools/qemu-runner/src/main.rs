use sadas_kernel::KMAIN_MESSAGE;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tools dir")
        .parent()
        .expect("repo root")
        .to_path_buf();

    let image_path = repo_root.join("target/sadas_boot.img");
    write_boot_sector(&image_path);

    run(
        Command::new("qemu-system-x86_64")
            .arg("-drive")
            .arg(format!("format=raw,file={}", image_path.display()))
            .arg("-nographic")
            .arg("-no-reboot")
            .arg("-no-shutdown"),
        "failed to launch qemu-system-x86_64",
    );
}

fn write_boot_sector(path: &PathBuf) {
    let mut sector = [0u8; 512];

    // Stage-0 loader performs minimal setup and jumps to a `kmain` routine.
    let code: [u8; 29] = [
        0x31, 0xC0, // xor ax, ax
        0x8E, 0xD8, // mov ds, ax
        0xEB, 0x02, // jmp short kmain
        0x90, // nop
        0x90, // nop
        // kmain:
        0xBE, 0x1D, 0x7C, // mov si, 0x7c1d (message)
        0xAC, // lodsb
        0x84, 0xC0, // test al, al
        0x74, 0x09, // jz hang
        0xB4, 0x0E, // mov ah, 0x0e
        0xBB, 0x07, 0x00, // mov bx, 0x0007
        0xCD, 0x10, // int 0x10
        0xEB, 0xF2, // jmp kmain_loop
        // hang:
        0xFA, // cli
        0xF4, // hlt
        0xEB, 0xFE, // jmp $
    ];

    let mut msg = KMAIN_MESSAGE.as_bytes().to_vec();
    msg.push(0);

    sector[..code.len()].copy_from_slice(&code);
    sector[code.len()..code.len() + msg.len()].copy_from_slice(&msg);
    sector[510] = 0x55;
    sector[511] = 0xAA;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create target directory");
    }
    fs::write(path, sector).expect("failed to write boot image");
}

fn run(cmd: &mut Command, fail_msg: &str) {
    let status = cmd.status().expect(fail_msg);
    if !status.success() {
        panic!("{fail_msg}: exit status {status}");
    }
}
