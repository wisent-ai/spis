use super::*;

/// The absolute program Stado's own service declaration names for this host's
/// queue agent.
///
/// One read of the declaration. A transport failure, a refusal, or a
/// declaration that names zero or several programs is returned as
/// `worker_agent_program_unstable` with what was observed, and the record is
/// planned again by `crawl resume` instead of this call pausing and asking
/// again (cli.md rule 8).
pub(crate) fn declared_worker_stado_program(host: &str) -> Result<String> {
    let mut programs = BTreeSet::new();
    let mut last_failure = String::new();
    let mut command = stado_command();
    command.args(["service", "list", "--json"]);
    match command_output(&mut command, "read declared Stado agent program") {
        Ok(output) if output.status.success() => {
            let services: Value = serde_json::from_slice(&output.stdout)
                .context("Stado service list is not JSON")?;
            programs = active_worker_stado_programs(&services, host);
            // Two overlapping service declarations that execute the same
            // program are still unambiguous.
            if programs.len() == 1 {
                return Ok(programs
                    .iter()
                    .next()
                    .expect("one active program")
                    .clone());
            }
        }
        Ok(output) => {
            last_failure = String::from_utf8_lossy(&output.stderr).trim().to_string();
        }
        Err(error) => {
            last_failure = error.to_string();
        }
    }
    if programs.is_empty() {
        // The registry can manage a unit while its declaration says nothing
        // about what that unit runs, and no command completes such an entry:
        // `service declare` demands an endpoint a queue agent does not have,
        // and `service adopt` refuses a name the registry already manages.
        // Such a host would refuse every record with `observed_count=0` while
        // the agent runs and is visible to `service label-print`. So ask the
        // host the same way this crawl asks
        // it for `cargo`: one allowlisted Stado probe, whose receipt names the
        // absolute executable the host itself resolved. An observation from
        // the host is evidence; a conventional path would be a guess.
        if let Ok(program) = executable_word_from_host_receipt(
            host,
            host_probe(host, &["stado", "registry", "doctor"])
                .get("stado_receipt")
                .unwrap_or(&Value::Null),
        ) {
            return Ok(program);
        }
    }
    if !last_failure.is_empty() && programs.is_empty() {
        bail!(
            "worker_agent_program_unstable: host={host} observed_count=0 retryable=true last_failure={last_failure}"
        )
    }
    bail!(
        "worker_agent_program_unstable: host={host} observed_count={} retryable=true",
        programs.len()
    )
}

/// Shell prefix built from the retained cargo receipt and Stado's service declaration.
///
/// `CARGO_TARGET_DIR` is part of the prefix because every record is submitted
/// with its own immutable checkout of the run's exact revision, and each one
/// otherwise rebuilds Spis inside it: fifty documentation records meant fifty
/// identical builds, each writing about 5 GiB of `target/` on a host with
/// 15 GiB to spare, before a page was fetched. One shared directory under the
/// host's declared build-cache root keeps the evidence identical - the run is
/// bound to one revision and cargo rebuilds whenever the revision differs -
/// while the fleet pays for the build once and the host's own janitor owns the
/// cache. It travels in the command, not in `--repo-extras`: the CLI
/// documents that flag as a shell snippet, the fleet agent renders it as pip
/// extras, and a record dies on `/bin/sh: pip: command not found` before a
/// single page is fetched.
pub(crate) fn resolved_worker_program(host: &str) -> Result<String> {
    let cargo = std::env::var("SPIS_PREFLIGHT_WORKER_PROGRAM")
        .context("crawler coordinator did not pass the retained worker-program receipt")?;
    if cargo.chars().any(char::is_whitespace) {
        bail!("host {host} resolved the worker program to {cargo:?}, which a command cannot carry");
    }
    let stado = declared_worker_stado_program(host)?;
    Ok(format!(
        "CARGO_TARGET_DIR=\"$HOME/.stado/build/spis-cargo-target\" SPIS_STADO_BIN='{}' {cargo}",
        stado.replace('\'', "'\\''")
    ))
}

