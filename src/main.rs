pub mod app;
mod database;
pub mod logging;
pub mod options;
pub mod parser;
pub mod repo;
pub mod scm;
pub mod util;
pub mod webhook;

use crate::app::AppState;
use crate::database::setup_schema;
use crate::options::process_arguments;
use crate::scm::poll_repos;
use crate::util::default_config_path;
use rusqlite::{Connection, Result};
use std::path::Path;
use std::time::Duration;

#[tokio::main]
async fn main() {
    logging::init();
    if let Some(config_dir) = default_config_path() {
        let sqlite_path = format!("{}/{}", config_dir, "db.sqlite");
        if let Ok(conn) = Connection::open(&sqlite_path) {
            if let Err(e) = setup_schema(&conn) {
                eprintln!("Failed to setup schema: {:?}", e);
            }

            if let Err(e) = load_env_variables(&config_dir) {
                eprintln!("environment variables not loaded: {}", e);
            }

            let mut state = AppState::new();
            state.set_db_conn(conn);

            if let Err(e) = initialize_state(&mut state, &config_dir) {
                eprintln!("Error initializing state: {}", e);
            }

            println!("Starting Git SCM polling...\n     config: {}", &config_dir);
            let interval = state.scm_internal.clone();
            poll_repos(state, Duration::from_secs(interval)).await;
        }
    }
}

fn initialize_state(state: &mut AppState, config_dir: &str) -> Result<(), anyhow::Error> {
    // state.restore_state();
    state.add_repos_from_config();
    Ok(process_arguments(state, config_dir))
}

fn load_env_variables(path: &str) -> Result<(), dotenv::Error> {
    let env_path = format!("{}.env", path);
    dotenv::from_path(Path::new(&env_path)).map(|_| println!("env: {}", env_path))
}
