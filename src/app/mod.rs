use crate::repo::{write_repo_to_config, Repo};
use chrono::Local;
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

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

pub fn default_config_path() -> String {
    let mut out = String::new();
    if let Ok(cur_user) = whoami::username() {
        if cur_user.contains("root") {
            out = "/root/.cache/phantomCI/{}/config/".to_string();
        } else {
            out = format!("/home/{}/.cache/phantomCI/config/", cur_user);
        }
        let _ = fs::create_dir_all(Path::new(&out));
    }
    out
}

// Struct to hold application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub repos: Arc<Mutex<HashMap<String, Repo>>>,
}

impl AppState {
    pub fn save_state(&self) {
        save_state(self.get_serializable());
    }
}

impl AppState {
    pub fn restore_state(&mut self) {
        #[warn(unused_assignments)]
        let mut state_path = String::new();
        state_path = get_state_path();
        if state_path.is_empty() {
            state_path = get_previous_state_path();
        }

        if Path::new(state_path.as_str()).exists() {
            let content = fs::read_to_string(&state_path).unwrap();
            let restored = serde_json::from_str::<SerializableState>(&content).unwrap();
            self.deserialize_restore(restored);
        }

        if let Ok(s) = self.repos.lock() {
            if s.len() > 0 {
                println!(
                    "Restored state:\n      repo: {}\n      path: {}\n",
                    s.len(),
                    state_path,
                );
            }
        }
    }
}

pub fn get_state_path() -> String {
    let mut short_stamp = Local::now().timestamp().to_string();
    short_stamp.truncate(8);
    format!(
        "{}{}",
        default_repo_work_path(".state".to_string()),
        short_stamp
    )
}

pub fn get_previous_state_path() -> String {
    let mut short_stamp = Local::now().timestamp().to_string();
    short_stamp.truncate(8);
    let mut num = short_stamp.parse::<i32>().unwrap();
    num = num - 1;
    format!(
        "{}{}",
        default_repo_work_path(".state".to_string()),
        num.to_string()
    )
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SerializableState {
    pub repos: HashMap<String, Repo>,
}

impl AppState {
    // Initialize a new AppState
    pub fn new() -> Self {
        Self {
            repos: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn get_serializable(&self) -> SerializableState {
        SerializableState {
            repos: self.repos.lock().unwrap().clone(),
        }
    }
    pub fn deserialize_restore(&mut self, state: SerializableState) {
        self.repos.lock().unwrap().clone_from(&state.repos)
    }

    // Add a new repository
    pub fn add_repo(&self, name: String, repo: Repo) {
        self.repos.lock().unwrap().insert(name, repo);
    }
}

pub fn save_state(app_state: SerializableState) {
    let path = get_state_path();

    let path_old = get_previous_state_path();

    let mut dir_only = path.clone();
    dir_only = dir_only.replace(path.split('/').last().unwrap(), "");

    if let Err(_e) = fs::create_dir_all(Path::new(dir_only.as_str())) {
        // println!("{:?}", e)
    }

    if let Err(_e) = fs::remove_file(Path::new(&path_old)) {
        // println!("{:?}", e)
    }
    // if let Err(_e) = fs::copy(&path, &path_old) {
    //     // println!("{:?}", e)
    // }
    if let Err(_e) = fs::remove_file(Path::new(&path)) {
        // println!("{:?}", e)
    }
    let state_string = serde_json::to_string(&app_state).unwrap();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(Path::new(&path))
    {
        let _ = file.write(state_string.as_ref());
    }
    // println!("Saving state .. {}", path);
}
