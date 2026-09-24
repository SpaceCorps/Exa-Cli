//! HTTP client for the Exa API, and the translation from HTTP status to [`ErrorCode`].
//!
//! One blocking agent per process: a CLI makes a handful of requests, so an async runtime
//! would cost more in startup than it could save.

use std::time::Duration;

use serde_json::{Value, json};
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};

const DEFAULT_BASE: &str = "https://api.exa.ai/";
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    api_key: String,
}

#[allow(dead_code)]
enum Method {
    Get,
    Post,
}

impl Client {
    pub fn new(api_key: &str) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(100)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("exa-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut base = std::env::var("EXA_API_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, api_key: api_key.trim().to_string() }
    }

    #[allow(dead_code)]
    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Post, path, Some(body))
    }

    #[allow(dead_code)]
    pub fn post_empty(&self, path: &str) -> Result<Value> {
        self.send(Method::Post, path, None)
    }

    /// Verifies that an API key works by sending a minimal test query.
    pub fn test_key(api_key: &str) -> Result<()> {
        let client = Client::new(api_key);
        let check_body = json!({
            "query": "exa",
            "numResults": 1
        });
        client.post("search", &check_body).map(|_| ())
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let clean_path = path.trim_start_matches('/');
        let url = format!("{}{}", self.base, clean_path);

        macro_rules! headers {
            ($req:expr) => {{ $req.header("x-api-key", &self.api_key).header("Accept", "application/json") }};
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Post, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send(&json[..])
            }
            (Method::Post, None) => {
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send_empty()
            }
        };

        let response = result.map_err(transport_error)?;
        read(response)
    }
}

fn read(mut response: Response<ureq::Body>) -> Result<Value> {
    let status = response.status().as_u16();
    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Object(Default::default()));
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the Exa API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    let parsed_error = serde_json::from_str::<Value>(body).ok().and_then(|v| {
        v.get("error").and_then(Value::as_str).or_else(|| v.get("message").and_then(Value::as_str)).map(str::to_string)
    });

    let e = match status {
        401 => Error::new(ErrorCode::AuthRequired, "The API key was rejected.")
            .fix("Set EXA_API_KEY, use --api-key <key>, or run: exa login"),
        403 => Error::new(ErrorCode::AuthRequired, "The API key is not permitted to perform this action.")
            .fix("Verify permissions for this key at https://dashboard.exa.ai/api-keys"),
        404 => Error::new(ErrorCode::NotFound, "The requested resource was not found."),
        429 => Error::new(ErrorCode::RateLimited, "Rate limited by the Exa API.")
            .fix("Wait before retrying or check your rate limits at https://dashboard.exa.ai"),
        400 | 422 => {
            let msg = parsed_error.unwrap_or_else(|| "The API refused the request.".to_string());
            Error::new(ErrorCode::InvalidInput, msg)
        }
        s if s >= 500 => Error::new(ErrorCode::Network, "The Exa API returned a server error.")
            .fix("Retry; if it persists, check https://status.exa.ai"),
        _ => Error::new(ErrorCode::Error, "The request failed."),
    };
    e.detail(detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(403, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(400, r#"{"error":"bad query"}"#).code, ErrorCode::InvalidInput);
        assert_eq!(status_error(422, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(500, "").code, ErrorCode::Network);
        assert_eq!(status_error(502, "").code, ErrorCode::Network);
    }
}
