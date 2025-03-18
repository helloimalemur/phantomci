use std::{fs, thread};
use std::path::Path;
use std::process::exit;
use tokio::process;

pub mod service;

pub fn default_repo_work_path_remove_cache_data() {
    println!("Stopping phantom_ci service...");
    let _ = process::Command::new("systemctl")
        .arg("stop")
        .arg("phantom_ci")
        .spawn();
    thread::sleep(std::time::Duration::from_secs(3));
    let mut out = String::new();
    println!("Removing phantom_ci caches...");
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = "/root/.cache/phantom_ci/".to_string();
        } else {
            out = format!("/home/{}/.cache/phantom_ci/", cur_user);
        }
        let _ = fs::remove_dir_all(Path::new(&out));
    }
    println!("Starting phantom_ci service...");
    let _ = process::Command::new("systemctl")
        .arg("start")
        .arg("phantom_ci")
        .spawn();
    exit(0);
}



pub fn default_repo_work_path(repo_name: String) -> String {
    let mut out = String::new();
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = format!("/root/.cache/phantom_ci/{}/", repo_name);
        } else {
            out = format!("/home/{}/.cache/phantom_ci/{}/", cur_user, repo_name);
        }
        let _ = fs::create_dir_all(Path::new(&out));
    }
    out
}

pub fn default_repo_work_path_delete(repo_name: String) -> String {
    let mut out = String::new();
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = format!("/root/.cache/phantom_ci/{}/", repo_name);
        } else {
            out = format!("/home/{}/.cache/phantom_ci/{}/", cur_user, repo_name);
        }
        let _ = fs::remove_dir_all(Path::new(&out));
    }
    out
}

pub fn default_config_path() -> Option<String> {
    let mut out = String::new();
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = "/root/.config/phantom_ci/".to_string();
        } else {
            out = format!("/home/{}/.config/phantom_ci/", cur_user);
        }
        if let Err(e) = fs::create_dir_all(Path::new(&out)) {
            eprintln!("{}", e);
        }
    }
    Some(out)
}
