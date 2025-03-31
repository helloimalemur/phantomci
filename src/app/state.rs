use crate::database::job::Job;
use crate::database::SqliteConnection;
use crate::options::{Arguments, Command};
use crate::repo::{create_default_config, load_repos_from_config, Repo};
use crate::util::service::configure_systemd;
use crate::util::{default_config_path, default_repo_work_path_delete};
use crate::util::{default_repo_work_path, default_repo_work_path_remove_cache_data};
use chrono::Local;
use clap::Parser;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::interval;

// Struct to hold application state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SerializedState {
    pub repos: HashMap<String, Repo>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub repos: Arc<Mutex<HashMap<String, Repo>>>,
    pub scm_internal: u64,
    pub db_conn: Option<Arc<Mutex<Connection>>>,
}

impl AppState {
    pub fn new() -> Self {
        if let Some(config_dir) = default_config_path() {
            if let Ok(c) = SqliteConnection::new() {
                let mut state = AppState {
                    repos: Arc::new(Mutex::new(HashMap::new())),
                    scm_internal: 15,
                    db_conn: Some(Arc::new(Mutex::new(c.conn))),
                };
                state.add_repos_from_config();
                state.process_arguments(config_dir.as_str());
                state
            } else {
                panic!("Failed to connect to SQLite database");
            }
        } else {
            panic!("unable to find config path");
        }
    }

    pub fn process_arguments(&mut self, config_dir: &str) {
        let repo_config = format!("{}Repo.toml", &config_dir);
        if !Path::new(&repo_config.as_str()).exists() {
            create_default_config(&repo_config);
        }
        let arguments = Arguments::parse();

        match arguments.command {
            None => {}
            Some(Command::Add {
                path: Some(repo_path),
                branch: Some(branch_name),
            }) => {
                if branch_name.len() == 0 {
                    println!("Branch name is empty");
                    exit(1);
                }
                if !repo_path.is_empty() {
                    let repo_name_only = repo_path
                        .split('/')
                        .last()
                        .to_owned()
                        .unwrap_or("0")
                        .to_string();
                    println!("Adding repo: {}", &repo_name_only);
                    Repo::new(
                        repo_name_only.clone(),
                        repo_path.to_owned(),
                        default_repo_work_path(repo_path.to_owned()).unwrap(),
                        None,
                        branch_name,
                        false,
                    )
                    .write_repo_to_config();
                    exit(0);
                }
            }

            Some(Command::Add {
                path: Some(path),
                branch: None,
            }) => {
                println!("Missing branch name: {}", &path);
                exit(1);
            }
            Some(Command::Add {
                path: None,
                branch: Some(branch),
            }) => {
                println!("Missing repo path: {}", &branch);
                exit(1);
            }
            Some(Command::Add {
                path: None,
                branch: None,
            }) => {
                println!("Missing repo path");
                exit(1);
            }
            Some(Command::Configure { sub }) => match sub.as_str() {
                "service" => {
                    configure_systemd();
                    exit(0);
                }
                &_ => {
                    println!("Invalid subcommand");
                    exit(1);
                }
            },
            Some(Command::Reset) => {
                default_repo_work_path_remove_cache_data();
            }
            Some(Command::List) => {
                let repo_config_path = format!("{}Repo.toml", config_dir);
                println!("Listing repos: {}", repo_config_path);
                let repo = load_repos_from_config(config_dir);
                for re in repo.iter() {
                    println!("{} - {}", re.path, re.target_branch);
                }
                exit(0);
            }
        }
    }

