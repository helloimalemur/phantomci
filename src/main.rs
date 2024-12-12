pub mod app;
pub mod options;
pub mod parser;
pub mod repo;
pub mod scm;
pub mod util;
pub mod webhook;

use crate::app::AppState;
use crate::options::process_arguments;
use crate::repo::{get_repo_from_config, prepare};
use crate::scm::poll_repos;
use crate::util::default_config_path;
use clap::Parser;
use std::path::Path;
use std::process::exit;
use std::time::Duration;

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

    get_repo_from_config(&config_dir)
        .iter_mut()
        .for_each(|repo| {
            prepare(repo);
            state.add_repo(repo.clone().name, repo.to_owned())
        });

    println!("Starting Git SCM polling...\n     config: {}", &config_dir);
    poll_repos(state, Duration::from_secs(15)).await;
}
