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
//!
//! Nothing the serving meets is dropped: a body that is not JSON is refused
//! with the parser's line and column, a pipe that cannot be read is said on
//! stderr, as a `{"type":"log","stream":"serve",…}` line and in the result's
//! `output_incomplete`, a job lock left by a job that panicked is said before
//! the next job takes it, and every wait on a job says on stderr what it
//! waits for (`stado_wait`).

mod endpoints;
mod pump;

use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};

use anyhow::{anyhow, Context, Result};
use http::StatusCode;
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

fn document(request: tiny_http::Request, status: StatusCode, body: Vec<u8>) {
    let header = tiny_http::Header::from_bytes("Content-Type", "application/json")
        .expect("a fixed header is well formed");
    let response = tiny_http::Response::from_data(body)
        .with_status_code(status.as_u16())
        .with_header(header);
    let asked = format!("{} {}", request.method(), request.url());
    // A client that left before its answer is said on stderr with the
    // request, so the serving log never shows an answer nobody received.
    if let Err(error) = request.respond(response) {
        eprintln!("spis serve: the answer {status} to {asked} could not be written: {error}");
    }
}

/// A JSON document's bytes. Writing a `Value` as text cannot fail, so
/// nothing stands in for a document that was not written.
fn bytes(value: &Value) -> Vec<u8> {
    value.to_string().into_bytes()
}

fn refuse(request: tiny_http::Request, refusal: Refusal) {
    let body = bytes(&json!({ "error": refusal.sentence }));
    document(request, refusal.status, body);
}

fn failed(sentence: String) -> Refusal {
    Refusal { status: StatusCode::INTERNAL_SERVER_ERROR, sentence }
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
        Err(error) => Err(bad_request(&format!("request body is not valid JSON: {error}"))),
    }
}

fn answer(mut request: tiny_http::Request, program: &Path, jobs: &Mutex<()>) {
    let method = request.method().clone();
    let path = request.url().to_string();
    let name = match (&method, path.strip_prefix("/v1/")) {
        (tiny_http::Method::Get, Some("health")) => {
            return document(request, StatusCode::OK, bytes(&json!({ "status": "ok" })));
        }
        (tiny_http::Method::Get, Some("docs-status")) => "docs-status".to_string(),
        (tiny_http::Method::Post, Some(name)) if !name.is_empty() && !name.contains('/') => {
            name.to_string()
        }
        _ => {
            let sentence = format!("unknown endpoint: {method} {path}");
            return refuse(request, Refusal { status: StatusCode::NOT_FOUND, sentence });
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

/// The one job lock every subcommand runs under. A job that panicked while
/// holding it left nothing half-written for the next (each job is its own
/// process), so the next job takes it, and the panic is said first.
fn job_lock(jobs: &Mutex<()>) -> MutexGuard<'_, ()> {
    match jobs.lock() {
        Ok(held) => held,
        Err(poisoned) => {
            eprintln!(
                "spis serve: an earlier job's thread panicked while it held the job lock; the next job runs after it"
            );
            poisoned.into_inner()
        }
    }
}

/// What a wait on one job names: the subcommand and its arguments.
fn job(argv: &[String]) -> String {
    format!("spis {}", argv.join(" "))
}

/// Run a docs subcommand and answer with the JSON document it prints,
/// unchanged; a refusal becomes `{"error": sentence}`.
fn run_document(request: tiny_http::Request, program: &Path, jobs: &Mutex<()>, argv: &[String]) {
    let completed = {
        let _job = job_lock(jobs);
        stado_wait::blocking(stado_wait::Kind::Process, job(argv), program.display(), || {
            Command::new(program).args(argv).output()
        })
    };
    match completed {
        Ok(output) if output.status.success() => document(request, StatusCode::OK, output.stdout),
        Ok(output) => {
            let sentence = endpoints::refusal_sentence(&String::from_utf8_lossy(&output.stderr));
            refuse(request, failed(sentence));
        }
        Err(error) => refuse(request, failed(format!("spis could not start: {error}"))),
    }
}

/// The result line of a run that ended without its own exit code: a failure
/// status the Desktop reads as one (`EXIT_FAILURE`), and the sentence.
fn unended(sentence: String) -> Value {
    json!({ "type": "result", "status": libc::EXIT_FAILURE, "json": { "error": sentence } })
}

/// Stream one subcommand's output as NDJSON on a connection that closes after
/// the result line. When the app goes away mid-stream the job still runs to
/// its end; only the writing stops.
fn stream(request: tiny_http::Request, program: &Path, jobs: &Mutex<()>, argv: &[String]) {
    let _job = job_lock(jobs);
    let spawned = Command::new(program)
        .args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => return refuse(request, failed(format!("spis could not start: {error}"))),
    };
    let (sender, receiver) = mpsc::channel();
    let mut pumps = Vec::new();
    if let Some(pipe) = child.stdout.take() {
        let sender = sender.clone();
        pumps.push(("stdout", std::thread::spawn(move || pump::pump(Box::new(pipe), "stdout", sender))));
    }
    if let Some(pipe) = child.stderr.take() {
        let sender = sender.clone();
        pumps.push(("stderr", std::thread::spawn(move || pump::pump(Box::new(pipe), "stderr", sender))));
    }
    drop(sender);

    let mut writer = request.into_writer();
    let mut connected = writer
        .write_all(STREAM_HEAD.as_bytes())
        .and_then(|()| writer.flush())
        .is_ok();
    let mut emit = |line: Value| -> bool {
        writer
            .write_all(format!("{line}\n").as_bytes())
            .and_then(|()| writer.flush())
            .is_ok()
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
    // A pipe that could not be read cut the output short: the stream and the result say so.
    let cut = pump::finish(pumps);
    for sentence in &cut {
        eprintln!("spis serve: {sentence}");
        if connected {
            connected = emit(json!({ "type": "log", "stream": "serve", "chunk": format!("{sentence}\n") }));
        }
    }
    let waited =
        stado_wait::blocking(stado_wait::Kind::Process, job(argv), program.display(), || child.wait());
    let mut result = match waited {
        // A run ended by a signal has no exit code; the result still says
        // it failed, and its sentence names how it ended instead of letting
        // a code read as spis's own exit.
        Ok(status) => match (status.success(), status.code()) {
            (true, Some(code)) => json!({ "type": "result", "status": code, "json": {} }),
            (false, Some(code)) => json!({
                "type": "result",
                "status": code,
                "json": { "error": endpoints::refusal_sentence(&stderr) },
            }),
            (_, None) => unended(format!(
                "spis was ended without an exit code ({status}): {}",
                endpoints::refusal_sentence(&stderr)
            )),
        },
        Err(error) => unended(format!("waiting for spis failed: {error}")),
    };
    if !cut.is_empty() {
        result["output_incomplete"] = json!(cut);
    }
    if connected {
        emit(result);
    }
}
