use crate::database::SqliteConnection;
use chrono::Local;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct JobLog {
    pub id: i32,
    pub repo: String,
    pub log_message: String,
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

    pub fn get_logs() -> Vec<JobLog> {
        if let Ok(sql) = SqliteConnection::new() {
            let mut logs: Vec<JobLog> = vec![];
            let res = sql.conn.prepare("SELECT * FROM job_logs");

            if let Ok(mut stmt) = sql.conn.prepare("SELECT * FROM job_logs") {
                let job_iter = stmt.query_map([], |row| {
                    Ok(JobLog {
                        id: row.get(0)?,
                        repo: row.get(1)?,
                        log_message: row.get(2)?,
                        logged_at: row.get(3)?,
                    })
                });

                if let Ok(job_logs_iter) = job_iter {
                    for job_logs in job_logs_iter {
                        if let Ok(log) = job_logs {
                            logs.push(log);
                        }
                    }
                }
            }

            logs
        } else {
            eprintln!("Error: unable to run query");
            vec![]
        }
    }

    pub fn get_logs_by_repo(repo: String) -> Vec<JobLog> {
        let mut logs = JobLog::get_logs();
        for log in logs.clone() {
            if log.repo == repo {
                logs.push(log);
            }
        }
        logs
    }
}
