use super::*;

/// Deliver one safe action on a fresh snapshot and retain what changed. Returns true when the
/// trajectory must stop here (the surface, owner or driver said so), false to observe again.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_step(
    driver: &CuaDriver,
    session: &str,
    pid: i64,
    window_id: i64,
    bundle_id: &str,
    transitions_dir: &Path,
    output: &Path,
    hash: &str,
    safe: Vec<(String, Action)>,
    selected: Action,
    transitions: &mut Vec<Value>,
    blocked: &mut Vec<Value>,
    trajectory: &mut Vec<Step>,
) -> Result<bool> {
    let transition_index = transitions.len() + 1;
    let pre_image = transitions_dir.join(format!("step-{transition_index:04}-before.png"));
    let pre = snapshot(driver, &session, pid, window_id, &pre_image)?;
    assert_target_surface(&pre, pid, window_id, &bundle_id)?;
    let active_owner_before = global_active_owner(driver, &session)?;
    if active_owner_before != bundle_id {
        blocked.push(json!({
            "state": hash,
            "delivered_input": Value::Null,
            "observed_state_change": Value::Null,
            "reason": format!("fresh global observation found active owner {active_owner_before:?}, not exact target {bundle_id:?}; input withheld"),
            "further_input_withheld": true,
        }));
        return Ok(true);
    }
    let step = Step {
        role: selected.role.clone(),
        label: selected.label.clone(),
    };
    let Some(action) = matching_action(&pre, &step) else {
        blocked.push(json!({
            "state": hash,
            "role": step.role,
            "label": step.label,
            "delivered_input": Value::Null,
            "observed_state_change": Value::Null,
            "reason": "fresh pre-action snapshot no longer exposed the selected token",
        }));
        return Ok(false);
    };
    if let Some(reason) = action_block_reason(&action) {
        blocked.push(json!({
            "state": hash,
            "role": action.role,
            "label": action.label,
            "delivered_input": Value::Null,
            "observed_state_change": Value::Null,
            "reason": reason,
        }));
        return Ok(false);
    }
    let before_hash = state_hash(&pre);
    let driver_response = match apply(driver, &session, pid, window_id, &action) {
        Ok(response) => response,
        Err(error) => {
            transitions.push(json!({
                "step": transition_index,
                "delivered_input": {
                    "role": action.role,
                    "label": action.label,
                    "delivery_status": "unknown",
                    "exact_driver_diagnostic": error.to_string(),
                },
                "observed_state_change": Value::Null,
            }));
            return Ok(true);
        }
    };
    let driver_effect = DriverResponse(&driver_response)
        .text("effect")
        .unwrap_or("missing")
        .to_string();
    let driver_route = DriverResponse(&driver_response)
        .text("route")
        .unwrap_or("missing")
        .to_string();
    let post_image = transitions_dir.join(format!("step-{transition_index:04}-after.png"));
    let post = match snapshot(driver, &session, pid, window_id, &post_image) {
        Ok(post) => post,
        Err(error) => {
            transitions.push(json!({
                "step": transition_index,
                "delivered_input": {
                    "role": action.role,
                    "label": action.label,
                    "driver_response": driver_response,
                },
                "observed_state_change": Value::Null,
                "exact_observation_diagnostic": error.to_string(),
            }));
            return Ok(true);
        }
    };
    let active_owner_after = global_active_owner(driver, &session)?;
    let post_snapshot = transitions_dir.join(format!("step-{transition_index:04}-after.json"));
    std::fs::write(
        &post_snapshot,
        serde_json::to_string_pretty(&post)? + "\n",
    )?;
    if let Err(error) = assert_target_surface(&post, pid, window_id, &bundle_id) {
        transitions.push(json!({
            "step": transition_index,
            "delivered_input": {
                "role": action.role,
                "label": action.label,
                "driver_response": driver_response,
            },
            "observed_state_change": {
                "snapshot": post_snapshot.strip_prefix(&output).unwrap_or(&post_snapshot),
                "screenshot": post_image.strip_prefix(&output).unwrap_or(&post_image),
                "exact_diagnostic": error.to_string(),
            },
        }));
        blocked.push(json!({
            "state": before_hash,
            "reason": error.to_string(),
            "further_input_withheld": true,
        }));
        return Ok(true);
    }
    let after_hash = state_hash(&post);
    let changed = before_hash != after_hash;
    transitions.push(json!({
        "step": transition_index,
        "delivered_input": {
            "role": action.role,
            "label": action.label,
            "driver_response": driver_response,
            "inspected_effect": driver_effect,
            "inspected_route": driver_route,
            "global_active_owner": active_owner_before,
        },
        "observed_state_change": {
            "changed": changed,
            "before_state": before_hash,
            "after_state": after_hash,
            "snapshot": post_snapshot.strip_prefix(&output).unwrap_or(&post_snapshot),
            "screenshot": post_image.strip_prefix(&output).unwrap_or(&post_image),
            "global_active_owner": active_owner_after,
        },
    }));
    trajectory.push(step);
    if active_owner_after != bundle_id {
        blocked.push(json!({
            "state": after_hash,
            "reason": format!("fresh global post-action observation found active owner {active_owner_after:?}, not exact target {bundle_id:?}; further input withheld"),
            "further_input_withheld": true,
        }));
        return Ok(true);
    }
    if driver_effect != "confirmed" {
        blocked.push(json!({
            "state": after_hash,
            "role": action.role,
            "label": action.label,
            "reason": format!(
                "driver action effect was {driver_effect:?}; fresh postcondition was retained, and further input was withheld"
            ),
            "further_input_withheld": true,
        }));
        return Ok(true);
    }
    if changed {
        for (_, alternative) in safe.into_iter().skip(1) {
            blocked.push(json!({
                "state": before_hash,
                "role": alternative.role,
                "label": alternative.label,
                "delivered_input": Value::Null,
                "observed_state_change": Value::Null,
                "reason": "single sequential trajectory took a different safe edge; this branch was not reset or inferred",
            }));
        }
    }
    Ok(false)
}
