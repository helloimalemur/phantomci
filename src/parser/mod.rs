use crate::database::joblog::JobLog;
use crate::repo::Repo;
use chrono::Local;
use config::Config;
use std::collections::{BTreeMap, HashMap};
use std::process;
use tokio::sync::mpsc::Sender;
use whoami::hostname;

pub async fn parse_workflow(file_path: &str, repo: Repo, tx_clone: Sender<String>) {
    let starting_message = format!(
        "Starting Workflow: {}\nTarget Branch: {}\nHost: {}",
        repo.path,
        repo.target_branch,
        hostname().unwrap_or_default()
    );
    println!("{}", starting_message);
    repo.send_webhook(starting_message, &repo).await;

    let commands = get_command_from_config((&file_path).to_string());
    let mut map = BTreeMap::<usize, WorkflowCommand>::new();
    for i in commands {
        map.insert(i.0.parse::<usize>().unwrap(), i.1.to_owned());
    }

    for e in map.iter() {
        run_command(repo.to_owned(), e.1.to_owned(), tx_clone.clone()).await
    }

    println!("========================================================");
    println!("========================================================");
}

async fn run_command(repo: Repo, command: WorkflowCommand, tx_clone: Sender<String>) {
    let cmd = command.clone();

    tokio::spawn(async move {
        let mut output = String::new();
        let root = cmd.run.split(' ').collect::<Vec<&str>>();
        let args = root[1..].to_vec();
        println!(
            "Running on {}:\n{}",
            hostname().unwrap_or_default(),
            command.run
        );
        if let Ok(o) = process::Command::new(root[0])
            .args(args)
            .current_dir(&repo.work_dir)
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
        {
            if let Ok(a) = o.wait_with_output() {
                let message = format!("{} :: {}", repo.name, String::from_utf8_lossy(&a.stderr));
                output = message;
            }
        }

        let mut log = JobLog {
            id: 0,
            repo: repo.path.clone(),
            log_message: output.to_string(),
            logged_at: Local::now().to_rfc3339(),
        };
        log.add_job_log();
        tx_clone.send(output.clone()).await.unwrap();
        repo.send_webhook(output.to_string(), &repo).await // troubleshooting
    });
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct WorkflowCommand {
    run: String,
}

fn get_command_from_config(path: String) -> HashMap<String, WorkflowCommand> {
    let workflow_commands = HashMap::<String, WorkflowCommand>::new();
    if let Ok(config_file) = Config::builder()
        .add_source(config::File::with_name(&path))
        .build()
    {
        if let Ok(map) = config_file.try_deserialize::<HashMap<String, WorkflowCommand>>() {
            map
        } else {
            workflow_commands
        }
    } else {
        workflow_commands
    }
}
