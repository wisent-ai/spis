use super::*;

pub(crate) fn save_state(state: &Value) -> Result<()> {
    let path = state_path()?;
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

/// The walkthrough's words in every fleet language (`texts/`, written by
/// wisent-texts' `texts bundle`): each screen's `title_key` and `body_key`,
/// whose English is the screen's presentation, and the walkthrough's own
/// lines.
pub(crate) fn texts() -> Result<&'static wisent_texts::Texts> {
    static TEXTS: std::sync::LazyLock<std::result::Result<wisent_texts::Texts, String>> =
        std::sync::LazyLock::new(|| {
            wisent_texts::Texts::bundle(include_str!("../../texts/bundle.json"))
                .map_err(|error| error.to_string())
        });
    TEXTS
        .as_ref()
        .map_err(|cause| anyhow::anyhow!("texts/bundle.json is the walkthrough's words: {cause}"))
}

/// One of the walkthrough's own lines in the reader's language.
pub(crate) fn say(key: &str, values: &[(&str, &str)]) -> Result<String> {
    Ok(texts()?.say(&wisent_texts::language(), key, values))
}

/// A screen in the reader's language: its title and body by their keys, the
/// presentation's English when a language lacks them. A screen without its
/// keys or presentation is refused by name.
pub(crate) fn render(screen: &Value) -> Result<()> {
    let field = |name: &str| -> Result<&str> {
        screen
            .get(name)
            .and_then(Value::as_str)
            .with_context(|| format!("canonical onboarding screen has no {name}"))
    };
    let presentation = screen
        .get("presentation")
        .and_then(Value::as_object)
        .context("canonical onboarding screen has no presentation")?;
    let english = |name: &str| -> Result<&str> {
        presentation
            .get(name)
            .and_then(Value::as_str)
            .with_context(|| format!("canonical onboarding screen has no presentation {name}"))
    };
    let language = wisent_texts::language();
    let words = texts()?;
    let read = |key: &str, fallback: &str| {
        let text = words.say(&language, key, &[]);
        if text == key { fallback.to_owned() } else { text }
    };
    let title = read(field("title_key")?, english("title")?);
    let body = read(field("body_key")?, english("body")?);
    println!("\n== {title} ==\n{body}");
    Ok(())
}

pub(crate) fn wait_for_enter() -> Result<()> {
    print!("\n{}", say("walkthrough.press_enter", &[])?);
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(())
}

/// `$XDG_STATE_HOME/spis/onboarding.json`, else `~/.local/state/spis/onboarding.json`.
/// With neither set the location is refused rather than read as `./.spis`, where
/// each directory the journey ran from kept its own progress.
pub(crate) fn state_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("XDG_STATE_HOME") {
        return Ok(PathBuf::from(path).join("spis/onboarding.json"));
    }
    Ok(crate::commands::docs_corpus::home_dir()?.join(".local/state/spis/onboarding.json"))
}
