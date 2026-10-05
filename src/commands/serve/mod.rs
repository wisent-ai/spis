//! `spis serve --port PORT`: the loopback HTTP/JSON backend Spis Desktop
//! talks to. It binds the loopback address (port 0 lets the system choose),
//! prints one line, `{"ready":true,"port":N}`, and from then on stdout carries
//! nothing.
//!
//! Every endpoint runs the spis subcommand the CLI would run, as a child of
//! this executable, one at a time. `GET /v1/health`, `GET /v1/docs-status` and
//! the `docs-search`/`docs-show` endpoints answer with a JSON document; every
//! other `POST /v1/<name>` streams NDJSON: zero or more
//! `{"type":"log","stream":"stdout"|"stderr","chunk":…}` lines, then exactly
//! one `{"type":"result","status":N,"json":{…}}` carrying the subcommand's exit
//! status and, on failure, `{"error": <its refusal sentence>}`. A refusal before
//! the stream starts is a non-2xx `{"error": sentence}`.

mod endpoints;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::Ipv4Addr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};

use endpoints::{bad_request, Refusal};

const STREAM_HEAD: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";

const USAGE: &str = "usage: spis serve --port PORT   (0 lets the system choose one)";

pub fn run(rest: &[String]) -> Result<()> {
    let port: u16 = match rest {
        [flag] if flag == "--help" || flag == "-h" => {
            println!("{USAGE}");
            return Ok(());
        }
        [flag, value] if flag == "--port" => super::parsed(Some(value), "--port")?,
        _ => return Err(super::usage(USAGE)),
    };
    let server = tiny_http::Server::http((Ipv4Addr::LOCALHOST, port))
        .map_err(|error| anyhow!("cannot listen on the loopback address, port {port}: {error}"))?;
    let bound = server
        .server_addr()
        .to_ip()
        .map(|address| address.port())
        .context("the listener reports no IP address")?;
    let mut stdout = std::io::stdout();
    writeln!(stdout, "{}", json!({ "ready": true, "port": bound }))?;
    stdout.flush()?;
    let program = std::env::current_exe().context("locate this spis executable")?;
    let jobs = Arc::new(Mutex::new(()));
    for request in server.incoming_requests() {
        let program = program.clone();
        let jobs = Arc::clone(&jobs);
        std::thread::spawn(move || answer(request, &program, &jobs));
    }
    Ok(())
}

fn document(request: tiny_http::Request, status: u16, body: Vec<u8>) {
    let header = tiny_http::Header::from_bytes("Content-Type", "application/json")
        .expect("a fixed header is well formed");
    let response = tiny_http::Response::from_data(body).with_status_code(status).with_header(header);
    let _ = request.respond(response);
}

fn refuse(request: tiny_http::Request, refusal: Refusal) {
    let body = serde_json::to_vec_pretty(&json!({ "error": refusal.sentence })).unwrap_or_default();
    document(request, refusal.status, body);
}

fn read_body(request: &mut tiny_http::Request) -> Result<Map<String, Value>, Refusal> {
    let mut raw = Vec::new();
    request
        .as_reader()
        .read_to_end(&mut raw)
        .map_err(|error| bad_request(&format!("the request body could not be read: {error}")))?;
    if raw.iter().all(u8::is_ascii_whitespace) {
        return Ok(Map::new());
    }
    match serde_json::from_slice::<Value>(&raw) {
        Ok(Value::Object(body)) => Ok(body),
        Ok(_) => Err(bad_request("request body must be a JSON object")),
        Err(_) => Err(bad_request("request body is not valid JSON")),
    }
}

