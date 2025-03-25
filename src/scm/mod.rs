use crate::app::AppState;
use crate::parser::parse_workflow;
use crate::repo::Repo;
use crate::util::default_repo_work_path;
use chrono::Local;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio::time::interval;

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
fn check_repo_changes(repo: &mut Repo) {
    println!("Checking repo changes... \n {}", &repo.name);
    if let Some(latest_sha) = fetch_latest_sha(repo) {
        // check sqlite
        // last sha
        if repo.last_sha.as_ref() != Some(&latest_sha) {
            println!("========================================================");
            println!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
            println!(
                "Change detected in repo: {}\nNew SHA: {}",
                repo.path, latest_sha
            );

            repo.last_sha = Some(latest_sha.to_owned());
            // write sha

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
            "{}workflow/{}.toml",
            default_repo_work_path(repo.path.split('/').last().unwrap().to_string()).unwrap(),
            repo.target_branch
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
pub async fn poll_repos(mut state: AppState, interval_duration: Duration) {
    let mut ticker = interval(interval_duration);

    loop {
        ticker.tick().await;
        let mut repos = state.repos.lock().unwrap().to_owned();
        for (_, repo) in repos.iter_mut() {
            check_repo_changes(repo);
            check_repo_triggered(repo).await
        }
        state.repos.lock().unwrap().clone_from(&repos);
        // state.add_repos_from_config();
        // state.save_state();
    }
}
