use super::*;

pub(crate) const KEYWORDS: &[(&str, &[&str])] = &[
    ("pricing", &["pricing", "plans", "plans-and-pricing"]),
    (
        "docs",
        &["docs", "documentation", "developers", "api", "guides"],
    ),
    (
        "signup",
        &["sign-up", "signup", "register", "get-started", "start"],
    ),
    ("about", &["about", "company", "customers", "careers"]),
    ("product", &["product", "features", "platform", "solutions"]),
];

pub(crate) fn heuristic_family(url: &str, text: &str) -> &'static str {
    let blob = format!("{} {}", url, text).to_lowercase();
    for (family, words) in KEYWORDS {
        if words.iter().any(|word| blob.contains(word)) {
            return family;
        }
    }
    "other"
}

/// Ask Brama to rank pages; `None` means fall back deterministically.
pub(crate) fn brama_rank(start_url: &str, links: &Links, limit: usize) -> Option<Vec<(String, String)>> {
    let router = std::env::var("MODEL_ROUTER_URL").ok()?;
    if router.is_empty() {
        return None;
    }
    let endpoint = if router.contains("/v1") {
        format!("{}/chat/completions", router.trim_end_matches('/'))
    } else {
        format!("{}/v1/chat/completions", router.trim_end_matches('/'))
    };
    let listing: Vec<String> = links
        .entries
        .iter()
        .take(80)
        .map(|(url, text)| format!("- {url} | {text}"))
        .collect();
    let payload = json!({
        "model": std::env::var("MODEL_ROUTER_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into()),
        "messages": [
            {"role": "system", "content": format!(
                "You classify pages of one product's website for an interface reference corpus. \
                 Return STRICT JSON: {{\"pages\": [{{\"url\": string, \"family\": \
                 one of {FAMILIES:?}]}}}} . Only use URLs from the list. Pick at most {limit}.")},
            {"role": "user", "content": format!(
                "Start page: {start_url}\nDiscovered links:\n{}", listing.join("\n"))},
        ],
    })
    .to_string();

    let parsed: Result<serde_json::Value> = (|| {
        let mut request = ureq::post(&endpoint)
            .timeout(Duration::from_secs(60))
            .set("Content-Type", "application/json");
        if let Ok(token) = std::env::var("MODEL_ROUTER_TOKEN") {
            if !token.is_empty() {
                request = request.set("Authorization", &format!("Bearer {token}"));
            }
        }
        let response = request.send_string(&payload)?;
        let mut body_bytes = Vec::new();
        let bytes_read = response.into_reader().read_to_end(&mut body_bytes)?;
        let _ = bytes_read;
        let body: serde_json::Value = serde_json::from_slice(&body_bytes)?;
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .context("no message content")?;
        let brace_open = content.find('{').context("no JSON object")?;
        let brace_close = content.rfind('}').context("no JSON object")? + 1;
        Ok(serde_json::from_str(&content[brace_open..brace_close])?)
    })();
    match parsed {
        Ok(parsed) => {
            let mut ranked: Vec<(String, String)> = Vec::new();
            for page in parsed["pages"].as_array().unwrap_or(&Vec::new()) {
                let (Some(url), Some(family)) = (page["url"].as_str(), page["family"].as_str())
                else {
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
                None
            } else {
                Some(ranked)
            }
        }
        Err(error) => {
            eprintln!("discover: Brama ranking unavailable ({error}); using keyword fallback");
            None
        }
    }
}

/// `spis discover <start-url> --catalog <slug> [--limit <n>] [--max-links <n>]`
pub fn run(rest: &[String]) -> Result<()> {
    let mut positionals: Vec<String> = Vec::new();
    let mut catalog: Option<String> = None;
    let mut limit: usize = 6;
    let mut max_links: usize = 120;
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
            "--limit" | "--max-links" => {
                let name = arg.clone();
                i += 1;
                let value = rest.get(i).cloned().ok_or_else(|| {
                    anyhow::anyhow!("discover: argument {name}: expected one argument")
                })?;
                let parsed: usize = value.parse().map_err(|_| {
                    anyhow::anyhow!("discover: argument {name}: invalid int value {value:?}")
                })?;
                if name == "--limit" {
                    limit = parsed;
                } else {
                    max_links = parsed;
                }
            }
            other => {
                if other.starts_with("--") {
                    bail!("discover: unrecognized argument {other}");
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

    let slug = if catalog.ends_with("-examples") {
        catalog.clone()
    } else {
        format!("{catalog}-examples")
    };
    let directory = std::path::PathBuf::from(&slug);

    let (html_bytes, _) = fetch(&start_url, 25)?;
    let links = extract_links(&start_url, &html_bytes, max_links);
    println!(
        "discovered {} same-origin links on {start_url}",
        links.entries.len()
    );
    if links.entries.is_empty() {
        bail!("discover: no same-origin links found");
    }

    let ranked = brama_rank(&start_url, &links, limit).unwrap_or_else(|| {
        let mut ranked: Vec<(String, String)> = Vec::new();
        for (url, text) in &links.entries {
            let family = heuristic_family(url, text);
            let family_count = ranked.iter().filter(|(_, f)| f == family).count();
            if family != "other" && family_count < std::cmp::max(1, limit / 3) {
                ranked.push((url.clone(), family.to_string()));
            }
        }
        ranked
    });
    let selected: Vec<(String, String)> = ranked.into_iter().take(limit).collect();
    if selected.is_empty() {
        bail!("discover: nothing selected for this corpus");
    }
    println!("Brama/heuristics selected {} page(s):", selected.len());
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
        let status = std::process::Command::new("python3")
            .args([
                "catalog-type.py",
                "add",
                &catalog,
                "--title",
                &format!("{title} examples"),
            ])
            .status()
            .context("run catalog-type.py")?;
        if !status.success() {
            bail!("discover: catalog-type.py add failed with {status}");
        }
    }

    for (url, family) in &selected {
        let thumb_url = format!("{THUMB}{url}");
        let (image_bytes, _) = fetch(&thumb_url, 40)?;
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
