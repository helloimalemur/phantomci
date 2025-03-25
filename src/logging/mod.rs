// logging.rs
use log::{error, info};

pub fn init() {
    env_logger::init();
}

pub fn log_job_start(job_id: u64) {
    info!("Job {} started", job_id);
}

pub fn log_job_end(job_id: u64, success: bool) {
    if success {
        info!("Job {} succeeded", job_id);
    } else {
        error!("Job {} failed", job_id);
    }
}
