pub mod app;
mod database;
pub mod logging;
pub mod options;
pub mod parser;
pub mod repo;
pub mod util;
pub mod webhook;

use crate::app::state::AppState;

#[tokio::main]
async fn main() {
    logging::init();
    let mut state = AppState::new();
    state.poll_repos().await;
}
