use std::fs;
use std::path::Path;
use std::process::exit;
use tokio::process;

pub mod service;

pub fn default_repo_work_path_remove_cache_data() {
    println!("Stopping phantomci service...");
    let _ = process::Command::new("systemctl")
        .arg("stop")
        .arg("phantomci")
        .spawn();
    let mut out = String::new();
    println!("Removing phantomci caches...");
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = "/root/.cache/phantomCI/".to_string();
        } else {
            out = format!("/home/{}/.cache/phantomCI/", cur_user);
        }
        let _ = fs::remove_dir_all(Path::new(&out));
    }
    println!("Starting phantomci service...");
    let _ = process::Command::new("systemctl")
        .arg("start")
        .arg("phantomci")
        .spawn();
    exit(0);
}



pub fn default_repo_work_path(repo_name: String) -> String {
    let mut out = String::new();
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = format!("/root/.cache/phantomCI/{}/", repo_name);
        } else {
            out = format!("/home/{}/.cache/phantomCI/{}/", cur_user, repo_name);
        }
        let _ = fs::create_dir_all(Path::new(&out));
    }
    out
}

pub fn default_repo_work_path_delete(repo_name: String) -> String {
    let mut out = String::new();
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = format!("/root/.cache/phantomCI/{}/", repo_name);
        } else {
            out = format!("/home/{}/.cache/phantomCI/{}/", cur_user, repo_name);
        }
        let _ = fs::remove_dir_all(Path::new(&out));
    }
    out
}

pub fn default_config_path() -> String {
    let mut out = String::new();
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = "/root/.config/phantomCI/".to_string();
        } else {
            out = format!("/home/{}/.config/phantomCI/", cur_user);
        }
        let _ = fs::create_dir_all(Path::new(&out));
    }
    out
}
