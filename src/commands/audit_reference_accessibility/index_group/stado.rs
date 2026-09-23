use super::*;

// ---------------------------------------------------------------------------
// Stado plumbing

/// Match capture-landing-pages.py: preserve the first useful Stado refusal
/// rather than replacing it with the usage banner printed after it.
pub(crate) fn stado(args: &[&str], parse_json: bool) -> Result<(Option<Value>, String)> {
    let Some(stado_bin) = which("stado") else {
        bail!("stado is not on PATH; hosts are reached through stado, never ssh");
    };
    let output = Command::new(stado_bin)
        .args(args)
        .output()
        .with_context(|| format!("run stado {}", args.join(" ")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        let combined = if stderr.trim().is_empty() {
            &stdout
        } else {
            &stderr
        };
        let lines: Vec<&str> = combined
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let said: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.starts_with("error") || lower.starts_with("warning")
            })
            .collect();
        let detail = said.first().is_some().then_some(said).unwrap_or(lines);
        let tail = if detail.is_empty() {
            format!("exit {}", output.status.code().unwrap_or(-1))
        } else {
            detail
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        };
        bail!("stado {}: {}", args.join(" "), tail);
    }
    if !parse_json {
        return Ok((None, stdout));
    }
    let value = load_json(&stdout, &format!("stado {}", args.join(" ")))?;
    Ok((Some(value), stdout))
}

pub(crate) fn load_json(text: &str, what: &str) -> Result<Value> {
    match serde_json::from_str::<Value>(text) {
        Ok(value) => Ok(value),
        Err(_) => {
            let start = text.find('{');
            let end = text.rfind('}');
            if let (Some(start), Some(end)) = (start, end) {
                if end > start {
                    if let Ok(value) = serde_json::from_str::<Value>(&text[start..=end]) {
                        return Ok(value);
                    }
                }
            }
            bail!(
                "{what}: expected JSON on stdout, got {:?}",
                text.trim().chars().take(200).collect::<String>()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Plan

pub(crate) fn validate_plan(
    document: &Value,
    expected_target: &str,
    expected: &[Reference],
) -> Result<Map<String, Value>> {
    let document = document
        .as_object()
        .ok_or_else(|| anyhow!("plan: document must be a JSON object"))?;
    let mut keys: Vec<&String> = document.keys().collect();
    keys.sort();
    let mut wanted: Vec<&str> = PLAN_KEYS.to_vec();
    wanted.sort_unstable();
    let keys_match = keys.len() == wanted.len()
        && keys
            .iter()
            .zip(wanted.iter())
            .all(|(k, w)| k.as_str() == *w);
    if !keys_match {
        bail!(
            "plan: keys must be exactly {}; got {}",
            wanted.join(", "),
            keys.iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if document.get("schema").and_then(Value::as_str) != Some(PLAN_SCHEMA) {
        bail!("plan: schema must be {PLAN_SCHEMA}");
    }
    let batch = document
        .get("batch")
        .and_then(Value::as_str)
        .filter(|b| looks_like_slug(b))
        .ok_or_else(|| anyhow!("plan: batch must be a lowercase Weles slug"))?;
    if document.get("target").and_then(Value::as_str) != Some(expected_target) {
        bail!("plan: target must be {expected_target:?}");
    }
    let captures = document
        .get("captures")
        .and_then(Value::as_array)
        .filter(|c| !c.is_empty())
        .ok_or_else(|| anyhow!("plan: captures must be a non-empty array"))?;
    if captures.len() != expected.len() {
        bail!(
            "plan: expected {} actions, got {}",
            expected.len(),
            captures.len()
        );
    }
    let mut prefixes: HashSet<String> = HashSet::new();
    for (position, (action, reference)) in captures.iter().zip(expected.iter()).enumerate() {
        let position = position + 1;
        let label = format!("{}: action {position}", reference.id());
        let action = action
            .as_object()
            .ok_or_else(|| anyhow!("{label}: expected an object"))?;
        let action_keys: HashSet<&String> = action.keys().collect();
        if action_keys.len() != ACTION_KEYS.len()
            || !ACTION_KEYS
                .iter()
                .all(|key| action_keys.contains(&key.to_string()))
        {
            bail!("{label}: keys must be exactly {}", ACTION_KEYS.join(", "));
        }
        let expected_action = reference.action(batch);
        for field in ACTION_KEYS {
            let expected_value = expected_action
                .get(*field)
                .ok_or_else(|| anyhow!("{label}: internal: missing {field}"))?;
            let got = action.get(*field).unwrap_or(&Value::Null);
            if expected_value != got {
                bail!("{label}: {field}: expected {expected_value}, got {got}");
            }
        }
        let prefix = action
            .get("artifact_prefix")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if prefixes.contains(prefix) {
            bail!("{label}: artifact_prefix: duplicate prefix {prefix:?}");
        }
        prefixes.insert(prefix.to_string());
    }
    Ok(document.clone())
}

pub(crate) fn atomic_json(path: &Path, payload: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_file_name(format!(".{}.{}.part", path_name(path), pid()));
    std::fs::write(&temporary, serde_json::to_string_pretty(payload)? + "\n")?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

pub(crate) fn path_name(path: &Path) -> &str {
    path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
}

pub(crate) fn plan_path_for(plan_arg: Option<&String>, batch: &str) -> Result<PathBuf> {
    let raw = match plan_arg {
        Some(arg) => PathBuf::from(expand_home(arg)),
        None => plan_dir().join(format!("{batch}.json")),
    };
    let resolved = lex_norm(Path::new(&raw));
    let work = lex_norm(&work_root());
    if !is_relative_to(&resolved, &work) {
        bail!(
            "--plan: {} is outside {}; plans belong under ~/.spis/work",
            resolved.display(),
            work.display()
        );
    }
    Ok(resolved)
}

pub(crate) fn expand_home(input: &str) -> String {
    if input == "~" {
        return std::env::var("HOME").unwrap_or(input.to_string());
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    input.to_string()
}
