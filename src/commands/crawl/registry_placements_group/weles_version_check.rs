use super::*;

/// Independently confirm the deployed Weles release on the pinned host.
///
/// The Stado service directory is one observation; `{endpoint}/version` on the
/// selected host is the second. Both must report the same `release_id` and
/// `source_revision`, and redirects are refused, so a relocated or rolled-back
/// worker cannot silently sign attempts as the planned release.
pub(crate) fn weles_version_check(host: &str, service: &RuntimeServiceIdentity) -> Value {
    let url = format!("{}/version", service.endpoint);
    let mut check = host_probe(
        host,
        &[
            "curl",
            "--fail",
            "--silent",
            "--show-error",
            "--max-redirs",
            "0",
            "--max-time",
            "20",
            "--header",
            "Accept: application/json",
            url.as_str(),
        ],
    );
    if check.get("ready").and_then(Value::as_bool) != Some(true) {
        return check;
    }
    let reported: Option<Value> = check
        .get("stdout")
        .and_then(Value::as_str)
        .and_then(|stdout| serde_json::from_str(stdout.trim()).ok());
    let field = |document: &Value, snake: &str, camel: &str| -> Option<String> {
        document
            .get(snake)
            .or_else(|| document.get(camel))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let agreed = reported.as_ref().is_some_and(|document| {
        field(document, "release_id", "releaseId").as_deref() == Some(service.release_id.as_str())
            && field(document, "source_revision", "sourceRevision").as_deref()
                == Some(service.source_revision.as_str())
    });
    if !agreed {
        check["ready"] = json!(false);
        check["error"] = json!(format!(
            "{url} does not report the service-directory release {} at revision {}",
            service.release_id, service.source_revision
        ));
    }
    check["weles_version"] = reported.unwrap_or(Value::Null);
    check
}

/// The program every Spis crawl worker IS, as an approved host-exec probe.
///
/// Every engine submits its worker as `cargo run --release -- <subcommand>`,
/// so cargo is not the documentation engine's private precondition: it is the
/// one precondition all six share, and the allowlist entry for it says so in
/// its own justification. It is listed first for every engine by
/// [`engine_preconditions`], and [`resolved_worker_program`] turns the same
/// probe into the absolute path the submitted command is built from.
pub(crate) const WORKER_PROGRAM: [&str; 2] = ["cargo", "--version"];

/// What one engine's worker needs on the host before it may be given a slot.
///
/// One declaration, read by [`host_preflight`] and by every engine's submit
/// path, because the alternative is what this fleet already paid for: the
/// documentation engine preflighted `hostname -f` and nothing else, so
/// job-545551889f9e88be30daa81f was declared ready, claimed a slot, ran for
/// sixteen minutes and died with `/bin/sh: cargo: command not found`. That
/// engine was fixed in place on 2026-09-03; the same hole was still open in
/// the five others, which is why the requirement now lives here rather than
/// in six separate match arms.
///
/// Every probe is an exact entry in Stado's host-exec allowlist and every one
/// of them resolves an absolute path per host. Nothing here is answered
/// through `PATH`: the submitted worker runs under a non-login `/bin/sh` that
/// reads no profile, so a probe that consulted the login shell's search path
/// would be answering a different question from the one the job asks.
pub(crate) fn engine_preconditions(engine: &str, catalog: &str) -> Vec<Vec<&'static str>> {
    let mut required = vec![WORKER_PROGRAM.to_vec()];
    required.extend(match (engine, catalog) {
        // The iOS worker drives a simulator through Appium's XCUITest
        // driver, and `simctl` is how it learns the host has one at all.
        //
        // `list devices available` and NOT `list devices booted --json`: the
        // allowlist matches an entry exactly, and the entry it carries is the
        // `available` form. The `booted --json` spelling this side used to
        // ask for came back "is not an approved host-exec command" on every
        // iOS preflight, which reads as a host with no simulator support and
        // is really a spelling this side chose -- the same defect the
        // `hostname -f` comment above records, still live in a second place.
        ("mobile", "ios-app-examples") => vec![
            vec!["appium", "--version"],
            vec!["appium", "driver", "list", "--installed"],
            vec!["xcrun", "simctl", "list", "devices", "available"],
        ],
        ("mobile", _) => vec![
            vec!["appium", "--version"],
            vec!["appium", "driver", "list", "--installed"],
            vec!["adb", "version"],
            vec!["adb", "devices", "-l"],
        ],
        // The desktop worker execs the Cua Driver bundle directly, so its
        // own readiness is checked by absolute candidate below rather than by
        // an argument-free entry here.
        ("desktop", _) => Vec::new(),
        // The web worker talks to Weles over HTTP; Node is what that host
        // runs the release with, and the release itself is checked by
        // `weles_version_check`.
        ("web", _) => vec![vec!["node", "--version"]],
        ("docs", _) => Vec::new(),
        // Both terminal engines drive the product inside a tmux session.
        ("cli", _) => vec![vec!["tmux", "-V"]],
        // The TUI worker additionally builds a fixture git repository
        // before it starts the product, so git is that family's second
        // precondition. It is probed at the paths a real installation uses
        // and never at `/usr/bin/git`, which on macOS is the `xcode-select`
        // shim and opens the Command Line Tools installer WINDOW on a host
        // that has none -- the allowlist entry excludes that path for
        // exactly this reason, so the probe cannot raise a dialog on an
        // unattended host.
        ("tui", _) => vec![vec!["tmux", "-V"], vec!["git", "--version"]],
        _ => Vec::new(),
    });
    required
}