    pub fn save_state(&self) {
        save_state(self.get_serialized_state());
    }
    pub fn restore_state(&mut self) {
        let mut state_path: String = get_state_path();
        if state_path.is_empty() {
            state_path = get_previous_state_path();
        }

        if Path::new(state_path.as_str()).exists() {
            let content = fs::read_to_string(&state_path).unwrap();
            let restored = serde_json::from_str::<SerializedState>(&content).unwrap();
            self.set_deserialize_state(restored);
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
    pub fn get_serialized_state(&self) -> SerializedState {
        SerializedState {
            repos: self.repos.lock().unwrap().to_owned(),
        }
    }
    pub fn set_deserialize_state(&mut self, state: SerializedState) {
        self.repos.lock().unwrap().clone_from(&state.repos)
    }

    pub fn add_repos_from_config(&mut self) {
        if let Some(config_dir) = default_config_path() {
            let mut left_out = self.get_serialized_state().repos.clone();
            println!("Loading repos from config:");
            load_repos_from_config(&config_dir)
                .iter_mut()
                .for_each(|repo| {
                    left_out.remove(&repo.name);
                    repo.prepare();
                    self.add_repo_to_state(repo.clone().name, repo.to_owned());
                    if !Job::check_exists(
                        String::from(&repo.path),
                        String::from(&repo.target_branch),
                    ) {
                        let mut job = Job {
                            id: 0,
                            repo: repo.path.clone(),
                            status: "".to_string(),
                            priority: 0,
                            created_at: "".to_string(),
                            updated_at: "".to_string(),
                            start_time: "".to_string(),
                            finish_time: "".to_string(),
                            error_message: "".to_string(),
                            result: "".to_string(),
                            sha: repo.last_sha.clone().unwrap_or("".to_string()),
                            target_branch: repo.target_branch.clone(),
                        };
                        job.add_job()
                    }
                });
            left_out.iter().for_each(|remove_repo| {
                println!("Removed from config: {}", remove_repo.1.name);
                self.repos.lock().unwrap().remove(remove_repo.0.as_str());
                default_repo_work_path_delete(remove_repo.1.name.clone()).unwrap();
            });
        }
    }

    pub fn add_repo_to_state(&mut self, repo_name: String, repo: Repo) {
        if let Ok(mut s) = self.repos.lock() {
            if !s.contains_key(&repo_name) {
                s.insert(repo_name, repo);
            }
        }
    }

    pub fn set_db_conn(&mut self, db_conn: Connection) {
        self.db_conn = Some(Arc::new(Mutex::new(db_conn)));
    }

    pub async fn poll_repos(&mut self) {
        println!(
            "Starting Git SCM polling...\n     config: {}",
            default_config_path().unwrap()
        );
        let interval_duration = Duration::new(self.scm_internal.clone(), 0);
        let mut ticker = interval(interval_duration);
        #[allow(unused)]
        let (mut tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

        loop {
            let tx_clone = tx.clone();
            // let mut rx_clone = &mut rx;

            ticker.tick().await;
            let mut repos = self.repos.lock().unwrap().to_owned();
            for (_, repo) in repos.iter_mut() {
                repo.check_repo_changes();
                repo.check_repo_triggered(tx_clone.clone()).await
            }
            self.repos.lock().unwrap().clone_from(&repos);

            drop(tx_clone);

            loop {
                match rx.try_recv() {
                    Ok(msg) => println!("Received: {}", msg),
                    Err(TryRecvError::Empty) => break, // nothing left to read
                    Err(TryRecvError::Disconnected) => {
                        // println!("Channel closed.");
                        break;
                    }
                }
            }
        }
    }
}

pub fn get_state_path() -> String {
    let mut short_stamp = Local::now().timestamp().to_string();
    short_stamp.truncate(8);
    format!(
        "{}{}",
        default_repo_work_path(".state".to_string()).unwrap(),
        short_stamp
    )
}

pub fn get_previous_state_path() -> String {
    let mut short_stamp = Local::now().timestamp().to_string();
    short_stamp.truncate(8);
    let mut num = short_stamp.parse::<i32>().unwrap();
    num -= 1;
    format!(
        "{}{}",
        default_repo_work_path(".state".to_string()).unwrap(),
        num
    )
}

pub fn save_state(app_state: SerializedState) {
    let path = get_state_path();

    let path_old = get_previous_state_path();

    let mut dir_only = path.to_owned();
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
