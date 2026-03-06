use sadas_net::http_get;

fn main() {
    let mut args = std::env::args().skip(1);
    let host = args.next().unwrap_or_else(|| "example.com".to_string());
    let path = args.next().unwrap_or_else(|| "/".to_string());

    match http_get(&host, &path) {
        Ok(resp) => {
            let preview: String = resp.chars().take(400).collect();
            println!("HTTP GET http://{}{}\n{}", host, path, preview);
        }
        Err(err) => {
            eprintln!("httpget failed: {:?}", err);
            std::process::exit(1);
        }
    }
}
