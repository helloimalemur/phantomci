use crate::app::{default_repo_work_path, AppState};
use crate::parser::parse_workflow;
use crate::repo::Repo;
use chrono::Local;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio::time::interval;

// Fetch the latest commit hash for a given repository
pub fn fetch_latest_sha(repo: &Repo) -> Option<String> {
    fetch_pull(repo);
    let output = Command::new("git")
        .arg("-C")
        .arg(repo.work_dir.to_string())
        .arg("rev-parse")
        .arg("HEAD")
        .output();
    // println!("{:?}", output);
    match output {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    }
}

pub fn fetch_pull(repo: &Repo) {
    if let Err(output) = Command::new("git")
        .arg("-C")
        .arg(repo.work_dir.to_string())
        .arg("stash")
        .output()
    {
        eprintln!("ERROR: {}", &output.to_string())
    }

    if let Err(output) = Command::new("git")
        .arg("-C")
        .arg(repo.work_dir.to_string())
        .arg("checkout")
        .arg(repo.target_branch.to_string())
        .output()
    {
        eprintln!("ERROR: {}", &output.to_string())
    }

    if let Err(output) = Command::new("git")
        .arg("-C")
        .arg(repo.work_dir.to_string())
        .arg("reset")
        .arg("--hard")
        .arg("HEAD")
        .output()
    {
        eprintln!("ERROR: {}", &output.to_string())
    }

    if let Err(output) = Command::new("git")
        .arg("-C")
        .arg(repo.work_dir.to_string())
        .arg("fetch")
        .output()
    {
        eprintln!("ERROR: {}", &output.to_string())
    }

    if let Err(output) = Command::new("git")
        .arg("-C")
        .arg(repo.work_dir.to_string())
        .arg("pull")
        .output()
    {
        eprintln!("ERROR: {}", &output.to_string())
    }
}

// Check for changes in a repository and handle them
fn check_repo_changes(repo: &mut Repo) {
    if let Some(latest_sha) = fetch_latest_sha(&repo) {
        if repo.last_sha.as_ref() != Some(&latest_sha) {
            println!("========================================================");
            println!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
            println!(
                "Change detected in repo: {}\nNew SHA: {}",
                repo.path, latest_sha
            );

            repo.last_sha = Some(latest_sha.to_owned());

            repo.triggered = true;

            println!("========================================================");
        }
    } else {
        eprintln!("Failed to fetch latest SHA for repo at {}", repo.path);
    }
}

// Check for changes in a repository and handle them
async fn check_repo_triggered(repo: &mut Repo) {
    if repo.triggered {
        // Parse workflow file
        // let workflow_path = Path::new(&repo.path).join(&repo.workflow_file);
        repo.triggered = false;
        let wp = format!(
            "{}workflow.toml",
            default_repo_work_path(repo.path.split('/').last().unwrap().to_string())
        );
        let workflow_path = Path::new(&wp);
        if workflow_path.exists() {
            parse_workflow(workflow_path.to_str().unwrap(), repo.to_owned()).await;
        } else {
            eprintln!(
                "Workflow file not found at {}",
                workflow_path.to_str().unwrap()
            );
        }
    }
}

// Main polling logic
pub async fn poll_repos(state: AppState, interval_duration: Duration) {
    let mut ticker = interval(interval_duration);

    loop {
        ticker.tick().await;
        let mut repos = state.repos.lock().unwrap().to_owned();
        for (_, repo) in repos.iter_mut() {
            check_repo_changes(repo);
            check_repo_triggered(repo).await
        }
        state.repos.lock().unwrap().clone_from(&repos);
        state.save_state();
    }
}
