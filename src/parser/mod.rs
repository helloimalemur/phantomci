use crate::database::job::Job;
use crate::database::joblog::JobLog;
use crate::repo::Repo;
use chrono::Local;
use config::Config;
use log::{error, info, warn};
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use whoami::hostname;

// Public entry point: parse workflow and run steps sequentially with fail-fast semantics
pub async fn parse_workflow(file_path: &str, repo: Repo, tx_clone: Sender<String>) {
    let host = hostname().unwrap_or_default();
    let starting_message = format!(
        "Starting workflow for {} [{}] on {}",
        repo.path, repo.target_branch, host
    );
    info!("{}", starting_message);
    println!("{}", starting_message);
    repo.send_webhook(starting_message.clone(), &repo).await;

    // Load workflow commands and order by numeric key
    let commands = get_command_from_config(file_path.to_string());
    let mut ordered = BTreeMap::<usize, WorkflowCommand>::new();
    for (k, v) in commands.into_iter() {
        match k.parse::<usize>() {
            Ok(i) => {
                ordered.insert(i, v);
            }
            Err(_) => {
                warn!("Ignoring non-numeric workflow key '{}' in {}", k, file_path);
            }
        }
    }

    if ordered.is_empty() {
        let msg = format!(
            "No workflow steps found in {} for {}:{}",
            file_path, repo.path, repo.target_branch
        );
        warn!("{}", msg);
        println!("{}", msg);
        Job::update_status(repo.path.clone(), repo.target_branch.clone(), "failed".to_string());
        Job::update_finished_time(repo.path.clone(), repo.target_branch.clone());
        let mut log = JobLog { id: 0, repo: repo.path.clone(), log_message: msg.clone(), logged_at: Local::now().to_rfc3339() };
        log.add_job_log();
        repo.send_webhook(msg, &repo).await;
        return;
    }

    let workflow_start = Instant::now();
    let mut all_ok = true;

    for (idx, cmd) in ordered.into_iter() {
        let step_desc = format!("[step {}] {}", idx, cmd.run);
        info!("Running {} in {}", step_desc, repo.work_dir);
        println!("Running {} on {}", step_desc, host);

        let t0 = Instant::now();
        let mut root_iter = cmd.run.split_whitespace();
        let program = match root_iter.next() {
            Some(p) => p.to_string(),
            None => {
                let msg = format!("Invalid empty command at step {}", idx);
                error!("{}", msg);
                let mut log = JobLog { id: 0, repo: repo.path.clone(), log_message: msg.clone(), logged_at: Local::now().to_string() };
                log.add_job_log();
                all_ok = false;
                break;
            }
        };
        let args: Vec<String> = root_iter.map(|s| s.to_string()).collect();

        let output_res = Command::new(&program)
            .args(&args)
            .current_dir(&repo.work_dir)
            .output()
            .await;

        let dt = t0.elapsed();

        match output_res {
            Ok(output) => {
                let success = output.status.success();
                let code = output.status.code();
                let stdout_s = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr_s = String::from_utf8_lossy(&output.stderr).to_string();

                let preview = |s: &str| -> String {
                    const LIM: usize = 4000; // keep logs reasonable
                    if s.len() > LIM { format!("{}…", &s[..LIM]) } else { s.to_string() }
                };

                let msg = if success {
                    format!(
                        "✅ {} succeeded in {:.2?} (code {:?})\nstdout:\n{}",
                        step_desc,
                        dt,
                        code,
                        preview(&stdout_s)
                    )
                } else {
                    format!(
                        "❌ {} failed in {:.2?} (code {:?})\nstdout:\n{}\nstderr:\n{}",
                        step_desc,
                        dt,
                        code,
                        preview(&stdout_s),
                        preview(&stderr_s)
                    )
                };

                let mut log = JobLog { id: 0, repo: repo.path.clone(), log_message: msg.clone(), logged_at: Local::now().to_rfc3339() };
                log.add_job_log();
                // Best-effort channel message
                let _ = tx_clone.send(msg.clone()).await;
                // Send webhook per-step only on failure to reduce noise
                if !success {
                    repo.send_webhook(msg.clone(), &repo).await;
                }

                if !success {
                    all_ok = false;
                    break;
                }
            }
            Err(e) => {
                let msg = format!(
                    "❌ {} failed to start in {:.2?}: {}",
                    step_desc, dt, e
                );
                error!("{}", msg);
                let mut log = JobLog { id: 0, repo: repo.path.clone(), log_message: msg.clone(), logged_at: Local::now().to_rfc3339() };
                log.add_job_log();
                let _ = tx_clone.send(msg.clone()).await;
                repo.send_webhook(msg.clone(), &repo).await;
                all_ok = false;
                break;
            }
        }
    }

    // Finalize job status and logging
    let total = workflow_start.elapsed();
    if all_ok {
        let msg = format!(
            "✅ Workflow completed successfully for {}:{} in {:.2?}",
            repo.path, repo.target_branch, total
        );
        info!("{}", msg);
        println!("{}", msg);
        Job::update_status(repo.path.clone(), repo.target_branch.clone(), "success".to_string());
        Job::update_finished_time(repo.path.clone(), repo.target_branch.clone());
        let mut log = JobLog { id: 0, repo: repo.path.clone(), log_message: msg.clone(), logged_at: Local::now().to_rfc3339() };
        log.add_job_log();
        repo.send_webhook(msg, &repo).await;
    } else {
        let msg = format!(
            "❌ Workflow failed for {}:{} after {:.2?}",
            repo.path, repo.target_branch, total
        );
        error!("{}", msg);
        println!("{}", msg);
        Job::update_status(repo.path.clone(), repo.target_branch.clone(), "failed".to_string());
        Job::update_finished_time(repo.path.clone(), repo.target_branch.clone());
        let mut log = JobLog { id: 0, repo: repo.path.clone(), log_message: msg.clone(), logged_at: Local::now().to_rfc3339() };
        log.add_job_log();
        repo.send_webhook(msg, &repo).await;
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct WorkflowCommand {
    run: String,
}

fn get_command_from_config(path: String) -> HashMap<String, WorkflowCommand> {
    let empty = HashMap::<String, WorkflowCommand>::new();
    if let Ok(config_file) = Config::builder()
        .add_source(config::File::with_name(&path))
        .build()
    {
        if let Ok(map) = config_file.try_deserialize::<HashMap<String, WorkflowCommand>>() {
            map
        } else {
            empty
        }
    } else {
        empty
    }
}
