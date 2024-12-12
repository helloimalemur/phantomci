use crate::repo::Repo;
use config::Config;
use std::collections::{BTreeMap, HashMap};
use std::process;
use whoami::hostname;

// Parse the workflow file
pub async fn parse_workflow(file_path: &str, repo: Repo) {
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
    // use btreemap to sort commands
    for e in map.iter() {
        run_command(repo.to_owned(), e.1.to_owned()).await
    }

    println!("========================================================");
    println!("========================================================");
}

async fn run_command(repo: Repo, command: WorkflowCommand) {
    let root = command.run.split(' ').collect::<Vec<&str>>();
    let args = root[1..].to_vec();
    println!(
        "Running on {}:\n{}",
        hostname().unwrap_or_default(),
        command.run
    );
    if let Ok(o) = process::Command::new(root[0])
        .args(args)
        .current_dir(&repo.work_dir)
        .spawn()
    {
        if let Ok(a) = o.wait_with_output() {
            let output = String::from_utf8_lossy(&a.stdout);
            println!("{}", &output);
            repo.send_webhook(output.to_string(), &repo).await // troubleshooting
        }
    }
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
