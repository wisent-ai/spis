//! Real CLI grammar, help isolation, generation and HTTP refusal transitions.
//! Every invocation and filesystem observation is retained under `.build/`.

mod backend;
mod docs;

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};

type Tree = BTreeMap<PathBuf, Option<Vec<u8>>>;

fn tree(root: &Path) -> Result<Tree> {
    fn visit(root: &Path, directory: &Path, observed: &mut Tree) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            let kind = entry.file_type()?;
            ensure!(!kind.is_symlink(), "fixture contains a symlink: {}", path.display());
            let relative = path.strip_prefix(root)?.to_path_buf();
            if kind.is_dir() {
                observed.insert(relative, None);
                visit(root, &path, observed)?;
            } else {
                ensure!(kind.is_file(), "fixture contains a special file: {}", path.display());
                observed.insert(relative, Some(fs::read(&path)?));
            }
        }
        Ok(())
    }
    let mut observed = BTreeMap::new();
    visit(root, root, &mut observed)?;
    Ok(observed)
}

fn retain_tree(path: &Path, observed: &Tree) -> Result<()> {
    let digests = observed.iter().map(|(path, bytes)| {
        (path, bytes.as_ref().map(|bytes| format!("{:x}", Sha256::digest(bytes))))
    }).collect::<BTreeMap<_, _>>();
    fs::write(path, serde_json::to_vec_pretty(&digests)?)
        .with_context(|| format!("retain filesystem observation {}", path.display()))
}

struct Observation {
    status: ExitStatus,
    output: String,
    unchanged: bool,
}

struct Run {
    root: PathBuf,
    work: PathBuf,
    sandbox: PathBuf,
    binary: PathBuf,
    path: String,
    commands: Vec<Value>,
    source: Value,
}