pub(crate) fn resolved_program_from_host_preflight(
    host: &str,
    arguments: &[&str],
    host_report: &Value,
) -> Result<String> {
    let check = host_report
        .get("checks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|check| check.get("command") == Some(&json!(arguments)))
        .with_context(|| format!("host preflight retained no `{}` probe", arguments.join(" ")))?;
    if check.get("ready").and_then(Value::as_bool) != Some(true) {
        bail!("host preflight `{}` probe was not ready", arguments.join(" "));
    }
    let receipt = check
        .get("stado_receipt")
        .context("ready host preflight probe retained no Stado receipt")?;
    executable_word_from_host_receipt(host, receipt)
}

/// Read the Stado program from the active queue-agent declaration for this host.
///
/// The agent is the component that will execute the job, so its managed service
/// declaration is the authoritative executable available inside that execution
/// boundary. It may temporarily point at a private repair build and later move
/// during a normal release; reading it for every submission follows either
/// transition without baking a shared or repair path into Spis.
///
/// Two spellings name the same thing and both are accepted. `agent --target
/// <host>` is what a coordinator declares for a machine it addresses by name,
/// and `agent --auto` is what a machine declares for itself - the shape
/// `lukasz-macbook` runs. Reading only the first refused every record on any
/// self-declared host with `worker_agent_program_unstable: observed_count=0`,
/// which reads as an unstable declaration rather than as a filter that never
/// matched.
pub fn active_worker_stado_programs(services: &Value, host: &str) -> BTreeSet<String> {
    services
        .as_array()
        .into_iter()
        .flatten()
        .filter(|service| service.get("host").and_then(Value::as_str) == Some(host))
        .filter(|service| service.get("state").and_then(Value::as_str) == Some("active"))
        .filter(|service| {
            service
                .get("args")
                .and_then(Value::as_array)
                .is_some_and(|arguments| {
                    let targets_this_host = arguments.windows(2).any(|pair| {
                        pair[0].as_str() == Some("--target") && pair[1].as_str() == Some(host)
                    });
                    let serves_its_own_host =
                        arguments.iter().any(|argument| argument.as_str() == Some("--auto"));
                    arguments.first().and_then(Value::as_str) == Some("agent")
                        && (targets_this_host || serves_its_own_host)
                })
        })
        .filter_map(|service| service.get("program").and_then(Value::as_str))
        .filter(|program| program.starts_with('/') && !program.chars().any(char::is_whitespace))
        .map(str::to_string)
        .collect()
}

pub fn worker_agent_program_retry_diagnostic(message: &str) -> Option<Value> {
    if !message.contains("worker_agent_program_unstable:") {
        return None;
    }
    let observed_count = message
        .split_once("observed_count=")
        .and_then(|(_, tail)| tail.split_whitespace().next())
        .and_then(|count| count.parse::<usize>().ok());
    Some(json!({
        "code": "worker_agent_program_unstable",
        "retryable": true,
        "observed_count": observed_count,
        "message": message,
    }))
}
