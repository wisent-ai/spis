//! Real refusals of `spis capture-widths` over a damaged catalog: a
//! `sources.json` without its `examples` list, one whose `examples` is not a
//! list, and a `references.json` whose entry is not a record object are each
//! refused naming the file and what it holds, where the command used to read
//! the catalog as empty and answer "nothing to capture" or "record not
//! found". A sound catalog still plans its captures in a dry run.
//!
//! The corpus is an isolated directory under `.build/`; every command, its
//! exit status and its streams are kept beside `report.json` there.

use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const CATALOG: &str = "capture-widths-probe";

struct Run {
    root: PathBuf,
    work: PathBuf,
    corpus: PathBuf,
    binary: PathBuf,
    report: Value,
}

impl Run {
    fn new() -> Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
        let build = root.join(".build");
        fs::create_dir_all(&build)?;
        ensure!(
            build.canonicalize()? == build,
            ".build must not redirect outside the checkout"
        );
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let work = build.join(format!("capture-widths-{}-{stamp}", std::process::id()));
        let corpus = work.join("corpus");
        fs::create_dir_all(&corpus)?;
        Ok(Self {
            root,
            work,
            corpus,
            binary: PathBuf::from(env!("CARGO_BIN_EXE_spis")),
            report: json!({"schema": "spis.capture-widths-refusals.v1", "state": "running", "commands": [], "checks": []}),
        })
    }

    fn run(&mut self, program: &Path, args: &[&str], cwd: &Path) -> Result<(bool, String, String)> {
        let number = self.report["commands"].as_array().unwrap().len();
        let stdout_path = self.work.join(format!("command-{number}.stdout"));
        let stderr_path = self.work.join(format!("command-{number}.stderr"));
        let status = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .stdout(File::create(&stdout_path)?)
            .stderr(File::create(&stderr_path)?)
            .status()
            .with_context(|| format!("start {}", program.display()))?;
        self.report["commands"].as_array_mut().unwrap().push(json!({
            "program": program, "args": args, "cwd": cwd, "exit_code": status.code(),
            "stdout": stdout_path, "stderr": stderr_path,
        }));
        Ok((
            status.success(),
            fs::read_to_string(stdout_path)?,
            fs::read_to_string(stderr_path)?,
        ))
    }

    fn capture_widths(&mut self) -> Result<(bool, String, String)> {
        let (binary, corpus) = (self.binary.clone(), self.corpus.clone());
        self.run(
            &binary,
            &[
                "capture-widths",
                CATALOG,
                "--host",
                "probe-host",
                "--dry-run",
            ],
            &corpus,
        )
    }

    fn check(&mut self, name: &str, passed: bool, observed: Value) -> Result<()> {
        self.report["checks"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": name, "passed": passed, "observed": observed.clone()}));
        ensure!(passed, "{name}: observed {observed}");
        Ok(())
    }

    fn catalog(&self, sources: Value, references: Value) -> Result<()> {
        let directory = self.corpus.join(format!("{CATALOG}-examples"));
        fs::create_dir_all(&directory)?;
        fs::write(
            directory.join("sources.json"),
            serde_json::to_vec_pretty(&sources)?,
        )?;
        fs::write(
            directory.join("references.json"),
            serde_json::to_vec_pretty(&references)?,
        )?;
        Ok(())
    }
}

fn sound_example() -> Value {
    json!({"name": "Probe", "source_url": "https://example.invalid/"})
}

fn sound_reference() -> Value {
    json!({"path": "references/probe/record.json"})
}

fn refusals(run: &mut Run) -> Result<()> {
    let root = run.root.clone();
    let (ok, revision, error) = run.run(Path::new("git"), &["rev-parse", "HEAD"], &root)?;
    ensure!(ok, "source revision: {error}");
    run.report["source_revision"] = json!(revision.trim());

    run.catalog(
        json!({"title": "no examples list"}),
        json!({"references": [sound_reference()]}),
    )?;
    let (ok, _, error) = run.capture_widths()?;
    run.check(
        "a sources.json without examples is refused naming the file and the key",
        !ok && error.contains("sources.json: examples is missing"),
        json!({"success": ok, "stderr": error}),
    )?;

    run.catalog(
        json!({"examples": "not a list"}),
        json!({"references": [sound_reference()]}),
    )?;
    let (ok, _, error) = run.capture_widths()?;
    run.check(
        "a sources.json whose examples is not a list is refused naming what it holds",
        !ok && error
            .contains("sources.json: examples holds \"not a list\" where a list is expected"),
        json!({"success": ok, "stderr": error}),
    )?;

    run.catalog(
        json!({"examples": [sound_example()]}),
        json!({"references": ["references/probe/record.json"]}),
    )?;
    let (ok, _, error) = run.capture_widths()?;
    run.check(
        "a references.json entry that is not a record object is refused naming its position",
        !ok && error.contains("references.json: references[0] holds \"references/probe/record.json\" where a record object is expected"),
        json!({"success": ok, "stderr": error}),
    )?;

    run.catalog(
        json!({"examples": [sound_example()]}),
        json!({"references": [sound_reference()]}),
    )?;
    let (ok, output, error) = run.capture_widths()?;
    run.check(
        "a sound catalog plans its captures in a dry run",
        ok && output.contains("dry run: planned"),
        json!({"success": ok, "stdout": output, "stderr": error}),
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let mut run = Run::new()?;
    let result = refusals(&mut run);
    run.report["state"] = json!(if result.is_ok() { "passed" } else { "failed" });
    if let Err(error) = &result {
        run.report["error"] = json!(format!("{error:#}"));
    }
    let report = run.work.join("report.json");
    fs::write(&report, serde_json::to_vec_pretty(&run.report)?)?;
    eprintln!("capture-widths refusals: {}", report.display());
    match result {
        Ok(()) => Ok(()),
        Err(error) => bail!("{error:#}"),
    }
}
