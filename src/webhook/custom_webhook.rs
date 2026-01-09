use std::process::Command;

pub async fn send_custom(url: &str, body: &str, content_type: &str) {
    let header = format!("Content-Type: {}", content_type);
    let status = Command::new("curl")
        .arg("--silent")
        .arg("--show-error")
        .arg("--fail")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg(header)
        .arg("-d")
        .arg(body)
        .arg(url)
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!(
                "failed to send Custom webhook (exit code {:?}) to {}",
                s.code(), url
            );
        }
        Err(e) => {
            eprintln!("failed to execute curl for Custom webhook: {}", e);
        }
    }
}