use std::path::Path;
use std::process::exit;
use clap::{Parser, Subcommand};
use crate::app::{default_repo_work_path, AppState};
use crate::app::service::configure_systemd;
use crate::repo::{create_default_config, write_repo_to_config, Repo};

#[derive(Debug, Clone, Parser)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Option<Command>
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Add {
        path: Option<String>,
    },
    Configure {
        sub: String,
}
}

pub fn process_arguments(app_state: &mut AppState, config_dir: &String) {
    let repo_config = format!("{}Repo.toml", &config_dir);
    if !Path::new(&repo_config.as_str()).exists() {
        create_default_config(&repo_config);
    }
    let arguments = Arguments::parse();

    match arguments.command {
        None => {}
        Some(Command::Add { path }) => {
            if let Some(p) = path {
                if !p.is_empty() {
                    println!("Add repo: {}", p);
                    write_repo_to_config(
                        Repo::new(p.clone(), default_repo_work_path(p.clone()), "workflow.toml".to_string(), None, "master".to_string(), false)
                    );
                }
            }
        }
        Some(Command::Configure { sub}) => {
            match sub.as_str() {
                "service" => {
                    configure_systemd();
                    exit(0);
                }
                &_ => {
                    println!("Invalid subcommand");
                    exit(1);
                }
            }
        }
    }

}