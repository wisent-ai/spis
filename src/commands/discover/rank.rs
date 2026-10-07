use super::*;

/// Ask Brama which discovered pages matter for the corpus and what family each
/// belongs to. There is no keyword fallback: words in a URL decided the family
/// before, so a pricing page at `/buy` was "other" and every link saying
/// "start" was a signup page. Every discovered link is listed; Brama's own
/// refusal, not a count chosen here, says when a page holds too many.
pub(crate) fn brama_rank(
    start_url: &str,
    links: &Links,
    limit: usize,
) -> Result<Vec<(String, String)>> {
    let listing: Vec<String> = links
        .entries
        .iter()
        .map(|(url, text)| format!("- {url} | {text}"))
        .collect();
    let parsed = crate::commands::brama::ask_json(
        "discover asks Brama which pages to keep",
        &format!(
            "You classify pages of one product's website for an interface reference corpus. \
             Return STRICT JSON: {{\"pages\": [{{\"url\": string, \"family\": \
             one of {FAMILIES:?}]}}}} . Only use URLs from the list. Pick at most {limit}."
        ),
        &format!(
            "Start page: {start_url}\nDiscovered links:\n{}",
            listing.join("\n")
        ),
    )?;
    let mut ranked: Vec<(String, String)> = Vec::new();
    for page in parsed["pages"].as_array().unwrap_or(&Vec::new()) {
        let (Some(url), Some(family)) = (page["url"].as_str(), page["family"].as_str()) else {
            continue;
        };
        if links.entries.iter().any(|(known, _)| known == url)
            && FAMILIES.contains(&family)
            && !ranked.iter().any(|(seen, _)| seen == url)
        {
            ranked.push((url.to_string(), family.to_string()));
        }
    }
    if ranked.is_empty() {
        bail!(
            "Brama selected no page from the {} discovered links",
            links.entries.len()
        );
    }
    Ok(ranked)
}

/// `spis discover <start-url> --catalog <slug> --limit <n>`: `--limit` is how
/// many pages the operator wants recorded from this site, and is required.
pub fn run(rest: &[String]) -> Result<()> {
    let mut positionals: Vec<String> = Vec::new();
    let mut catalog: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        match arg.as_str() {
            "--catalog" => {
                i += 1;
                catalog = Some(rest.get(i).cloned().ok_or_else(|| {
                    anyhow::anyhow!("discover: argument --catalog: expected one argument")
                })?);
            }
            "--limit" => {
                i += 1;
                let value = rest.get(i).cloned().ok_or_else(|| {
                    anyhow::anyhow!("discover: argument --limit: expected one argument")
                })?;
                limit = Some(value.parse().map_err(|_| {
                    anyhow::anyhow!("discover: argument --limit: invalid int value {value:?}")
                })?);
            }
            other => {
                if other.starts_with("--") {
                    return Err(crate::commands::usage(format!(
                        "discover: unrecognized argument {other}"
                    )));
                }
                positionals.push(arg);
            }
        }
        i += 1;
    }
    let start_url = positionals.first().cloned().ok_or_else(|| {
        anyhow::anyhow!("discover: the following arguments are required: start_url")
    })?;
    let catalog = catalog.ok_or_else(|| {
        anyhow::anyhow!("discover: the following arguments are required: --catalog")
    })?;
    let limit = limit.ok_or_else(|| {
        anyhow::anyhow!(
            "discover: the following arguments are required: --limit (how many pages to record from this site)"
        )
    })?;

    let slug = if catalog.ends_with("-examples") {
        catalog.clone()
    } else {
        format!("{catalog}-examples")
    };
    let directory = std::path::PathBuf::from(&slug);

    let (html_bytes, _) = fetch(&start_url)?;
    let links = extract_links(&start_url, &html_bytes);
    println!(
        "discovered {} same-origin links on {start_url}",
        links.entries.len()
    );
    if links.entries.is_empty() {
        bail!("discover: no same-origin links found");
    }

    let ranked = brama_rank(&start_url, &links, limit)?;
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
        let thumb_url = format!("{THUMB}{url}");
        let (image_bytes, _) = fetch(&thumb_url)?;
        let tmp_name: String = url
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_lowercase() || c.is_ascii_digit() {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-")
            .chars()
            .take(60)
            .collect();
        let tmp = std::path::PathBuf::from(format!("/tmp/{tmp_name}.png"));
        std::fs::write(&tmp, &image_bytes).with_context(|| format!("write {}", tmp.display()))?;
        let last_segment: String = url
            .split_once("://")
            .map(|(_, r)| r)
            .unwrap_or(url)
            .split(['?', '#'])
            .next()
            .unwrap_or("")
            .trim_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .chars()
            .take(40)
            .collect();
        let capitalized_family = {
            let mut chars = family.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        };
        reference_record::add(&AddArgs {
            catalog: slug.clone(),
            name: format!("{capitalized_family} \u{2014} {last_segment}"),
            source_url: url.clone(),
            category: family.clone(),
            selection_note: format!("auto-discovered from {start_url}; family {family}"),
            visual: tmp.display().to_string(),
            owner: Some(netloc_of(url).to_string()),
        })?;
    }

    super::reference_contract::regenerate_index()?;
    println!("done: catalog regenerated and consistent");
    Ok(())
}
