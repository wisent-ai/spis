//! The first-use walkthrough reads in the person's language: the real `spis
//! onboarding` runs with its state in an isolated directory and a language
//! from the environment, and its first screen, prompt and lines must be that
//! language's; a language the fleet lacks reads the journey's English, and
//! every key of texts/catalogue.json reads in every fleet language without
//! falling back. Every run's command, status and output are retained under
//! `.build/real-tests/onboarding/<run>/report.json`.
//!
//!   cargo test --test onboarding-languages

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct Run {
    root: PathBuf,
    report: Vec<Value>,
}

impl Run {
    fn walkthrough(&mut self, name: &str, language: &str, input: &str) -> Result<String> {
        let state = self.root.join(name);
        fs::create_dir_all(&state)?;
        let mut child = Command::new(env!("CARGO_BIN_EXE_spis"))
            .arg("onboarding")
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", &state)
            .env("XDG_STATE_HOME", &state)
            .env("LC_ALL", language)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("start spis onboarding")?;
        child
            .stdin
            .take()
            .context("spis onboarding has no stdin")?
            .write_all(input.as_bytes())?;
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        self.report.push(json!({
            "run": name,
            "language": language,
            "status": output.status.code(),
            "stdout": stdout,
            "stderr": String::from_utf8_lossy(&output.stderr),
        }));
        ensure!(
            output.status.success(),
            "spis onboarding in {language} exited {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(stdout)
    }
}

fn source_revision(root: &Path) -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "unknown (not a git checkout)".to_owned())
}

fn main() -> Result<()> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = manifest
        .join(".build/real-tests/onboarding")
        .join(format!("{stamp}-{}", std::process::id()));
    fs::create_dir_all(&root)?;
    let mut run = Run { root: root.clone(), report: Vec::new() };
    let outcome = check(&mut run, manifest);
    fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&json!({
            "source_revision": source_revision(manifest),
            "runs": run.report,
            "verdict": if outcome.is_ok() { "passed" } else { "failed" },
            "failure": outcome.as_ref().err().map(|error| format!("{error:#}")),
        }))?,
    )?;
    outcome?;
    println!("passed: {}", root.join("report.json").display());
    Ok(())
}

fn check(run: &mut Run, manifest: &Path) -> Result<()> {
    let bundle: Value = serde_json::from_slice(&fs::read(manifest.join("texts/bundle.json"))?)?;
    let catalogue: Value = serde_json::from_slice(&fs::read(manifest.join("texts/catalogue.json"))?)?;
    let keys: Vec<&String> = catalogue["texts"]
        .as_object()
        .context("texts/catalogue.json holds no texts")?
        .keys()
        .collect();
    let translations = bundle["translations"].as_object().context("texts/bundle.json holds no translations")?;
    ensure!(translations.len() == 65, "the bundle holds {} languages, the fleet 65 besides English", translations.len());
    for (code, words) in translations {
        for key in &keys {
            let text = words[key.as_str()].as_str().unwrap_or_default();
            ensure!(!text.is_empty() && text != key.as_str(), "{code} lacks {key}");
        }
    }

    // Polish: the first screen, the prompt, then the second screen.
    let polish = run.walkthrough("pl", "pl_PL.UTF-8", "\n")?;
    let pl = &translations["pl"];
    for key in [
        "spis.onboarding.evidence_corpus.title",
        "walkthrough.press_enter",
        "spis.onboarding.command_surface.title",
    ] {
        let text = pl[key].as_str().context("pl lacks a key")?;
        ensure!(polish.contains(text), "the Polish walkthrough does not show {key} ({text}): {polish}");
    }
    ensure!(!polish.contains("Press Enter to continue."), "the Polish walkthrough shows the English prompt: {polish}");

    // Japanese: a script far from Latin, read from the same LC_ALL.
    let japanese = run.walkthrough("ja", "ja_JP.UTF-8", "")?;
    let ja_title = translations["ja"]["spis.onboarding.evidence_corpus.title"].as_str().unwrap_or_default();
    ensure!(japanese.contains(ja_title), "the Japanese walkthrough does not show its title: {japanese}");

    // A language the fleet does not speak reads the journey's English.
    let other = run.walkthrough("xx", "xx_XX.UTF-8", "")?;
    ensure!(
        other.contains("Start from an evidence corpus you already have") && other.contains("Press Enter to continue."),
        "a language the fleet lacks does not read English: {other}"
    );
    Ok(())
}
