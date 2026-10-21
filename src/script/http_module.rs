//! Rune's `http` module over the system's `curl`, which brings TLS, proxies and the OS's certificates without IVE
//! linking a TLS stack. `http::get(url)` and `http::request(#{ method, url, headers, body, timeout })` answer
//! `Ok(#{ status, headers, body })` or `Err(reason)`; a 404 is an `Ok` with `status: 404`.

use std::collections::BTreeMap;
use std::process::{Output, Stdio};

use rune::{ContextError, Module, Value};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const METHODS: [&str; 7] = ["GET", "HEAD", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"];
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const MAX_TIMEOUT_SECONDS: u64 = 600;
const MAX_URL_BYTES: usize = 8 * 1024;
const MAX_HEADERS: usize = 100;
/// For a request body and a response body alike.
const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;
/// Separates what `--write-out` adds to curl's stderr from curl's own error message.
const MARKER: &str = "\n<<ive-http>>";

type Answer = Result<Value, String>;

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("http")?;
    module.function("get", |url: String| send(Ok(HttpRequest::get(url)))).build()?;
    module.function("request", |options: Value| send(HttpRequest::from_rune(&options))).build()?;
    Ok(module)
}

/// What `http::request` takes; `method` defaults to `GET`, `timeout` (seconds, for the whole exchange) to 30.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpRequest {
    #[serde(default = "get_method")]
    method: String,
    url: String,
    #[serde(default)]
    headers: BTreeMap<String, String>,
    body: Option<String>,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn get_method() -> String {
    "GET".to_owned()
}

fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT_SECONDS
}

impl HttpRequest {
    fn get(url: String) -> Self {
        Self { method: get_method(), url, headers: BTreeMap::new(), body: None, timeout: DEFAULT_TIMEOUT_SECONDS }
    }

    fn from_rune(options: &Value) -> Result<Self, String> {
        let invalid = |error: String| format!("http::request takes #{{ method, url, headers, body, timeout }}: {error}");
        let json = serde_json::to_value(options).map_err(|error| invalid(error.to_string()))?;
        serde_json::from_value(json).map_err(|error| invalid(error.to_string()))
    }

    /// Rejects what curl would misread (a URL that is an option, a header with a line break) or what is unreasonable.
    fn validated(self) -> Result<Self, String> {
        let url = &self.url;
        if url.len() > MAX_URL_BYTES || url.chars().any(char::is_control) || url.contains(' ') {
            return Err(format!("http: URL {url:?} is longer than {MAX_URL_BYTES} bytes or has spaces or control characters"));
        }
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(format!("http: URL {url:?} is not http:// or https://"));
        }
        if !METHODS.contains(&self.method.as_str()) {
            return Err(format!("http: method {:?} is none of {}", self.method, METHODS.join(", ")));
        }
        if !(1..=MAX_TIMEOUT_SECONDS).contains(&self.timeout) {
            return Err(format!("http: timeout {} is not 1-{MAX_TIMEOUT_SECONDS} seconds", self.timeout));
        }
        self.validated_payload()
    }

    fn validated_payload(self) -> Result<Self, String> {
        if self.headers.len() > MAX_HEADERS {
            return Err(format!("http: {} headers are more than {MAX_HEADERS}", self.headers.len()));
        }
        if let Some((name, _)) = self.headers.iter().find(|(name, value)| !is_header_name(name) || value.chars().any(|c| matches!(c, '\r' | '\n' | '\0'))) {
            return Err(format!("http: header {name:?} has a name that is not a token or a value with a line break"));
        }
        if self.body.as_ref().is_some_and(|body| body.len() > MAX_BODY_BYTES) {
            return Err(format!("http: the body is longer than {MAX_BODY_BYTES} bytes"));
        }
        Ok(self)
    }

    fn curl_args(&self) -> Vec<String> {
        // --disable first: no ~/.curlrc changes what a script asked for
        let mut args: Vec<String> = ["--disable", "--silent", "--show-error", "--location", "--max-redirs", "10", "--proto", "=http,https", "--proto-redir", "=http,https"]
            .map(str::to_owned)
            .into();
        args.extend(["--max-time".to_owned(), self.timeout.to_string(), "--max-filesize".to_owned(), MAX_BODY_BYTES.to_string()]);
        args.extend(self.method_args());
        for (name, value) in &self.headers {
            args.extend(["--header".to_owned(), format!("{name}: {value}")]);
        }
        if self.body.is_some() {
            args.extend(["--data-binary".to_owned(), "@-".to_owned()]);
        }
        args.extend(["--write-out".to_owned(), format!("%{{stderr}}{MARKER}%{{response_code}} %{{header_json}}")]);
        args.extend(["--url".to_owned(), self.url.clone()]);
        args
    }

    /// curl sends HEAD with `--head` (with `--request HEAD` it waits for a body that never comes).
    fn method_args(&self) -> Vec<String> {
        if self.method == "HEAD" {
            return vec!["--head".to_owned()];
        }
        vec!["--request".to_owned(), self.method.clone()]
    }
}

fn is_header_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
}

