use super::*;

pub(crate) const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

pub(crate) const REPRESENTATIVE_INPUT_CLASSES: &[(&str, &str)] = &[
    ("arrow keys", "context-ambiguous navigation or mutation input"),
    ("focus keys", "context-ambiguous focus or commit input"),
    ("escape/cancel keys", "context-ambiguous cancel or discard input"),
    ("paging keys", "context-ambiguous navigation input"),
    ("activation keys", "ambiguous activation, confirmation, or toggle input"),
    ("digit keys", "application-specific numeric input"),
    ("text keys", "application-specific letter, search, help, or command input"),
    ("mouse and pointer", "context-ambiguous pointing, scrolling, or activation input"),
    ("paste and other input", "unclassified input has no authorized exception"),
];

pub(crate) struct TmuxSession {
    pub(crate) name: String,
    pub(crate) socket: PathBuf,
    pub(crate) environment: BTreeMap<OsString, OsString>,
}

impl Drop for TmuxSession {
    fn drop(&mut self) {
        // Short deadline: cleanup must never block the worker (finding 4).
        let mut command = Command::new("tmux");
        command
            .env_clear()
            .env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin")
            .args(["-S", self.socket.to_string_lossy().as_ref()])
            .args(["kill-session", "-t", &self.name]);
        let _ = super::crawl::bounded_command_output(
            &mut command,
            "close private TUI PTY",
            Duration::from_secs(5),
            64 * 1024,
        );
    }
}

/// Typed record failure so the worker report can name a stable machine code
/// instead of a free-text diagnostic.
#[derive(Debug)]
pub(crate) struct RecordFailure {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl std::fmt::Display for RecordFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for RecordFailure {}

pub(crate) fn failure_code(error: &anyhow::Error) -> &'static str {
    error
        .chain()
        .find_map(|cause| {
            cause
                .downcast_ref::<RecordFailure>()
                .map(|failure| failure.code)
        })
        .unwrap_or("tui_record_failed")
}

pub(crate) fn safe_component(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("{name} must contain only letters, digits, '-' or '_'");
    }
    Ok(())
}

pub(crate) fn attempt_root(
    base: &Path,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<PathBuf> {
    super::crawl::native_attempt_root(base, manifest)
}

pub(crate) fn revision() -> Result<String> { super::crawl::build_revision() }

pub(crate) fn binary_candidates(name: &str) -> Vec<String> {
    let lower = name.to_lowercase();
    vec![
        match lower.as_str() {
            "midnight commander" => "mc".to_string(),
            "github cli dashboard" => "gh-dash".to_string(),
            "bottom" => "btm".to_string(),
            other => other.to_string(),
        },
        lower.replace(' ', "-"),
        lower.replace(' ', ""),
    ]
}

pub(crate) fn delivery_secret_bindings(
    manifest: &super::crawl::RuntimeManifest,
) -> Result<Vec<(String, String)>> {
    let value = serde_json::to_value(manifest)?;
    let secrets = value
        .pointer("/delivery/secret_env")
        .and_then(Value::as_object)
        .context("TUI runtime manifest has no typed delivery.secret_env map")?;
    let mut bindings = Vec::with_capacity(secrets.len());
    for (name, reference) in secrets {
        let reference = reference
            .as_str()
            .filter(|value| !value.is_empty())
            .with_context(|| format!("TUI delivery secret reference is invalid for environment key {name:?}"))?;
        if name.is_empty()
            || !name
                .chars()
                .enumerate()
                .all(|(index, character)| {
                    character == '_'
                        || character.is_ascii_alphabetic()
                        || (index > 0 && character.is_ascii_digit())
                })
        {
            bail!("TUI delivery secret binding has invalid environment key {name:?}");
        }
        if [
            "PATH",
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "XDG_STATE_HOME",
            "XDG_RUNTIME_DIR",
            "GIT_CONFIG_GLOBAL",
            "GIT_CONFIG_NOSYSTEM",
            "KUBECONFIG",
            "DOCKER_HOST",
            "AWS_EC2_METADATA_DISABLED",
            "TERM",
            "NO_COLOR",
            "LANG",
        ]
        .contains(&name.as_str())
        {
            bail!("TUI delivery secret key {name:?} would override the isolated worker environment");
        }
        bindings.push((name.clone(), reference.to_string()));
    }
    Ok(bindings)
}

pub(crate) fn isolated_environment(
    manifest: &super::crawl::RuntimeManifest,
    fixture: &Path,
) -> Result<BTreeMap<OsString, OsString>> {
    let home = fixture.join("home");
    let mut environment = BTreeMap::new();
    environment.insert("PATH".into(), "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin".into());
    environment.insert("HOME".into(), home.clone().into_os_string());
    environment.insert("XDG_CONFIG_HOME".into(), home.join(".config").into_os_string());
    environment.insert("XDG_DATA_HOME".into(), home.join(".local/share").into_os_string());
    environment.insert("XDG_CACHE_HOME".into(), home.join(".cache").into_os_string());
    environment.insert("XDG_STATE_HOME".into(), home.join(".local/state").into_os_string());
    environment.insert("XDG_RUNTIME_DIR".into(), fixture.join("runtime").into_os_string());
    environment.insert("GIT_CONFIG_GLOBAL".into(), fixture.join("gitconfig").into_os_string());
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert("KUBECONFIG".into(), fixture.join("kubeconfig").into_os_string());
    environment.insert("DOCKER_HOST".into(), format!("unix://{}", fixture.join("docker.sock").display()).into());
    environment.insert("AWS_EC2_METADATA_DISABLED".into(), "true".into());
    environment.insert("TERM".into(), "xterm-256color".into());
    environment.insert("NO_COLOR".into(), "1".into());
    environment.insert("LANG".into(), "C.UTF-8".into());
    for (name, _) in delivery_secret_bindings(manifest)? {
        let value = std::env::var_os(&name)
            .with_context(|| format!("TUI worker did not receive manifest-bound secret environment key {name}"))?;
        environment.insert(name.into(), value);
    }
    Ok(environment)
}
