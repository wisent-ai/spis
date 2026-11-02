//! Real lifecycle of `spis catalog-type`: add, edit (title and rename) and
//! remove one catalog beside the checked-in interface families, plus the
//! refusals a person meets.
//!
//! The corpus is an isolated directory under `.build/` whose interface
//! families are symbolic links to the checkout's catalogs, so the real records
//! are read and never written. The family set is read from the checkout's
//! generated `example-catalogs.json`. Every command, its exit status and its
//! streams are kept beside `report.json` in the run directory.

use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const PROBE: &str = "code-quality-lifecycle-probe-examples";
const RENAMED: &str = "code-quality-lifecycle-renamed-examples";

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
        let work = build.join(format!(
            "catalog-type-lifecycle-{}-{stamp}",
            std::process::id()
        ));
        let corpus = work.join("corpus");
        fs::create_dir_all(&corpus)?;
        fs::create_dir(work.join("config"))?;
        Ok(Self {
            root,
            work,
            corpus,
            binary: PathBuf::from(env!("CARGO_BIN_EXE_spis")),
            report: json!({"schema": "spis.catalog-type-lifecycle.v1", "state": "running", "commands": [], "checks": []}),
        })
    }

    fn run(&mut self, program: &Path, args: &[&str], cwd: &Path) -> Result<(i32, String, String)> {
        let number = self.report["commands"].as_array().unwrap().len();
        let stdout_path = self.work.join(format!("command-{number}.stdout"));
        let stderr_path = self.work.join(format!("command-{number}.stderr"));
        let status = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .env("XDG_CONFIG_HOME", self.work.join("config"))
            .stdout(File::create(&stdout_path)?)
            .stderr(File::create(&stderr_path)?)
            .status()
            .with_context(|| format!("start {}", program.display()))?;
        self.report["commands"].as_array_mut().unwrap().push(json!({
            "program": program, "args": args, "cwd": cwd, "exit_code": status.code(),
            "stdout": stdout_path, "stderr": stderr_path,
        }));
        Ok((
            status.code().unwrap_or(-1),
            fs::read_to_string(stdout_path)?,
            fs::read_to_string(stderr_path)?,
        ))
    }

    fn spis(&mut self, args: &[&str]) -> Result<(i32, String, String)> {
        let (binary, corpus) = (self.binary.clone(), self.corpus.clone());
        self.run(&binary, args, &corpus)
    }

    fn check(&mut self, name: &str, passed: bool, observed: Value) -> Result<()> {
        self.report["checks"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": name, "passed": passed, "observed": observed.clone()}));
        ensure!(passed, "{name}: observed {observed}");
        Ok(())
    }

    fn index(&self) -> Result<Value> {
        json_file(&self.corpus.join("example-catalogs.json"))
    }
}

fn json_file(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("decode {}", path.display()))
}

