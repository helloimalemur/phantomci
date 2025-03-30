use anyhow::Error;
use rusqlite::Connection;
use crate::util::default_sqlite_path;

pub mod job;
pub mod joblog;


pub struct SqliteConnection {
    pub conn: Connection,
}

impl SqliteConnection {
    pub fn new() -> Result<SqliteConnection, Error> {
        let conn = if let Ok(sqlite_path) = default_sqlite_path() {
            Connection::open(sqlite_path)?
        } else {
            panic!("unable to connect to database")
        };

        let mut sqlite = SqliteConnection { conn };

        if let Err(e) = sqlite.setup_schema() {
            eprintln!("failed to setup schema: {}", e);
        }

        Ok(sqlite)
    }

    pub fn setup_schema(&mut self) -> Result<(), anyhow::Error> {
        if let Err(e) = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo TEXT NOT NULL,           -- Repo of the job (target project)
            status TEXT NOT NULL,                -- Job status: 'pending', 'running', 'success', 'failed'
            priority INTEGER DEFAULT 0,          -- Optional priority field for scheduling decisions
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,  -- When the job was created
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,  -- Last update timestamp
            start_time DATETIME,                 -- When the job execution started
            finish_time DATETIME,                -- When the job execution ended
            error_message TEXT,                  -- Error message if the job failed
            result TEXT,                        -- Any result output or summary from the job execution
            sha TEXT                        -- Any result output or summary from the job execution
        )",
            (),
        ) {
            eprintln!("Error: {}", e);
        }

        if let Err(e) = self.conn.execute(
            "CREATE TABLE IF NOT EXISTS job_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo INTEGER NOT NULL,             -- Foreign key reference to the jobs table
            log_message TEXT NOT NULL,           -- Log message content
            logged_at DATETIME DEFAULT CURRENT_TIMESTAMP,  -- When this log was recorded
            FOREIGN KEY(repo) REFERENCES jobs(repo) ON DELETE CASCADE
        )",
            (),
        ) {
            eprintln!("Error: {}", e);
        }

        Ok(())
    }
}