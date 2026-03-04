use std::env;

fn usage() {
    println!("privacy-firewall usage:");
    println!("  privacy-firewall check <app-id>");
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check") => {
            let app = args.next().unwrap_or_default();
            if app.is_empty() {
                usage();
                return;
            }
            match sadas_core::network_allowed(&app) {
                Ok(true) => println!("network allowed for {app}"),
                Ok(false) => println!("network denied for {app}"),
                Err(err) => println!("error: {err}"),
            }
        }
        _ => usage(),
    }
}
