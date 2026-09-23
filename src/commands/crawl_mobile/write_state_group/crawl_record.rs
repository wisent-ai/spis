use super::*;

pub(crate) fn crawl_record(
    appium: &Appium,
    platform: Platform,
    record: &Record,
    manifest: &super::crawl::RuntimeManifest,
    root: &Path,
    max_states: usize,
    max_depth: usize,
) -> Result<Value> {
    let app_id = manifest.runtime_product.identifier.clone();
    let identity_source = manifest.runtime_product.identity_source.clone();
    let execution = manifest
        .execution_identity
        .as_ref()
        .context("mobile runtime manifest has no exact device identity")?;
    // Pinned exactly once per record and reused by both readiness observations.
    let readiness_helper = pinned_readiness_helper()?;
    let readiness_before = readiness_observation(manifest, &app_id, &readiness_helper)?;
    let output = root.join(&record.slug);
    std::fs::create_dir_all(&output)?;

    let (session, capabilities) = appium
        .create_session(platform, &app_id, execution)
        .with_context(|| format!("launch {} ({app_id})", record.name))?;
    let result = (|| -> Result<Value> {
        verify_session_capabilities(&capabilities, platform, &app_id, execution)?;
        let readiness_after = readiness_observation(manifest, &app_id, &readiness_helper)?;
        appium.start_recording(&session)?;

        let mut states = HashSet::new();
        let mut attempted = HashSet::<String>::new();
        let mut reported_gaps = HashSet::<String>::new();
        let mut trajectory = Vec::<PathStep>::new();
        let mut graph = Vec::new();
        let mut transitions = Vec::new();
        let mut blocked = Vec::new();
        let mut surface_observations = 0usize;

        for action_index in 0..=max_depth {
            if states.len() >= max_states {
                blocked.push(json!({
                    "reason": "state limit reached on the single observed trajectory; unexplored branches remain explicit gaps",
                    "max_states": max_states,
                }));
                break;
            }
            let current = inspect_surface(appium, &session)?;
            surface_observations += 1;
            if let Some(reason) = current.refusal_reason(platform, &app_id) {
                blocked.push(json!({
                    "delivered_input": Value::Null,
                    "observed_surface": current,
                    "reason": reason,
                    "further_input_withheld": true,
                }));
                break;
            }
            let state_id = hash_text(&current.source);
            let available = actions(&current.source, platform);
            if states.insert(state_id.clone()) {
                let index = states.len();
                let (screenshot, screenshot_owner_before, screenshot_owner_after) =
                    exact_surface_screenshot(appium, &session, platform, &app_id)?;
                write_state(
                    &output,
                    index,
                    &current.source,
                    &screenshot,
                    None,
                    &trajectory,
                    &available,
                )?;
                graph.push(json!({
                    "state": state_id,
                    "index": index,
                    "trajectory_depth": trajectory.len(),
                    "delivered_inputs": trajectory,
                    "observed_state": {
                        "active_owner_before": current.active_owner_before,
                        "active_owner_after": current.active_owner_after,
                        "screenshot_active_owner_before": screenshot_owner_before,
                        "screenshot_active_owner_after": screenshot_owner_after,
                        "alert_text": current.alert_text,
                        "source": format!("state-{index:04}/source.xml"),
                        "screenshot": format!("state-{index:04}/screenshot.png"),
                    },
                    "available_actions": available.iter().map(|action| json!({
                        "selector": action.selector,
                        "label": action.label,
                        "kind": action.kind,
                        "destructive": action.destructive,
                        "withheld_reason": action_block_reason(action),
                    })).collect::<Vec<_>>(),
                }));
            }

            let mut safe = Vec::new();
            for action in available {
                let identity = format!("{state_id}|{}|{}", action.selector, action.label);
                if let Some(reason) = action_block_reason(&action) {
                    if reported_gaps.insert(identity) {
                        blocked.push(json!({
                            "state": state_id,
                            "selector": action.selector,
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
                        "state": state_id,
                        "selector": action.selector,
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

            let before = inspect_surface(appium, &session)?;
            surface_observations += 1;
            if let Some(reason) = before.refusal_reason(platform, &app_id) {
                blocked.push(json!({
                    "state": state_id,
                    "delivered_input": Value::Null,
                    "observed_surface": before,
                    "reason": reason,
                    "further_input_withheld": true,
                }));
                break;
            }
            let Some(action) = actions(&before.source, platform)
                .into_iter()
                .find(|action| action.selector == selected.selector && action.label == selected.label)
            else {
                blocked.push(json!({
                    "state": state_id,
                    "selector": selected.selector,
                    "label": selected.label,
                    "delivered_input": Value::Null,
                    "observed_state_change": Value::Null,
                    "reason": "fresh pre-action source no longer exposed the selected control",
                }));
                continue;
            };
            if let Some(reason) = action_block_reason(&action) {
                blocked.push(json!({
                    "state": state_id,
                    "selector": action.selector,
                    "label": action.label,
                    "delivered_input": Value::Null,
                    "observed_state_change": Value::Null,
                    "reason": reason,
                }));
                continue;
            }
            let before_hash = hash_text(&before.source);
            appium
                .click(&session, &action.selector)
                .with_context(|| format!("deliver independently safe mobile action {:?}", action.label))?;
            let after = inspect_surface(appium, &session)?;
            surface_observations += 1;
            let after_hash = hash_text(&after.source);
            let changed = before_hash != after_hash;
            transitions.push(json!({
                "step": transitions.len() + 1,
                "delivered_input": {
                    "kind": "click",
                    "selector": action.selector,
                    "label": action.label,
                    "driver_acknowledged": true,
                },
                "observed_state_change": {
                    "changed": changed,
                    "before_source_sha256": before_hash,
                    "after_source_sha256": after_hash,
                    "active_owner_before_source": before.active_owner_before,
                    "active_owner_after_source": before.active_owner_after,
                    "active_owner_before_post_source": after.active_owner_before,
                    "active_owner_after_post_source": after.active_owner_after,
                    "alert_before": before.alert_text,
                    "alert_after": after.alert_text,
                },
            }));
            trajectory.push(PathStep {
                selector: action.selector.clone(),
                label: action.label.clone(),
            });
            if let Some(reason) = after.refusal_reason(platform, &app_id) {
                blocked.push(json!({
                    "state": after_hash,
                    "delivered_input": Value::Null,
                    "observed_surface": after,
                    "reason": reason,
                    "further_input_withheld": true,
                }));
                break;
            }
            if changed {
                for (_, alternative) in safe.into_iter().skip(1) {
                    blocked.push(json!({
                        "state": before_hash,
                        "selector": alternative.selector,
                        "label": alternative.label,
                        "delivered_input": Value::Null,
                        "observed_state_change": Value::Null,
                        "reason": "single sequential trajectory took a different safe edge; this branch was not reset or inferred",
                    }));
                }
            }
        }

        let recording = appium.stop_recording(&session)?;
        let recording_path = recording
            .as_deref()
            .filter(|bytes| !bytes.is_empty())
            .map(|bytes| {
                let path = output.join("trajectory.mp4");
                std::fs::write(&path, bytes)?;
                Ok::<_, anyhow::Error>(path)
            })
            .transpose()?;
        let report = json!({
            "schema": "wisent.mobile-crawl-run.v1",
            "catalog": platform.appium_name(),
            "record": record.slug,
            "name": record.name,
            "record_path": record.path,
            "app_id": app_id,
            "app_identity_source": identity_source,
            "source_revision": manifest.source_revision,
            "source_input_sha256": manifest.source_input_sha256,
            "runtime_manifest": manifest,
            "runtime_execution_identity": execution,
            "runtime_readiness_before_appium_launch": readiness_before,
            "runtime_readiness_after_capability_verification": readiness_after,
            "pinned_readiness_helper": {
                "path": readiness_helper.path,
                "sha256": readiness_helper.sha256,
                "version": readiness_helper.version,
            },
            "appium_session_capabilities": capabilities,
            "driver_url": appium.base,
            "surface_observations": surface_observations,
            "states": graph,
            "states_seen": states.len(),
            "transitions": transitions,
            "blocked_edges": blocked,
            "max_states": max_states,
            "max_depth": max_depth,
            "evidence_observations": {
                "executed_trajectory": trajectory,
                "accessibility_artifacts": graph.iter().filter_map(|state| state.pointer("/observed_state/source").cloned()).collect::<Vec<_>>(),
                "motion_artifacts": recording_path.iter().map(|path| path.strip_prefix(&output).unwrap_or(path)).collect::<Vec<_>>(),
                "canonical_interactions": [],
                "canonical_journey": Value::Null,
                "canonical_accessibility": Value::Null,
                "canonical_motion_analysis": Value::Null,
                "gaps": [
                    "Only one genuinely sequential observed trajectory was retained; alternative branches were not reset or inferred.",
                    "Destructive, input, permission-like, notification-like, and ambiguous controls were withheld before delivery.",
                    "Alert/source/owner observations are retained as observations; no hard-coded claim that a system dialog was absent is emitted."
                ]
            },
        });
        std::fs::write(
            output.join("crawl.json"),
            serde_json::to_string_pretty(&report)? + "\n",
        )?;
        Ok(report)
    })();
    appium.delete_session(&session);
    result
}
