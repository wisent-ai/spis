use crate::support::{digest, json_file, Fixture, Run};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path};

fn inspect(run: &mut Run, expected_exit: i32, name: &str) -> Result<Value> {
    let (code, stdout, stderr) = run.spis(&[
        "check-upstream-drift",
        "--skip-network",
        "--write-report",
        "--strict",
    ])?;
    ensure!(
        code == expected_exit,
        "{name}: exit {code}\n{stdout}\n{stderr}"
    );
    ensure!(
        !run.corpus.join("upstream-drift.json").exists(),
        "operational drift report leaked into the public corpus root"
    );
    let report = json_file(&run.corpus.join(".build/upstream-drift.json"))?;
    run.report["journeys"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": name, "exit_code": code, "observation": report}));
    Ok(report)
}

pub fn run(run: &mut Run, fixture: &Fixture) -> Result<()> {
    let catalog = fixture.catalog_dir.file_name().context("catalog name")?;
    let root = run.corpus.join(catalog);
    let references = json_file(&root.join("references.json"))?;
    let pointer = references["references"]
        .as_array()
        .context("catalog references")?
        .iter()
        .find(|row| row["index"].as_u64() == Some(fixture.record_index))
        .context("selected reference is absent")?;
    let relative = Path::new(pointer["path"].as_str().context("reference path")?);
    ensure!(
        relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
            && relative.file_name().and_then(|name| name.to_str()) == Some("reference.json"),
        "fixture reference escapes its catalog"
    );
    let record = root.join(relative);
    let data = json_file(&record)?;
    let state = data["states"]
        .as_array()
        .context("fixture requires retained state evidence")?
        .iter()
        .find(|state| state["local_path"].is_string() && state["sha256"].is_string())
        .context("fixture requires a real state artifact and recorded SHA-256")?;
    let local = Path::new(state["local_path"].as_str().context("state path")?);
    ensure!(
        !local.as_os_str().is_empty()
            && local
                .components()
                .all(|part| matches!(part, Component::Normal(_))),
        "state artifact must stay in its reference directory"
    );
    let media = record
        .parent()
        .context("reference directory")?
        .join(local)
        .canonicalize()?;
    ensure!(
        media.starts_with(&run.corpus),
        "only isolated copied media may be changed"
    );
    let source = fixture
        .catalog_dir
        .join(relative)
        .parent()
        .context("source reference directory")?
        .join(local);
    let original_hash = digest(&source)?;
    ensure!(
        state["sha256"] == original_hash && digest(&media)? == original_hash,
        "fixture bytes do not match their measured provenance"
    );
    let before = inspect(run, 0, "private-drift-report")?;
    ensure!(
        before["local_media_missing"] == json!([])
            && before["local_media_hash_mismatch"] == json!([]),
        "intact fixture reports drift"
    );
    let backup = run.work.join("drift-original-media");
    fs::copy(&media, &backup)?;
    let refusal = (|| -> Result<()> {
        OpenOptions::new()
            .append(true)
            .open(&media)?
            .write_all(&[0])?;
        let changed = inspect(run, 1, "real-media-hash-mismatch")?;
        let mismatches = changed["local_media_hash_mismatch"]
            .as_array()
            .context("mismatch identities")?;
        let mut identified = false;
        for mismatch in mismatches {
            let path = run
                .corpus
                .join(mismatch.as_str().context("mismatch file identity")?);
            if path.canonicalize()? == media {
                identified = true;
            }
        }
        ensure!(
            identified,
            "strict refusal did not identify the actual changed media file"
        );
        Ok(())
    })();
    fs::copy(&backup, &media).context("restore only the isolated media copy")?;
    ensure!(
        digest(&media)? == original_hash,
        "isolated media restoration failed"
    );
    refusal?;
    let restored = inspect(run, 0, "restored-media-integrity")?;
    ensure!(
        restored["local_media_verified"] == before["local_media_verified"]
            && restored["local_media_missing"] == json!([])
            && restored["local_media_hash_mismatch"] == json!([]),
        "restored real bytes did not restore measured integrity"
    );
    ensure!(
        digest(&source)? == original_hash,
        "source media changed during isolated qualification"
    );
    run.report["drift_media"] = json!({"source": source, "isolated_copy": media, "sha256": original_hash, "source_unchanged": true});
    Ok(())
}
