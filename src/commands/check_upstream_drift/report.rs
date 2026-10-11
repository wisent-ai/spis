use super::*;

pub(crate) const REPORT: &str = ".build/upstream-drift.json";

pub(crate) const USER_AGENT: &str = crate::USER_AGENT;

pub(crate) const SCHEMA: &str = "wisent.upstream-drift-report.v1";

pub(crate) const GONE_CODES: &[u16] = &[400, 404, 410];

pub(crate) const GUARDED_CODES: &[u16] = &[401, 403, 405, 429, 451, 501, 503];

#[derive(Default)]
pub(crate) struct Drift {
    pub(crate) readme_changed: Vec<Value>,
    pub(crate) readme_unreachable: Vec<Value>,
    pub(crate) readme_unchanged: usize,
    pub(crate) sources_gone: Vec<Value>,
    pub(crate) sources_guarded: Vec<Value>,
    pub(crate) sources_unresolved: Vec<Value>,
    pub(crate) sources_ok: usize,
    pub(crate) sources_skipped: usize,
    pub(crate) media_missing: Vec<String>,
    pub(crate) media_hash_mismatch: Vec<String>,
    pub(crate) media_ok: usize,
}

impl Drift {
    /// Guarded sources are not drift: an authenticated product answering 401
    /// is behaving as recorded. A gone or unresolvable source is drift.
    pub(crate) fn any_drift(&self) -> bool {
        !self.readme_changed.is_empty()
            || !self.readme_unreachable.is_empty()
            || !self.sources_gone.is_empty()
            || !self.sources_unresolved.is_empty()
            || !self.media_missing.is_empty()
            || !self.media_hash_mismatch.is_empty()
    }
}

pub(crate) fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().user_agent(USER_AGENT).build()
}

/// Run `f` over `items` on a bounded worker pool (mirrors ThreadPoolExecutor).
pub(crate) fn pool<T: Sync, F: Fn(&T) + Sync>(items: &[T], workers: usize, f: F) {
    if items.is_empty() {
        return;
    }
    let next = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..workers.min(items.len()) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= items.len() {
                    break;
                }
                f(&items[i]);
            });
        }
    });
}

