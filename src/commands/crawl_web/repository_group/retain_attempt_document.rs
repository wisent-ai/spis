use super::*;

/// Retains one operational attempt document: the task status, the cancellation, the
/// official provenance, the attempt envelope and the failure diagnostic. The submission
/// does not pass through here — the bridge persists that one itself, through `--output`.
///
/// `relative` must name a path outside the `weles/` and `recordings/` subtrees, and this
/// function refuses anything else instead of merely asking for it.
/// `crawl::apply_web_attempt` merges exactly those two subtrees into the shared record
/// directory, and `crawl::write_immutable_file` refuses a destination that already holds
/// different bytes. Every document written through here is named after its role instead
/// of its content, and a second attempt of the same record carries a different attempt
/// id, task id and digests, so a fixed name inside a merged subtree would permanently
/// block the second import of that record. These documents therefore stay with the
/// attempt: in the attempt root, in the published archive, and in the attempt-private
/// `crawl/{attempt_id}` tree the importer installs verbatim.
///
/// The temp file staged HERE lives BESIDE the attempt root, next to the published archive
/// and its lock, so none of its bytes are ever audited by `audit_attempt_tree`, archived,
/// or installed into the record; `prune_stale_attempt_temporaries` sweeps what a killed
/// run left there. The bridge's `--output` staging is a separate mechanism that does sit
/// inside the attempt root, and the bridge unlinks it on every path it controls.
///
/// Unlike `crawl::atomic_json_write` this leaves no `.{name}.lock` sibling: `create_new`
/// plus the atomic `rename` below leave the destination either absent or one complete
/// document, so nothing an advisory lock would add is needed here.
pub(crate) fn retain_attempt_document(attempt_root: &Path, relative: &str, value: &Value) -> Outcome<()> {
    use std::io::Write;
    let io_failed = |message: &str| WorkerFailure::new("web_worker_io_failed", message);
    if !is_portable_relative(relative) {
        return Err(io_failed(
            "a retained attempt document name is not a portable relative path",
        ));
    }
    // The distinction between `weles-status.json` and `weles/status.json` is one
    // character, and getting it wrong reintroduces the permanent second-import failure,
    // so the merged subtrees are refused here rather than trusted to every call site.
    if relative
        .split('/')
        .next()
        .is_some_and(|leading| matches!(leading, "weles" | "recordings"))
    {
        return Err(io_failed(
            "an operational attempt document may not be retained inside a merged record subtree",
        ));
    }
    let owner = attempt_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| io_failed("the attempt root has no UTF-8 name"))?;
    let staging = attempt_root
        .parent()
        .ok_or_else(|| io_failed("the attempt root has no staging parent"))?;
    let destination = attempt_root.join(relative);
    let parent = destination
        .parent()
        .ok_or_else(|| io_failed("the retained document has no parent directory"))?;
    std::fs::create_dir_all(parent)?;
    let temporary = staging.join(format!(
        ".{owner}.{}.{}.tmp",
        relative.replace('/', "."),
        std::process::id()
    ));
    let _ = std::fs::remove_file(&temporary);
    let bytes = serde_json::to_vec_pretty(value)?;
    let result = (|| -> Outcome<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        std::fs::rename(&temporary, &destination)?;
        std::fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

/// Is `file_name` a document `retain_attempt_document` staged for THIS attempt, whose
/// process is gone? Recognises only `.{attempt_id}.{document}.{pid}.tmp`, so a live
/// sibling worker's staged write, the archive, the archive lock and the docs crawler's
/// `.{owner}.{artifact}.{pid}-{sequence}.tmp` names are all left alone.
pub(crate) fn is_stale_attempt_temporary(file_name: &str, owner: &str) -> bool {
    let Some(body) = file_name
        .strip_prefix('.')
        .and_then(|rest| rest.strip_suffix(".tmp"))
    else {
        return false;
    };
    let Some(rest) = body
        .strip_prefix(owner)
        .and_then(|rest| rest.strip_prefix('.'))
    else {
        return false;
    };
    let Some((document, pid)) = rest.rsplit_once('.') else {
        return false;
    };
    if !is_portable_component(document) {
        return false;
    }
    let Ok(pid) = pid.parse::<i32>() else {
        return false;
    };
    pid > 0 && !process_is_live(pid)
}

/// Drops staged documents an earlier run of this attempt orphaned between `create_new`
/// and `rename`. They sit beside the attempt root, outside the audited and archived set,
/// and carry no secret, so nothing breaks without this — they would simply accumulate on
/// the host. Mirrors `crawl_docs::prune_stale_temporaries`.
pub(crate) fn prune_stale_attempt_temporaries(attempt_root: &Path) -> Result<()> {
    let owner = attempt_root
        .file_name()
        .and_then(|value| value.to_str())
        .context("the attempt root has no UTF-8 name")?;
    let staging = attempt_root
        .parent()
        .context("the attempt root has no staging parent")?;
    let entries = match std::fs::read_dir(staging) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("list the attempt staging directory {}", staging.display()))
        }
    };
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if is_stale_attempt_temporary(file_name, owner) && entry.file_type()?.is_file() {
            let path = entry.path();
            std::fs::remove_file(&path)
                .with_context(|| format!("remove the stale staged document {}", path.display()))?;
        }
    }
    Ok(())
}

