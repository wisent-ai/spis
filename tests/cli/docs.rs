//! Public namespace isolation and the real documentation generation lifecycle.

use super::{tree, Run};
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::fs;

pub(super) fn exercise(run: &mut Run) -> Result<()> {
    run.invoke(&["docs"], Some("spis docs corpus"))?;
    run.invoke(&["docs", "--help"], Some("spis docs site generate"))?;
    run.invoke(&["docs", "corpus"], Some("spis docs corpus"))?;
    run.invoke(&["docs", "corpus", "search", "--not-a-flag", "--help"], Some("spis docs corpus"))?;
    run.invoke(&["docs", "site"], Some("spis docs site generate"))?;
    run.invoke(&["docs", "site", "generate", "--help"], Some("spis docs site generate"))?;
    run.invoke(&["docs-corpus", "status"], None)?;
    run.invoke(&["docs-site"], None)?;
    run.invoke(&["docs", "undefined"], None)?;
    run.invoke(&["docs", "site", "undefined"], None)?;
    run.invoke(&["docs", "corpus", "undefined"], None)?;
    run.invoke(&["docs", "corpus", "status", "--query", "needle"], None)?;
    run.invoke(&["docs", "corpus", "search"], None)?;
    run.invoke(&["docs", "corpus", "search", "--query"], None)?;
    run.invoke(&["docs", "corpus", "search", "--query", ""], None)?;
    run.invoke(&["docs", "corpus", "search", "--query", "needle", "--url", "unused"], None)?;
    run.invoke(&["docs", "corpus", "search", "--query", "needle", "--limit", "not-a-count"], None)?;
    // Nonzero count boundary: https://doc.rust-lang.org/std/num/type.NonZeroUsize.html
    run.invoke(&["docs", "corpus", "search", "--query", "needle", "--limit", "0"], None)?;
    run.invoke(&["docs", "corpus", "show", "--site", "declared-site"], None)?;
    run.invoke(&["docs", "corpus", "show", "--site", "declared-site", "--url", "unused", "--query", "needle"], None)?;
    run.invoke(&["docs", "corpus", "import"], None)?;
    run.invoke(&["docs", "corpus", "import", "--attempt-receipt"], None)?;
    run.invoke(&["docs", "corpus", "import", "--query", "needle"], None)?;
    run.invoke(&["docs", "site", "generate", "--site-content"], None)?;
    generation(run)
}

fn generation(run: &mut Run) -> Result<()> {
    let settings = tree(&run.sandbox.join("config"))?;
    let home = tree(&run.sandbox.join("home"))?;
    let landing = run.sandbox.join("landing");
    let payload = landing.join("product-docs.json");
    let path = payload.to_str().context("generated documentation path is not UTF-8")?;
    let absent = tree(&run.sandbox)?;
    let missing = run.capture(&["docs", "site", "generate", "--check", "--site-content", path])?;
    // Operational failure exit: https://spis.wisent.com/docs/cli
    ensure!(missing.status.code() == Some(1), "missing generated outputs were not refused");
    ensure!(tree(&run.sandbox)? == absent, "checking absent outputs created or modified state");
    fs::create_dir(&landing)?;
    let generated = run.capture(&["docs", "site", "generate", "--site-content", path])?;
    ensure!(generated.status.success(), "initial generation failed; inspect retained command streams");
    ensure!(tree(&run.sandbox.join("config"))? == settings,
        "generation changed corpus adoption configuration");
    ensure!(tree(&run.sandbox.join("home"))? == home,
        "generation changed the isolated home directory");
    let baseline = fs::read(&payload).context("generation did not persist its selected DocPage payload")?;
    let _: Value = serde_json::from_slice(&baseline).context("generated payload is not JSON")?;
    let persisted = tree(&run.sandbox)?;
    let current = run.capture(&["docs", "site", "generate", "--check", "--site-content", path])?;
    ensure!(current.status.success(), "checking freshly generated documentation failed");
    ensure!(tree(&run.sandbox)? == persisted, "--check changed freshly generated outputs or configuration");

    let stale = b"a replaced documentation payload\n";
    fs::write(&payload, stale)?;
    let before_refusal = tree(&run.sandbox)?;
    let refused = run.capture(&["docs", "site", "generate", "--check", "--site-content", path])?;
    // Operational failure exit: https://spis.wisent.com/docs/cli
    ensure!(refused.status.code() == Some(1), "stale documentation was not an operational refusal");
    ensure!(tree(&run.sandbox)? == before_refusal, "--check repaired or otherwise modified stale output");

    let repaired = run.capture(&["docs", "site", "generate", "--site-content", path])?;
    ensure!(repaired.status.success(), "regenerating stale documentation failed");
    ensure!(fs::read(&payload)? == baseline, "regeneration did not restore the source-derived payload");
    let restored = tree(&run.sandbox)?;
    let rechecked = run.capture(&["docs", "site", "generate", "--check", "--site-content", path])?;
    ensure!(rechecked.status.success(), "regenerated documentation still fails its own check");
    ensure!(tree(&run.sandbox)? == restored, "checking regenerated documentation modified persisted state");
    Ok(())
}
