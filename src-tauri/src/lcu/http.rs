use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::{header, Method};
use serde_json::{json, Value};
use tokio::time::sleep;

use super::types::LockfileInfo;

#[cfg(debug_assertions)]
fn log_request(method: &Method, url: &str, body: &str) {
    let body_preview = if body.len() > 200 {
        format!("{}...", &body[..200])
    } else {
        body.to_string()
    };
    println!("[LCU-HTTP] >>> {} {} body={}", method, url, body_preview);
}

#[cfg(not(debug_assertions))]
fn log_request(_method: &Method, _url: &str, _body: &str) {}

#[cfg(debug_assertions)]
fn log_response(method: &Method, url: &str, status: reqwest::StatusCode, body: &str) {
    let body_preview = if body.len() > 200 {
        format!("{}...", &body[..200])
    } else {
        body.to_string()
    };
    println!("[LCU-HTTP] <<< {} {} status={} body={}", method, url, status, body_preview);
}

#[cfg(not(debug_assertions))]
fn log_response(_method: &Method, _url: &str, _status: reqwest::StatusCode, _body: &str) {}

#[derive(Clone)]
pub struct LcuHttpClient {
    base_url: String,
    authorization_header: String,
    client: reqwest::Client,
}

impl LcuHttpClient {
    pub fn from_lockfile(lockfile: &LockfileInfo) -> Result<Self> {
        let auth_raw = format!("riot:{}", lockfile.password);
        let authorization_header = format!("Basic {}", general_purpose::STANDARD.encode(auth_raw));

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(10))
            .build()
            .context("failed building LCU HTTP client")?;

        Ok(Self {
            base_url: format!("https://127.0.0.1:{}", lockfile.port),
            authorization_header,
            client,
        })
    }

    pub async fn post_json(&self, path: &str, body: Option<Value>) -> Result<Value> {
        self.request_json(Method::POST, path, body).await
    }

    pub async fn patch_json(&self, path: &str, body: Value) -> Result<Value> {
        self.request_json(Method::PATCH, path, Some(body)).await
    }

    async fn request_json(&self, method: Method, path: &str, body: Option<Value>) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let mut attempt: usize = 0;

        loop {
            let body_preview = body.as_ref().map(|b| b.to_string()).unwrap_or_default();
            log_request(&method, &url, &body_preview);

            let mut request = self
                .client
                .request(method.clone(), &url)
                .header(header::AUTHORIZATION, self.authorization_header.as_str())
                .header(header::CONTENT_TYPE, "application/json");

            if let Some(body_json) = body.clone() {
                request = request.json(&body_json);
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    let response_text = response.text().await.unwrap_or_default();
                    log_response(&method, &url, status, &response_text);

                    if !status.is_success() {
                        if attempt == 0 && status.is_server_error() {
                            attempt += 1;
                            sleep(Duration::from_millis(500)).await;
                            continue;
                        }

                        return Err(anyhow!(
                            "LCU request {} {} failed with {}: {}",
                            method,
                            path,
                            status,
                            response_text
                        ));
                    }

                    if response_text.trim().is_empty() {
                        return Ok(json!({}));
                    }

                    // Check if response is valid JSON
                    match serde_json::from_str::<Value>(&response_text) {
                        Ok(json_body) => {
                            // Check for LCU-level error wrapped in 2xx
                            if let Some(error_code) = json_body.get("errorCode").and_then(|v| v.as_str()) {
                                return Err(anyhow!(
                                    "LCU error in 2xx response for {} {}: {} - {}",
                                    method, path, error_code,
                                    json_body.get("message").and_then(|v| v.as_str()).unwrap_or("")
                                ));
                            }
                            return Ok(json_body);
                        }
                        Err(e) => {
                            return Err(anyhow!(
                                "LCU response body for {} {} was not valid JSON: {} (body: {})",
                                method, path, e, &response_text[..response_text.len().min(200)]
                            ));
                        }
                    }
                }
                Err(error) => {
                    if attempt == 0 {
                        attempt += 1;
                        sleep(Duration::from_millis(500)).await;
                        continue;
                    }
                    return Err(anyhow!("LCU request {} {} failed: {}", method, path, error));
                }
            }
        }
    }
}

