use anyhow::{bail, ensure, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub catalog_dir: PathBuf,
    pub record_index: u64,
    pub target: String,
}

pub struct Run {
    pub work: PathBuf,
    pub corpus: PathBuf,
    pub report: Value,
    pub batch_prefix: String,
    root: PathBuf,
    binary: PathBuf,
}

pub fn digest(path: &Path) -> Result<String> {
    let mut input = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hex::encode(hash.finalize()))
}

fn copy_data(source: &Path, destination: &Path, inventory: &mut Vec<Value>) -> Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    ensure!(
        !metadata.file_type().is_symlink(),
        "fixture contains a symlink: {}",
        source.display()
    );
    ensure!(
        source.file_name().and_then(|name| name.to_str()) != Some(".git"),
        "fixtures must contain corpus data, not a checkout"
    );
    if metadata.is_dir() {
        fs::create_dir(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_data(
                &entry.path(),
                &destination.join(entry.file_name()),
                inventory,
            )?;
        }
    } else if metadata.is_file() {
        let mut input = File::open(source)?;
        let mut output = File::create(destination)?;
        let mut hash = Sha256::new();
        let mut bytes = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            output.write_all(&buffer[..count])?;
            hash.update(&buffer[..count]);
            bytes += count as u64;
        }
        inventory.push(
            json!({"source": source, "retained": destination, "bytes": bytes,
            "sha256": hex::encode(hash.finalize())}),
        );
    } else {
        bail!("fixture contains a non-regular file: {}", source.display());
    }
    Ok(())
}

impl Run {
    pub fn new() -> Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
        let build = root.join(".build");
        fs::create_dir_all(&build)?;
        ensure!(
            build.canonicalize()? == build,
            ".build must not redirect outside the checkout"
        );
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let batch_prefix = format!("accessibility-live-{}-{stamp}", std::process::id());
        let work = build.join(&batch_prefix);
        fs::create_dir(&work)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&work, fs::Permissions::from_mode(0o700))?;
        }
        let corpus = work.join("corpus");
        fs::create_dir(&corpus)?;
        fs::create_dir(work.join("config"))?;
        Ok(Self {
            root,
            work,
            corpus,
            batch_prefix,
            binary: PathBuf::from(env!("CARGO_BIN_EXE_spis")),
            report: json!({"schema": "spis.accessibility-qualification.v1", "state": "blocked", "commands": [], "journeys": []}),
        })
    }

    fn command(
        report: &mut Value,
        work: &Path,
        program: &Path,
        args: &[&str],
        cwd: &Path,
    ) -> Result<(i32, String, String)> {
        let number = report["commands"].as_array().unwrap().len();
        let stdout_path = work.join(format!("command-{number}.stdout"));
        let stderr_path = work.join(format!("command-{number}.stderr"));
        let config = work.join("config");
        let status = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .env("XDG_CONFIG_HOME", &config)
            .stdout(File::create(&stdout_path)?)
            .stderr(File::create(&stderr_path)?)
            .status();
        let mut observation = json!({"program": program, "args": args, "cwd": cwd,
            "xdg_config_home": config, "stdout": stdout_path, "stderr": stderr_path});
        match &status {
            Ok(status) => {
                observation["exit_code"] = json!(status.code());
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    observation["signal"] = json!(status.signal());
                }
            }
            Err(error) => observation["spawn_error"] = json!(error.to_string()),
        }
        report["commands"].as_array_mut().unwrap().push(observation);
        let status = status.with_context(|| format!("start {}", program.display()))?;
        Ok((
            status.code().unwrap_or(-1),
            fs::read_to_string(stdout_path)?,
            fs::read_to_string(stderr_path)?,
        ))
    }

    pub fn spis(&mut self, args: &[&str]) -> Result<(i32, String, String)> {
        Self::command(
            &mut self.report,
            &self.work,
            &self.binary,
            args,
            &self.corpus,
        )
    }

    pub fn prepare(&mut self) -> Result<Fixture> {
        let (code, revision, error) = Self::command(
            &mut self.report,
            &self.work,
            Path::new("git"),
            &["rev-parse", "HEAD"],
            &self.root,
        )?;
        ensure!(code == 0, "source revision: {error}");
        self.report["source_revision"] = json!(revision.trim());
        let args = [
            "status",
            "--porcelain",
            "--untracked-files=normal",
            "--",
            "src",
            "build.rs",
            "Cargo.toml",
            "Cargo.lock",
            "tests/accessibility",
        ];
        let (code, state, error) = Self::command(
            &mut self.report,
            &self.work,
            Path::new("git"),
            &args,
            &self.root,
        )?;
        self.report["source_state"] = json!(state);
        ensure!(
            code == 0 && state.is_empty(),
            "commit the executable and qualification source before running: {error}"
        );
        self.report["binary_sha256"] = json!(digest(&self.binary)?);
        let file = std::env::var_os("SPIS_ACCESSIBILITY_FIXTURE").context(
            "SPIS_ACCESSIBILITY_FIXTURE must name an actual catalog fixture and Stado target",
        )?;
        let file = PathBuf::from(file);
        let bytes = fs::read(&file).with_context(|| format!("read fixture {}", file.display()))?;
        self.report["fixture_sha256"] = json!(hex::encode(Sha256::digest(&bytes)));
        let mut fixture: Fixture = serde_json::from_slice(&bytes)?;
        fixture.catalog_dir = fixture.catalog_dir.canonicalize()?;
        let catalog = fixture
            .catalog_dir
            .file_name()
            .and_then(|name| name.to_str())
            .context("catalog directory has no name")?;
        ensure!(
            catalog.ends_with("-examples")
                && catalog
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
            "fixture must name a real catalog directory"
        );
        ensure!(
            fixture.record_index > 0 && !fixture.target.trim().is_empty(),
            "fixture requires a positive record_index and a Stado-selected target"
        );
        let destination = self.corpus.join(catalog);
        ensure!(
            !destination.starts_with(&fixture.catalog_dir),
            "the fixture cannot contain its destination"
        );
        let mut inventory = Vec::new();
        copy_data(&fixture.catalog_dir, &destination, &mut inventory)?;
        self.report["inputs"] = json!(inventory);
        self.report["fixture"] = json!({"catalog_dir": fixture.catalog_dir, "record_index": fixture.record_index, "target": fixture.target});
        Ok(fixture)
    }
}
