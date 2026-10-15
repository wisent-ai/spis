use super::*;

/// Why `stado machine status` gave no job record: the diagnostic it answered
/// (or one built from what it printed), and whether Stado said the job does
/// not exist.
#[derive(Debug)]
pub(crate) struct LookupFailure {
    pub(crate) diagnostic: Value,
    pub(crate) not_found: bool,
}

/// The job record `stado machine status JOB` answers for `job_id`.
pub(crate) fn machine_status(job_id: &str) -> std::result::Result<Value, LookupFailure> {
    let mut command = crate::commands::crawl::stado_command();
    command.args(["machine", "status", job_id]);
    let output = command_output(&mut command, "read Stado machine job status").map_err(|error| {
        LookupFailure {
            diagnostic: json!({"code": "transport_error", "retryable": true, "message": format!("{error:#}")}),
            not_found: false,
        }
    })?;
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let document: Value = serde_json::from_slice(&output.stdout).map_err(|error| LookupFailure {
        diagnostic: json!({"code": "invalid_response", "retryable": true, "message": error.to_string(), "stderr": stderr}),
        not_found: false,
    })?;
    if !output.status.success() || document.get("ok").and_then(Value::as_bool) != Some(true) {
        // Stado's refusal is its `error` object; an answer that carries none
        // is said to be one, with the status and stderr it ended with.
        let error = match document.get("error") {
            Some(error) => error.clone(),
            None => json!({
                "code": "status_failed",
                "retryable": true,
                "message": format!(
                    "stado machine status {job_id} answered no error object ({}): {stderr}",
                    output.status
                ),
            }),
        };
        let not_found = matches!(
            error
                .get("code")
                .and_then(Value::as_str)
                .map(str::to_ascii_uppercase)
                .as_deref(),
            Some("NOT_FOUND" | "JOB_NOT_FOUND")
        );
        return Err(LookupFailure {
            diagnostic: error,
            not_found,
        });
    }
    document
        .pointer("/result/job")
        .cloned()
        .ok_or_else(|| LookupFailure {
            diagnostic: json!({"code": "invalid_response", "retryable": true, "message": "Stado status has no result.job"}),
            not_found: false,
        })
}
