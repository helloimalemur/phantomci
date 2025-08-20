use crate::database::job::Job;
use crate::parser::parse_workflow;
use crate::util::{default_config_path, default_repo_work_path};
use crate::webhook::{Webhook, WebhookConfig, WebhookType};
use chrono::Local;
use config::Config;
use std::collections::HashMap;
use std::env::consts::OS;
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::{exit, Command};
use std::{env, fs};
use tokio::sync::mpsc::Sender;

// Struct to represent a repository
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Repo {
    pub name: String,
    pub path: String,
    pub work_dir: String,
    pub last_sha: Option<String>,
    pub target_branch: String,
    pub triggered: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Repos {
    pub path: String,
    pub target_branch: Option<String>,
}

impl Default for Repo {
    fn default() -> Repo {
        Repo {
            name: "".to_string(),
            path: "".to_string(),
            work_dir: "".to_string(),
            last_sha: None,
            target_branch: "master".to_string(),
            triggered: false,
        }
    }
}

impl Repo {
    pub fn new(
        name: String,
        path: String,
        work_dir: String,
        last_sha: Option<String>,
        target_branch: String,
        triggered: bool,
    ) -> Repo {
        Repo {
            name,
            path,
            work_dir,
            last_sha,
            target_branch,
            triggered,
        }
    }

    pub fn prepare(&mut self) {
        // clone if not exist
        println!("Preparing {}", self.name);
        env::set_var("GIT_SSH_COMMAND", "ssh -o StrictHostKeyChecking=no");
        if !Path::new(&self.work_dir).exists()
            && fs::create_dir_all(Path::new(&self.work_dir)).is_ok()
        {
            println!("Created directory: {}", self.work_dir);
            println!("Cloning: {}", self.work_dir);
            self.clone_repo();
            self.get_default_branch();
        }

        let branch = self.target_branch.clone();
        self.last_sha = self.git_latest_sha(&branch);

        let mut job = Job {
            id: 0,
            repo: self.path.clone(),
            status: "".to_string(),
            priority: 0,
            created_at: "".to_string(),
            updated_at: "".to_string(),
            start_time: "".to_string(),
            finish_time: "".to_string(),
            error_message: "".to_string(),
            result: "".to_string(),
            sha: String::from(self.clone().last_sha.unwrap_or_default()),
            target_branch: self.target_branch.clone(),
        };
        job.add_job();
    }

    pub async fn send_webhook(&self, message: String, repo: &Repo) {
        let title = repo.path.split('/').last().unwrap_or(repo.path.as_str());

        if let Ok(discord_url) = env::var("DISCORD_WEBHOOK_URL") {
            let webhook = Webhook::new(WebhookConfig::new(
                title,
                discord_url.as_str(),
                WebhookType::Discord,
                &message,
            ));
            webhook.send().await;
        }
        if let Ok(slack_url) = env::var("SLACK_WEBHOOK_URL") {
            let webhook = Webhook::new(WebhookConfig::new(
                title,
                slack_url.as_str(),
                WebhookType::Slack,
                &message,
            ));
            webhook.send().await;
        }
    }

    pub fn check_repo_changes(&mut self) {
        let branch = self.target_branch.clone();
        if let Some(latest_sha) = self.git_latest_sha(&branch) {
            // read last known SHA from DB (empty string if none)
            let last_sha = self.get_sha_by_repo();

            if last_sha.is_empty() {
                // first-time initialization
                self.set_sha_by_repo(latest_sha.clone());
                self.last_sha = Some(latest_sha);
                return;
            }

            if last_sha != latest_sha {
                self.set_sha_by_repo(latest_sha.clone());
                Job::update_status(
                    self.path.clone(),
                    self.target_branch.clone(),
                    "running".to_string(),
                );
                self.triggered = true;
                self.last_sha = Some(latest_sha.clone());

                println!("========================================================");
                println!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
                println!(
                    "Change detected in repo: {}\nNew SHA: {}",
                    self.path, latest_sha
                );
                println!("========================================================");
            }
        } else {
            eprintln!("Failed to fetch latest SHA for repo at {}", self.path);
        }
    }

    // Check for changes in a repository and handle them
    pub async fn check_repo_triggered(&mut self, tx_clone: Sender<String>) {
        if self.triggered {
            Job::update_start_time(self.path.clone(), self.target_branch.clone());

            // Parse workflow file
            self.triggered = false;
            let repo_name = self.path.split('/').last().unwrap_or("").to_string();
            if let Some(base) = default_repo_work_path(repo_name) {
                let wp = format!("{}/workflow/{}.toml", base, self.target_branch);
                let workflow_path = Path::new(&wp);
                if workflow_path.exists() {
                    if let Some(wp_str) = workflow_path.to_str() {
                        parse_workflow(wp_str, self.to_owned(), tx_clone).await;
                    } else {
                        eprintln!("Invalid workflow path");
                    }
                } else {
                    eprintln!("Workflow file not found at {}", wp);
                }
            } else {
                eprintln!("Failed to determine default repo work path; skipping workflow parse");
            }
        }
    }

    pub fn git_latest_sha(&mut self, branch: &str) -> Option<String> {
        if let Err(e) = self.fetch_pull() {
            eprintln!("Error: {}", e)
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("rev-parse")
            .arg(format!("origin/{}", branch))
            .output();

        match output {
            Ok(output) if output.status.success() => {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            }
            Ok(output) => {
                eprintln!("Error: scm polling error: {:?}", output);
                None
            }
            _ => {
                eprintln!("Error: scm polling error: {}", self.name);
                None
            }
        }
    }

    pub fn fetch_pull(&mut self) -> Result<(), anyhow::Error> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("fetch")
            .arg("--all")
            .arg("--prune")
            .output()?;
        if !out.status.success() {
            anyhow::bail!(
                "git fetch failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(())
    }

    pub fn write_repo_to_config(&mut self) {
        let name = self.path.split('/').last().unwrap();
        let config_entry = format!(
            "[{}]\n\
    path = \"{}\"\n\
    target_branch = \"{}\"\n\n\
    ",
            name, self.path, self.target_branch
        );
        if let Some(config_path) = default_config_path() {
            let repo_config = format!("{}Repo.toml", config_path);
            if Path::new(&repo_config.as_str()).exists() {
                if let Ok(mut f) = OpenOptions::new().append(true).open(repo_config) {
                    if let Err(e) = f.write_all(config_entry.as_ref()) {
                        println!("{:?}", e);
                    }
                }
            }
        }
    }

    fn clone_repo(&mut self) {
        let p = self
            .work_dir
            .replace(self.work_dir.split('/').last().unwrap(), "");
        if let Ok(_output) = Command::new("git")
            .arg("-C")
            .arg(p)
            .arg("clone")
            .arg(&self.path)
            .output()
        {
            let git_repo_path = format!("{}/.git", self.work_dir);
            if Path::new(&git_repo_path).exists() {
                println!("Cloned successfully: {}", self.path);
            } else {
                println!("Failed to clone: {}", self.path);
                let _ = fs::remove_dir_all(Path::new(&self.work_dir));
            }
        }
    }

    fn get_default_branch(&mut self) -> String {
        let mut head_branch = "master".to_string();

        if let Ok(output) = Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("remote")
            .arg("show")
            .arg("origin")
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(s) = stdout
                    .lines()
                    .find(|l| l.contains("HEAD branch:"))
                    .map(|l| l.replace("HEAD branch:", ""))
                {
                    head_branch = s.trim().to_string();
                    println!("Default branch: {}", head_branch);
                }
            } else {
                eprintln!(
                    "git remote show origin failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        if self.target_branch.is_empty() {
            self.target_branch = head_branch.to_owned();
        }

        head_branch
    }

    fn get_sha_by_repo(&self) -> String {
        let repo = String::from(&self.path);
        let branch = String::from(&self.target_branch);
        let jobs = Job::get_jobs_by_repo(repo, branch);
        let mut sha = String::new();
        for job in jobs {
            sha = job.sha;
        }
        sha
    }

    fn set_sha_by_repo(&self, latest_sha: String) {
        Job::update_sha(
            String::from(&self.path),
            self.target_branch.clone(),
            latest_sha.clone(),
        );
    }
}

pub fn load_repos_from_config(config_dir: &str) -> Vec<Repo> {
    let repo_config = format!("{}Repo.toml", &config_dir);
    let mut repos = vec![];
    if let Ok(config_file) = Config::builder()
        .add_source(config::File::with_name(repo_config.as_str()))
        .build()
    {
        if let Ok(map) = config_file.try_deserialize::<HashMap<String, Repos>>() {
            map.iter().for_each(|r| {
                // println!("{:?}", r);
                repos.push(Repo {
                    name: r.0.to_string(),
                    path: r.1.path.to_string(),
                    work_dir: repo_work_dir(r.1),
                    last_sha: None,
                    target_branch: r.1.to_owned().target_branch.unwrap_or("master".to_string()),
                    triggered: false,
                })
            });
            repos
        } else {
            vec![]
        }
    } else {
        let repo_config = format!("{}Repo.toml", &config_dir);
        create_default_config(&repo_config);
        println!(
            "Config not found !! default config created, please edit;\n{}",
            repo_config
        );
        exit(1);
    }
}

pub fn create_default_config(path: &String) {
    let default_config = r#"
##[sys-compare]
##path = "git@github.com:helloimalemur/sys-compare"
##target_branch = "master"

"#;
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(default_config.as_ref());
    }
}

pub fn repo_work_dir(repo: &Repos) -> String {
    let rand = rand::random::<u64>();
    let cur_user = whoami::username().unwrap_or_else(|_| "unknown".to_string());
    // Resolve a stable repo name tail or fall back to random
    let repo_name = repo
        .path
        .split('/')
        .last()
        .map(|s| s.to_string())
        .unwrap_or_else(|| rand.to_string());

    if cur_user.contains("root") {
        match OS {
            "linux" => format!("/root/.cache/phantom_ci/{}", repo_name),
            "macos" => format!(
                "/var/root/.cache/phantom_ci/{}",
                repo_name
            ),
            &_ => format!("/tmp/phantom_ci/{}", repo_name),
        }
    } else {
        match OS {
            "linux" => format!(
                "/home/{}/.cache/phantom_ci/{}",
                cur_user, repo_name
            ),
            "macos" => format!(
                "/Users/{}/Library/Caches/com.helloimalemur.phantom_ci/{}",
                cur_user, repo_name
            ),
            &_ => format!("/tmp/phantom_ci/{}", repo_name),
        }
    }
}
