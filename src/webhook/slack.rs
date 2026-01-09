use serde_json::json;
use std::process::Command;

/// Build a minimal Slack-compatible JSON payload.
/// Slack Incoming Webhooks accept `{ "text": "..." }`.
pub fn build_slack_payload(text: &str, title: Option<&str>) -> String {
    let content = if let Some(t) = title {
        // Simple formatting: bold title followed by message
        format!("*{}*\n{}", t, text)
    } else {
        text.to_string()
    };
    json!({ "text": content }).to_string()
}

/// Send a Slack webhook using the system `curl` binary to avoid adding
/// extra HTTP client dependencies.
pub async fn send_slack(url: &str, text: &str, title: Option<&str>) {
    let payload = build_slack_payload(text, title);

    // Use curl for a simple POST
    let status = Command::new("curl")
        .arg("--silent")
        .arg("--show-error")
        .arg("--fail")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(payload)
        .arg(url)
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!(
                "failed to send Slack webhook (exit code {:?}) to {}",
                s.code(), url
            );
        }
        Err(e) => {
            eprintln!("failed to execute curl for Slack webhook: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::build_slack_payload;
    use serde_json::Value;

    #[test]
    fn test_build_payload_with_title() {
        let out = build_slack_payload("hello", Some("title"));
        let val: Value = serde_json::from_str(&out).expect("valid json");
        assert_eq!(val["text"], "*title*\nhello");
    }

    #[test]
    fn test_build_payload_without_title() {
        let out = build_slack_payload("hello", None);
        let val: Value = serde_json::from_str(&out).expect("valid json");
        assert_eq!(val["text"], "hello");
    }
}