use crate::app::state::AppState;
use crate::repo::{create_default_config, load_repos_from_config, Repo};
use crate::util::service::configure_systemd;
use crate::util::{default_repo_work_path, default_repo_work_path_remove_cache_data};
use clap::{Parser, Subcommand};
use std::path::Path;
use std::process::exit;

#[derive(Debug, Clone, Parser)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Add {
        path: Option<String>,
        branch: Option<String>,
    },
    Configure {
        sub: String,
    },
    List,
    Reload,
}