/// `gh api <path> --cache 0`, answering the JSON or the reason it has none:
/// gh could not start, gh refused (its exit status and stderr), or gh
/// answered something that is not JSON.
pub(crate) fn gh_json(path: &str) -> std::result::Result<Value, String> {
    let out = Command::new("gh")
        .args(["api", path, "--cache", "0"])
        .output()
        .map_err(|error| format!("gh api {path} could not start: {error}"))?;
    if !out.status.success() {
        return Err(format!(
            "gh api {path} exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|error| format!("gh api {path} answered no JSON: {error}"))
}

pub(crate) fn check_readmes(drift: &Mutex<Drift>) -> Result<()> {
    let sources: Value = lib::read_json("readme-examples/sources.json")?;
    // A sources file without its list, or an entry without its repository
    // and README path, is a broken input, not "no README to check".
    let entries = sources
        .get("repositories")
        .or_else(|| sources.get("examples"))
        .or_else(|| sources.get("sources"))
        .and_then(|v| v.as_array())
        .cloned()
        .context("readme-examples/sources.json has no repositories, examples or sources array")?;
    let mut checked = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let repo = entry.get("repository").and_then(|v| v.as_str());
        let readme_path = entry.get("readme_path").and_then(|v| v.as_str());
        let (Some(repo), Some(readme_path)) = (repo, readme_path) else {
            anyhow::bail!(
                "readme-examples/sources.json entry {index} has no repository or readme_path string: {entry}"
            );
        };
        checked.push((entry, repo, readme_path));
    }

    pool(&checked, 8, |&(entry, repo, readme_path)| {
        let recorded = entry.get("readme_blob_sha");
        let unreachable = |cause: String| {
            json!({
                "repository": repo,
                "readme_path": readme_path,
                "cause": cause,
            })
        };
        let answer = gh_json(&format!("repos/{repo}/contents/{readme_path}"));
        let mut guard = drift.lock();
        match answer {
            Err(cause) => guard.readme_unreachable.push(unreachable(cause)),
            Ok(data) => match data.get("sha").and_then(|v| v.as_str()) {
                None => guard.readme_unreachable.push(unreachable(format!(
                    "gh answered no sha string for {repo}/{readme_path}: {data}"
                ))),
                Some(current) => {
                    if Some(current) != recorded.and_then(|v| v.as_str()) {
                        guard.readme_changed.push(json!({
                            "repository": repo,
                            "readme_path": readme_path,
                            "recorded_sha": recorded.cloned().unwrap_or(Value::Null),
                            "current_sha": current,
                            "snapshot": entry.get("filename").cloned().unwrap_or(Value::Null),
                        }));
                    } else {
                        guard.readme_unchanged += 1;
                    }
                }
            },
        }
    });
    Ok(())
}

/// Return (state, detail) where state is reachable | gone | guarded | unresolved.
pub(crate) fn url_state(client: &ureq::Agent, url: &str) -> (&'static str, Value) {
    match client.head(url).call() {
        Ok(resp) => ("reachable", json!(resp.status())),
        Err(ureq::Error::Status(code, resp)) => {
            if GUARDED_CODES.contains(&resp.status()) {
                // HEAD refused or rate limited; try one byte.
                match client.get(url).set("Range", "bytes=0-0").call() {
                    Ok(r) => return ("reachable", json!(r.status())),
                    Err(ureq::Error::Status(inner, _)) => {
                        if GONE_CODES.contains(&inner) {
                            return ("gone", json!(inner));
                        }
                        return ("guarded", json!(inner));
                    }
                    Err(error) => {
                        // HEAD was refused and the one-byte GET did not
                        // arrive at all: nothing says the source is still
                        // there, so it is unresolved, with both answers.
                        return (
                            "unresolved",
                            json!(format!("HEAD {} then GET failed: {error}", resp.status())),
                        );
                    }
                }
            }
            if GONE_CODES.contains(&code) {
                return ("gone", json!(code));
            }
            ("guarded", json!(code))
        }
        Err(error) => ("unresolved", json!(error.to_string())),
    }
}

/// (url, where, expected_state) for every recorded upstream reference.
pub(crate) fn collect_urls() -> Vec<(String, String, Option<String>)> {
    let mut pairs: Vec<(String, String, Option<String>)> = Vec::new();
    for sources in sorted_glob_examples_files("sources.json") {
        let Ok(data) = lib::read_json::<Value>(sources.to_str().unwrap()) else {
            continue;
        };
        let catalog = sources
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        for example in data
            .get("examples")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[])
        {
            let name = example
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            if let Some(url) = example.get("source_url").and_then(|v| v.as_str()) {
                pairs.push((
                    url.to_string(),
                    format!("{catalog}/{name}/source_url"),
                    example
                        .get("source_url_state")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                ));
            }
            if let Some(visual) = example.get("visual") {
                for key in ["source_page_url", "source_image_url"] {
                    if let Some(url) = visual.get(key).and_then(|v| v.as_str()) {
                        pairs.push((
                            url.to_string(),
                            format!("{catalog}/{name}/{key}"),
                            visual
                                .get(&format!("{key}_state"))
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                        ));
                    }
                }
            }
        }
    }
    for record in reference_records() {
        let Ok(data) = lib::read_json::<Value>(record.to_str().unwrap()) else {
            continue;
        };
        let catalog = record
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let dir_name = record
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        for entry in data
            .get("motion")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[])
        {
            if let Some(url) = entry.get("source_url").and_then(|v| v.as_str()) {
                pairs.push((
                    url.to_string(),
                    format!("{catalog}/{dir_name}/motion"),
                    entry
                        .get("source_url_state")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                ));
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    pairs.retain(|(url, _, _)| seen.insert(url.clone()));
    pairs
}
