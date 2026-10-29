//! Real serve requests must validate caller fields before corpus activation.

use super::{retain_tree, tree, Run};
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::process::{Command, Stdio};
use std::thread;

#[derive(Deserialize)]
struct Ready {
    ready: bool,
    port: u16,
}

pub(super) fn exercise(run: &mut Run) -> Result<()> {
    let directory = run.work.join("backend");
    fs::create_dir(&directory)?;
    fs::create_dir(directory.join("calls"))?;
    let mut stdout = File::create(directory.join("stdout"))?;
    let stderr = File::create(directory.join("stderr"))?;
    let args = ["serve", "--port", "0"];
    let environment = json!({
        "PATH": run.path, "HOME": run.sandbox.join("home"),
        "XDG_CONFIG_HOME": run.sandbox.join("config"),
        "TMPDIR": run.sandbox.join("tmp"),
    });
    fs::write(
        directory.join("command.json"),
        serde_json::to_vec_pretty(&json!({
            "program": run.binary, "args": args, "environment_cleared": true,
            "environment": environment, "cwd": run.sandbox.join("corpus"),
        }))?,
    )?;
    let mut child = Command::new(&run.binary)
        .args(args)
        .env_clear()
        .env("PATH", &run.path)
        .env("HOME", run.sandbox.join("home"))
        .env("XDG_CONFIG_HOME", run.sandbox.join("config"))
        .env("TMPDIR", run.sandbox.join("tmp"))
        .current_dir(run.sandbox.join("corpus"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .spawn()
        .context("start the real Spis loopback backend")?;
    let mut drain = None;
    let mut requests = Vec::new();
    let result = (|| -> Result<()> {
        let pipe = child
            .stdout
            .take()
            .context("serve child has no owned stdout pipe")?;
        let mut reader = BufReader::new(pipe);
        let mut line = Vec::new();
        let read = reader.read_until(b'\n', &mut line);
        stdout.write_all(&line)?;
        read.context("read the real backend readiness announcement")?;
        drain = Some(thread::spawn(move || {
            std::io::copy(&mut reader, &mut stdout)
        }));
        let ready: Ready = serde_json::from_slice(&line)
            .context("decode the retained backend readiness announcement")?;
        ensure!(ready.ready, "serve announced that it was not ready");
        // Reserved port zero: https://www.rfc-editor.org/rfc/rfc6335#section-6
        ensure!(ready.port != 0, "serve announced an unbound port");
        refusal(
            run,
            &directory,
            ready.port,
            "/v1/docs-search",
            json!({"query": ""}),
            "query",
            &mut requests,
        )?;
        refusal(
            run,
            &directory,
            ready.port,
            "/v1/docs-search",
            json!({"query": "retained query", "site": false}),
            "site",
            &mut requests,
        )?;
        refusal(
            run,
            &directory,
            ready.port,
            "/v1/docs-search",
            json!({"query": "retained query", "limit": false}),
            "limit",
            &mut requests,
        )?;
        refusal(
            run,
            &directory,
            ready.port,
            "/v1/docs-show",
            json!({"site": "caller-site", "url": false}),
            "url",
            &mut requests,
        )?;
        refusal(
            run,
            &directory,
            ready.port,
            "/v1/catalog-remove",
            json!({"slug": "caller-catalog", "force": "yes"}),
            "force",
            &mut requests,
        )?;
        Ok(())
    })();
    // Teardown always runs after startup or request failure, only for this child.
    let kill = child.kill();
    let exit = child.wait();
    let drained = drain.map(|handle| {
        handle
            .join()
            .map_err(|panic| anyhow::anyhow!("stdout drain panicked: {panic:?}"))
            .and_then(|read| read.context("retain the backend stdout stream"))
    });
    fs::write(
        directory.join("outcome.json"),
        serde_json::to_vec_pretty(&json!({
            "pid": child.id(), "requests": requests,
            "error": result.as_ref().err().map(|error| format!("{error:#}")),
            "termination_error": kill.as_ref().err().map(ToString::to_string),
            "exit_code": exit.as_ref().ok().and_then(|status| status.code()),
            "exit_status": exit.as_ref().ok().map(ToString::to_string),
            "wait_error": exit.as_ref().err().map(ToString::to_string),
            "stdout_reader_started": drained.is_some(),
            "stdout_error": drained.as_ref().and_then(|read| read.as_ref().err()).map(|error| format!("{error:#}")),
        }))?,
    )?;
    result?;
    kill.context("terminate the owned backend after HTTP qualification")?;
    exit.context("observe the owned backend termination")?;
    if let Some(drained) = drained {
        drained?;
    }
    Ok(())
}

fn refusal(
    run: &Run,
    directory: &std::path::Path,
    port: u16,
    endpoint: &str,
    body: Value,
    field: &str,
    requests: &mut Vec<Value>,
) -> Result<()> {
    let call = directory.join("calls").join(requests.len().to_string());
    fs::create_dir(&call)?;
    let before = tree(&run.sandbox)?;
    retain_tree(&call.join("before.json"), &before)?;
    let bytes = serde_json::to_vec(&body)?;
    // HTTP/1.0 closes each response; framing comes from the measured body size.
    // https://www.rfc-editor.org/rfc/rfc1945#section-7.2.2
    let request = format!(
        "POST {endpoint} HTTP/1.0\r\nHost: {}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        Ipv4Addr::LOCALHOST, bytes.len());
    let mut wire = request.into_bytes();
    wire.extend_from_slice(&bytes);
    fs::write(call.join("request"), &wire)?;
    let mut socket = TcpStream::connect((Ipv4Addr::LOCALHOST, port))
        .with_context(|| format!("connect to the announced backend for {endpoint}"))?;
    socket
        .write_all(&wire)
        .with_context(|| format!("send {endpoint}"))?;
    let mut response = Vec::new();
    socket
        .read_to_end(&mut response)
        .with_context(|| format!("read {endpoint}"))?;
    fs::write(call.join("response"), &response)?;
    let after = tree(&run.sandbox)?;
    retain_tree(&call.join("after.json"), &after)?;
    let text = std::str::from_utf8(&response).context("backend response is not UTF-8")?;
    let (headers, payload) = text
        .split_once("\r\n\r\n")
        .context("backend response has no header boundary")?;
    let mut status_line = headers
        .lines()
        .next()
        .context("backend response has no status line")?
        .split_whitespace();
    status_line
        .next()
        .context("backend response has no protocol version")?;
    let status = status_line
        .next()
        .context("backend response has no status code")?
        .parse::<u16>()?;
    requests.push(json!({"endpoint": endpoint, "body": body, "status": status,
        "field": field, "request": call.join("request"), "response": call.join("response"),
        "fixture_unchanged": before == after}));
    ensure!(
        before == after,
        "{endpoint} changed corpus or configuration while refusing {field}"
    );
    ensure!(
        status == http::StatusCode::BAD_REQUEST.as_u16(),
        "{endpoint} did not refuse malformed {field} before corpus activation: HTTP {status}"
    );
    let envelope: Value =
        serde_json::from_str(payload).context("refusal is not a JSON envelope")?;
    let cause = envelope["error"]
        .as_str()
        .context("refusal has no string error field")?;
    ensure!(
        cause.contains(field),
        "{endpoint} refusal does not identify failed field {field}; inspect retained response"
    );
    Ok(())
}
