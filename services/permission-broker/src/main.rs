use std::env;

fn usage() {
    println!("permission-broker usage:");
    println!("  permission-broker set-network <app-id> <allow|deny>");
    println!("  permission-broker get-network <app-id>");
    println!("  permission-broker list");
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("set-network") => {
            let app = args.next().unwrap_or_default();
            let mode = args.next().unwrap_or_default();
            let allowed = mode == "allow";
            if app.is_empty() || (mode != "allow" && mode != "deny") {
                usage();
                return;
            }
            match sadas_core::set_network_permission(&app, allowed) {
                Ok(()) => println!("updated {app} network={mode}"),
                Err(err) => println!("error: {err}"),
            }
        }
        Some("get-network") => {
            let app = args.next().unwrap_or_default();
            if app.is_empty() {
                usage();
                return;
            }
            match sadas_core::network_allowed(&app) {
                Ok(true) => println!("{app}: allow"),
                Ok(false) => println!("{app}: deny"),
                Err(err) => println!("error: {err}"),
            }
        }
        Some("list") => match sadas_core::list_permissions() {
            Ok(lines) => {
                if lines.is_empty() {
                    println!("no explicit permission overrides");
                }
                for line in lines {
                    println!("{line}");
                }
            }
            Err(err) => println!("error: {err}"),
        },
        _ => usage(),
    }
}
