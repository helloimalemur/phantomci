use std::{fs, process};

fn default_systemd_service_dir(f: String) -> String {
    format!("/usr/lib/systemd/system/{}.service", f)
}

fn default_systemd_service_file() -> &'static str {
    r#"
[Unit]
Description=phantom_ci
After=network.target

[Service]
User=root
Group=root
Type=simple
RemainAfterExit=no
Restart=always
ExecStart=/root/.cargo/bin/phantom_ci

[Install]
WantedBy=default.target
"#
}

pub fn configure_systemd() {
    let _ = process::Command::new("systemctl")
        .arg("stop")
        .arg("phantom_ci")
        .output();

    let service_file = default_systemd_service_dir("phantom_ci".to_string());
    println!("installing service.. {}", &service_file);
    let _ = fs::remove_file(&service_file);
    if let Err(e) = fs::write(&service_file, default_systemd_service_file()) {
        println!("unable to install {}: {}", &service_file, e);
    }

    println!("\nservice installed\nplease run:\nsystemctl daemon-reload\nsystemctl enable phantom_ci\nsystemctl start phantom_ci");
}
