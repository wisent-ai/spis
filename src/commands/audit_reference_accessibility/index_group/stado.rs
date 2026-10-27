use super::*;

// ---------------------------------------------------------------------------
// Stado plumbing

/// Match capture-landing-pages.py: preserve the first useful Stado refusal
/// rather than replacing it with the usage banner printed after it.
pub(crate) fn stado(args: &[&str], parse_json: bool) -> Result<(Option<Value>, String)> {
    let Some(stado_bin) = which("stado") else {
        bail!("stado is not on PATH; hosts are reached through stado, never ssh");
    };
    // The wait is announced (what, where) by stado_wait, as every spis
    // child wait is.
    let output = stado_wait::output(Command::new(stado_bin).args(args))
        .with_context(|| format!("run stado {}", args.join(" ")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        // Stado's refusal is what it wrote to stderr, or to stdout when it
        // wrote nothing there, quoted whole.
        let said = match stderr.trim() {
            "" => stdout.trim(),
            text => text,
        };
        if said.is_empty() {
            bail!(
                "stado {} ended with {} and said nothing",
                args.join(" "),
                output.status
            );
        }
        bail!("stado {}: {said}", args.join(" "));
    }
    if !parse_json {
        return Ok((None, stdout));
    }
    // `--json` writes one JSON document on stdout and its waits on stderr,
    // so stdout is read as it is; text around it is a defect to name.
    let value = serde_json::from_str::<Value>(&stdout).with_context(|| {
        format!(
            "stado {}: expected one JSON document on stdout, got {:?}",
            args.join(" "),
            stdout.trim()
        )
    })?;
    Ok((Some(value), stdout))
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
    for ((action, reference), position) in captures
        .iter()
        .zip(expected.iter())
        .zip(crate::commands::reference_contract::ordinals())
    {
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
            // The key set was checked to be exactly ACTION_KEYS above.
            let got = action
                .get(*field)
                .ok_or_else(|| anyhow!("{label}: {field}: missing"))?;
            if expected_value != got {
                bail!("{label}: {field}: expected {expected_value}, got {got}");
            }
        }
        let prefix = action
            .get("artifact_prefix")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("{label}: artifact_prefix: must be text"))?;
        if prefixes.contains(prefix) {
            bail!("{label}: artifact_prefix: duplicate prefix {prefix:?}");
        }
        prefixes.insert(prefix.to_string());
    }
    Ok(document.clone())
}

/// Write `payload` whole: a hidden, per-process sibling renamed into place.
pub(crate) fn atomic_json(path: &Path, payload: &Value) -> Result<()> {
    let name = path
        .file_name()
        .with_context(|| format!("{} names no file to write", path.display()))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create {} for {}", parent.display(), path.display()))?;
    }
    let mut sibling = std::ffi::OsString::from(".");
    sibling.push(name);
    sibling.push(format!(".{}.part", pid()));
    let temporary = path.with_file_name(sibling);
    std::fs::write(&temporary, serde_json::to_string_pretty(payload)? + "\n")
        .with_context(|| format!("write {}", temporary.display()))?;
    std::fs::rename(&temporary, path)
        .with_context(|| format!("move {} into place as {}", temporary.display(), path.display()))?;
    Ok(())
}

pub(crate) fn plan_path_for(plan_arg: Option<&String>, batch: &str) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("locate the selected corpus directory")?;
    let work = lex_norm(&cwd.join(work_root()));
    let raw = match plan_arg {
        Some(arg) => expand_home(arg)?,
        None => work.join("plans").join(format!("{batch}.json")),
    };
    let resolved = lex_norm(&cwd.join(raw));
    if !is_relative_to(&resolved, &work) {
        bail!(
            "--plan: {} is outside {}; plans belong under .build/accessibility-audit",
            resolved.display(),
            work.display()
        );
    }
    Ok(resolved)
}

/// Expand a leading `~` to the user's home through `docs_corpus::home_dir`,
/// which refuses with the cause when HOME is unset; a `~/` left in place
/// named a directory called `~` under the working directory.
pub(crate) fn expand_home(input: &str) -> Result<PathBuf> {
    if input == "~" {
        return crate::commands::docs_corpus::home_dir();
    }
    match input.strip_prefix("~/") {
        Some(rest) => Ok(crate::commands::docs_corpus::home_dir()?.join(rest)),
        None => Ok(PathBuf::from(input)),
    }
}