/// Runs one bridge operation through the single shared invoker in
/// `crate::weles_provenance`, which owns the script digest pin, the data-URL execution of
/// the verified bytes, the cleared environment, the process group, the stream bounds and
/// the canonical trust path for every operation this repository runs.
///
/// `output` is the durable destination `submit` requires and `get` refuses; `network`
/// hands the bridge the protected config, and is the only difference between the
/// credentialed task operations and the secretless local verification.
pub(crate) fn run_bridge(
    attempt_root: &Path,
    private: &PrivateBridge,
    label: &str,
    command: &Value,
    output: Option<&Path>,
    network: bool,
) -> Outcome<Vec<u8>> {
    // Re-read and re-validated for every operation: the exact bytes this process
    // accepted are the bytes the child is given, and an unprovisioned or altered trust
    // document stops the attempt here instead of inside the child.
    let trust = weles::CanonicalTrust::load()
        .map_err(|message| WorkerFailure::new("weles_trust_unavailable", message))?;
    weles::run_bridge_command(&weles::BridgeInvocation {
        command,
        trust: &trust,
        working_dir: attempt_root,
        output,
        config: network.then_some(private.config.as_path()),
        timeout: if network {
            weles::NETWORK_BRIDGE_TIMEOUT
        } else {
            weles::VERIFY_BRIDGE_TIMEOUT
        },
    })
    .map_err(|failure| {
        // Only the typed bridge code is surfaced; bridge stderr is never echoed, so no
        // delivered secret can reach the worker report or the Stado job log.
        WorkerFailure::new(
            &format!("weles_bridge_{}", failure.code.replace('-', "_")),
            format!("the official Weles bridge refused the {label} operation"),
        )
    })
}

/// Reads back the durable, request-bound submission the bridge persisted.
pub(crate) fn read_submission(path: &Path) -> Outcome<weles::WelesSubmission> {
    let text = path.to_str().ok_or_else(|| {
        WorkerFailure::new("web_worker_io_failed", "retained document path is not UTF-8")
    })?;
    Ok(crate::read_json(text)?)
}

/// Cancels a task that is still live at Weles through the bridge's `cancel` operation.
///
/// The cancellation key is derived from the retained submission's own idempotency key,
/// itself a pure function of the immutable attempt, so a resubmitted identical attempt
/// cancels exactly the same task exactly once; Weles refuses a second cancellation that
/// carries a different key or reason for the same task.
pub(crate) fn cancel_task(
    attempt_root: &Path,
    private: &PrivateBridge,
    identity: &weles::WelesServiceIdentity,
    identity_value: &Value,
    expected_task: &Value,
    submission: &weles::WelesSubmission,
    reason: &str,
    collected: &mut Collected,
) -> Outcome<weles::WelesCancellation> {
    let idempotency_key = format!(
        "spis-cancel-{}",
        crate::sha256_hex(format!("{}\0cancel", submission.idempotency_key).as_bytes())
    );
    let command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "cancel",
        "serviceIdentity": identity_value,
        "taskId": submission.task_id,
        "expectedTask": expected_task,
        "reason": reason,
        "idempotencyKey": idempotency_key,
    });
    let stdout = run_bridge(attempt_root, private, "cancel", &command, None, true)?;
    let cancellation: weles::WelesCancellation = serde_json::from_slice(&stdout)?;
    // Retained before it is judged: a cancellation this worker refuses is still the exact
    // document Weles returned for this attempt.
    retain_attempt_document(
        attempt_root,
        "weles-cancellation.json",
        &serde_json::to_value(&cancellation)?,
    )?;
    collected.cancellation = Some(cancellation.clone());
    ensure(
        cancellation.schema == weles::CANCELLATION_SCHEMA
            && cancellation.task_id == submission.task_id
            && cancellation.organization_id == submission.organization_id
            && cancellation.origin == submission.origin
            && cancellation.action == submission.action,
        "weles_cancellation_invalid",
        "the retained cancellation does not name this exact Weles task",
    )?;
    ensure(
        cancellation.idempotency_key == idempotency_key,
        "weles_cancellation_invalid",
        "the retained cancellation carries a different idempotency key",
    )?;
    ensure(
        cancellation.request_identity == submission.request_identity
            && cancellation.service_identity == *identity,
        "weles_cancellation_invalid",
        "the retained cancellation request/service identity differs from the submission",
    )?;
    Ok(cancellation)
}
