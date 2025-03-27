use crate::parser::parse_workflow;
use crate::repo::Repo;
use crate::util::{default_config_path, default_repo_work_path};
use chrono::Local;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio::time::interval;
use crate::app::state::AppState;

// Fetch the latest commit hash for a given repository
pub fn fetch_latest_sha(repo: &Repo) -> Option<String> {
    if let Err(e) = fetch_pull(repo) {
        eprintln!("Error: {}", e)
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(&repo.work_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => {
            eprintln!("Error: scm polling error: {}", repo.name);
            None
        }
    }
}

pub fn fetch_pull(repo: &Repo) -> Result<(), anyhow::Error> {
    Command::new("git")
        .arg("-C")
        .arg(&repo.work_dir)
        .arg("stash")
        .output()?;

    Command::new("git")
        .arg("-C")
        .arg(&repo.work_dir)
        .arg("checkout")
        .arg(&repo.target_branch)
        .output()?;

    Command::new("git")
        .arg("-C")
        .arg(&repo.work_dir)
        .arg("reset")
        .arg("--hard")
        .arg("HEAD")
        .output()?;

    Command::new("git")
        .arg("-C")
        .arg(&repo.work_dir)
        .arg("fetch")
        .output()?;

    Command::new("git")
        .arg("-C")
        .arg(&repo.work_dir)
        .arg("pull")
        .output()?;
    Ok(())
}

// Check for changes in a repository and update repo
