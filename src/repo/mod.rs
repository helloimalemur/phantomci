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
            clone_repo(self);
            get_default_branch(self);
        }

        self.last_sha = self.fetch_latest_sha()
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
        println!("Checking repo changes... \n {}", &self.name);
        if let Some(latest_sha) = self.fetch_latest_sha() {
            // check sqlite
            // last sha
            if self.last_sha.as_ref() != Some(&latest_sha) {
                println!("========================================================");
                println!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
                println!(
                    "Change detected in repo: {}\nNew SHA: {}",
                    self.path, latest_sha
                );

                self.last_sha = Some(latest_sha.to_owned());
                // write sha

                self.triggered = true;

                println!("========================================================");
            }
        } else {
            eprintln!("Failed to fetch latest SHA for repo at {}", self.path);
        }
    }

    // Check for changes in a repository and handle them
    pub async fn check_repo_triggered(&mut self) {
        if self.triggered {
            // Parse workflow file
            // let workflow_path = Path::new(&repo.path).join(&repo.workflow_file);
            self.triggered = false;
            let wp = format!(
                "{}workflow/{}.toml",
                default_repo_work_path(self.path.split('/').last().unwrap().to_string()).unwrap(),
                self.target_branch
            );
            let workflow_path = Path::new(&wp);
            if workflow_path.exists() {
                parse_workflow(workflow_path.to_str().unwrap(), self.to_owned()).await;
            } else {
                eprintln!(
                    "Workflow file not found at {}",
                    workflow_path.to_str().unwrap()
                );
            }
        }
    }

    pub fn fetch_latest_sha(&mut self) -> Option<String> {
        if let Err(e) = self.fetch_pull() {
            eprintln!("Error: {}", e)
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("rev-parse")
            .arg("HEAD")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            }
            _ => {
                eprintln!("Error: scm polling error: {}", self.name);
                None
            }
        }
    }

    pub fn fetch_pull(&mut self) -> Result<(), anyhow::Error> {
        Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("stash")
            .output()?;

        Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("checkout")
            .arg(&self.target_branch)
            .output()?;

        Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("reset")
            .arg("--hard")
            .arg("HEAD")
            .output()?;

        Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("fetch")
            .output()?;

        Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("pull")
            .output()?;
        Ok(())
    }
}

pub fn write_repo_to_config(repo: Repo) {
    let name = repo.path.split('/').last().unwrap();
    let config_entry = format!(
        "[{}]\n\
    path = \"{}\"\n\
    target_branch = \"{}\"\n\n\
    ",
        name, repo.path, repo.target_branch
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
    let cur_user = whoami::username().unwrap();
    if cur_user.contains("root") {
        match OS {
            "linux" => {
                format!(
                    "/root/.cache/phantom_ci/{}",
                    repo.path
                        .split('/')
                        .last()
                        .unwrap_or(rand.to_string().as_str())
                )
            }
            "macos" => {
                format!(
                    "/var/root/.cache/phantom_ci/{}",
                    repo.path
                        .split('/')
                        .last()
                        .unwrap_or(rand.to_string().as_str())
                )
            }
            &_ => {
                panic!("Unsupported OS!");
            }
        }
    } else {
        match OS {
            "linux" => {
                format!(
                    "/home/{}/.cache/phantom_ci/{}",
                    cur_user,
                    repo.path
                        .split('/')
                        .last()
                        .unwrap_or(rand.to_string().as_str())
                )
            }
            "macos" => {
                format!(
                    "/Users/{}/Library/Caches/com.helloimalemur.phantom_ci/{}",
                    cur_user,
                    repo.path
                        .split('/')
                        .last()
                        .unwrap_or(rand.to_string().as_str())
                )
            }
            &_ => {
                panic!("Unsupported OS!");
            }
        }
    }
}

fn clone_repo(repo: &Repo) {
    let p = repo
        .work_dir
        .replace(repo.work_dir.split('/').last().unwrap(), "");
    if let Ok(_output) = Command::new("git")
        .arg("-C")
        .arg(p)
        .arg("clone")
        .arg(&repo.path)
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
        .arg(&repo.work_dir)
        .arg("remote")
        .arg("show")
        .arg(&repo.path)
        .output()
    {
        let lines = output.stdout.lines();
        if let Some(s) = lines
            .map(|l| l.unwrap())
            .filter(|l| l.contains("HEAD branch:"))
            .map(|l| l.replace("HEAD branch:", ""))
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
