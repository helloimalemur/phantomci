use crate::util::default_sqlite_path;
use anyhow::Error;
use rusqlite::{params, Connection};
use std::path::Path;
use rusqlite::fallible_iterator::FallibleIterator;

enum JobColumn {
    Id,
    Description,
    Status,
    Priority,
    CreatedAt,
    UpdatedAt,
    StartTime,
    FinishTime,
    ErrorMessage,
    Result,
}

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
    pub fn write(&mut self) {
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

    // pub fn read_by_date_range() -> Vec<Job> {}
    // pub fn read_by_id_range() -> Vec<Job> {}
    pub fn get_jobs() -> Vec<Job> {
        if let Ok(sql) = SqliteConnection::new() {
            let mut jobs: Vec<Job> = vec![];
            let res = sql.conn.prepare(
                "SELECT * FROM jobs",
            );

            if let Ok(mut stmt) = sql.conn.prepare("SELECT * FROM jobs") {
                let job_iter = stmt.query_map([], |row| {
                    Ok(Job {
                        id: row.get(0)?,
                        description: row.get(1)?,
                        status: row.get(2)?,
                        priority: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                        start_time: row.get(6)?,
                        finish_time: row.get(7)?,
                        error_message: row.get(8)?,
                        result: row.get(9)?,
                    })
                });

                if let Ok(job_iter) = job_iter {
                    for job in job_iter {
                        if let Ok(job) = job {
                            jobs.push(job);
                        }
                    }
                }
            }

            jobs
        } else {
            eprintln!("Error: unable to run query");
            vec![]
        }
    }

    pub fn get_jobs_by_status(status: String) -> Vec<Job> {
        if let Ok(sql) = SqliteConnection::new() {
            let mut jobs: Vec<Job> = vec![];
            let res = sql.conn.prepare(
                "SELECT * FROM jobs",
            );

            if let Ok(mut stmt) = sql.conn.prepare("SELECT * FROM jobs") {
                let job_iter = stmt.query_map([], |row| {
                    Ok(Job {
                        id: row.get(0)?,
                        description: row.get(1)?,
                        status: row.get(2)?,
                        priority: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                        start_time: row.get(6)?,
                        finish_time: row.get(7)?,
                        error_message: row.get(8)?,
                        result: row.get(9)?,
                    })
                });

                if let Ok(job_iter) = job_iter {
                    for job in job_iter {
                        if let Ok(job) = job {
                            if job.status == status {
                                jobs.push(job);
                            }
                        }
                    }
                }
            }

            jobs
        } else {
            eprintln!("Error: unable to run query");
            vec![]
        }
    }

    // fn read_by(job_column: JobColumn, column_value: String) -> Job {
    //     match job_column {
    //         JobColumn::Id => {}
    //         JobColumn::Description => {}
    //         JobColumn::Status => {}
    //         JobColumn::Priority => {}
    //         JobColumn::CreatedAt => {}
    //         JobColumn::UpdatedAt => {}
    //         JobColumn::StartTime => {}
    //         JobColumn::FinishTime => {}
    //         JobColumn::ErrorMessage => {}
    //         JobColumn::Result => {}
    //     }
    // }

    // fn fetch_latest_sha(&self) -> Option<String> {
    //
    // }
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
        let conn = if let Ok(sqlite_path) = default_sqlite_path() {
            if let Err(e) = load_env_variables(&sqlite_path) {
                eprintln!("environment variables not loaded: {}", e);
            }

            Connection::open(sqlite_path)?
        } else {
            panic!("unable to connect to database")
        };

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
            status: "running".to_string(),
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

    #[test]
    fn test_read_by_status() {
        let jobs = Job::get_jobs_by_status("running".to_string());
        println!("{:?}", jobs);
    }
}
