use rusqlite::Connection;

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
