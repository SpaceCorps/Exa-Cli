//! Drives the built binary against an in-process mock of the Exa API. Every test gets its
//! own config directory and the plaintext store, so nothing touches a real keystore or account.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    /// Routes are (method, path-with-query, status, body). Unmatched requests get a 404.
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let routes = Arc::new(Mutex::new(routes));
        let log2 = log.clone();
        let routes2 = routes.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes2.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let path = parts.next().unwrap_or("").trim_start_matches('/').to_string();
                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap());
                    log.lock().unwrap().push(Recorded { method: method.clone(), path: path.clone(), headers, body });

                    let (status, resp) = {
                        let mut r = routes.lock().unwrap();
                        let matches: Vec<usize> = r
                            .iter()
                            .enumerate()
                            .filter(|(_, (m, p, _, _))| *m == method && *p == path)
                            .map(|(i, _)| i)
                            .collect();
                        if matches.is_empty() {
                            (404, json!({"error": "not found"}))
                        } else if matches.len() > 1 {
                            let idx = matches[0];
                            let (_, _, s, b) = r.remove(idx);
                            (s, b)
                        } else {
                            let idx = matches[0];
                            let (_, _, s, ref b) = r[idx];
                            (s, b.clone())
                        }
                    };

                    let text = if status == 204 { String::new() } else { resp.to_string() };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "exa-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_exa"))
            .args(args)
            .env("EXA_CONFIG_DIR", &self.dir)
            .env("EXA_SECRET_STORE", "plaintext")
            .env("EXA_API_URL", &self.api)
            .env_remove("EXA_API_KEY")
            .output()
            .unwrap()
    }

    fn run_with_env(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_exa"));
        cmd.args(args)
            .env("EXA_CONFIG_DIR", &self.dir)
            .env("EXA_SECRET_STORE", "plaintext")
            .env("EXA_API_URL", &self.api)
            .env_remove("EXA_API_KEY");
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.output().unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    fn json_with_env(&self, args: &[&str], envs: &[(&str, &str)]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run_with_env(&all, envs);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    /// Adds account `work` with key `exa_test`.
    fn with_account(self) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", "work", "--api-key", "exa_test"]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn search_verify_route() -> Route {
    (
        "POST",
        "search",
        200,
        json!({
            "results": [
                {
                    "title": "Exa AI",
                    "url": "https://exa.ai",
                    "id": "1"
                }
            ]
        }),
    )
}

#[test]
fn accounts_lifecycle() {
    let mock = Mock::start(vec![search_verify_route()]);
    let env = Env::new(&mock).with_account();

    assert_eq!(mock.last("POST").headers.iter().find(|(k, _)| k == "x-api-key").unwrap().1, "exa_test");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["name"], "work");
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");
    assert_eq!(out["secretStore"], "plaintext");

    let (_, out, _) = env.json(&["accounts", "list", "--check"]);
    assert_eq!(out["accounts"][0]["keyStatus"], "valid");

    // Duplicate account name requires --force
    let (code, _, err) = env.json(&["accounts", "add", "WORK", "--api-key", "x"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    let (code, out, _) = env.json(&["accounts", "test", "Work"]);
    assert_eq!(code, 0);
    assert_eq!(out["keyStatus"], "valid");

    // No terminal and no --yes refuses
    let (code, _, err) = env.json(&["accounts", "remove", "work"]);
    assert_eq!(code, 6);
    assert_eq!(err["remediation"], "exa accounts remove work --yes");

    let (code, out, _) = env.json(&["accounts", "remove", "work", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    let (_, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(out["count"], 0);
}

#[test]
fn api_key_from_stdin() {
    let mock = Mock::start(vec![search_verify_route()]);
    let env = Env::new(&mock);
    let mut child = Command::new(env!("CARGO_BIN_EXE_exa"))
        .args(["accounts", "add", "piped", "--api-key-stdin", "--json"])
        .env("EXA_CONFIG_DIR", &env.dir)
        .env("EXA_SECRET_STORE", "plaintext")
        .env("EXA_API_URL", &env.api)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"exa_piped\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let key = mock.last("POST").headers.into_iter().find(|(k, _)| k == "x-api-key").unwrap().1;
    assert_eq!(key, "exa_piped");
}

#[test]
fn config_is_readable_yaml_without_secrets() {
    let mock = Mock::start(vec![search_verify_route()]);
    let env = Env::new(&mock).with_account();
    let yaml = std::fs::read_to_string(env.dir.join("config.yaml")).unwrap();
    assert!(yaml.contains("work:"), "{yaml}");
    assert!(!yaml.contains("exa_test"), "the key leaked into config.yaml");
}

#[test]
fn search_command_payload() {
    let mock = Mock::start(vec![
        search_verify_route(),
        (
            "POST",
            "search",
            200,
            json!({
                "results": [
                    {
                        "title": "Quantum Computing 2026",
                        "url": "https://example.com/quantum",
                        "text": "Detailed quantum text"
                    }
                ]
            }),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "search",
        "quantum computing",
        "--account",
        "work",
        "--num",
        "5",
        "--type",
        "deep",
        "--category",
        "research paper",
        "--include-domains",
        "arxiv.org,nature.com",
        "--exclude-domains",
        "pinterest.com",
        "--start-date",
        "2026-01-01",
        "--end-date",
        "2026-06-01",
        "--highlights",
        "--highlights-chars",
        "2000",
        "--text",
        "--text-chars",
        "5000",
        "--summary",
        "--user-location",
        "US",
    ]);

    assert_eq!(code, 0, "{err}");
    assert_eq!(out["results"][0]["title"], "Quantum Computing 2026");

    let last_req = mock.last("POST");
    assert_eq!(last_req.path, "search");
    let body = last_req.body.unwrap();
    assert_eq!(body["query"], "quantum computing");
    assert_eq!(body["numResults"], 5);
    assert_eq!(body["type"], "deep");
    assert_eq!(body["category"], "research paper");
    assert_eq!(body["userLocation"], "US");
    assert_eq!(body["startPublishedDate"], "2026-01-01");
    assert_eq!(body["endPublishedDate"], "2026-06-01");
    assert_eq!(body["includeDomains"], json!(["arxiv.org", "nature.com"]));
    assert_eq!(body["excludeDomains"], json!(["pinterest.com"]));
    assert_eq!(body["contents"]["highlights"]["maxCharacters"], 2000);
    assert_eq!(body["contents"]["text"]["maxCharacters"], 5000);
    assert_eq!(body["contents"]["summary"], true);
}

#[test]
fn contents_command_payload() {
    let mock = Mock::start(vec![
        search_verify_route(),
        (
            "POST",
            "contents",
            200,
            json!({
                "results": [
                    {
                        "url": "https://example.com/a",
                        "text": "Extracted text content"
                    }
                ]
            }),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "contents",
        "https://example.com/a,https://example.com/b",
        "--account",
        "work",
        "--text",
        "--highlights",
        "--highlights-chars",
        "1500",
        "--highlights-query",
        "methodology",
        "--summary",
        "--summary-query",
        "key takeaways",
        "--max-age",
        "24",
        "--subpages",
        "3",
        "--subpage-target",
        "about,docs",
    ]);

    assert_eq!(code, 0, "{err}");
    assert_eq!(out["results"][0]["url"], "https://example.com/a");

    let last_req = mock.last("POST");
    assert_eq!(last_req.path, "contents");
    let body = last_req.body.unwrap();
    assert_eq!(body["urls"], json!(["https://example.com/a", "https://example.com/b"]));
    assert_eq!(body["text"], true);
    assert_eq!(body["highlights"]["maxCharacters"], 1500);
    assert_eq!(body["highlights"]["query"], "methodology");
    assert_eq!(body["summary"]["query"], "key takeaways");
    assert_eq!(body["maxAgeHours"], 24);
    assert_eq!(body["subpages"], 3);
    assert_eq!(body["subpageTarget"], json!(["about", "docs"]));
}

#[test]
fn contents_validation_requires_content_mode() {
    let mock = Mock::start(vec![search_verify_route()]);
    let env = Env::new(&mock).with_account();

    let (code, _, err) = env.json(&["contents", "https://example.com", "--account", "work"]);
    assert_eq!(code, 6);
    assert!(err["error"].as_str().unwrap().contains("Specify at least one content mode"));
}

#[test]
fn answer_command_payload() {
    let mock = Mock::start(vec![
        search_verify_route(),
        (
            "POST",
            "answer",
            200,
            json!({
                "answer": "Quantum computing utilizes superposition and entanglement.",
                "citations": ["https://example.com/quantum"]
            }),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["answer", "What is quantum computing?", "--account", "work", "--text"]);
    assert_eq!(code, 0, "{err}");
    assert!(out["answer"].as_str().unwrap().contains("superposition"));

    let last_req = mock.last("POST");
    assert_eq!(last_req.path, "answer");
    let body = last_req.body.unwrap();
    assert_eq!(body["query"], "What is quantum computing?");
    assert_eq!(body["text"], true);
}

#[test]
fn find_similar_command_payload() {
    let mock = Mock::start(vec![
        search_verify_route(),
        (
            "POST",
            "findSimilar",
            200,
            json!({
                "results": [
                    {
                        "url": "https://similar.example.com",
                        "title": "Similar Page"
                    }
                ]
            }),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "find-similar",
        "https://example.com/original",
        "--account",
        "work",
        "--num",
        "3",
        "--include-domains",
        "tech.com",
        "--exclude-domains",
        "spam.com",
        "--text",
        "--summary",
    ]);

    assert_eq!(code, 0, "{err}");
    assert_eq!(out["results"][0]["url"], "https://similar.example.com");

    let last_req = mock.last("POST");
    assert_eq!(last_req.path, "findSimilar");
    let body = last_req.body.unwrap();
    assert_eq!(body["url"], "https://example.com/original");
    assert_eq!(body["numResults"], 3);
    assert_eq!(body["includeDomains"], json!(["tech.com"]));
    assert_eq!(body["excludeDomains"], json!(["spam.com"]));
    assert_eq!(body["contents"]["text"], true);
    assert_eq!(body["contents"]["summary"], true);
}

#[test]
fn direct_api_key_and_env_var() {
    let mock = Mock::start(vec![
        ("POST", "search", 200, json!({"results": [{"title": "Direct Key Search"}]})),
        ("POST", "search", 200, json!({"results": [{"title": "Env Key Search"}]})),
    ]);
    let env = Env::new(&mock);

    // 1. Direct --api-key
    let (code, out, err) = env.json(&["search", "direct test", "--api-key", "key_direct"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["results"][0]["title"], "Direct Key Search");
    assert_eq!(mock.last("POST").headers.iter().find(|(k, _)| k == "x-api-key").unwrap().1, "key_direct");

    // 2. EXA_API_KEY environment variable
    let (code, out, err) = env.json_with_env(&["search", "env test"], &[("EXA_API_KEY", "key_from_env")]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["results"][0]["title"], "Env Key Search");
    assert_eq!(mock.last("POST").headers.iter().find(|(k, _)| k == "x-api-key").unwrap().1, "key_from_env");
}

#[test]
fn auth_is_required_when_unconfigured() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, _, err) = env.json(&["search", "anything"]);
    assert_eq!(code, 7);
    assert_eq!(err["code"], "no_account");
    assert!(err["remediation"].as_str().unwrap().contains("exa login"));
}

#[test]
fn http_errors_map_to_exit_codes() {
    let mock = Mock::start(vec![
        search_verify_route(),
        ("POST", "search", 401, json!({"error": "Invalid API key"})),
        ("POST", "search", 403, json!({"error": "Forbidden feature"})),
        ("POST", "search", 404, json!({"error": "Resource missing"})),
        ("POST", "search", 429, json!({"error": "Too many requests"})),
        ("POST", "search", 500, json!({"error": "Internal server crash"})),
    ]);
    let env = Env::new(&mock).with_account();

    // 401 -> 3 (AuthRequired)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");

    // 403 -> 3 (AuthRequired)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");

    // 404 -> 4 (NotFound)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 4);
    assert_eq!(err["code"], "not_found");

    // 429 -> 5 (RateLimited)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 5);
    assert_eq!(err["code"], "rate_limited");

    // 500 -> 2 (Network)
    let (code, _, err) = env.json(&["search", "test", "-a", "work"]);
    assert_eq!(code, 2);
    assert_eq!(err["code"], "network");
}

#[test]
fn parse_errors_are_envelopes() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    // Missing positional query argument
    let (code, _, err) = env.json(&["search"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");
    assert!(err["error"].as_str().unwrap().contains("<QUERY>"));

    // --help exits successfully
    let out = env.run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn yaml_is_the_default() {
    let mock = Mock::start(vec![
        search_verify_route(),
        ("POST", "search", 200, json!({"results": [{"title": "YAML Output Result", "url": "https://yaml.org"}]})),
    ]);
    let env = Env::new(&mock).with_account();

    let out = env.run(&["search", "yaml test", "-a", "work"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("title: YAML Output Result"), "{stdout}");
    assert!(stdout.contains("url: https://yaml.org"), "{stdout}");
}

#[test]
fn agent_readme_as_data() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(out["tool"], "exa");
    assert_eq!(out["exitCodes"]["7"], "no_account - run exa accounts list or set EXA_API_KEY");

    let md = String::from_utf8(env.run(&["agent-readme"]).stdout).unwrap();
    assert!(md.starts_with("# exa - agent operating manual"));
}

#[test]
fn login_command_lifecycle() {
    let mock = Mock::start(vec![search_verify_route(), search_verify_route(), search_verify_route()]);
    let env = Env::new(&mock);

    // Initial login
    let (code, out, err) = env.json(&["login", "--api-key", "exa_login_key"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["status"], "logged_in");
    assert_eq!(out["name"], "default");
    assert_eq!(out["secretStore"], "plaintext");

    // Login without --force refuses duplicate
    let (code, _, err) = env.json(&["login", "--api-key", "exa_login_key_2"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    // Login with --force succeeds
    let (code, out, _) = env.json(&["login", "--api-key", "exa_login_key_2", "--force"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "logged_in");

    // Login with named account
    let (code, out, _) = env.json(&["login", "production", "--api-key", "exa_prod_key"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "production");
}
