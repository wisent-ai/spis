use crate::support::{digest, json_file, Fixture, Run};
use anyhow::{ensure, Context, Result};
use serde_json::json;
use std::fs;
use std::path::{Component, Path};

pub fn run(run: &mut Run, fixture: &Fixture) -> Result<()> {
    let catalog = fixture
        .catalog_dir
        .file_name()
        .and_then(|name| name.to_str())
        .context("catalog name")?;
    let catalog_root = run.corpus.join(catalog);
    let references = json_file(&catalog_root.join("references.json"))?;
    let pointer = references["references"]
        .as_array()
        .context("fixture references")?
        .iter()
        .find(|row| row["index"].as_u64() == Some(fixture.record_index))
        .context("selected real record is absent")?;
    let relative = Path::new(pointer["path"].as_str().context("reference path")?);
    ensure!(
        relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
            && relative.file_name().and_then(|name| name.to_str()) == Some("reference.json"),
        "reference path must remain in its catalog"
    );
    let original = fixture.catalog_dir.join(relative);
    let original_hash = digest(&original)?;
    let record = catalog_root.join(relative);
    let result = (|| {
        let selection = fixture.record_index.to_string();
        let base = [
            "audit-reference-accessibility",
            "--catalog",
            catalog,
            "--records",
            &selection,
            "--target",
            &fixture.target,
        ];
        let outside = run.corpus.join("outside-plan.json");
        let mut refusal = Vec::with_capacity(base.len() + 2);
        refusal.extend_from_slice(&base);
        refusal.extend([
            "--plan",
            outside
                .to_str()
                .context("qualification path must be UTF-8")?,
        ]);
        let (code, _, stderr) = run.spis(&refusal)?;
        ensure!(code == 2, "outside-plan refusal: exit {code}: {stderr}");
        ensure!(
            !outside.exists(),
            "refused plan was written outside private audit storage"
        );
        ensure!(
            digest(&record)? == original_hash,
            "refusal mutated the selected reference"
        );
        run.report["journeys"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "outside-plan-refusal", "state": "passed"}));
        for supplied in [
            None,
            Some(".build/accessibility-audit/plans/operator-selected.json"),
        ] {
            let name = if supplied.is_some() {
                "relative"
            } else {
                "default"
            };
            let batch = format!("{}-{name}", run.batch_prefix);
            let state = run.corpus.join(".build/accessibility-audit");
            let plan = match supplied {
                Some(path) => run.corpus.join(path),
                None => state.join("plans").join(format!("{batch}.json")),
            };
            let mut args = Vec::with_capacity(base.len() + 4);
            args.extend_from_slice(&base);
            args.extend(["--batch", &batch]);
            if let Some(path) = supplied {
                args.extend(["--plan", path]);
            }
            let (code, stdout, stderr) = run.spis(&args)?;
            ensure!(
                code == 0,
                "real {name} audit failed: exit {code}\n{stdout}\n{stderr}"
            );
            ensure!(
                !run.corpus.join("accessibility-audit-index.json").exists(),
                "operational index leaked to the public corpus root"
            );
            let index = json_file(&state.join("index.json"))?;
            ensure!(
                index["batch"] == batch && index["target"] == fixture.target,
                "retained index does not identify this execution"
            );
            ensure!(
                index["plan"].as_str() == plan.to_str(),
                "retained plan is not the requested private plan"
            );
            ensure!(
                index["totals"] == json!({"planned": 1, "complete": 1, "failed": 0, "pending": 0}),
                "audit did not complete the selected record: {}",
                index["totals"]
            );
            let row = index["records"]
                .as_array()
                .context("index records")?
                .first()
                .context("selected result")?;
            ensure!(
                row["status"] == "complete" && row["index"].as_u64() == Some(fixture.record_index),
                "wrong record completed"
            );
            let persisted = json_file(&record)?;
            let measurement = &persisted["accessibility"]["measurement"];
            ensure!(
                persisted["accessibility"]["measured"] == true,
                "record did not retain the measurement"
            );
            let media = record
                .parent()
                .context("reference directory")?
                .join("media/accessibility");
            let raw = media.join("axe.json");
            let summary = json_file(&media.join("axe-summary.json"))?;
            let sha = digest(&raw)?;
            let bytes = fs::metadata(&raw)?.len();
            ensure!(
                measurement["raw_sha256"] == sha
                    && measurement["raw_bytes"].as_u64() == Some(bytes),
                "persisted measurement disagrees with retained raw bytes"
            );
            ensure!(
                summary["sha256"] == sha && summary["bytes"].as_u64() == Some(bytes),
                "downloaded summary disagrees with retained raw bytes"
            );
            ensure!(
                measurement["captured_at"] == summary["captured_at"]
                    && measurement["violation_count"] == summary["violation_count"],
                "persisted measurement disagrees with the actual observed audit"
            );
            let slug = relative
                .parent()
                .and_then(Path::file_name)
                .context("reference slug")?;
            let staged = state
                .join("staging")
                .join(&batch)
                .join(catalog)
                .join(slug)
                .join("axe.json");
            ensure!(
                digest(&staged)? == sha,
                "private staged artifact differs from the installed measurement"
            );
            run.report["journeys"].as_array_mut().unwrap().push(json!({"name": format!("live-{name}-plan"),
                "state": "passed", "index": index, "plan_sha256": digest(&plan)?, "staged_raw": staged,
                "raw_sha256": sha, "raw_bytes": bytes, "record_sha256": digest(&record)?}));
        }
        Ok(())
    })();
    let unchanged = digest(&original)? == original_hash;
    run.report["original_record"] =
        json!({"path": original, "sha256": original_hash, "unchanged": unchanged});
    ensure!(
        unchanged,
        "qualification changed the source corpus instead of its isolated copy"
    );
    result
}
