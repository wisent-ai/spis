use super::*;

pub(crate) const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

pub(crate) const CATALOG: &str = "cli-examples";

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) binary: String,
}

pub(crate) struct Invocation {
    pub(crate) argv: Vec<String>,
    pub(crate) output: String,
    pub(crate) exit_status: Option<i32>,
    pub(crate) timed_out: bool,
    pub(crate) state_path: String,
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
        .unwrap_or("cli_record_failed")
}

pub(crate) fn safe_component(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{flag} must contain only letters, digits, '.', '-' or '_'");
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

pub(crate) fn binary_for(slug: &str) -> String {
    let tail = slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug);
    match tail {
        "github-cli-gh" => "gh",
        "gitlab-cli-glab" => "glab",
        "go-command" => "go",
        "npm-cli" => "npm",
        "homebrew" => "brew",
        "docker-cli" => "docker",
        "opentofu-cli" => "tofu",
        "ansible-command-line-tools" => "ansible",
        "terraform-cli" => "terraform",
        "google-cloud-cli-gcloud" => "gcloud",
        "azure-cli" => "az",
        "digitalocean-cli-doctl" => "doctl",
        "oracle-cloud-infrastructure-cli" => "oci",
        "httpie-cli" => "http",
        "gnu-wget" => "wget",
        "iproute2" => "ip",
        "openssl-command-line-tools" => "openssl",
        "vault-cli" => "vault",
        "sqlite-command-line-shell" => "sqlite3",
        "duckdb-cli" => "duckdb",
        "mongodb-shell-mongosh" => "mongosh",
        other => other,
    }
    .to_string()
}

pub(crate) fn records(selected: Option<&str>) -> Result<Vec<Record>> {
    let directory = super::corpus::data_root()
        .join(CATALOG)
        .join("references");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    paths.sort();
    let mut records = Vec::new();
    for path in paths {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if selected.is_some_and(|value| {
            value != slug && value != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug)
        }) {
            continue;
        }
        let document: Value = serde_json::from_slice(&std::fs::read(path.join("reference.json"))?)?;
        records.push(Record {
            slug: slug.to_string(),
            name: document
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            binary: binary_for(slug),
        });
    }
    if records.is_empty() {
        bail!("no matching CLI records");
    }
    Ok(records)
}

pub(crate) fn delivery_secret_bindings(
    manifest: &super::crawl::RuntimeManifest,
) -> Result<Vec<(String, String)>> {
    let value = serde_json::to_value(manifest)?;
    let secrets = value
        .pointer("/delivery/secret_env")
        .and_then(Value::as_object)
        .context("CLI runtime manifest has no typed delivery.secret_env map")?;
    let mut bindings = Vec::with_capacity(secrets.len());
    for (name, reference) in secrets {
        let reference = reference
            .as_str()
            .filter(|value| !value.is_empty())
            .with_context(|| format!("CLI delivery secret reference is invalid for environment key {name:?}"))?;
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
            bail!("CLI delivery secret binding has invalid environment key {name:?}");
        }
        if [
            "PATH",
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "GIT_CONFIG_GLOBAL",
            "GIT_CONFIG_NOSYSTEM",
            "KUBECONFIG",
            "DOCKER_HOST",
            "AWS_EC2_METADATA_DISABLED",
            "TERM",
            "PAGER",
            "GIT_PAGER",
            "MANPAGER",
            "NO_COLOR",
            "CI",
            "LANG",
        ]
        .contains(&name.as_str())
        {
            bail!("CLI delivery secret key {name:?} would override the isolated worker environment");
        }
        bindings.push((name.clone(), reference.to_string()));
    }
    Ok(bindings)
}

pub(crate) fn delivery_secret_names(manifest: &super::crawl::RuntimeManifest) -> Result<Vec<String>> {
    Ok(delivery_secret_bindings(manifest)?
        .into_iter()
        .map(|(name, _)| name)
        .collect())
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
    environment.insert("GIT_CONFIG_GLOBAL".into(), fixture.join("gitconfig").into_os_string());
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert("KUBECONFIG".into(), fixture.join("kubeconfig").into_os_string());
    environment.insert("DOCKER_HOST".into(), format!("unix://{}", fixture.join("docker.sock").display()).into());
    environment.insert("AWS_EC2_METADATA_DISABLED".into(), "true".into());
    environment.insert("TERM".into(), "xterm-256color".into());
    environment.insert("PAGER".into(), "cat".into());
    environment.insert("GIT_PAGER".into(), "cat".into());
    environment.insert("MANPAGER".into(), "cat".into());
    environment.insert("NO_COLOR".into(), "1".into());
    environment.insert("CI".into(), "1".into());
    environment.insert("LANG".into(), "C.UTF-8".into());
    for name in delivery_secret_names(manifest)? {
        let value = std::env::var_os(&name)
            .with_context(|| format!("CLI worker did not receive manifest-bound secret environment key {name}"))?;
        environment.insert(name.into(), value);
    }
    Ok(environment)
}
