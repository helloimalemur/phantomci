pub mod app;
pub mod options;
pub mod parser;
pub mod repo;
pub mod scm;
pub mod util;
pub mod webhook;

use crate::app::AppState;
use crate::options::process_arguments;
use crate::scm::poll_repos;
use crate::util::default_config_path;
use std::path::Path;
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
    state.add_repos_from_config();
    process_arguments(&mut state, &config_dir);

    println!("Starting Git SCM polling...\n     config: {}", &config_dir);
    poll_repos(state, Duration::from_secs(15)).await;
}
