use super::*;

/// The option of a page that does not belong in the corpus.
const NOT_KEPT: &str = "not-kept";
/// Each discovered link is asked under this prefix and its position.
const LINK: &str = "link-";

/// Ask Brama which discovered pages matter for the corpus and what family each
/// belongs to: one typed decision, one `choice` per discovered link among the
/// declared page families and not keeping it. There is no keyword fallback:
/// words in a URL decided the family before, so a pricing page at `/buy` was
/// "other" and every link saying "start" was a signup page. Every discovered
/// link is asked; Brama's own refusal, not a count chosen here, says when a
/// page holds too many. The kept pages come back most confident first, so the
/// caller's `--limit` keeps the ones Brama was surest of.
pub(crate) fn brama_rank(start_url: &str, links: &PageLinks) -> Result<Vec<(String, String)>> {
    let purpose = "discover asks Brama which pages to keep";
    let mut options: Vec<(String, String)> = PageFamily::names()?
        .into_iter()
        .map(|family| match family.as_str() {
            Some(name) => Ok((name.to_string(), format!("a page of the {name} family"))),
            None => bail!("discover: page family {family} is not a name"),
        })
        .collect::<Result<_>>()?;
    options.push((
        NOT_KEPT.to_string(),
        "the page does not belong in an interface reference corpus".to_string(),
    ));
    let listing: Vec<String> = links
        .entries
        .iter()
        .enumerate()
        .map(|(index, (url, text))| format!("{index}: {url} | {text}"))
        .collect();
    let state = format!(
        "Start page: {start_url}\nDiscovered links:\n{}",
        listing.join("\n")
    );
    let questions: serde_json::Map<String, serde_json::Value> = links
        .entries
        .iter()
        .enumerate()
        .map(|(index, (url, _))| {
            let instructions = format!(
                "The state lists the discovered links of one product's website. For an interface \
                 reference corpus, which page family is the link at index {index} ({url}), or \
                 does it not belong in the corpus?"
            );
            (
                format!("{LINK}{index}"),
                crate::commands::brama::choice(&instructions, &options),
            )
        })
        .collect();
    let answers = crate::commands::brama::decide(purpose, &state, &questions)?;
    let mut kept: Vec<(f64, String, String)> = Vec::new();
    for (index, (url, _)) in links.entries.iter().enumerate() {
        let (family, confidence) =
            crate::commands::brama::chosen(purpose, &answers[&format!("{LINK}{index}")], &options)?;
        if family != NOT_KEPT && !kept.iter().any(|(_, seen, _)| seen == url) {
            kept.push((confidence, url.clone(), family));
        }
    }
    if kept.is_empty() {
        bail!(
            "Brama kept no page of the {} discovered links",
            links.entries.len()
        );
    }
    kept.sort_by(|(left, _, _), (right, _, _)| right.total_cmp(left));
    Ok(kept
        .into_iter()
        .map(|(_, url, family)| (url, family))
        .collect())
}

