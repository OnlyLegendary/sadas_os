use sadas_kernel::PHASE1_BOOT_LINES;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunnerMode {
    BuildOnly,
    BuildAndRun,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BootFlavor {
    Legacy,
    Uefi,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RunnerConfig {
    mode: RunnerMode,
    flavor: BootFlavor,
}

fn main() {
    if let Err(err) = run_main() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args(env::args().skip(1))?;
    let repo_root = find_repo_root()?;

    match config.flavor {
        BootFlavor::Legacy => {
            let image_path = repo_root.join("target/sadas_boot.img");
            write_legacy_boot_sector(&image_path)?;
            if config.mode == RunnerMode::BuildOnly {
                println!("Built legacy boot image: {}", image_path.display());
                return Ok(());
            }

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
        BootFlavor::Uefi => {
            let esp_dir = build_uefi_esp_layout(&repo_root)?;
            if config.mode == RunnerMode::BuildOnly {
                println!("Built UEFI ESP layout: {}", esp_dir.display());
                return Ok(());
            }

            let qemu_binary = resolve_qemu_binary()?;
            let (ovmf_code, ovmf_vars) = resolve_ovmf_paths()?;

            run(
                Command::new(&qemu_binary)
                    .arg("-machine")
                    .arg("q35")
                    .arg("-m")
                    .arg("2048")
                    .arg("-drive")
                    .arg(format!(
                        "if=pflash,format=raw,readonly=on,file={}",
                        ovmf_code.display()
                    ))
                    .arg("-drive")
                    .arg(format!("if=pflash,format=raw,file={}", ovmf_vars.display()))
                    .arg("-drive")
                    .arg(format!("format=raw,file=fat:rw:{}", esp_dir.display()))
                    .arg("-serial")
                    .arg("stdio")
                    .arg("-no-reboot")
                    .arg("-no-shutdown"),
                &format!("failed to launch {} in UEFI mode", qemu_binary.display()),
            )
        }
    }
}

fn parse_args<I>(args: I) -> Result<RunnerConfig, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = String>,
{
    let mut mode = RunnerMode::BuildAndRun;
    let mut flavor = BootFlavor::Legacy;

    for arg in args {
        match arg.as_str() {
            "--build-only" => mode = RunnerMode::BuildOnly,
            "--run" => mode = RunnerMode::BuildAndRun,
            "--uefi" => flavor = BootFlavor::Uefi,
            "--legacy" => flavor = BootFlavor::Legacy,
            "-h" | "--help" => {
                println!(
                    "Usage: cargo run -p sadas-qemu-runner -- [--build-only|--run] [--legacy|--uefi]"
                );
                println!("  --build-only  generate boot artifact without launching QEMU");
                println!("  --run         generate artifact and run QEMU (default)");
                println!("  --legacy      use raw boot-sector image flow (default)");
                println!("  --uefi        build UEFI ESP layout and boot with OVMF");
                return Ok(RunnerConfig { mode, flavor });
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument: {arg}"),
                )
                .into())
            }
        }
    }

    Ok(RunnerConfig { mode, flavor })
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

fn write_legacy_boot_sector(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut sector = [0u8; 512];

    let code: [u8; 29] = [
        0x31, 0xC0, 0x8E, 0xD8, 0xEB, 0x02, 0x90, 0x90, 0xBE, 0x1D, 0x7C, 0xAC, 0x84, 0xC0, 0x74,
        0x09, 0xB4, 0x0E, 0xBB, 0x07, 0x00, 0xCD, 0x10, 0xEB, 0xF2, 0xFA, 0xF4, 0xEB, 0xFE,
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
        fs::create_dir_all(parent)?;
    }
    fs::write(path, sector)?;
    Ok(())
}

fn build_uefi_esp_layout(repo_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let esp_root = repo_root.join("target/esp");
    let efi_boot = esp_root.join("EFI/BOOT");
    fs::create_dir_all(&efi_boot)?;

    let bootloader = build_uefi_bootloader(repo_root)?;
    let efi_target = efi_boot.join("BOOTX64.EFI");
    fs::copy(&bootloader, &efi_target).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!(
                "failed to copy bootloader {} -> {}: {err}",
                bootloader.display(),
                efi_target.display()
            ),
        )
    })?;

    let kernel_path = esp_root.join("kernel.elf");
    let initfs_path = esp_root.join("initfs.cpio");
    fs::write(&kernel_path, b"SADAS_KERNEL_ELF_PLACEHOLDER\n")?;
    fs::write(&initfs_path, b"SADAS_INITFS_PLACEHOLDER\n")?;
    fs::write(
        esp_root.join("cmdline.txt"),
        b"console=serial loglevel=info root=/dev/sda2\n",
    )?;

    Ok(esp_root)
}

fn build_uefi_bootloader(repo_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let status = Command::new("cargo")
        .current_dir(repo_root)
        .arg("build")
        .arg("-p")
        .arg("sadas-bootloader-uefi")
        .arg("--target")
        .arg("x86_64-unknown-uefi")
        .status()?;

    if !status.success() {
        return Err(io::Error::other(
            "failed to build UEFI bootloader (try: rustup target add x86_64-unknown-uefi)",
        )
        .into());
    }

    Ok(repo_root.join("target/x86_64-unknown-uefi/debug/sadas-bootloader-uefi.efi"))
}

fn resolve_ovmf_paths() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let code_candidates = [
        env::var_os("OVMF_CODE").map(PathBuf::from),
        Some(PathBuf::from("/usr/share/OVMF/OVMF_CODE.fd")),
        Some(PathBuf::from("/usr/share/edk2-ovmf/x64/OVMF_CODE.fd")),
    ];
    let vars_candidates = [
        env::var_os("OVMF_VARS").map(PathBuf::from),
        Some(PathBuf::from("/usr/share/OVMF/OVMF_VARS.fd")),
        Some(PathBuf::from("/usr/share/edk2-ovmf/x64/OVMF_VARS.fd")),
    ];

    let code = code_candidates
        .into_iter()
        .flatten()
        .find(|p| p.exists())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OVMF_CODE.fd not found"))?;

    let vars_template = vars_candidates
        .into_iter()
        .flatten()
        .find(|p| p.exists())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OVMF_VARS.fd not found"))?;

    let vars_runtime = PathBuf::from("target/OVMF_VARS.fd");
    if let Some(parent) = vars_runtime.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&vars_template, &vars_runtime)?;

    Ok((code, vars_runtime))
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

    #[test]
    fn parse_args_supports_uefi_build_only() {
        let cfg =
            parse_args(vec!["--build-only".to_string(), "--uefi".to_string()]).expect("valid args");
        assert_eq!(cfg.mode, RunnerMode::BuildOnly);
        assert_eq!(cfg.flavor, BootFlavor::Uefi);
    }
}
