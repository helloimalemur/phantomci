use crate::repo::Repo;
use config::Config;
use std::collections::{BTreeMap, HashMap};
use std::{fs, process};
use whoami::hostname;
use crate::app::default_config_path;
use crate::webhook::{Webhook, WebhookConfig, WebhookType};

// Parse the workflow file
pub async fn parse_workflow(file_path: &str, repo: Repo) {
    if let Ok(content) = fs::read_to_string(file_path) {
        let starting_message = format!("Starting Workflow: {}\nTarget Branch: {}", repo.path, repo.target_branch);
        println!("{}", starting_message);
        repo.send_webhook(starting_message, &repo).await;
        // println!("Parsed workflow file: \n{}", content);
        let commands = get_command_from_config((&file_path).to_string());
        let mut map = BTreeMap::<usize, WorkflowCommand>::new();
        for i in commands {
            map.insert(i.0.parse::<usize>().unwrap(), i.1.to_owned());
        }
        // use btreemap to sort commands
        map.iter()
            .for_each(|e| run_command(repo.to_owned(), e.1.to_owned()));
        println!("========================================================");
        println!("========================================================");
    } else {
        eprintln!("Failed to read workflow file at {}", file_path);
    }
}

fn run_command(repo: Repo, command: WorkflowCommand) {
    let root = command.run.split(' ').collect::<Vec<&str>>();
    let args = root[1..].to_vec();
    println!(
        "Running on {}:\n{}",
        hostname().unwrap_or_default(),
        command.run
    );
    if let Ok(o) = process::Command::new(root[0])
        .args(args)
        .current_dir(repo.work_dir.to_string())
        .spawn()
    {
        if let Ok(a) = o.wait_with_output() {
            println!("{}", String::from_utf8_lossy(&a.stdout));
        }
        // if let Some(stdout) = o.stdout {
        //     let mut reader = BufReader::new(stdout);
        //     let then = Local::now();
        //     let mut sec_past = 0i64;
        //     while !reader.buffer().is_empty() && sec_past > 10 {
        //         sec_past = Local::now().signed_duration_since(then).num_seconds();
        //         // println!("{}",
        //         // for (a,e) in reader.buffer().iter().enumerate() {
        //         //     println!("{}", e.to_string())
        //         // }
        //         for (ind, line) in BufRead::lines(reader.buffer()).enumerate() {
        //             if let Ok(li) =  line {
        //                 println!("{}", li)
        //             }
        //         }
        //     }
        // }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct WorkflowCommand {
    run: String,
}

fn get_command_from_config(path: String) -> HashMap<String, WorkflowCommand> {
    let mut workflow_commands = HashMap::<String, WorkflowCommand>::new();
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
