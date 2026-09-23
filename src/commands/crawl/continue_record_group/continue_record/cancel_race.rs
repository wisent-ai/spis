use super::*;

/// A cancel intent that arrived while the attempt was being submitted: dispatch it against the
/// retained Stado job (status first), and record whether it was dispatched or is still pending.
pub(super) fn settle_cancel_race(run_id: &str, catalog: &str, record_name: &str) -> Result<()> {
    let retained = record_snapshot(run_id, catalog, record_name)?;
    if !retained.get("cancel_intent").is_some_and(Value::is_object) {
        return Ok(());
    }
    let Some(job_id) = retained.get("stado_job_id").and_then(Value::as_str) else {
        return Ok(());
    };
    let cancellation = match machine_status(job_id) {
        Ok(job) if terminal_machine_state(machine_state(&job)) => {
            Ok(json!({"state": "noop_terminal", "observed_job": job}))
        }
        Ok(job) => {
            let output = stado_command()
                .args(["machine", "cancel", job_id])
                .output()
                .context("cancel Stado job after submission race")?;
            if output.status.success() {
                let response = serde_json::from_slice(&output.stdout)
                    .unwrap_or_else(|_| json!({"stdout": String::from_utf8_lossy(&output.stdout).trim()}));
                Ok(json!({"state": "cancel_dispatched", "observed_job": job, "response": response}))
            } else {
                Err(anyhow!(
                    "Stado refused race cancellation: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ))
            }
        }
        Err(error) => Err(anyhow!(
            "status-first race cancellation failed: {}",
            error.diagnostic
        )),
    };
    mutate_record(run_id, catalog, record_name, |entry| {
        match cancellation {
            Ok(result) => {
                entry["state"] = json!("cancelled");
                entry["cancel_result"] = result;
                entry["diagnostic"] = Value::Null;
            }
            Err(error) => {
                entry["state"] = json!("cancel_pending");
                entry["diagnostic"] = json!({
                    "code": "cancel_dispatch_failed",
                    "message": error.to_string(),
                });
            }
        }
        Ok(())
    })
}