async fn send(request: Result<HttpRequest, String>) -> Answer {
    let request = request?.validated()?;
    let output = run_curl(&request).await?;
    let response = response_of(&output).map_err(|error| format!("http: {} {}: {error}", request.method, request.url))?;
    serde_json::from_value(response).map_err(|error| format!("http: cannot pass the response to the script: {error}"))
}

async fn run_curl(request: &HttpRequest) -> Result<Output, String> {
    let mut child = Command::new("curl")
        .args(request.curl_args())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // a script task that is dropped stops its transfer
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| format!("http needs curl on PATH: {error}"))?;
    let mut stdin = child.stdin.take().expect("stdin is piped");
    let body = request.body.clone().unwrap_or_default();
    // written while curl runs, so a body larger than the pipe does not wait on a reader that never comes
    let write = async move { stdin.write_all(body.as_bytes()).await };
    let (written, output) = tokio::join!(write, child.wait_with_output());
    let output = output.map_err(|error| format!("http: curl did not finish: {error}"))?;
    // when curl failed, its reason says more than the broken pipe
    if output.status.success()
        && let Err(error) = written
    {
        return Err(format!("http: cannot send the body to curl: {error}"));
    }
    Ok(output)
}

/// `#{ status, headers, body }` from curl's output: headers with lowercase names, a repeated one joined by `, `.
fn response_of(output: &Output) -> Result<serde_json::Value, String> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let (message, written_out) = stderr.rsplit_once(MARKER).unwrap_or((&stderr, ""));
    if !output.status.success() {
        return Err(message.trim().to_owned());
    }
    let (status, headers) = written_out.split_once(' ').ok_or_else(|| format!("curl wrote out {written_out:?}, not a status and headers"))?;
    let status: u16 = status.parse().map_err(|_| format!("curl wrote out status {status:?}"))?;
    let headers: BTreeMap<String, Vec<String>> = serde_json::from_str(headers).map_err(|error| format!("curl wrote out headers IVE cannot read: {error}"))?;
    let headers: BTreeMap<String, String> = headers.into_iter().map(|(name, values)| (name, values.join(", "))).collect();
    Ok(serde_json::json!({ "status": status, "headers": headers, "body": String::from_utf8_lossy(&output.stdout) }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Answers one request with `response`, and hands back what it received.
    fn serve_once(response: &'static str) -> (String, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://127.0.0.1:{}/path", listener.local_addr().unwrap().port());
        let served = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut received = vec![0; 64 * 1024];
            let read = stream.read(&mut received).unwrap();
            stream.write_all(response.as_bytes()).unwrap();
            String::from_utf8_lossy(&received[..read]).into_owned()
        });
        (url, served)
    }

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
    }

    #[test]
    fn should_answer_status_headers_and_body() {
        // given
        let (url, served) = serve_once("HTTP/1.1 201 Created\r\nContent-Length: 2\r\nX-Kind: a\r\nX-Kind: b\r\nConnection: close\r\n\r\nok");
        let options = HttpRequest { method: "POST".to_owned(), url, headers: BTreeMap::from([("X-Token".to_owned(), "t1".to_owned())]), body: Some("hello".to_owned()), timeout: 5 };

        // when
        let answer = block_on(send(Ok(options))).unwrap();

        // then
        let json = serde_json::to_value(&answer).unwrap();
        assert_eq!(json["status"], 201);
        assert_eq!(json["headers"]["x-kind"], "a, b");
        assert_eq!(json["body"], "ok");
        let received = served.join().unwrap();
        assert!(received.starts_with("POST /path HTTP/1.1\r\n"), "{received}");
        assert!(received.contains("X-Token: t1\r\n") && received.ends_with("hello"), "{received}");
    }

    #[test]
    fn should_answer_an_error_status_as_ok() {
        // given
        let (url, _served) = serve_once("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");

        // when
        let answer = block_on(send(Ok(HttpRequest::get(url)))).unwrap();

        // then
        assert_eq!(serde_json::to_value(&answer).unwrap()["status"], 404);
    }

    #[test]
    fn should_fail_with_curls_reason_when_nothing_listens() {
        // given
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}/");

        // when
        let answer = block_on(send(Ok(HttpRequest::get(url.clone()))));

        // then
        assert!(answer.as_ref().is_err_and(|error| error.starts_with(&format!("http: GET {url}: curl: (7)"))), "{answer:?}");
    }

    #[test]
    fn should_reject_what_curl_would_misread() {
        // given
        let header = |name: &str, value: &str| HttpRequest { headers: BTreeMap::from([(name.to_owned(), value.to_owned())]), ..HttpRequest::get("https://example.test/".to_owned()) };
        let requests = [
            HttpRequest::get("--output=/tmp/x".to_owned()),
            HttpRequest::get("file:///etc/passwd".to_owned()),
            HttpRequest::get("https://example.test/a b".to_owned()),
            HttpRequest { method: "get --head".to_owned(), ..HttpRequest::get("https://example.test/".to_owned()) },
            HttpRequest { timeout: 0, ..HttpRequest::get("https://example.test/".to_owned()) },
            header("X-A", "1\r\nX-Injected: 2"),
            header("X A", "1"),
        ];

        // when
        let rejected: Vec<bool> = requests.into_iter().map(|request| request.validated().is_err()).collect();

        // then
        assert_eq!(rejected, [true; 7]);
    }
}
