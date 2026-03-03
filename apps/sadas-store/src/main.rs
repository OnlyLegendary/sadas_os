use std::env;

fn usage() {
    println!("Sadas Store (Flatpak frontend)");
    println!("  sadas-store catalog");
    println!("  sadas-store install <app-id>");
    println!("  sadas-store installed");
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("catalog") => {
            for app in sadas_core::STORE_CATALOG {
                println!("{app}");
            }
        }
        Some("install") => {
            let app = args.next().unwrap_or_default();
            if app.is_empty() {
                usage();
                return;
            }
            match sadas_core::install_app(&app) {
                Ok(()) => println!("installed via Flatpak: {app}"),
                Err(err) => println!("error: {err}"),
            }
        }
        Some("installed") => match sadas_core::list_installed() {
            Ok(apps) => {
                if apps.is_empty() {
                    println!("no apps installed");
                }
                for app in apps {
                    println!("{app}");
                }
            }
            Err(err) => println!("error: {err}"),
        },
        _ => usage(),
    }
}
