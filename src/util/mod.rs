use std::env::consts::OS;
use std::path::Path;
use std::process::exit;
use std::{fs, thread};
use tokio::process;

pub mod service;

pub fn default_repo_work_path_remove_cache_data() {
    println!("Stopping phantom_ci service...");
    let _ = process::Command::new("systemctl")
        .arg("stop")
        .arg("phantom_ci")
        .spawn();
    thread::sleep(std::time::Duration::from_secs(3));
    println!("Removing phantom_ci caches...");
    let out: String = match OS {
        "linux" => match whoami::username().is_ok_and(|a| a.eq("root")) {
            true => "/root/.cache/phantom_ci/".to_string(),
            false => {
                if let Ok(user) = whoami::username() {
                    user
                } else {
                    panic!("unable to determine user name");
                }
            }
        },
        "macos" => {
            match whoami::username().is_ok_and(|a| a.eq("root")) {
                true => "/var/root/.cache/phantom_ci/".to_string(),
                false => {
                    if let Ok(user) = whoami::username() {
                        format!(
                            "/Users/{}/Library/Caches/com.helloimalemur.phantom_ci/",
                            user
                        )
                        // format!("/Users/{}/Library/Application Support/com.helloimalemur.phantom_ci/", user)
                    } else {
                        panic!("unable to determine user name");
                    }
                }
            }
        }

        _ => {
            panic!("invalid platform")
        }
    };

    let _ = fs::remove_dir_all(Path::new(&out));
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

pub fn default_repo_work_path_delete(repo_name: String) -> Option<String> {
    if let Ok(cur_user) = whoami::username() {
        Some(match OS {
            "linux" => {
                if cur_user.contains("root") {
                    let path = "/root/.cache/phantom_ci/".to_string();
                    let _ = fs::create_dir_all(&path);
                    path
                } else {
                    let path = format!("/home/{}/.cache/phantom_ci/", cur_user).to_string();
                    let _ = fs::create_dir_all(&path);
                    path
                }
            },
            "macos" => {
                if cur_user.contains("root") {
                    let path = "/var/root/.cache/phantom_ci/".to_string();
                    let _ = fs::create_dir_all(&path);
                    path
                } else {
                    let path = format!("/Users/{}/Library/Caches/com.helloimalemur.phantom_ci/", cur_user).to_string();
                    let _ = fs::create_dir_all(&path);
                    path
                }
            },
            &_ => {
                panic!("invalid platform")
            },
        })
    } else {
        panic!("unable to determine user name");
    }
}

pub fn default_config_path() -> Option<String> {
    if let Ok(cur_user) = whoami::username() {
        Some(match OS {
            "linux" => {
                if cur_user.contains("root") {
                    let path = "/root/.config/phantom_ci/".to_string();
                    let _ = fs::create_dir_all(&path);
                    path
                } else {
                    let path = format!("/home/{}/.config/phantom_ci/", cur_user).to_string();
                    let _ = fs::create_dir_all(&path);
                    path
                }
            },
            "macos" => {
                if cur_user.contains("root") {
                    let path = "/var/root/.config/phantom_ci/".to_string();
                    let _ = fs::create_dir_all(&path);
                    path
                } else {
                    let path = format!("/Users/{}/Library/Application\\ Support/com.helloimalemur.phantom_ci/", cur_user).to_string();
                    let _ = fs::create_dir_all(&path);
                    path
                }
            },
            &_ => {
                panic!("invalid platform")
            },
        })
    } else {
        panic!("unable to determine user name");
    }
}
