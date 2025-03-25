mod state;

use crate::app::state::{get_previous_state_path, get_state_path, save_state};
use crate::repo::{get_repo_from_config, Repo};
use crate::util::{default_config_path, default_repo_work_path_delete};
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};

// Struct to hold application state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SerializableState {
    pub repos: HashMap<String, Repo>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub repos: Arc<Mutex<HashMap<String, Repo>>>,
    pub scm_internal: u64,
    pub db_conn: Option<Arc<Mutex<Connection>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            repos: Arc::new(Mutex::new(HashMap::new())),
            scm_internal: 15,
            db_conn: None,
        }
    }
    pub fn save_state(&self) {
        save_state(self.get_serializable());
    }
    pub fn restore_state(&mut self) {
        let mut state_path: String = get_state_path();
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
    pub fn get_serializable(&self) -> SerializableState {
        SerializableState {
            repos: self.repos.lock().unwrap().to_owned(),
        }
    }
    pub fn deserialize_restore(&mut self, state: SerializableState) {
        self.repos.lock().unwrap().clone_from(&state.repos)
    }

    // Add a new repository
    pub fn add_repos_from_config(&mut self) {
        if let Some(config_dir) = default_config_path() {
            let mut left_out = self.get_serializable().repos.clone();
            println!("Loading repos from config:");
            get_repo_from_config(&config_dir)
                .iter_mut()
                .for_each(|repo| {
                    left_out.remove(&repo.name);
                    repo.prepare();
                    self.add_repo(repo.clone().name, repo.to_owned())
                });
            left_out.iter().for_each(|remove_repo| {
                println!("Removed from config: {}", remove_repo.1.name);
                self.repos.lock().unwrap().remove(remove_repo.0.as_str());
                default_repo_work_path_delete(remove_repo.1.name.clone()).unwrap();
            });
        }
    }

    pub fn add_repo(&mut self, repo_name: String, repo: Repo) {
        if let Ok(mut s) = self.repos.lock() {
            if !s.contains_key(&repo_name) {
                s.insert(repo_name, repo);
            }
        }
    }

    pub fn set_db_conn(&mut self, db_conn: Connection) {
        self.db_conn = Some(Arc::new(Mutex::new(db_conn)));
    }
}
