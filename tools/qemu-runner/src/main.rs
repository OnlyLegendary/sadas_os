use sadas_kernel::PHASE1_BOOT_LINES;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if let Err(err) = run_main() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = find_repo_root()?;
    let image_path = repo_root.join("target/sadas_boot.img");
    write_boot_sector(&image_path)?;

    let qemu_binary = resolve_qemu_binary()?;

    run(
        Command::new(&qemu_binary)
            .arg("-drive")
            .arg(format!("format=raw,file={}", image_path.display()))
            .arg("-nographic")
            .arg("-no-reboot")
            .arg("-no-shutdown"),
        &format!("failed to launch {}", qemu_binary.display()),
    )
}

fn find_repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "failed to determine repo root from CARGO_MANIFEST_DIR",
            )
            .into()
        })
}

fn write_boot_sector(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut message = String::new();
    for line in PHASE1_BOOT_LINES {
        message.push_str(line);
        message.push('\n');
    }

    let mut msg = message.into_bytes();
    msg.push(0);

    sector[..code.len()].copy_from_slice(&code);
    sector[code.len()..code.len() + msg.len()].copy_from_slice(&msg);
    sector[510] = 0x55;
    sector[511] = 0xAA;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!(
                    "failed to create target directory {}: {err}",
                    parent.display()
                ),
            )
        })?;
    }
    fs::write(path, sector).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("failed to write boot image {}: {err}", path.display()),
        )
    })?;
    Ok(())
}

fn resolve_qemu_binary() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut checked_paths: Vec<String> = Vec::new();

    if let Some(path) = env::var_os("QEMU_SYSTEM_X86_64") {
        let candidate = PathBuf::from(path);
        checked_paths.push(format!("QEMU_SYSTEM_X86_64={}", candidate.display()));
        if candidate.exists() {
            return Ok(candidate);
        }
    } else {
        checked_paths.push("QEMU_SYSTEM_X86_64 (not set)".to_string());
    }

    let binary_name = default_qemu_binary_name();
    checked_paths.push(format!("PATH lookup: {binary_name}"));
    if let Some(path) = lookup_in_path(binary_name)? {
        return Ok(path);
    }

    if cfg!(windows) {
        for candidate in windows_common_qemu_paths() {
            checked_paths.push(candidate.display().to_string());
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "QEMU not found. Install QEMU and add it to PATH, or set QEMU_SYSTEM_X86_64 to the full executable path. Checked: {}",
            checked_paths.join("; ")
        ),
    )
    .into())
}

fn default_qemu_binary_name() -> &'static str {
    if cfg!(windows) {
        "qemu-system-x86_64.exe"
    } else {
        "qemu-system-x86_64"
    }
}

fn lookup_in_path(binary_name: &str) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let tool = if cfg!(windows) { "where" } else { "which" };

    let output = match Command::new(tool).arg(binary_name).output() {
        Ok(output) => output,
        Err(_) => return Ok(None),
    };

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    let first = stdout.lines().find(|line| !line.trim().is_empty());
    Ok(first.map(|line| PathBuf::from(line.trim())))
}

fn windows_common_qemu_paths() -> Vec<PathBuf> {
    [
        r"C:\Program Files\qemu\qemu-system-x86_64.exe",
        r"C:\Program Files\QEMU\qemu-system-x86_64.exe",
        r"C:\Program Files (x86)\qemu\qemu-system-x86_64.exe",
        r"C:\Program Files (x86)\QEMU\qemu-system-x86_64.exe",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

fn run(cmd: &mut Command, fail_msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    let status = cmd
        .status()
        .map_err(|err| io::Error::new(err.kind(), format!("{fail_msg}: {err}")))?;
    if !status.success() {
        return Err(io::Error::other(format!("{fail_msg}: exit status {status}")).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_path_probe_order_is_stable() {
        let actual: Vec<std::ffi::OsString> = windows_common_qemu_paths()
            .into_iter()
            .map(|p| p.into_os_string())
            .collect();
        let expected: Vec<std::ffi::OsString> = vec![
            std::ffi::OsString::from(r"C:\Program Files\qemu\qemu-system-x86_64.exe"),
            std::ffi::OsString::from(r"C:\Program Files\QEMU\qemu-system-x86_64.exe"),
            std::ffi::OsString::from(r"C:\Program Files (x86)\qemu\qemu-system-x86_64.exe"),
            std::ffi::OsString::from(r"C:\Program Files (x86)\QEMU\qemu-system-x86_64.exe"),
        ];

        assert_eq!(actual, expected);
    }
}
