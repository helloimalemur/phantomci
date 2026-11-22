use crate::database::job::Job;
use crate::database::joblog::JobLog;
use crate::repo::{parse_timeout_secs, Repo};
use chrono::Local;
use config::Config;
use log::{error, info, warn};
use std::collections::{BTreeMap, HashMap};
use std::process::{Output, Stdio};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use whoami::hostname;

/// Upper bound for one workflow step. Override with PHANTOMCI_STEP_TIMEOUT_SECS.
/// Steps run one after another for every repo, so a step that never exits (for example a
/// download that stalls inside a `docker build`) would otherwise stop all builds.
const DEFAULT_STEP_TIMEOUT_SECS: u64 = 7200;

fn step_timeout() -> Duration {
    parse_timeout_secs(
        std::env::var("PHANTOMCI_STEP_TIMEOUT_SECS").ok().as_deref(),
        DEFAULT_STEP_TIMEOUT_SECS,
    )
}

/// Like `Command::output()`, but after `timeout` kills the step and everything it started,
/// then returns an `ErrorKind::TimedOut` error.
async fn run_step(cmd: &mut Command, timeout: Duration) -> std::io::Result<Output> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    // Own process group, so a timeout also reaches the processes the step started (such as
    // a `docker build` client, whose exit cancels the build). They inherit the output pipes,
    // so leaving them alive could keep the runner waiting.
    #[cfg(unix)]
    cmd.process_group(0);
    let child = cmd.spawn()?;
    let pid = child.id();
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(res) => res,
        Err(_) => {
            // Dropping the timed-out future already killed the step itself (kill_on_drop).
            #[cfg(unix)]
            if let Some(pid) = pid {
                // SAFETY: kill(2) with a negative pid signals that process group. The kernel
                // does not reuse an id while a group with that id still has members.
                unsafe {
                    libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
                }
            }
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!(
                    "timed out after {}s; killed it and the processes it started",
                    timeout.as_secs()
                ),
            ))
        }
    }
}

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

        let output_res = run_step(
            Command::new(&program)
                .args(&args)
                .current_dir(&repo.work_dir),
            step_timeout(),
        )
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
                let msg = if e.kind() == std::io::ErrorKind::TimedOut {
                    format!("❌ {} {}", step_desc, e)
                } else {
                    format!("❌ {} failed to start in {:.2?}: {}", step_desc, dt, e)
                };
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_step_returns_output_of_fast_steps() {
        let out = run_step(
            Command::new("sh")
                .arg("-c")
                .arg("echo out; echo err >&2; exit 3"),
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(out.status.code(), Some(3));
        assert_eq!(String::from_utf8_lossy(&out.stdout), "out\n");
        assert_eq!(String::from_utf8_lossy(&out.stderr), "err\n");
    }

    #[tokio::test]
    async fn run_step_kills_hung_step_and_its_children() {
        // The background `sleep` stands in for a `docker build` client the step started: it
        // holds the output pipes, and the timeout must still end the step on time.
        let pid_file =
            std::env::temp_dir().join(format!("phantomci-step-test-{}", std::process::id()));
        let start = Instant::now();
        let err = run_step(
            Command::new("sh").arg("-c").arg(format!(
                "sleep 300 & echo $! > '{}'; sleep 300",
                pid_file.display()
            )),
            Duration::from_millis(500),
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "{:?}",
            start.elapsed()
        );

        let child_pid = std::fs::read_to_string(&pid_file).unwrap();
        let _ = std::fs::remove_file(&pid_file);
        tokio::time::sleep(Duration::from_millis(200)).await;
        // Gone, or a zombie waiting to be reaped by init: either way no longer running.
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", child_pid.trim()));
        if let Ok(stat) = stat {
            let state = stat.rsplit(')').next().unwrap_or("").trim_start();
            assert!(
                state.starts_with('Z'),
                "child of the timed-out step still running: {stat}"
            );
        }
    }
}
