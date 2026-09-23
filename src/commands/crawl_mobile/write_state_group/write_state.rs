use super::*;

pub(crate) fn write_state(
    directory: &Path,
    index: usize,
    source: &str,
    screenshot: &[u8],
    video: Option<&[u8]>,
    path: &[PathStep],
    actions: &[Action],
) -> Result<()> {
    let state = directory.join(format!("state-{index:04}"));
    std::fs::create_dir_all(&state)?;
    std::fs::write(state.join("source.xml"), source)?;
    std::fs::write(state.join("screenshot.png"), screenshot)?;
    if let Some(video) = video.filter(|video| !video.is_empty()) {
        std::fs::write(state.join("trajectory.mp4"), video)?;
    }
    std::fs::write(
        state.join("state.json"),
        serde_json::to_string_pretty(&json!({
            "schema": "wisent.mobile-crawl-state.v1",
            "source_sha256": hash_text(source),
            "path": path.iter().map(|step| json!({
                "selector": step.selector,
                "label": step.label,
                "kind": "click",
            })).collect::<Vec<_>>(),
            "actions": actions.iter().map(|action| json!({
                "selector": action.selector,
                "label": action.label,
                "destructive": action.destructive,
                "kind": "click",
            })).collect::<Vec<_>>(),
        }))? + "\n",
    )?;
    Ok(())
}
