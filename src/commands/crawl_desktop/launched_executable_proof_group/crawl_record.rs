use super::*;

pub(crate) fn crawl_record(
    driver: &CuaDriver,
    record: &Record,
    manifest: &super::crawl::RuntimeManifest,
    root: &Path,
    max_states: usize,
    max_depth: usize,
) -> Result<Value> {
    let bundle_id = manifest
        .prepared_proof
        .as_ref()
        .context("desktop runtime manifest has no exact prepared product identity")?
        .product_identifier
        .clone();
    let session = format!("spis-{}", record.slug.replace('_', "-"));
    call(
        driver,
        "start_session",
        &json!({"session": session, "capture_scope": "window"}),
    )?;
    let _session_guard = SessionGuard {
        driver: driver.clone(),
        session: session.clone(),
    };
    let session_state = call(driver, "get_session_state", &json!({"session": session}))?;
    if DriverResponse(&session_state).text("capture_scope") != Some("window") {
        bail!("Cua Driver did not establish the required strict window session: {session_state}");
    }

    let output = root.join(&record.slug);
    let states_dir = output.join("states");
    let transitions_dir = output.join("transitions");
    let recording_dir = output.join("trajectory");
    std::fs::create_dir_all(&states_dir)?;
    std::fs::create_dir_all(&transitions_dir)?;
    std::fs::create_dir_all(&recording_dir)?;

    // These are deliberately the last product checks before the exact-bundle
    // launch. No catalog display name or PATH lookup participates.
    let readiness_helper = pinned_readiness_helper()?;
    let readiness = verify_desktop_executable(&bundle_id, manifest, &readiness_helper)?;
    // Cold launch: any pre-existing instance of this bundle is terminated
    // first, because launching an already-running bundle only activates the
    // stale instance and the screenshot would not be evidence of a clean
    // launch (finding 5).
    let pre_existing = terminate_pre_existing(driver, &session, &bundle_id)?;
    let launched = launch(driver, record, &bundle_id, &session)?;
    // Declared after the session guard so it drops first: the app is
    // terminated before the driver session is ended (finding 5).
    let _app_guard = AppGuard {
        driver: driver.clone(),
        session: session.clone(),
        pid: launched.pid,
    };
    if pre_existing.contains(&launched.pid) {
        return Err(anyhow::Error::new(RecordFailure {
            code: "desktop_launch_not_cold",
            message: format!(
                "launched pid {} was already running before the launch; the app was not started cold",
                launched.pid
            ),
        }));
    }
    let executable_proof = launched_executable_proof(&launched, manifest)?;
    let cold_launch_proof = json!({
        "pre_existing_pids": pre_existing,
        "terminated_pre_existing_pids": pre_existing,
        "launched_pid": launched.pid,
        "pid_existed_before_launch": false,
        "launch_response": launched.response,
    });
    let (pid, window_id) = (launched.pid, launched.window_id);
    call(
        driver,
        "start_recording",
        &json!({
            "session": session,
            "output_dir": recording_dir,
            "record_video": false,
        }),
    )?;
    let mut recording_guard = RecordingGuard {
        driver: driver.clone(),
        session: session.clone(),
        active: true,
    };

    let mut trajectory = Vec::<Step>::new();
    let mut seen_states = HashSet::new();
    let mut attempted = HashSet::<String>::new();
    let mut reported_gaps = HashSet::<String>::new();
    let mut graph = Vec::new();
    let mut transitions = Vec::new();
    let mut blocked = Vec::new();

    for action_index in 0..=max_depth {
        if seen_states.len() >= max_states {
            blocked.push(json!({
                "reason": "state limit reached on the single observed trajectory; unexplored branches remain explicit gaps",
                "max_states": max_states,
            }));
            break;
        }
        let observation_path = states_dir.join(format!("observation-{:04}.png", action_index + 1));
        let current = snapshot(driver, &session, pid, window_id, &observation_path)?;
        assert_target_surface(&current, pid, window_id, &bundle_id)?;
        let hash = state_hash(&current);
        let available = actions(&current);
        if seen_states.insert(hash.clone()) {
            let index = seen_states.len();
            let snapshot_path = states_dir.join(format!("state-{index:04}.json"));
            std::fs::write(
                &snapshot_path,
                serde_json::to_string_pretty(&current)? + "\n",
            )?;
            graph.push(json!({
                "state": hash,
                "index": index,
                "trajectory_depth": trajectory.len(),
                "delivered_inputs": trajectory,
                "observed_state": {
                    "snapshot": snapshot_path.strip_prefix(&output).unwrap_or(&snapshot_path),
                    "screenshot": observation_path.strip_prefix(&output).unwrap_or(&observation_path),
                },
                "available_actions": available.iter().map(|action| json!({
                    "role": action.role,
                    "label": action.label,
                    "destructive": action.destructive,
                    "kind": "click",
                    "withheld_reason": action_block_reason(action),
                })).collect::<Vec<_>>(),
            }));
        }

        let mut safe = Vec::new();
        for action in available {
            let identity = format!("{}|{}|{}", hash, action.role, normalize(&action.label));
            if let Some(reason) = action_block_reason(&action) {
                if reported_gaps.insert(identity) {
                    blocked.push(json!({
                        "state": hash,
                        "role": action.role,
                        "label": action.label,
                        "delivered_input": Value::Null,
                        "observed_state_change": Value::Null,
                        "reason": reason,
                    }));
                }
            } else if !attempted.contains(&identity) {
                safe.push((identity, action));
            }
        }
        if action_index == max_depth {
            for (_, action) in safe {
                blocked.push(json!({
                    "state": hash,
                    "role": action.role,
                    "label": action.label,
                    "delivered_input": Value::Null,
                    "observed_state_change": Value::Null,
                    "reason": "trajectory depth limit reached before this independently safe branch",
                }));
            }
            break;
        }
        let Some((attempt_identity, selected)) = safe.first().cloned() else {
            break;
        };
        attempted.insert(attempt_identity);

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
            break;
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
            continue;
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
            continue;
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
                break;
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
                break;
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
            break;
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
            break;
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
            break;
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
    }

    let recording = recording_guard
        .stop()
        .transpose()?
        .context("Cua Driver did not return session-scoped recording metadata")?;
    let recording_path = output.join("recording.json");
    std::fs::write(
        &recording_path,
        serde_json::to_string_pretty(&recording)? + "\n",
    )?;
    let report = json!({
        "schema": "wisent.desktop-crawl-run.v1",
        "record": record.slug,
        "name": record.name,
        "driver": "cua-driver",
        "pinned_driver": {
            "path": driver.path,
            "sha256": driver.sha256,
            "version": driver.version,
        },
        "pinned_readiness_helper": {
            "path": readiness_helper.path,
            "sha256": readiness_helper.sha256,
            "version": readiness_helper.version,
        },
        "cold_launch_proof": cold_launch_proof,
        "launched_executable_proof": executable_proof,
        "bundle_id": bundle_id,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "runtime_execution_identity": manifest.execution_identity,
        "fresh_runtime_readiness_observation": readiness,
        "resource_lease": manifest.resource_lease,
        "capture_scope": "window",
        "record_video": false,
        "fresh_snapshot_before_after_each_action": true,
        "states": graph,
        "states_seen": seen_states.len(),
        "transitions": transitions,
        "blocked_edges": blocked,
        "max_states": max_states,
        "max_depth": max_depth,
        "evidence_observations": {
            "executed_trajectory": trajectory,
            "accessibility_artifacts": graph.iter().filter_map(|state| state.pointer("/observed_state/snapshot").cloned()).collect::<Vec<_>>(),
            "motion_artifacts": [recording_path.strip_prefix(&output).unwrap_or(&recording_path)],
            "canonical_interactions": [],
            "canonical_journey": Value::Null,
            "canonical_accessibility": Value::Null,
            "canonical_motion_analysis": Value::Null,
            "gaps": [
                "Only one genuinely sequential observed trajectory was retained; alternative branches were not reset or inferred.",
                "Destructive, input, and ambiguous controls were withheld before delivery.",
                "No screen-reader, focus-order, live-region, or reduced-motion variant was executed."
            ]
        },
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}
