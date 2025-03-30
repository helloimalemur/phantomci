use crate::database::SqliteConnection;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::params;
use std::path::Path;

enum JobColumn {
    Id,
    Repo,
    Status,
    Priority,
    CreatedAt,
    UpdatedAt,
    StartTime,
    FinishTime,
    ErrorMessage,
    Result,
    Sha
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: i32,
    pub repo: String,
    pub status: String,
    pub priority: i32,
    pub created_at: String,
    pub updated_at: String,
    pub start_time: String,
    pub finish_time: String,
    pub error_message: String,
    pub result: String,
    pub sha: String
}

impl Job {
    pub fn add_job(&mut self) {

        let connection = SqliteConnection::new();
        let conn = connection.unwrap().conn;

        match conn.execute(
            "INSERT INTO jobs (repo, status, priority, created_at, updated_at, start_time, finish_time, error_message, result, sha) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![self.repo, self.status, self.priority, self.created_at, self.updated_at, self.start_time, self.finish_time, self.error_message, self.result, self.sha],
        ) {
            Ok(_) => (
                println!("Added successfully")
            ),
            Err(error) => {
                println!("{}", error)
            }
        }
    }

    pub fn check_exists(repo: String) -> bool {
        let mut contained = false;
        let jobs = Job::get_jobs();
        for job in jobs {
            if job.repo == repo {
                contained = true;
            }
        }
        contained
    }


    pub fn update_sha(repo: String, sha: String) {
        let connection = SqliteConnection::new();
        let conn = connection.unwrap().conn;

        match conn.execute(
            "UPDATE jobs
            SET sha = ?1
            WHERE repo = ?2",
            params![
                sha,
                repo
        ],
        ) {
            Ok(rows_updated) => {
                if rows_updated > 0 {
                    println!("SHA Update successful");
                } else {
                    println!("No matching job to update");
                }
            },
            Err(error) => {
                println!("Update error: {}", error);
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
                        repo: row.get(1)?,
                        status: row.get(2)?,
                        priority: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                        start_time: row.get(6)?,
                        finish_time: row.get(7)?,
                        error_message: row.get(8)?,
                        result: row.get(9)?,
                        sha: row.get(10)?,
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
        let mut jobs = Job::get_jobs();
        for job in jobs.clone() {
            if job.status == status {
                jobs.push(job);
            }
        }
        jobs
    }

    pub fn get_jobs_by_repo(repo: String) -> Vec<Job> {
        let mut jobs = Job::get_jobs();
        for job in jobs.clone() {
            if job.repo == repo {
                jobs.push(job);
            }
        }
        jobs
    }
}



pub fn load_env_variables(path: &str) -> rusqlite::Result<(), dotenv::Error> {
    let env_path = format!("{}.env", path);
    dotenv::from_path(Path::new(&env_path)).map(|_| println!("env: {}", env_path))
}

#[cfg(test)]
mod tests {
    use crate::database::job::Job;
    use crate::util::default_config_path;

    #[test]
    fn test_insert() {
        let config_dir = default_config_path().unwrap();
        let sqlite_path = format!("{}{}", config_dir, "db.sqlite");
        println!("SQLite database path: {}", sqlite_path);

        let mut job = Job {
            id: 0,
            repo: "".to_string(),
            status: "running".to_string(),
            priority: 0,
            created_at: "".to_string(),
            updated_at: "".to_string(),
            start_time: "".to_string(),
            finish_time: "".to_string(),
            error_message: "".to_string(),
            result: "".to_string(),
            sha: "".to_string(),
        };

        job.add_job();
    }

    #[test]
    fn test_read_by_status() {
        let jobs = Job::get_jobs_by_status("running".to_string());
        println!("{:?}", jobs);
        for job in jobs {
            assert_eq!(job.status, "running");
        }
    }
}
