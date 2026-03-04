use std::env;

fn usage() {
    println!("Sadas Settings – Privacy");
    println!("  sadas-settings privacy-list");
    println!("  sadas-settings privacy-network <app-id> <allow|deny>");
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("privacy-list") => match sadas_core::list_permissions() {
            Ok(entries) => {
                if entries.is_empty() {
                    println!("no overrides (all denied by default)");
                }
                for entry in entries {
                    println!("{entry}");
                }
            }
            Err(err) => println!("error: {err}"),
        },
        Some("privacy-network") => {
            let app = args.next().unwrap_or_default();
            let mode = args.next().unwrap_or_default();
            if app.is_empty() || (mode != "allow" && mode != "deny") {
                usage();
                return;
            }
            let allowed = mode == "allow";
            match sadas_core::set_network_permission(&app, allowed) {
                Ok(()) => println!("privacy updated: {app} network={mode}"),
                Err(err) => println!("error: {err}"),
            }
        }
        _ => usage(),
    }
}
