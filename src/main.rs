pub mod app;
mod database;
pub mod logging;
pub mod options;
pub mod parser;
pub mod repo;
pub mod util;
pub mod webhook;

use crate::app::state::AppState;
use crate::database::SqliteConnection;
use crate::util::default_config_path;

#[tokio::main]
async fn main() {
    logging::init();
    let mut state = AppState::new();
    state.poll_repos().await;
}
