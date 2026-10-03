mod journeys;
mod support;

use anyhow::Result;
use serde_json::json;
use std::fs;

fn main() -> Result<()> {
    let mut run = support::Run::new()?;
    let result = (|| {
        let fixture = run.prepare()?;
        run.report["state"] = json!("running");
        journeys::run(&mut run, &fixture)
    })();
    match &result {
        Ok(()) => run.report["state"] = json!("passed"),
        Err(error) => {
            if run.report["state"] == "running" {
                run.report["state"] = json!("failed");
            }
            run.report["error"] = json!(format!("{error:#}"));
        }
    }
    let report = run.work.join("report.json");
    fs::write(&report, serde_json::to_vec_pretty(&run.report)?)?;
    eprintln!("Accessibility qualification: {}", report.display());
    result
}