pub(crate) fn host_preflight(
    catalog: &str,
    engine: &str,
    host: &str,
    service_identity: Option<&RuntimeServiceIdentity>,
) -> Value {
    // `hostname -f` and not bare `hostname`: Stado's host-exec allowlist
    // matches an entry exactly and never appends operator words, and the entry
    // it carries is the fully-qualified form. `df -h` is the same kind of
    // observation and the coordinator needs it: how many records may be in
    // flight at once is a question about this host's free space, and nothing
    // else in the crawl ever asked it.
    let mut commands: Vec<Vec<&str>> = vec![vec!["hostname", "-f"], vec!["df", "-h"]];
    commands.extend(engine_preconditions(engine, catalog));
    let mut checks: Vec<Value> = commands.iter().map(|command| host_probe(host, command)).collect();
    let desktop_driver_ready = if engine == "desktop" {
        let candidates = [
            "/Applications/CuaDriver.app/Contents/MacOS/cua-driver",
            "/Applications/CuaDriver.app/Contents/MacOS/CuaDriver",
        ];
        let driver_checks: Vec<Value> = candidates
            .iter()
            .map(|candidate| host_probe(host, &[*candidate, "doctor", "--json"]))
            .collect();
        let ready = driver_checks
            .iter()
            .any(|check| check.get("ready").and_then(Value::as_bool) == Some(true));
        checks.extend(driver_checks);
        ready
    } else {
        true
    };
    let admission = match (engine, service_identity) {
        ("web", Some(service)) => {
            let check = weles_version_check(host, service);
            let ready = check.get("ready").and_then(Value::as_bool) == Some(true);
            checks.push(check);
            ready
        }
        ("web", None) => false,
        _ => true,
    };
    let ready = checks
        .iter()
        .take(commands.len())
        .all(|check| check.get("ready").and_then(Value::as_bool) == Some(true))
        && desktop_driver_ready
        && admission;
    let retryable = !ready && host_preflight_is_retryable(&json!({"checks": checks}));
    let diagnostic = retryable.then(|| {
        let retryable_checks = checks
            .iter()
            .filter(|check| check.pointer("/diagnostic/retryable").and_then(Value::as_bool) == Some(true))
            .filter_map(|check| check.get("diagnostic").cloned())
            .collect::<Vec<_>>();
        json!({
            "code": "host_check_retryable",
            "retryable": true,
            "message": "a host capability check answered with a refusal it marks retryable",
            "checks": retryable_checks,
        })
    });
    json!({
        "schema": "wisent.crawl-host-preflight.v2",
        "catalog": catalog,
        "engine": engine,
        "host": host,
        "ready": ready,
        "retryable": retryable,
        "diagnostic": diagnostic,
        "checks": checks,
        "service_identity": service_identity,
    })
}

pub(crate) fn observed_hostname(host_report: &Value) -> Result<String> {
    let value = host_report
        .get("checks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|check| {
            // The approved spelling is `hostname -f`, so the retained check
            // carries two words. Matching only the one-word form silently
            // dropped the observed hostname and every record then refused with
            // runtime_identity_or_readiness_unavailable.
            check.get("command").and_then(Value::as_array).is_some_and(|command| {
                command.len() == 2
                    && command[0].as_str() == Some("hostname")
                    && command[1].as_str() == Some("-f")
            })
        })
        .and_then(|check| check.get("stdout"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| {
            !value.is_empty()
                && !value.chars().any(char::is_whitespace)
                && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b".-_".contains(&byte))
        })
        .context("host preflight has no exact observed hostname")?;
    Ok(value.to_string())
}

/// The host's free space on the volume its work lands on, in GiB, as the
/// retained `df -h` probe of this preflight reported it.
///
/// `None` when the probe is missing or unparseable, and the caller treats that
/// as "assume the tightest case" rather than "assume room".
pub(crate) fn observed_free_gib(host_report: &Value) -> Option<f64> {
    let stdout = host_report
        .get("checks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|check| {
            check.get("command").and_then(Value::as_array).is_some_and(|command| {
                command.len() == 2
                    && command[0].as_str() == Some("df")
                    && command[1].as_str() == Some("-h")
            })
        })
        .and_then(|check| check.get("stdout"))
        .and_then(Value::as_str)?;
    // The root volume's line, whatever its device is called: the mount point
    // is the last field and the available size is the fourth.
    stdout
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            (fields.len() >= 6 && fields.last() == Some(&"/")).then(|| fields[3].to_string())
        })
        .find_map(|available| parse_size_gib(&available))
}

/// A `df -h` size such as `12Gi`, `980Mi` or `1.2Ti` in GiB.
pub(crate) fn parse_size_gib(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let (number, scale) = match trimmed.chars().last()? {
        'K' | 'k' => (&trimmed[..trimmed.len() - 1], 1.0 / (1024.0 * 1024.0)),
        'M' | 'm' => (&trimmed[..trimmed.len() - 1], 1.0 / 1024.0),
        'G' | 'g' => (&trimmed[..trimmed.len() - 1], 1.0),
        'T' | 't' => (&trimmed[..trimmed.len() - 1], 1024.0),
        'i' => {
            let head = &trimmed[..trimmed.len() - 1];
            return parse_size_gib(head);
        }
        _ => (trimmed, 1.0 / (1024.0 * 1024.0 * 1024.0)),
    };
    number.parse::<f64>().ok().map(|size| size * scale)
}
