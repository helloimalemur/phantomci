#[derive(Debug, Clone)]
pub struct JobLog {
    pub id: i32,
    pub job_id: i32,
    pub log_message: String,
    pub logged_at: String,
}