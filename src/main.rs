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
use crate::database::create_connection;
use crate::scm::poll_repos;
use crate::util::default_config_path;

#[tokio::main]
async fn main() {
    logging::init();
    if let Some(config_dir) = default_config_path() {
        if let Ok(conn) = create_connection(config_dir.clone()) {
            let mut state = AppState::new(conn, config_dir.clone());
            poll_repos(&mut state).await;
        }
    } else {
        panic!("No config file specified");
    }
}
