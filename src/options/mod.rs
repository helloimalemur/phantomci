use crate::app::AppState;
use crate::repo::{create_default_config, write_repo_to_config, Repo};
use crate::util::{default_config_path, default_repo_work_path, default_repo_work_path_remove_data};
use crate::util::service::configure_systemd;
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
    Add { path: Option<String> },
    Configure { sub: String },
    Reload,
}

pub fn process_arguments(_app_state: &mut AppState, config_dir: &String) {
    let repo_config = format!("{}Repo.toml", &config_dir);
    if !Path::new(&repo_config.as_str()).exists() {
        create_default_config(&repo_config);
    }
    let arguments = Arguments::parse();

    match arguments.command {
        None => {}
        Some(Command::Add { path: Some(p) }) => {
            if !p.is_empty() {
                println!("Add repo: {}", p);
                write_repo_to_config(Repo::new(
                    p.split('/').last().to_owned().unwrap_or("0").to_string(),
                    p.to_owned(),
                    default_repo_work_path(p.to_owned()),
                    "workflow.toml".to_string(),
                    None,
                    "master".to_string(),
                    false,
                ));
            }
        }
        Some(Command::Add { path: None }) => {}
        Some(Command::Configure { sub }) => match sub.as_str() {
            "service" => {
                configure_systemd();
                exit(0);
            }
            &_ => {
                println!("Invalid subcommand");
                exit(1);
            }
        },
        Some(Command::Reload) => {
            default_repo_work_path_remove_data();
        }
    }
}
