const BUILTINS: &[&str] = &["help", "ls", "cat", "echo", "reboot", "shutdown", "ps"];

fn main() {
    let _ = sadas_userland::write_console(b"sadas> ");
    let mut help = String::from("builtins: ");
    for (i, cmd) in BUILTINS.iter().enumerate() {
        help.push_str(cmd);
        if i + 1 != BUILTINS.len() {
            help.push_str(", ");
        }
    }
    help.push('\n');
    let _ = sadas_userland::write_console(help.as_bytes());
}
