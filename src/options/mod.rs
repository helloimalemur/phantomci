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

pub fn process_arguments(_app_state: &mut AppState, config_dir: &str) {
    let repo_config = format!("{}Repo.toml", &config_dir);
    if !Path::new(&repo_config.as_str()).exists() {
        create_default_config(&repo_config);
    }
    let arguments = Arguments::parse();

    match arguments.command {
        None => {}
        Some(Command::Add {
            path: Some(repo_path),
            branch: Some(branch_name),
        }) => {
            if branch_name.len() == 0 {
                println!("Branch name is empty");
                exit(1);
            }
            if !repo_path.is_empty() {
                let repo_name_only = repo_path
                    .split('/')
                    .last()
                    .to_owned()
                    .unwrap_or("0")
                    .to_string();
                println!("Adding repo: {}", &repo_name_only);
                Repo::new(
                    repo_name_only.clone(),
                    repo_path.to_owned(),
                    default_repo_work_path(repo_path.to_owned()).unwrap(),
                    None,
                    branch_name,
                    false,
                )
                .write_repo_to_config();
                exit(0);
            }
        }

        Some(Command::Add {
            path: Some(path),
            branch: None,
        }) => {
            println!("Missing branch name: {}", &path);
            exit(1);
        }
        Some(Command::Add {
            path: None,
            branch: Some(branch),
        }) => {
            println!("Missing repo path: {}", &branch);
            exit(1);
        }
        Some(Command::Add {
            path: None,
            branch: None,
        }) => {
            println!("Missing repo path");
            exit(1);
        }
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
            default_repo_work_path_remove_cache_data();
        }
        Some(Command::List) => {
            let repo_config_path = format!("{}Repo.toml", config_dir);
            println!("Listing repos: {}", repo_config_path);
            let repo = load_repos_from_config(config_dir);
            for re in repo.iter() {
                println!("{} - {}", re.path, re.target_branch);
            }
            exit(0);
        }
    }
}