/// `spis discover <start-url> --catalog <slug> --limit <n>`: `--limit` is how
/// many pages the operator wants recorded from this site, and is required.
pub fn run(rest: &[String]) -> Result<()> {
    let mut positionals: Vec<String> = Vec::new();
    let mut catalog: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut arguments = rest.iter();
    while let Some(arg) = arguments.next() {
        match arg.as_str() {
            "--catalog" => {
                catalog = Some(arguments.next().cloned().ok_or_else(|| {
                    crate::commands::usage("discover: argument --catalog: expected one argument")
                })?);
            }
            "--limit" => {
                let value = arguments.next().ok_or_else(|| {
                    crate::commands::usage("discover: argument --limit: expected one argument")
                })?;
                limit = Some(value.parse().map_err(|error| {
                    crate::commands::usage(format!(
                        "discover: argument --limit: invalid int value {value:?}: {error}"
                    ))
                })?);
            }
            other => {
                if other.starts_with("--") {
                    return Err(crate::commands::usage(format!(
                        "discover: unrecognized argument {other}"
                    )));
                }
                positionals.push(arg.clone());
            }
        }
    }
    let start_url = positionals.first().cloned().ok_or_else(|| {
        crate::commands::usage("discover: the following arguments are required: start_url")
    })?;
    let catalog = catalog.ok_or_else(|| {
        crate::commands::usage("discover: the following arguments are required: --catalog")
    })?;
    let limit = limit.ok_or_else(|| {
        crate::commands::usage(
            "discover: the following arguments are required: --limit (how many pages to record from this site)"
        )
    })?;

    let slug = if catalog.ends_with("-examples") {
        catalog.clone()
    } else {
        format!("{catalog}-examples")
    };
    let directory = std::path::PathBuf::from(&slug);

    let start = url::Url::parse(&start_url).map_err(|error| {
        crate::commands::usage(format!(
            "discover: start_url {start_url:?} is not a URL: {error}"
        ))
    })?;
    let page = crate::commands::collect_example_images::fetch(start.as_str(), "text/html")?;
    let links = page_links(&start, &String::from_utf8_lossy(&page.data))?;
    for (target, cause) in &links.unresolved {
        eprintln!("discover: {start_url} links to {target:?}, which does not resolve: {cause}");
    }
    println!(
        "discovered {} same-origin links on {start_url}",
        links.entries.len()
    );
    if links.entries.is_empty() {
        bail!("discover: no same-origin links found");
    }

    let ranked = brama_rank(&start_url, &links)?;
    let selected: Vec<(String, String)> = ranked.into_iter().take(limit).collect();
    if selected.is_empty() {
        bail!("discover: nothing selected for this corpus");
    }
    println!("Brama selected {} page(s):", selected.len());
    for (url, family) in &selected {
        println!("  [{family}] {url}");
    }

    // Ensure the catalog exists, then reuse the tested record scaffolder.
    if !directory.is_dir() {
        let title_base = catalog.replace("-examples", "").to_lowercase();
        let mut title_chars = title_base.chars();
        let title = match title_chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + title_chars.as_str(),
            None => title_base,
        };
        crate::commands::catalog_type::run(&[
            "add".to_string(),
            catalog.clone(),
            "--title".to_string(),
            format!("{title} examples"),
        ])
        .context("discover: scaffolding the catalog with spis catalog-type add")?;
    }

    for (url, family) in &selected {
        // The record's visual is the page's own image of the interface, as
        // collect-example-images chooses it; a page without one is refused
        // with that cause, never rendered by a third-party screenshot service.
        let (_, probe) = crate::commands::collect_example_images::select_image(url)
            .with_context(|| format!("discover: {url} gives no image of its interface"))?;
        // The image is staged in Spis's own work area, named by its URL's
        // digest, and taken back once the record holds its copy; nothing is
        // written to /tmp and no name is cut.
        let staging = crate::commands::capture_widths::work_root()?.join("discover");
        std::fs::create_dir_all(&staging)
            .with_context(|| format!("create {}", staging.display()))?;
        let tmp = staging.join(format!(
            "{}.{}",
            crate::sha256_hex(url.as_bytes()),
            probe.format
        ));
        std::fs::write(&tmp, &probe.payload).with_context(|| format!("write {}", tmp.display()))?;
        // A page is named by its last path segment; the site's root page,
        // which has none, by its host.
        let parsed =
            url::Url::parse(url).with_context(|| format!("discover: {url} is not a URL"))?;
        let last_segment = parsed
            .path_segments()
            .and_then(|segments| segments.filter(|segment| !segment.is_empty()).last())
            .or_else(|| parsed.host_str())
            .with_context(|| format!("discover: {url} names neither a path nor a host"))?
            .to_string();
        let capitalized_family = {
            let mut chars = family.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        };
        let added = reference_record::add(&AddArgs {
            catalog: slug.clone(),
            name: format!("{capitalized_family} \u{2014} {last_segment}"),
            source_url: url.clone(),
            category: family.clone(),
            selection_note: format!("auto-discovered from {start_url}; family {family}"),
            visual: tmp.display().to_string(),
            owner: Some(parsed.origin().ascii_serialization()),
        });
        crate::discarding(added, &tmp)?;
        crate::discard(&tmp)?;
    }

    super::reference_contract::regenerate_index()?;
    println!("done: catalog regenerated and consistent");
    Ok(())
}