impl Run {
    fn new() -> Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
        let build = root.join(".build");
        fs::create_dir_all(&build)?;
        ensure!(build.canonicalize()? == build, ".build redirects outside the checkout");
        let parent = build.join("cli-contract");
        fs::create_dir_all(&parent)?;
        ensure!(parent.canonicalize()? == parent, "CLI report parent redirects outside the checkout");
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let work = parent.join(format!("{}-{stamp}", std::process::id()));
        fs::create_dir(&work)?;
        fs::create_dir(work.join("calls"))?;
        fs::create_dir(work.join("source"))?;
        let sandbox = work.join("sandbox");
        fs::create_dir_all(sandbox.join("config/spis"))?;
        fs::create_dir_all(sandbox.join("home"))?;
        fs::create_dir_all(sandbox.join("corpus"))?;
        fs::create_dir_all(sandbox.join("tmp"))?;
        fs::write(sandbox.join("config/spis/corpus.json"), b"not a corpus configuration\n")?;
        Ok(Self {
            root, work, sandbox,
            binary: PathBuf::from(env!("CARGO_BIN_EXE_spis")),
            path: std::env::var("PATH").context("CLI qualification requires a UTF-8 PATH")?,
            commands: Vec::new(),
            source: Value::Null,
        })
    }

    fn identify_source(&mut self) -> Result<()> {
        let source = self.work.join("source");
        let revision = Command::new("git").args(["rev-parse", "HEAD"])
            .env_clear().env("PATH", &self.path).env("HOME", self.sandbox.join("home"))
            .current_dir(&self.root).output().context("read source revision")?;
        fs::write(source.join("revision.stdout"), &revision.stdout)?;
        fs::write(source.join("revision.stderr"), &revision.stderr)?;
        self.source = json!({
            "revision_exit": revision.status.code(),
            "build_revision": env!("SPIS_GIT_REVISION"),
            "build_revision_refusal": env!("SPIS_GIT_REVISION_REFUSAL"),
        });
        ensure!(revision.status.success(), "git could not read the source revision");
        let revision_text = String::from_utf8(revision.stdout)?.trim().to_string();
        self.source["observed_revision"] = json!(revision_text);
        ensure!(!env!("SPIS_GIT_REVISION").is_empty(),
            "product build has no exact source revision: {}", env!("SPIS_GIT_REVISION_REFUSAL"));
        ensure!(revision_text == env!("SPIS_GIT_REVISION"),
            "checkout revision differs from the product build's revision");
        let state = Command::new("git").args(["status", "--porcelain"])
            .env_clear().env("PATH", &self.path).env("HOME", self.sandbox.join("home"))
            .current_dir(&self.root).output().context("observe source tree state")?;
        fs::write(source.join("state.stdout"), &state.stdout)?;
        fs::write(source.join("state.stderr"), &state.stderr)?;
        self.source["worktree_exit"] = json!(state.status.code());
        ensure!(state.status.success() && state.stdout.is_empty(),
            "source cannot be bound to the clean product build; inspect retained source state");
        let mut digests = BTreeMap::new();
        for (path, bytes) in tree(&self.root.join("src"))? {
            if let Some(bytes) = bytes {
                digests.insert(path, format!("{:x}", Sha256::digest(bytes)));
            }
        }
        self.source = json!({
            "revision": revision_text, "build_revision": env!("SPIS_GIT_REVISION"),
            "revision_command": ["git", "rev-parse", "HEAD"],
            "revision_exit": revision.status.code(), "worktree_exit": state.status.code(),
            "source_files_sha256": digests,
            "manifest_sha256": format!("{:x}", Sha256::digest(fs::read(self.root.join("Cargo.toml"))?)),
            "lock_sha256": format!("{:x}", Sha256::digest(fs::read(self.root.join("Cargo.lock"))?)),
            "binary_sha256": format!("{:x}", Sha256::digest(fs::read(&self.binary)?)),
            "journey_main_sha256": format!("{:x}", Sha256::digest(fs::read(self.root.join("tests/cli/main.rs"))?)),
            "journey_docs_sha256": format!("{:x}", Sha256::digest(fs::read(self.root.join("tests/cli/docs.rs"))?)),
            "journey_backend_sha256": format!("{:x}", Sha256::digest(fs::read(self.root.join("tests/cli/backend.rs"))?)),
        });
        Ok(())
    }

    fn capture(&mut self, args: &[&str]) -> Result<Observation> {
        let before = tree(&self.sandbox)?;
        let directory = self.work.join("calls").join(self.commands.len().to_string());
        fs::create_dir(&directory)?;
        retain_tree(&directory.join("before.json"), &before)?;
        let stdout = directory.join("stdout");
        let stderr = directory.join("stderr");
        let status = Command::new(&self.binary).args(args)
            .env_clear().env("PATH", &self.path)
            .current_dir(self.sandbox.join("corpus"))
            .env("HOME", self.sandbox.join("home"))
            .env("XDG_CONFIG_HOME", self.sandbox.join("config"))
            .env("TMPDIR", self.sandbox.join("tmp"))
            .stdout(File::create(&stdout)?).stderr(File::create(&stderr)?)
            .status().with_context(|| format!("start {} with {args:?}", self.binary.display()))?;
        let after = tree(&self.sandbox)?;
        retain_tree(&directory.join("after.json"), &after)?;
        let unchanged = before == after;
        self.commands.push(json!({
            "program": self.binary, "args": args,
            "cwd": self.sandbox.join("corpus"), "environment_cleared": true,
            "environment": {
                "PATH": self.path, "HOME": self.sandbox.join("home"),
                "XDG_CONFIG_HOME": self.sandbox.join("config"),
                "TMPDIR": self.sandbox.join("tmp"),
            },
            "exit_code": status.code(), "status": status.to_string(),
            "stdout": stdout, "stderr": stderr, "fixture_unchanged": unchanged,
        }));
        let output = fs::read_to_string(&stdout)?;
        Ok(Observation { status, output, unchanged })
    }

    fn invoke(&mut self, args: &[&str], help: Option<&str>) -> Result<()> {
        let observed = self.capture(args)?;
        let expected_status = match help {
            Some(invocation) => observed.status.success() && observed.output.contains(invocation),
            None => observed.status.code() == Some(2), // Usage: https://spis.wisent.com/docs/cli
        };
        ensure!(observed.unchanged, "{args:?} changed isolated corpus or configuration state");
        ensure!(expected_status,
            "{args:?}: unexpected status {} or missing public grammar; inspect retained streams",
            observed.status);
        Ok(())
    }

    fn engine(&mut self, engine: &str) -> Result<()> {
        let invocation = format!("spis crawler run {engine}");
        self.invoke(&["crawler", "run", engine, "--not-an-option", "--help"], Some(&invocation))?;
        let retired = format!("crawl-{engine}");
        self.invoke(&[&retired], None)
    }
}

fn exercise(run: &mut Run) -> Result<()> {
    run.identify_source()?;
    run.invoke(&["crawler"], Some("spis crawler run"))?;
    run.invoke(&["crawler", "run", "--help"], Some("spis crawler run"))?;
    run.invoke(&["crawler", "run"], None)?;
    run.invoke(&["crawler", "undefined"], None)?;
    run.invoke(&["crawler", "run", "undefined"], None)?;
    run.engine("cli")?;
    run.engine("docs")?;
    run.engine("desktop")?;
    run.engine("mobile")?;
    run.engine("web")?;
    run.engine("tui")?;
    docs::exercise(run)?;
    backend::exercise(run)
}

fn main() -> Result<()> {
    let mut run = Run::new()?;
    let result = exercise(&mut run);
    let report = run.work.join("report.json");
    fs::write(&report, serde_json::to_vec_pretty(&json!({
        "schema": "spis.cli-contract.v1",
        "state": if result.is_ok() { "passed" } else { "failed" },
        "source": run.source, "commands": run.commands,
        "error": result.as_ref().err().map(|error| format!("{error:#}")),
        "backend_report": run.work.join("backend/outcome.json").is_file().then(|| run.work.join("backend/outcome.json")),
        "qualification": "CLI grammar, filesystem isolation, documentation generation lifecycle and real HTTP field refusals; no retrieved-corpus, engine, native client or graphical qualification",
    }))?)?;
    eprintln!("CLI contract: {}", report.display());
    result
}
