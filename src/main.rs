pub mod app;
pub mod parser;
pub mod repo;
pub mod scm;
pub mod options;
pub mod webhook;
pub mod util;

use std::path::Path;
use std::process::exit;
use crate::app::{AppState};
use crate::options::process_arguments;
use crate::repo::{get_repo_from_config, prepare};
use crate::scm::poll_repos;
use clap::Parser;
use std::time::Duration;
use crate::util::default_config_path;

#[tokio::main]
async fn main() {
    let config_dir = default_config_path();

    let env_path = format!("{}.env", config_dir);
    if dotenv::from_path(Path::new(&env_path)).is_ok() {
        println!("Loaded variables from .env")
    }


    let mut state = AppState::new();
    state.restore_state();
    process_arguments(&mut state, &config_dir);

    get_repo_from_config(&config_dir).iter_mut().for_each(|mut repo| {
        prepare(&mut repo);
        state.add_repo(repo.clone().name, repo.to_owned())
    });


    // println!("{:?}", state);
    println!("Starting Git SCM polling...\n     config: {}", &config_dir);
    // println!("{}", serde_json::to_string(&state.get_serializable()).unwrap());
    poll_repos(state, Duration::from_secs(15)).await; // Poll every 60 seconds
}
