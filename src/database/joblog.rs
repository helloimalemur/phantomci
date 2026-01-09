use crate::database::SqliteConnection;
use chrono::Local;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct JobLog {
    #[allow(unused)]
    pub id: i32,
    pub repo: String,
    pub log_message: String,
    #[allow(unused)]
    pub logged_at: String,
}

impl JobLog {
    pub fn add_job_log(&mut self) {
        let connection = SqliteConnection::new();
        let conn = connection.unwrap().conn;

        match conn.execute(
            "INSERT INTO job_logs (repo, log_message, logged_at) values (?1, ?2, ?3)",
            params![self.repo, self.log_message, Local::now().to_rfc3339()],
        ) {
            Ok(_) => println!("Wrote job log successfully"),
            Err(error) => println!("{}", error),
        }
    }

    // Fetch all logs ordered by newest first
    #[allow(dead_code)]
    pub fn get_logs() -> Vec<JobLog> {
        if let Ok(sql) = SqliteConnection::new() {
            let mut logs: Vec<JobLog> = vec![];
            if let Ok(mut stmt) = sql
                .conn
                .prepare("SELECT id, repo, log_message, logged_at FROM job_logs ORDER BY logged_at DESC")
            {
                let job_iter = stmt.query_map([], |row| {
                    Ok(JobLog {
                        id: row.get(0)?,
                        repo: row.get(1)?,
                        log_message: row.get(2)?,
                        logged_at: row.get(3)?,
                    })
                });

                if let Ok(job_logs_iter) = job_iter {
                    logs.extend(job_logs_iter.flatten());
                }
            }

            logs
        } else {
            eprintln!("Error: unable to run query");
            vec![]
        }
    }

    // Fetch newest logs with a limit
    pub fn get_logs_limited(limit: usize) -> Vec<JobLog> {
        if let Ok(sql) = SqliteConnection::new() {
            let mut logs: Vec<JobLog> = vec![];
            let query = if limit == 0 {
                "SELECT id, repo, log_message, logged_at FROM job_logs ORDER BY logged_at DESC".to_string()
            } else {
                format!(
                    "SELECT id, repo, log_message, logged_at FROM job_logs ORDER BY logged_at DESC LIMIT {}",
                    limit
                )
            };
            if let Ok(mut stmt) = sql.conn.prepare(&query) {
                let job_iter = stmt.query_map([], |row| {
                    Ok(JobLog {
                        id: row.get(0)?,
                        repo: row.get(1)?,
                        log_message: row.get(2)?,
                        logged_at: row.get(3)?,
                    })
                });
                if let Ok(job_logs_iter) = job_iter {
                    logs.extend(job_logs_iter.flatten());
                }
            }
            logs
        } else {
            eprintln!("Error: unable to run query");
            vec![]
        }
    }

    // Fetch logs for a repo (exact match) with optional limit
    pub fn get_logs_by_repo(repo: &str, limit: usize) -> Vec<JobLog> {
        if let Ok(sql) = SqliteConnection::new() {
            let mut logs: Vec<JobLog> = vec![];
            let base = "SELECT id, repo, log_message, logged_at FROM job_logs WHERE repo = ?1 ORDER BY logged_at DESC";
            let query = if limit == 0 {
                base.to_string()
            } else {
                format!("{} LIMIT {}", base, limit)
            };
            if let Ok(mut stmt) = sql.conn.prepare(&query) {
                let job_iter = stmt.query_map(params![repo], |row| {
                    Ok(JobLog {
                        id: row.get(0)?,
                        repo: row.get(1)?,
                        log_message: row.get(2)?,
                        logged_at: row.get(3)?,
                    })
                });
                if let Ok(job_logs_iter) = job_iter {
                    logs.extend(job_logs_iter.flatten());
                }
            }
            logs
        } else {
            eprintln!("Error: unable to run query");
            vec![]
        }
    }
}
