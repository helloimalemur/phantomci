use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

fn default_systemd_service_dir(f: String) -> String {
    format!("/usr/lib/systemd/system/{}.service", f)
}

fn default_systemd_service_file() -> &'static str {
    let x = r#"
[Unit]
Description=phantom_ci
After=network.target

[Service]
User=root
Group=root
Type=simple
RemainAfterExit=no
Restart=always
ExecStart=phantom_ci

[Install]
WantedBy=default.target
"#;
    x
}

pub fn configure_systemd() {
    let service_file = default_systemd_service_dir("phantom_ci".to_string());
    println!("installing service.. {}", &service_file);
    if let Err(e) = fs::write(&service_file, default_systemd_service_file()) {
        println!("unable to install {}: {}", &service_file, e);
    }
}