fn slugs(index: &Value) -> Vec<String> {
    index["catalogs"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|entry| entry["slug"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn lifecycle(run: &mut Run) -> Result<()> {
    let root = run.root.clone();
    let (code, revision, error) = run.run(Path::new("git"), &["rev-parse", "HEAD"], &root)?;
    ensure!(code == 0, "source revision: {error}");
    run.report["source_revision"] = json!(revision.trim());
    let (code, state, error) = run.run(
        Path::new("git"),
        &[
            "status",
            "--porcelain",
            "--",
            "src",
            "build.rs",
            "Cargo.toml",
            "Cargo.lock",
            "tests/catalog_type",
        ],
        &root,
    )?;
    ensure!(
        code == 0 && state.is_empty(),
        "commit the executable and test source before running: {state}{error}"
    );

    let families = slugs(&json_file(&root.join("example-catalogs.json"))?);
    ensure!(
        !families.is_empty(),
        "the checkout's example-catalogs.json lists no catalogs"
    );
    for family in &families {
        std::os::unix::fs::symlink(root.join(family), run.corpus.join(family))?;
    }
    run.report["families"] = json!(families);

    let (code, _, error) = run.spis(&["generate-example-catalogs"])?;
    run.check(
        "baseline index regenerates",
        code == 0,
        json!({"exit": code, "stderr": error}),
    )?;

    // add
    let (code, out, error) = run.spis(&[
        "catalog-type",
        "add",
        "code-quality-lifecycle-probe",
        "--title",
        "Lifecycle probe",
        "--description",
        "Catalog created by the lifecycle test.",
    ])?;
    run.check(
        "add succeeds",
        code == 0,
        json!({"exit": code, "stdout": out, "stderr": error}),
    )?;
    let sources = json_file(&run.corpus.join(PROBE).join("sources.json"))?;
    run.check(
        "add scaffolds zero records",
        sources["status"] == "scaffolded"
            && sources["count"] == 0
            && sources["title"] == "Lifecycle probe",
        sources.clone(),
    )?;
    let index = run.index()?;
    let listed = slugs(&index);
    let mut expected = families.clone();
    expected.push(PROBE.into());
    run.check(
        "index lists the families, then the new catalog",
        listed == expected,
        json!(listed),
    )?;
    let stats = json_file(&run.corpus.join("catalog-stats.json"))?;
    run.check(
        "stats count the new catalog",
        stats["catalog_count"] == json!(families.len() + 1),
        stats,
    )?;
    let (code, out, error) = run.spis(&["verify-reference-evidence", "--catalog", PROBE])?;
    run.check(
        "verifier accepts the empty catalog",
        code == 0,
        json!({"exit": code, "stdout": out, "stderr": error}),
    )?;

    // refusals
    let (code, _, error) = run.spis(&[
        "catalog-type",
        "add",
        "code-quality-lifecycle-probe",
        "--title",
        "Again",
    ])?;
    run.check(
        "duplicate add is refused",
        code != 0 && error.contains("already exists"),
        json!({"exit": code, "stderr": error}),
    )?;
    let (code, _, error) = run.spis(&["catalog-type", "add", "Bad_Slug", "--title", "Bad"])?;
    run.check(
        "invalid slug is refused",
        code != 0 && error.contains("kebab-case"),
        json!({"exit": code, "stderr": error}),
    )?;
    let (code, _, error) = run.spis(&["catalog-type", "add", "code-quality-untitled"])?;
    let left = run.corpus.join("code-quality-untitled-examples").exists();
    run.check(
        "add without a title is refused and leaves nothing",
        code != 0 && error.contains("--title") && !left,
        json!({"exit": code, "stderr": error, "directory_left": left}),
    )?;

    let missing = families.last().context("no family to withhold")?.clone();
    fs::remove_file(run.corpus.join(&missing))?;
    let (code, _, error) = run.spis(&[
        "catalog-type",
        "add",
        "code-quality-refused",
        "--title",
        "Refused",
        "--description",
        "Catalog whose regeneration is refused.",
    ])?;
    let left = run.corpus.join("code-quality-refused-examples").exists();
    std::os::unix::fs::symlink(root.join(&missing), run.corpus.join(&missing))?;
    run.check(
        "refused regeneration removes the new catalog",
        code != 0 && error.contains("was removed again") && error.contains(&missing) && !left,
        json!({"exit": code, "stderr": error, "directory_left": left}),
    )?;

    // edit
    let (code, out, error) = run.spis(&[
        "catalog-type",
        "edit",
        PROBE,
        "--title",
        "Renamed probe",
        "--rename",
        "code-quality-lifecycle-renamed",
    ])?;
    run.check(
        "edit succeeds",
        code == 0,
        json!({"exit": code, "stdout": out, "stderr": error}),
    )?;
    let renamed = json_file(&run.corpus.join(RENAMED).join("sources.json"))?;
    let renamed_index = json_file(&run.corpus.join(RENAMED).join("references.json"))?;
    let gone = !run.corpus.join(PROBE).exists();
    run.check(
        "edit renames the directory and stores the title",
        gone && renamed["title"] == "Renamed probe"
            && renamed["catalog"] == RENAMED
            && renamed_index["catalog"] == RENAMED,
        json!({"old_gone": gone, "sources": renamed, "references": renamed_index}),
    )?;
    let listed = slugs(&run.index()?);
    run.check(
        "index follows the rename",
        listed.last().map(String::as_str) == Some(RENAMED)
            && !listed.iter().any(|slug| slug == PROBE),
        json!(listed),
    )?;

    // remove
    let (code, out, error) = run.spis(&["catalog-type", "remove", RENAMED])?;
    run.check(
        "remove succeeds",
        code == 0,
        json!({"exit": code, "stdout": out, "stderr": error}),
    )?;
    let gone = !run.corpus.join(RENAMED).exists();
    let listed = slugs(&run.index()?);
    run.check(
        "remove deletes the catalog and its index entry",
        gone && listed == families,
        json!({"directory_gone": gone, "listed": listed}),
    )?;
    let (code, _, error) = run.spis(&["catalog-type", "remove", RENAMED])?;
    run.check(
        "removing a missing catalog is refused",
        code != 0 && error.contains("does not exist"),
        json!({"exit": code, "stderr": error}),
    )?;
    curated(run)
}

/// `curate-catalog` with a per-source category column: the categories reach
/// the records and the index counts them; a malformed selector is refused
/// and writes nothing.
fn curated(run: &mut Run) -> Result<()> {
    const CURATED: &str = "code-quality-curated-probe-examples";
    let selector = run.work.join("curated.selector");
    fs::write(
        &selector,
        format!(
            "catalog: {CURATED}\ntitle: Curated probe\ndescription: Catalog curated by the lifecycle test.\ncategory: general\ncount: 3\n\n\
             First deck\thttps://example.com/first.pdf\tconsulting\n\
             Second deck\thttps://example.com/second.pdf\tconsulting\n\
             Third deck\thttps://example.com/third.pdf\n"
        ),
    )?;
    let path = selector
        .to_str()
        .context("selector path is not UTF-8")?
        .to_string();
    let (code, out, error) = run.spis(&["curate-catalog", "--selector", &path])?;
    run.check(
        "validation counts every category",
        code == 0 && out.contains("consulting 2, general 1") && !run.corpus.join(CURATED).exists(),
        json!({"exit": code, "stdout": out, "stderr": error}),
    )?;
    let (code, out, error) = run.spis(&["curate-catalog", "--selector", &path, "--apply"])?;
    run.check(
        "apply writes the curated records",
        code == 0,
        json!({"exit": code, "stdout": out, "stderr": error}),
    )?;
    let (code, _, error) = run.spis(&["generate-example-catalogs"])?;
    run.check(
        "index regenerates with the curated catalog",
        code == 0,
        json!({"exit": code, "stderr": error}),
    )?;
    let sources = json_file(&run.corpus.join(CURATED).join("sources.json"))?;
    let recorded: Vec<&str> = sources["examples"]
        .as_array()
        .map(|examples| {
            examples
                .iter()
                .filter_map(|e| e["category"].as_str())
                .collect()
        })
        .unwrap_or_default();
    run.check(
        "each record keeps its own category, or the header's",
        recorded == ["consulting", "consulting", "general"],
        json!(recorded),
    )?;
    let index = run.index()?;
    let entry = index["catalogs"]
        .as_array()
        .and_then(|list| list.iter().find(|entry| entry["slug"] == CURATED))
        .cloned()
        .unwrap_or(Value::Null);
    run.check(
        "index counts the records of every category",
        entry["categories"]
            == json!([
                {"category": "consulting", "count": 2},
                {"category": "general", "count": 1},
            ]),
        entry.clone(),
    )?;

    let broken = run.work.join("broken.selector");
    fs::write(
        &broken,
        format!(
            "catalog: code-quality-broken-examples\ntitle: Broken\ndescription: Refused selector.\ncategory: general\ncount: 1\n\n\
             Only deck\thttps://example.com/only.pdf\t\n"
        ),
    )?;
    let broken_path = broken
        .to_str()
        .context("selector path is not UTF-8")?
        .to_string();
    let (code, _, error) = run.spis(&["curate-catalog", "--selector", &broken_path, "--apply"])?;
    let left = run.corpus.join("code-quality-broken-examples").exists();
    run.check(
        "an empty category column is refused and writes nothing",
        code != 0 && error.contains("empty category column") && !left,
        json!({"exit": code, "stderr": error, "directory_left": left}),
    )?;

    let (code, out, error) = run.spis(&["catalog-type", "remove", CURATED, "--force"])?;
    let listed = slugs(&run.index()?);
    run.check(
        "the curated catalog is removed with its records",
        code == 0 && !run.corpus.join(CURATED).exists() && !listed.iter().any(|s| s == CURATED),
        json!({"exit": code, "stdout": out, "stderr": error, "listed": listed}),
    )?;
    replaced_sources(run)
}

/// `check-upstream-drift` against real public addresses: a page whose site now
/// redirects it to the home page and a `.pdf` address now served as an HTML
/// page are gone, a live page is reachable. The corpus holds only this
/// catalog, so no other source is fetched.
fn replaced_sources(run: &mut Run) -> Result<()> {
    const CATALOG: &str = "code-quality-drift-probe-examples";
    let corpus = run.work.join("drift-corpus");
    fs::create_dir(&corpus)?;
    let selector = run.work.join("drift.selector");
    fs::write(
        &selector,
        format!(
            "catalog: {CATALOG}\ntitle: Drift probe\ndescription: Sources whose sites replaced them.\ncategory: probe\ncount: 3\n\n\
             Removed article\thttps://openviewpartners.com/blog/the-greatest-sales-deck-ive-ever-seen-its-zuoras-and-its-brilliant-heres-why\n\
             Removed deck\thttps://karpathy.ai/stateofgpt.pdf\n\
             Live article\thttps://sequoiacap.com/article/writing-a-business-plan\n"
        ),
    )?;
    let path = selector.to_str().context("selector path is not UTF-8")?.to_string();
    let binary = run.binary.clone();
    let (code, _, error) = run.run(
        &binary,
        &["curate-catalog", "--selector", &path, "--apply"],
        &corpus,
    )?;
    ensure!(code == 0, "drift probe catalog: {error}");
    let (code, out, error) = run.run(
        &binary,
        &[
            "check-upstream-drift",
            "--skip-readme",
            "--write-report",
            "--strict",
        ],
        &corpus,
    )?;
    let report = json_file(&corpus.join(".build/upstream-drift.json"))?;
    let gone: Vec<String> = report["upstream_urls_gone"]
        .as_array()
        .map(|items| items.iter().map(|item| item.to_string()).collect())
        .unwrap_or_default();
    let named = |needle: &str| gone.iter().any(|item| item.contains(needle));
    run.check(
        "a source redirected home and a .pdf served as HTML are gone; a live page is not",
        code != 0
            && gone.len() == 2
            && named("home page")
            && named("HTML page")
            && !named("sequoiacap"),
        json!({"exit": code, "stdout": out, "stderr": error, "gone": report["upstream_urls_gone"]}),
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let mut run = Run::new()?;
    let result = lifecycle(&mut run);
    run.report["state"] = json!(if result.is_ok() { "passed" } else { "failed" });
    if let Err(error) = &result {
        run.report["error"] = json!(format!("{error:#}"));
    }
    let report = run.work.join("report.json");
    fs::write(&report, serde_json::to_vec_pretty(&run.report)?)?;
    eprintln!("catalog-type lifecycle: {}", report.display());
    match result {
        Ok(()) => Ok(()),
        Err(error) => bail!("{error:#}"),
    }
}
