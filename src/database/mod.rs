use crate::util::default_config_path;
use anyhow::Error;
use rusqlite::{params, Connection};
use std::path::Path;

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

impl Job {
    fn write(&mut self) {
        let connection = SqliteConnection::new();
        let conn = connection.unwrap().conn;

        match conn.execute(
            "INSERT INTO jobs (description, status, priority, created_at, updated_at, start_time, finish_time, error_message, result) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![self.description, self.status, self.priority, self.created_at, self.updated_at, self.start_time, self.finish_time, self.error_message, self.result],
        ) {
            Ok(_) => (
                println!("success")
                ),
            Err(error) => {
                println!("{}", error)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobLog {
    pub id: i32,
    pub job_id: i32,
    pub log_message: String,
    pub logged_at: String,
}

pub struct SqliteConnection {
    pub conn: Connection,
}

impl SqliteConnection {
    pub fn new() -> Result<SqliteConnection, Error> {
        let config_dir = default_config_path().unwrap();

        if let Err(e) = load_env_variables(&config_dir) {
            eprintln!("environment variables not loaded: {}", e);
        }

        let conn = Connection::open(sqlite_path)?;

        Ok(SqliteConnection { conn })
    }

    pub fn setup_schema(&mut self) -> Result<(), anyhow::Error> {
        if let Err(e) = self.conn.execute(
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

        if let Err(e) = self.conn.execute(
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
}

fn load_env_variables(path: &str) -> rusqlite::Result<(), dotenv::Error> {
    let env_path = format!("{}.env", path);
    dotenv::from_path(Path::new(&env_path)).map(|_| println!("env: {}", env_path))
}

#[cfg(test)]
mod tests {
    use crate::database::Job;
    use crate::util::default_config_path;

    #[test]
    fn test_insert() {
        let config_dir = default_config_path().unwrap();
        let sqlite_path = format!("{}{}", config_dir, "db.sqlite");
        println!("SQLite database path: {}", sqlite_path);

        let mut job = Job {
            id: 0,
            description: "".to_string(),
            status: "".to_string(),
            priority: 0,
            created_at: "".to_string(),
            updated_at: "".to_string(),
            start_time: "".to_string(),
            finish_time: "".to_string(),
            error_message: "".to_string(),
            result: "".to_string(),
        };

        job.write();
    }
}
