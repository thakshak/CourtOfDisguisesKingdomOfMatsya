use battle_of_matsya::server::run_server;
use std::env;
use std::path::PathBuf;

fn main() {
    let mut port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--port" || arg == "-p" {
            if let Some(val) = args.next() {
                if let Ok(parsed) = val.parse::<u16>() {
                    port = parsed;
                }
            }
        }
    }

    let assets_dir = PathBuf::from("assets");
    let addr = format!("0.0.0.0:{port}");
    if let Err(err) = run_server(&addr, assets_dir) {
        eprintln!("Server failed: {err}");
        std::process::exit(1);
    }
}
