use super::*;

pub fn run(rest: &[String]) -> Result<()> {
    let mut check = false;
    let site_content = PathBuf::from(DOCS_CONTENT);
    let mut landing_content = None;
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--check" => check = true,
            "--site-content" => {
                let path = arguments
                    .next()
                    .context("--site-content requires a path")?;
                landing_content = Some(PathBuf::from(path));
            }
            "--help" | "-h" => {
                println!(
                    "usage: spis docs-site [--check] [--site-content PATH]\n\
                     Generates {DOCS_BRIEF}, {DOCS_PLAN} and canonical DocPage JSON from this\n\
                     repository's own tables. The JSON is written to {DOCS_CONTENT}; pass the\n\
                     landing checkout's src/content/product-docs.json with --site-content to\n\
                     update that deployed consumer in the same operation. --check writes nothing\n\
                     and exits 1 when any selected output no longer matches the product."
                );
                return Ok(());
            }
            other => bail!("unknown argument: {other}"),
        }
    }
    let version = env!("CARGO_PKG_VERSION");
    let plan = plan(version);
    let brief = brief(version);
    let content = docs_content(&plan)?;
    let root = PathBuf::from(".");
    let mut stale = Vec::new();
    write_if_changed(
        &root.join(DOCS_BRIEF),
        &format!("{}\n", serde_json::to_string_pretty(&brief)?),
        check,
        &mut stale,
    )?;
    write_if_changed(
        &root.join(DOCS_PLAN),
        &format!("{}\n", serde_json::to_string_pretty(&plan)?),
        check,
        &mut stale,
    )?;
    write_if_changed(&site_content, &content, check, &mut stale)?;
    if let Some(path) = landing_content.as_ref().filter(|path| *path != &site_content) {
        write_if_changed(path, &content, check, &mut stale)?;
    }
    if check {
        if stale.is_empty() {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schema": "wisent.spis-docs-site.v1",
                    "state": "current",
                    "site_content": site_content,
                    "landing_content": landing_content,
                    "families": super::crawl::CATALOGS.len(),
                    "commands": super::SUBCOMMANDS.len(),
                }))?
            );
            return Ok(());
        }
        bail!(
            "committed documentation no longer matches the product: {}. Run `spis docs-site` \
             and commit the result; do not edit the generated files by hand",
            stale.join(", ")
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "wisent.spis-docs-site.v1",
            "state": "generated",
            "wrote": [DOCS_BRIEF, DOCS_PLAN, DOCS_CONTENT],
            "landing_content": landing_content,
            "families": super::crawl::CATALOGS.len(),
            "commands": super::SUBCOMMANDS.len(),
            "corpus_bound": super::docs_corpus::MAX_PAGE_RECORDS,
        }))?
    );
    let _ = lib::now_iso_utc();
    Ok(())
}
