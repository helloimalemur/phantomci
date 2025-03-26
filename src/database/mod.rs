use std::path::Path;
use anyhow::Error;
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: i32,
    pub description: String,
    pub status: String,
    pub priority: i32,
    pub created_at: String,
    pub updated_at: String,
    pub start_time: String,
    pub finish_time: String,
    pub error_message: String,
    pub result: String,
}

#[derive(Debug, Clone)]
pub struct JobLog {
    pub id: i32,
    pub job_id: i32,
    pub log_message: String,
    pub logged_at: String,
}

fn load_env_variables(path: &str) -> rusqlite::Result<(), dotenv::Error> {
    let env_path = format!("{}.env", path);
    dotenv::from_path(Path::new(&env_path)).map(|_| println!("env: {}", env_path))
}

pub fn create_connection(config_dir: String) -> Result<Connection, Error> {
    let sqlite_path = format!("{}/{}", config_dir, "db.sqlite");
    if let Err(e) = load_env_variables(&config_dir) {
        eprintln!("environment variables not loaded: {}", e);
    }
    if let Ok(conn) = Connection::open(&sqlite_path) {
        if let Err(e) = setup_schema(&conn) {
            eprintln!("Failed to setup schema: {:?}", e);
        }
        Ok(conn)
    } else {
        anyhow::bail!("Failed to connect to database");
    }
}

pub fn setup_schema(db: &Connection) -> Result<(), anyhow::Error> {
    if let Err(e) = db.execute(
        "CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            description TEXT NOT NULL,           -- Description of the job (e.g., build commands, target project)
            status TEXT NOT NULL,                -- Job status: 'pending', 'running', 'success', 'failed'
            priority INTEGER DEFAULT 0,          -- Optional priority field for scheduling decisions
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,  -- When the job was created
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,  -- Last update timestamp
            start_time DATETIME,                 -- When the job execution started
            finish_time DATETIME,                -- When the job execution ended
            error_message TEXT,                  -- Error message if the job failed
            result TEXT                        -- Any result output or summary from the job execution
        )",
        (),
    ) {
        eprintln!("Error: {}", e);
    } else {
        println!("Table Created: jobs");
    }

    if let Err(e) = db.execute(
        "CREATE TABLE IF NOT EXISTS job_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            job_id INTEGER NOT NULL,             -- Foreign key reference to the jobs table
            log_message TEXT NOT NULL,           -- Log message content
            logged_at DATETIME DEFAULT CURRENT_TIMESTAMP,  -- When this log was recorded
            FOREIGN KEY(job_id) REFERENCES jobs(id) ON DELETE CASCADE
        )",
        (),
    ) {
        eprintln!("Error: {}", e);
    } else {
        println!("Table Created: job_logs");
    }

    Ok(())
}
