//! Spis's declared refusals through the real binary: `spis refusals check`
//! passes on this checkout and refuses one that lacks the tests, naming each
//! cause; `list` and `show` print every cause with the exit status it
//! leaves; an unknown subcommand exits 2. Every command and its streams are
//! kept beside `report.json` under `.build/`.

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Run {
    root: PathBuf,
    work: PathBuf,
    binary: PathBuf,
    commands: Vec<Value>,
}

impl Run {
    fn new() -> Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
        let work = root
            .join(".build/real-tests/refusals")
            .join(std::process::id().to_string());
        if work.exists() {
            fs::remove_dir_all(&work)?;
        }
        fs::create_dir_all(&work)?;
        Ok(Self {
            root,
            work,
            binary: PathBuf::from(env!("CARGO_BIN_EXE_spis")),
            commands: Vec::new(),
        })
    }

    /// `spis args` run in `cwd`: the exit status, stdout and stderr.
    fn spis(&mut self, args: &[&str], cwd: &Path) -> Result<(Option<i32>, String, String)> {
        let output = Command::new(&self.binary)
            .args(args)
            .current_dir(cwd)
            .output()
            .with_context(|| format!("run spis {args:?}"))?;
        let (stdout, stderr) = (
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        );
        self.commands.push(json!({"args": args, "cwd": cwd, "exit": output.status.code(),
            "stdout": stdout, "stderr": stderr}));
        Ok((output.status.code(), stdout, stderr))
    }
}

/// `check` passes on this checkout, and `show` answers every cause with
/// the exit status the fleet declares for its code: 69 when retryable, 2
/// for a usage refusal, 1 otherwise.
fn every_declared_refusal_names_a_test_this_checkout_holds(run: &mut Run) -> Result<()> {
    let root = run.root.clone();
    let (code, out, err) = run.spis(&["refusals", "check"], &root)?;
    ensure!(code == Some(0), "check failed on this checkout: {err}");
    let checked: Value = serde_json::from_str(&out)?;
    ensure!(checked["problems"] == json!([]), "{checked}");

    let (code, out, err) = run.spis(&["refusals", "show", "spis"], &root)?;
    ensure!(code == Some(0), "show failed: {err}");
    let operation: Value = serde_json::from_str(&out)?;
    let causes = operation["causes"].as_array().context("the operation lists its causes")?;
    ensure!(
        causes.len() as u64 == checked["causes"].as_u64().unwrap_or_default(),
        "{operation}"
    );
    for cause in causes {
        let expected = if cause["usage"] == true {
            2
        } else if cause["retryable"] == true {
            69
        } else {
            1
        };
        ensure!(cause["exit"] == expected, "{cause}");
    }

    let (code, _, err) = run.spis(&["refusals", "show", "nothing"], &root)?;
    ensure!(code == Some(1), "an unknown operation exited {code:?}: {err}");
    ensure!(err.contains("no operation nothing declares its refusals"), "{err}");
    let (code, _, err) = run.spis(&["no-such-subcommand"], &root)?;
    ensure!(code == Some(2), "an unknown subcommand exited {code:?}: {err}");
    ensure!(err.contains("unknown subcommand: no-such-subcommand"), "{err}");
    Ok(())
}

/// A checkout without the tests the causes name is refused, each cause
/// named with the file it could not read.
fn check_names_each_cause_a_checkout_lacks(run: &mut Run) -> Result<()> {
    let bare = run.work.join("bare");
    fs::create_dir_all(&bare)?;
    let (code, _, err) = run.spis(
        &["refusals", "check", "--root", bare.to_str().context("UTF-8 path")?],
        &bare,
    )?;
    ensure!(code == Some(1), "a checkout without tests exited {code:?}: {err}");
    ensure!(err.contains("do not meet the error standard"), "{err}");
    ensure!(err.contains("cause catalog-damaged") && err.contains("cannot be read"), "{err}");
    Ok(())
}

fn main() -> Result<()> {
    let mut run = Run::new()?;
    let result = every_declared_refusal_names_a_test_this_checkout_holds(&mut run)
        .and_then(|()| check_names_each_cause_a_checkout_lacks(&mut run));
    let report = run.work.join("report.json");
    fs::write(
        &report,
        serde_json::to_vec_pretty(&json!({
            "state": if result.is_ok() { "passed" } else { "failed" },
            "commands": run.commands,
            "error": result.as_ref().err().map(|error| format!("{error:#}")),
        }))?,
    )?;
    eprintln!("refusals: {}", report.display());
    result
}
