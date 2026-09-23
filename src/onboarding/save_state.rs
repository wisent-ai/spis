use super::*;

pub(crate) fn save_state(state: &Value) -> Result<()> {
    let path = state_path();
    let parent = path
        .parent()
        .context("onboarding state path has no parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create onboarding state directory {}", parent.display()))?;

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = path.with_extension(format!("json.{}.{}.tmp", std::process::id(), nonce));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .with_context(|| format!("create onboarding state {}", temporary.display()))?;
    let body = serde_json::to_vec(state)?;
    file.write_all(&body)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(&temporary, &path)
        .with_context(|| format!("replace onboarding state {}", path.display()))?;
    Ok(())
}

pub(crate) fn render(screen: &Value) -> Result<()> {
    let presentation = screen
        .get("presentation")
        .and_then(Value::as_object)
        .context("canonical onboarding screen has no presentation")?;
    let title = presentation
        .get("title")
        .and_then(Value::as_str)
        .context("canonical onboarding screen has no presentation title")?;
    let body = presentation
        .get("body")
        .and_then(Value::as_str)
        .context("canonical onboarding screen has no presentation body")?;
    println!("\n== {title} ==\n{body}");
    Ok(())
}

pub(crate) fn wait_for_enter() -> Result<()> {
    print!("\nPress Enter to continue.");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(())
}

pub(crate) fn state_path() -> PathBuf {
    if let Some(path) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(path).join("spis/onboarding.json");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local/state/spis/onboarding.json");
    }
    PathBuf::from(".spis/onboarding.json")
}
