use crate::app::default_config_path;
use crate::scm::fetch_latest_sha;
use config::Config;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::{exit, Command};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use crate::webhook::{Webhook, WebhookConfig, WebhookType};

// Struct to represent a repository
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Repo {
    pub path: String,
    pub work_dir: String,
    pub workflow_file: String,
    pub last_sha: Option<String>,
    pub target_branch: String,
    pub triggered: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Repos {
    pub path: String,
    pub target_branch: Option<String>,
}

impl Repo {
    pub fn default() -> Repo {
        Repo {
            path: "".to_string(),
            work_dir: "".to_string(),
            workflow_file: "".to_string(),
            last_sha: None,
            target_branch: "master".to_string(),
            triggered: false,
        }
    }

    pub fn new(
        path: String,
        work_dir: String,
        workflow_file: String,
        last_sha: Option<String>,
        target_branch: String,
        triggered: bool,
    ) -> Repo {
        Repo {
            path,
            work_dir,
            workflow_file,
            last_sha,
            target_branch,
            triggered,
        }
    }

    pub fn send_webhook(&self, message: String, repo: &Repo) {
        let config_path = default_config_path();
        if let Ok(config) = Config::builder()
            .add_source(config::File::with_name(&config_path))
            .build() {
            if let Ok(map) = config.try_deserialize::<HashMap<String, String>>() {
                let title = repo.path.split('/').last().unwrap_or(repo.path.as_str());

                if let Ok(discord_url) = env::var("DISCORD_WEBHOOK_URL") {
                    Webhook::new(WebhookConfig::new(title, discord_url.as_str(), WebhookType::Discord, &message));
                }
                if let Ok(slack_url) = env::var("SLACK_WEBHOOK_URL") {
                    Webhook::new(WebhookConfig::new(title, slack_url.as_str(), WebhookType::Slack, &message));
                }
            }
        }
    }
}

pub fn write_repo_to_config(repo: Repo) {
    let name = repo.path.split('/').last().unwrap();
    let config_entry = format!("[{}]\n\
    path = \"{}\"\n\
    target_branch = \"master\"\n\n\
    ", name, repo.path);

    let repo_config = format!("{}Repo.toml", default_config_path());
    if Path::new(&repo_config.as_str()).exists() {
        if let Ok(mut f) = OpenOptions::new()
            .append(true)
            .open(repo_config) {
            if let Err(e) = f.write_all(config_entry.as_ref()) {
                println!("{:?}", e);
            }
        }
    }
}

pub fn get_repo_from_config(config_dir: &String) -> Vec<Repo> {
    let repo_config = format!("{}Repo.toml", &config_dir);
    let mut repos = vec![];
    if let Ok(config_file) = Config::builder()
        .add_source(config::File::with_name(&repo_config.as_str()))
        .build()
    {
        if let Ok(map) = config_file.try_deserialize::<HashMap<String, Repos>>() {
            map.iter().for_each(|r| {
                repos.push(Repo {
                    path: r.1.path.to_string(),
                    work_dir: repo_work_dir(r.1),
                    workflow_file: "workflow.toml".to_string(),
                    last_sha: None,
                    target_branch: r.1.to_owned().target_branch.unwrap_or("master".to_string()),
                    triggered: false,
                })
            });
            if repos.is_empty() {
                println!("Config empty !!\nUpdate: {}", repo_config);
                exit(1);
            }
            repos
        } else {
            panic!("Config not found !!")
        }
    } else {
        panic!("Config not found !!")
    }
}

pub fn create_default_config(path: &String) {
    let default_config = r#"
##[sys-compare]
##path = "https://github.com/helloimalemur/sys-compare"
##target_branch = "master"

"#;
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path) {
        let _ = file.write_all(default_config.as_ref());
    }
}

pub fn repo_work_dir(repo: &Repos) -> String {
    let rand = rand::random::<u64>();
    let cur_user = whoami::username().unwrap();
    if cur_user.contains("root") {
        format!(
            "/root/.cache/phantomCI/{}",
            repo.path
                .split('/')
                .last()
                .unwrap_or(rand.to_string().as_str())
                .to_string()
        )
    } else {
        format!(
            "/home/{}/.cache/phantomCI/{}",
            cur_user,
            repo.path
                .split('/')
                .last()
                .unwrap_or(rand.to_string().as_str())
                .to_string()
        )
    }
}

pub fn prepare(repo: &mut Repo) {
    // clone if not exist
    env::set_var("GIT_SSH_COMMAND", "ssh -o StrictHostKeyChecking=no");
    if !Path::new(&repo.work_dir).exists() {
        if fs::create_dir_all(Path::new(&repo.work_dir)).is_ok() {
            clone_repo(repo);
            get_default_branch(repo);
        }
    }

    repo.last_sha = fetch_latest_sha(repo)
}

fn clone_repo(repo: &Repo) {
    let p = repo
        .work_dir
        .replace(repo.work_dir.split('/').last().unwrap(), "");
    if let Ok(_output) = Command::new("git")
        .arg("-C")
        .arg(p)
        .arg("clone")
        .arg(repo.path.to_string())
        .output()
    {
        let git_repo_path = format!("{}/.git", repo.work_dir);
        if Path::new(&git_repo_path).exists() {
            println!("Cloned successfully: {}", repo.path);
        } else {
            println!("Failed to clone: {}", repo.path);
            let _ = fs::remove_dir(Path::new(&repo.work_dir));
        }
    }
}

fn get_default_branch(repo: &mut Repo) -> String {
    let mut head_branch = "master".to_string();

    if let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(repo.work_dir.to_owned())
        .arg("remote")
        .arg("show")
        .arg(repo.path.to_string())
        .output()
    {
        let lines = output.stdout.lines();
        #[warn(unused_mut)]
        if let Some(s) = lines
            .map(|l| l.unwrap())
            .filter(|l| l.contains("HEAD branch:"))
            .map(|mut l| l.replace("HEAD branch:", ""))
            .next()
        {
            head_branch = s.trim().to_string();
            println!("Default branch: {}", head_branch);
        }
    }

    if repo.target_branch.is_empty() {
        repo.target_branch = head_branch.to_owned();
    }

    head_branch
}