fn answer(mut request: tiny_http::Request, program: &Path, jobs: &Mutex<()>) {
    let method = request.method().clone();
    let path = request.url().to_string();
    let name = match (&method, path.strip_prefix("/v1/")) {
        (tiny_http::Method::Get, Some("health")) => {
            let body = serde_json::to_vec_pretty(&json!({ "status": "ok" })).unwrap_or_default();
            return document(request, 200, body);
        }
        (tiny_http::Method::Get, Some("docs-status")) => "docs-status".to_string(),
        (tiny_http::Method::Post, Some(name)) if !name.is_empty() && !name.contains('/') => name.to_string(),
        _ => {
            let sentence = format!("unknown endpoint: {method} {path}");
            return refuse(request, Refusal { status: 404, sentence });
        }
    };
    let body = match read_body(&mut request) {
        Ok(body) => body,
        Err(refusal) => return refuse(request, refusal),
    };
    let argv = match endpoints::argv(&name, &body) {
        Ok(argv) => argv,
        Err(refusal) => return refuse(request, refusal),
    };
    if endpoints::answers_document(&name) {
        run_document(request, program, jobs, &argv);
    } else {
        stream(request, program, jobs, &argv);
    }
}

/// Run a docs subcommand and answer with the JSON document it prints,
/// unchanged; a refusal becomes `{"error": sentence}`.
fn run_document(request: tiny_http::Request, program: &Path, jobs: &Mutex<()>, argv: &[String]) {
    let completed = {
        let _job = jobs.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        Command::new(program).args(argv).output()
    };
    match completed {
        Ok(output) if output.status.success() => document(request, 200, output.stdout),
        Ok(output) => {
            let sentence = endpoints::refusal_sentence(&String::from_utf8_lossy(&output.stderr));
            refuse(request, Refusal { status: 500, sentence });
        }
        Err(error) => {
            let sentence = format!("spis could not start: {error}");
            refuse(request, Refusal { status: 500, sentence });
        }
    }
}

/// Each line a pipe yields, sent as `(stream_name, line)` until the pipe ends.
fn pump(pipe: Box<dyn Read + Send>, stream_name: &'static str, sender: mpsc::Sender<(&'static str, String)>) {
    let mut reader = BufReader::new(pipe);
    let mut line = Vec::new();
    while reader.read_until(b'\n', &mut line).map(|read| read > 0).unwrap_or(false) {
        if sender.send((stream_name, String::from_utf8_lossy(&line).into_owned())).is_err() {
            return;
        }
        line.clear();
    }
}

/// Stream one subcommand's output as NDJSON on a connection that closes after
/// the result line. When the app goes away mid-stream the job still runs to
/// its end; only the writing stops.
fn stream(request: tiny_http::Request, program: &Path, jobs: &Mutex<()>, argv: &[String]) {
    let _job = jobs.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let spawned = Command::new(program)
        .args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            let sentence = format!("spis could not start: {error}");
            return refuse(request, Refusal { status: 500, sentence });
        }
    };
    let (sender, receiver) = mpsc::channel();
    let mut pumps = Vec::new();
    if let Some(pipe) = child.stdout.take() {
        let sender = sender.clone();
        pumps.push(std::thread::spawn(move || pump(Box::new(pipe), "stdout", sender)));
    }
    if let Some(pipe) = child.stderr.take() {
        let sender = sender.clone();
        pumps.push(std::thread::spawn(move || pump(Box::new(pipe), "stderr", sender)));
    }
    drop(sender);

    let mut writer = request.into_writer();
    let mut connected = writer.write_all(STREAM_HEAD.as_bytes()).and_then(|()| writer.flush()).is_ok();
    let mut emit = |line: Value| -> bool {
        writer.write_all(format!("{line}\n").as_bytes()).and_then(|()| writer.flush()).is_ok()
    };
    let mut stderr = String::new();
    for (stream_name, chunk) in receiver {
        if stream_name == "stderr" {
            stderr.push_str(&chunk);
        }
        if connected {
            connected = emit(json!({ "type": "log", "stream": stream_name, "chunk": chunk }));
        }
    }
    for pump in pumps {
        let _ = pump.join();
    }
    let result = match child.wait() {
        Ok(status) if status.success() => json!({ "type": "result", "status": 0, "json": {} }),
        Ok(status) => json!({
            "type": "result",
            "status": status.code().unwrap_or(1),
            "json": { "error": endpoints::refusal_sentence(&stderr) },
        }),
        Err(error) => json!({
            "type": "result",
            "status": 1,
            "json": { "error": format!("waiting for spis failed: {error}") },
        }),
    };
    if connected {
        emit(result);
    }
}
