use crate::scm::fetch_latest_sha;
use config::Config;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;
use std::process::{exit, Command};
use std::{env, fs};

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
}

pub fn get_repo_from_config() -> Vec<Repo> {
    let mut repos = vec![];
    if let Ok(config_file) = Config::builder()
        .add_source(config::File::with_name("config/Repo"))
        .build()
    {
        if let Ok(map) = config_file.try_deserialize::<HashMap<String, Repos>>() {
            map.iter().for_each(|r| {
                repos.push(Repo {
                    path: r.1.path.to_string(),
                    work_dir: repo_work_dir(r.1),
                    workflow_file: "workflow.toml".to_string(),
                    last_sha: None,
                    target_branch: r.1.clone().target_branch.unwrap_or("master".to_string()),
                    triggered: false,
                })
            });
            if repos.is_empty() {
                panic!("Config empty !!")
            }
            repos
        } else {
            panic!("Config not found !!")
        }
    } else {
        panic!("Config not found !!")
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
        }
    }
}

fn get_default_branch(repo: &mut Repo) -> String {
    let mut head_branch = "master".to_string();

    if let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(repo.work_dir.clone())
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
        repo.target_branch = head_branch.clone();
    }

    head_branch
}
