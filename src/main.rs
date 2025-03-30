pub mod app;
mod database;
pub mod logging;
pub mod options;
pub mod parser;
pub mod repo;
pub mod util;
pub mod webhook;

use crate::app::state::AppState;
use crate::util::default_config_path;

#[tokio::main]
async fn main() {
    logging::init();
    if let Some(config) = default_config_path() {
        if let Err(e) = database::load_env_variables(config.as_str()) {
            eprintln!("environment variables not loaded: {}", e);
        }
    }
    let mut state = AppState::new();
    state.poll_repos().await;
}
