use serde::Serialize;
use serde_json::{json, Value};

const DEFAULT_URL: &str = "http://127.0.0.1:9077/ingest";

fn url() -> String {
    std::env::var("DAEMONAI_URL").unwrap_or_else(|_| DEFAULT_URL.to_string())
}

#[derive(Debug, Clone, Serialize)]
struct Payload {
    data: Value,
    severity: String,
    app: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<String>,
}

pub fn send(data: Value, severity: &str, kind: Option<&str>, app: Option<&str>) {
    let payload = Payload {
        data,
        severity: severity.to_string(),
        app: app.unwrap_or("rust").to_string(),
        kind: kind.map(String::from),
        channel: None,
        file: None,
        line: None,
        function: None,
    };

    let body = match serde_json::to_vec(&payload) {
        Ok(b) => b,
        Err(_) => return,
    };

    #[cfg(feature = "blocking")]
    {
        let _ = ureq::post(&url())
            .header("Content-Type", "application/json")
            .send(&body);
    }

    #[cfg(all(feature = "async", not(feature = "blocking")))]
    {
        let url = url();
        tokio::spawn(async move {
            let _ = reqwest::Client::new()
                .post(&url)
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await;
        });
    }
}

pub fn log(message: &str) {
    send(json!({"message": message}), "info", None, None);
}

pub fn warn(message: &str) {
    send(json!({"message": message}), "warn", None, None);
}

pub fn error(message: &str) {
    send(json!({"message": message}), "error", None, None);
}

pub fn query(sql: &str, duration_ms: f64) {
    send(
        json!({"sql": sql, "duration_ms": duration_ms}),
        "debug",
        Some("query"),
        None,
    );
}

/// Send with automatic file/line capture via macro
#[macro_export]
macro_rules! observe {
    ($data:expr) => {
        $crate::send($data, "debug", None, None)
    };
    ($data:expr, $severity:expr) => {
        $crate::send($data, $severity, None, None)
    };
}
