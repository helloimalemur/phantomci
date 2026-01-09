use crate::database::job::Job;
use crate::parser::parse_workflow;
use crate::util::{default_config_path, default_repo_work_path};
use crate::webhook::{Webhook, WebhookConfig, WebhookType};
use chrono::Local;
use config::Config;
use std::collections::HashMap;
use std::env::consts::OS;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};
use std::{env, fs};
use tokio::sync::mpsc::Sender;

// Lightweight Git client abstraction for easier testing
trait GitClient {
    fn has_remote_branch(&self, work_dir: &str, branch: &str) -> bool;
    fn remote_default_branch(&self, work_dir: &str) -> Option<String>;
}

struct SystemGitClient {}

impl GitClient for SystemGitClient {
    fn has_remote_branch(&self, work_dir: &str, branch: &str) -> bool {
        let remote_ref = format!("refs/remotes/origin/{}", branch);
        match Command::new("git")
            .arg("-C")
            .arg(work_dir)
            .arg("show-ref")
            .arg("--verify")
            .arg("--quiet")
            .arg(&remote_ref)
            .output()
        {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    fn remote_default_branch(&self, work_dir: &str) -> Option<String> {
        match Command::new("git")
            .arg("-C")
            .arg(work_dir)
            .arg("remote")
            .arg("show")
            .arg("origin")
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .find(|l| l.contains("HEAD branch:"))
                    .map(|l| l.replace("HEAD branch:", ""))
                    .map(|s| s.trim().to_string())
            }
            _ => None,
        }
    }
}

