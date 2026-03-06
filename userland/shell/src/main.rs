use std::path::Path;

const BUILTINS: &[&str] = &[
    "help", "ls", "cat", "echo", "write", "reboot", "shutdown", "ps",
];

fn print_line(line: &str) {
    let _ = sadas_userland::write_console(line.as_bytes());
}

fn main() {
    print_line("sadas> booting shell\n");

    let root_path = Path::new("system/state/rootfs.img");
    let mut fs = match sadas_fs::open_default_root(root_path) {
        Ok(fs) => fs,
        Err(err) => {
            print_line(&format!("storage error: {:?}\n", err));
            return;
        }
    };

    if fs.list_dir().map(|files| files.is_empty()).unwrap_or(true) {
        let _ = fs.write_file("welcome.txt", b"Welcome to Sadas shell\n");
    }

    let _ = fs.write_file("last_boot.txt", b"shell started\n");

    let mut help = String::from("builtins: ");
    for (i, cmd) in BUILTINS.iter().enumerate() {
        help.push_str(cmd);
        if i + 1 != BUILTINS.len() {
            help.push_str(", ");
        }
    }
    help.push('\n');
    print_line(&help);

    match fs.list_dir() {
        Ok(files) => print_line(&format!("ls / -> {}\n", files.join(" "))),
        Err(err) => print_line(&format!("ls error: {:?}\n", err)),
    }

    match fs.read_file("welcome.txt") {
        Ok(content) => {
            let rendered = String::from_utf8_lossy(&content);
            print_line(&format!("cat /welcome.txt -> {}", rendered));
        }
        Err(err) => print_line(&format!("cat error: {:?}\n", err)),
    }

    print_line("echo persistence-ready\n");
    print_line("ps\n  1 init\n  2 shell\n");
}
