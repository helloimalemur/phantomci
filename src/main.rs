pub mod app;
pub mod repo;
pub mod scm;
pub mod parser;

use crate::app::AppState;
use crate::repo::{get_repo_from_config, prepare};
use crate::scm::poll_repos;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut state = AppState::new();
    state.restore_state();

    get_repo_from_config().iter_mut().for_each(|mut repo| {
        // println!("{:?}", repo);
        prepare(&mut repo);
        state.add_repo(repo.path.to_string(), repo.clone())
    });

    // println!("{:?}", state);
    println!("Starting Git SCM polling app...");
    // println!("{}", serde_json::to_string(&state.get_serializable()).unwrap());
    poll_repos(state, Duration::from_secs(15)).await; // Poll every 60 seconds
}