// Struct to represent a repository
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Repo {
    pub name: String,
    pub path: String,
    pub work_dir: String,
    pub last_sha: Option<String>,
    pub target_branch: String,
    pub triggered: bool,
    pub ssh_key_path: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Repos {
    pub path: String,
    pub target_branch: Option<String>,
    pub ssh_key_path: Option<String>,
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
            ssh_key_path: None,
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
            ssh_key_path: None,
        }
    }

    pub fn prepare(&mut self) {
        // clone if not exist
        println!("Preparing {}", self.name);
        // Build SSH command, optionally with a specific identity file
        let ssh_cmd = match self
            .ssh_key_path
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            Some(key) => format!("ssh -o StrictHostKeyChecking=no -i {}", key),
            None => "ssh -o StrictHostKeyChecking=no".to_string(),
        };
        env::set_var("GIT_SSH_COMMAND", ssh_cmd);
        if !Path::new(&self.work_dir).exists()
            && fs::create_dir_all(Path::new(&self.work_dir)).is_ok()
        {
            println!("Created directory: {}", self.work_dir);
            println!("Cloning: {}", self.work_dir);
            self.clone_repo();
            self.get_default_branch();
        }

        // If target branch is empty (e.g., not specified in config), resolve via remote default now
        if self.target_branch.is_empty() {
            let git = SystemGitClient {};
            let _ = self.resolve_effective_branch_with("", &git);
        }

        let branch = self.target_branch.clone();
        self.last_sha = self.git_latest_sha(&branch);

    }

    // Resolve an effective branch based on a preferred branch name and remote state.
    // Behavior:
    // - If preferred is empty: use remote default if available.
    // - If preferred exists on remote: use it.
    // - Else: fall back to remote default if available.
    // - Update self.target_branch consistently when a choice is made.
    fn resolve_effective_branch_with(
        &mut self,
        preferred: &str,
        git: &dyn GitClient,
    ) -> Option<String> {
        let p = preferred.trim();

        if p.is_empty() {
            if let Some(def) = git.remote_default_branch(&self.work_dir) {
                println!(
                    "Using remote default branch '{}' for {}",
                    def, self.path
                );
                self.target_branch = def.clone();
                return Some(def);
            }
            return None;
        }

        if git.has_remote_branch(&self.work_dir, p) {
            if self.target_branch != p {
                println!("Using preferred branch '{}' for {}", p, self.path);
            }
            self.target_branch = p.to_string();
            return Some(p.to_string());
        }

        // Fallback to remote default
        if let Some(def) = git.remote_default_branch(&self.work_dir) {
            if def != p {
                // Expected fallback -> info-level style
                println!(
                    "Preferred branch '{}' not found; falling back to remote default '{}'",
                    p, def
                );
            }
            self.target_branch = def.clone();
            return Some(def);
        }

        // Both missing -> error
        eprintln!(
            "Neither preferred branch '{}' nor remote default branch exist for {}",
            p, self.path
        );
        None
    }

    pub async fn send_webhook(&self, message: String, repo: &Repo) {
        let title = repo
            .path
            .rsplit('/')
            .next()
            .unwrap_or(repo.path.as_str());

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
        if let Ok(custom_url) = env::var("CUSTOM_WEBHOOK_URL") {
            let webhook = Webhook::new(WebhookConfig::new(
                title,
                custom_url.as_str(),
                WebhookType::Custom,
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
                // Persist the new SHA first
                self.set_sha_by_repo(latest_sha.clone());

                // Ensure working copy is updated to the latest remote state for the target branch
                if let Err(e) = self.pull_branch() {
                    eprintln!(
                        "Failed to update working tree for {} on {}: {}",
                        self.path, self.target_branch, e
                    );
                }

                // Mark job running and trigger workflow processing
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
            let repo_name = self
                .path
                .rsplit('/')
                .next()
                .unwrap_or("")
                .to_string();
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
            eprintln!("Error during git fetch: {}", e);
            // Do not continue if fetch failed; prevent using potentially stale data
            return None;
        }

        // Resolve an effective branch using remote existence/defaults
        let git = SystemGitClient {};
        let resolved = self.resolve_effective_branch_with(branch, &git);
        let Some(effective) = resolved else {
            eprintln!(
                "Unable to resolve a branch for repo {}; neither preferred nor remote default exists",
                self.path
            );
            return None;
        };

        // Now safe to rev-parse origin/<effective>
        let cmd_desc = format!("git -C {} rev-parse origin/{}", &self.work_dir, effective);
        match Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("rev-parse")
            .arg(format!("origin/{}", effective))
            .output()
        {
            Ok(output) if output.status.success() => {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            }
            Ok(output) => {
                let code = output.status.code().unwrap_or(-1);
                eprintln!(
                    "git rev-parse failed (exit code {})\ncommand: {}\nstderr: {}\nstdout: {}",
                    code,
                    cmd_desc,
                    String::from_utf8_lossy(&output.stderr),
                    String::from_utf8_lossy(&output.stdout)
                );
                None
            }
            Err(err) => {
                eprintln!("Failed to execute {}: {}", cmd_desc, err);
                None
            }
        }
    }

    pub fn fetch_pull(&mut self) -> Result<(), anyhow::Error> {
        // Ensure we have a valid repo directory
        let git_dir = format!("{}/.git", self.work_dir);
        if !Path::new(&git_dir).exists() {
            // Attempt a fresh clone if the repo is missing
            self.clone_repo();
        }

        // Ensure the remote URL is correct (some environments may have stale/malformed origin)
        let current_url = Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .output();

        let needs_update = match current_url {
            Ok(o) if o.status.success() => {
                let url = String::from_utf8_lossy(&o.stdout).trim().to_string();
                url != self.path
            }
            _ => true, // no origin or failed; ensure it's set below
        };

        if needs_update {
            // Try to set the correct origin URL
            let _ = Command::new("git")
                .arg("-C")
                .arg(&self.work_dir)
                .arg("remote")
                .arg("remove")
                .arg("origin")
                .output();
            let set_out = Command::new("git")
                .arg("-C")
                .arg(&self.work_dir)
                .arg("remote")
                .arg("add")
                .arg("origin")
                .arg(&self.path)
                .output()?;
            if !set_out.status.success() {
                anyhow::bail!(
                    "failed to set origin: {}",
                    String::from_utf8_lossy(&set_out.stderr)
                );
            }
        }

        // Now fetch normally
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("fetch")
            .arg("--all")
            .arg("--prune")
            .output()?;
        if !out.status.success() {
            let code = out.status.code().unwrap_or(-1);
            let cmd_desc = format!("git -C {} fetch --all --prune", &self.work_dir);
            anyhow::bail!(
                "git fetch failed (exit code {})\ncommand: {}\nstderr: {}\nstdout: {}",
                code,
                cmd_desc,
                String::from_utf8_lossy(&out.stderr),
                String::from_utf8_lossy(&out.stdout)
            );
        }
        Ok(())
    }

    // Ensure local working copy reflects the latest remote state for the target branch
    pub fn pull_branch(&mut self) -> Result<(), anyhow::Error> {
        // Resolve branch to pull (preferred target_branch or remote default)
        let git = SystemGitClient {};
        let preferred = self.target_branch.clone();
        let resolved = self.resolve_effective_branch_with(&preferred, &git);
        let branch = match resolved {
            Some(b) => b,
            None => {
                eprintln!(
                    "Unable to resolve branch to pull for {}; neither preferred nor remote default exists",
                    self.path
                );
                return Ok(());
            }
        };

        // Always fetch first to ensure remote refs are up to date
        let _ = self.fetch_pull();

        // Ensure the local branch exists and is set to track the remote branch
        let checkout_output = Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("checkout")
            .arg("-B")
            .arg(&branch)
            .arg(format!("origin/{}", &branch))
            .output()?;
        if !checkout_output.status.success() {
            eprintln!(
                "git checkout -B failed for {}/{}\nstderr: {}\nstdout: {}",
                self.path,
                branch,
                String::from_utf8_lossy(&checkout_output.stderr),
                String::from_utf8_lossy(&checkout_output.stdout)
            );
        }

        // Try a fast-forward pull; if it fails, hard reset to remote
        let pull_output = Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("pull")
            .arg("--ff-only")
            .arg("origin")
            .arg(&branch)
            .output()?;
        if !pull_output.status.success() {
            eprintln!(
                "git pull --ff-only failed for {}/{} (will try hard reset)\nstderr: {}\nstdout: {}",
                self.path,
                branch,
                String::from_utf8_lossy(&pull_output.stderr),
                String::from_utf8_lossy(&pull_output.stdout)
            );

            // Fallback: hard reset to the remote branch tip
            let reset_output = Command::new("git")
                .arg("-C")
                .arg(&self.work_dir)
                .arg("reset")
                .arg("--hard")
                .arg(format!("origin/{}", &branch))
                .output()?;
            if !reset_output.status.success() {
                let code = reset_output.status.code().unwrap_or(-1);
                anyhow::bail!(
                    "git reset --hard origin/{} failed (exit code {})\nstderr: {}\nstdout: {}",
                    branch,
                    code,
                    String::from_utf8_lossy(&reset_output.stderr),
                    String::from_utf8_lossy(&reset_output.stdout)
                );
            }
        }

        println!("Updated working copy for {} on branch {}", self.path, branch);
        Ok(())
    }

    pub fn write_repo_to_config(&mut self) {
        let name = self.path.rsplit('/').next().unwrap();
        let config_entry = format!(
            "[{}]\n\
    path = \"{}\"\n\
    target_branch = \"{}\"\n\
    ssh_key_path = \"{}\"\n\n\
    ",
            name,
            self.path,
            self.target_branch,
            self.ssh_key_path.clone().unwrap_or_default()
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
        // Determine parent directory robustly
        let parent = Path::new(&self.work_dir)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Path::new("/").to_path_buf());

        if let Err(e) = std::fs::create_dir_all(&parent) {
            eprintln!("Failed to create directory {}: {}", parent.display(), e);
        }
        match Command::new("git")
            .arg("-C")
            .arg(&parent)
            .arg("clone")
            .arg(&self.path)
            .arg(&self.work_dir) // explicitly clone into target dir
            .output()
        {
            Ok(output) => {
                let git_repo_path = format!("{}/.git", self.work_dir);
                if output.status.success() && Path::new(&git_repo_path).exists() {
                    let code = output.status.code().unwrap_or(0);
                    println!("git clone succeeded (exit code {}): {}", code, self.path);
                } else {
                    let code = output.status.code().unwrap_or(-1);
                    eprintln!(
                        "git clone failed (exit code {}) for {}\ncommand: git -C {} clone {} {}\nstderr: {}\nstdout: {}",
                        code,
                        self.path,
                        parent.display(),
                        self.path,
                        self.work_dir,
                        String::from_utf8_lossy(&output.stderr),
                        String::from_utf8_lossy(&output.stdout)
                    );
                    let _ = fs::remove_dir_all(Path::new(&self.work_dir));
                }
            }
            Err(err) => {
                eprintln!(
                    "Failed to execute git clone for {}: {}\ncommand: git -C {} clone {} {}",
                    self.path,
                    err,
                    parent.display(),
                    self.path,
                    self.work_dir
                );
                let _ = fs::remove_dir_all(Path::new(&self.work_dir));
            }
        }
    }

    fn get_default_branch(&mut self) -> String {
        let mut head_branch = "master".to_string();

        let cmd_desc = format!("git -C {} remote show origin", &self.work_dir);
        match Command::new("git")
            .arg("-C")
            .arg(&self.work_dir)
            .arg("remote")
            .arg("show")
            .arg("origin")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(s) = stdout
                        .lines()
                        .find(|l| l.contains("HEAD branch:"))
                        .map(|l| l.replace("HEAD branch:", ""))
                    {
                        head_branch = s.trim().to_string();
                        // Informational log only; not an error
                        println!("Resolved remote default branch: {}", head_branch);
                    }
                } else {
                    let code = output.status.code().unwrap_or(-1);
                    eprintln!(
                        "git remote show origin failed (exit code {})\ncommand: {}\nstderr: {}\nstdout: {}",
                        code,
                        cmd_desc,
                        String::from_utf8_lossy(&output.stderr),
                        String::from_utf8_lossy(&output.stdout)
                    );
                }
            }
            Err(err) => {
                eprintln!("Failed to execute {}: {}", cmd_desc, err);
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
                    // When not specified in config, leave empty so we can resolve remote default later
                    target_branch: r.1
                        .to_owned()
                        .target_branch
                        .unwrap_or("".to_string()),
                    triggered: false,
                    ssh_key_path: r.1.ssh_key_path.clone().and_then(|s| {
                        let t = s.trim().to_string();
                        if t.is_empty() { None } else { Some(t) }
                    }),
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
## Example repo configuration
##[sys-compare]
##path = "git@github.com:helloimalemur/sys-compare"
##target_branch = "main"  # Optional; if omitted, PhantomCI will use the remote's default branch
##ssh_key_path = "/home/youruser/.ssh/id_ed25519"  # Optional: specify a custom SSH key

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
        .rsplit('/')
        .next()
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockGitClient {
        present: Vec<String>,
        default: Option<String>,
    }

    impl GitClient for MockGitClient {
        fn has_remote_branch(&self, _work_dir: &str, branch: &str) -> bool {
            self.present.iter().any(|b| b == branch)
        }
        fn remote_default_branch(&self, _work_dir: &str) -> Option<String> {
            self.default.clone()
        }
    }

    fn dummy_repo() -> Repo {
        Repo {
            name: "test".into(),
            path: "git@example.com:org/repo.git".into(),
            work_dir: "/tmp/phantom_ci-test".into(),
            last_sha: None,
            target_branch: "".into(),
            triggered: false,
            ssh_key_path: None,
        }
    }

    #[test]
    fn resolver_uses_preferred_when_exists() {
        let mut repo = dummy_repo();
        repo.target_branch = "stale".into();
        let git = MockGitClient {
            present: vec!["feature".into()],
            default: Some("main".into()),
        };
        let res = repo.resolve_effective_branch_with("feature", &git);
        assert_eq!(res.as_deref(), Some("feature"));
        assert_eq!(repo.target_branch, "feature");
    }

    #[test]
    fn resolver_falls_back_to_default_when_preferred_missing() {
        let mut repo = dummy_repo();
        let git = MockGitClient {
            present: vec![],
            default: Some("main".into()),
        };
        let res = repo.resolve_effective_branch_with("feature", &git);
        assert_eq!(res.as_deref(), Some("main"));
        assert_eq!(repo.target_branch, "main");
    }

    #[test]
    fn resolver_none_when_both_missing() {
        let mut repo = dummy_repo();
        let git = MockGitClient {
            present: vec![],
            default: None,
        };
        let res = repo.resolve_effective_branch_with("feature", &git);
        assert!(res.is_none());
        // target_branch should remain unchanged (empty)
        assert_eq!(repo.target_branch, "");
    }

    #[test]
    fn resolver_uses_default_when_preferred_empty() {
        let mut repo = dummy_repo();
        let git = MockGitClient {
            present: vec![],
            default: Some("main".into()),
        };
        let res = repo.resolve_effective_branch_with("", &git);
        assert_eq!(res.as_deref(), Some("main"));
        assert_eq!(repo.target_branch, "main");
    }
}
