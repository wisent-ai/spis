use super::*;

pub(crate) const USAGE: &str = "usage: spis docs corpus status | search --query T [--site S] [--limit N] | show --site S --url U | import --attempt-receipt FILE";

enum Operation<'a> {
    Status,
    Search {
        query: &'a str,
        site: Option<&'a str>,
        limit: Option<std::num::NonZeroUsize>,
    },
    Show {
        site: &'a str,
        url: &'a str,
    },
    Import {
        receipt: &'a str,
    },
}

fn text<'a>(arguments: &mut std::slice::Iter<'a, String>, flag: &str) -> Result<&'a str> {
    let value = arguments
        .next()
        .ok_or_else(|| crate::commands::usage(format!("{flag} needs a value")))?;
    if value.is_empty() {
        return Err(crate::commands::usage(format!(
            "{flag} needs a non-empty value"
        )));
    }
    Ok(value.as_str())
}

fn search(arguments: &[String]) -> Result<Operation<'_>> {
    let mut query = None;
    let mut site = None;
    let mut limit = None;
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--query" => query = Some(text(&mut arguments, "--query")?),
            "--site" => site = Some(text(&mut arguments, "--site")?),
            "--limit" => limit = Some(crate::commands::parsed(arguments.next(), "--limit")?),
            other => return Err(crate::commands::usage(format!("unknown argument: {other}"))),
        }
    }
    Ok(Operation::Search {
        query: query.ok_or_else(|| crate::commands::usage("--query required"))?,
        site,
        limit,
    })
}

fn show(arguments: &[String]) -> Result<Operation<'_>> {
    let mut site = None;
    let mut url = None;
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--site" => site = Some(text(&mut arguments, "--site")?),
            "--url" => url = Some(text(&mut arguments, "--url")?),
            other => return Err(crate::commands::usage(format!("unknown argument: {other}"))),
        }
    }
    Ok(Operation::Show {
        site: site.ok_or_else(|| crate::commands::usage("show needs --site <slug>"))?,
        url: url.ok_or_else(|| crate::commands::usage("show needs --url <url>"))?,
    })
}

fn import(arguments: &[String]) -> Result<Operation<'_>> {
    let mut receipt = None;
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--attempt-receipt" => receipt = Some(text(&mut arguments, "--attempt-receipt")?),
            other => return Err(crate::commands::usage(format!("unknown argument: {other}"))),
        }
    }
    Ok(Operation::Import {
        receipt: receipt
            .ok_or_else(|| crate::commands::usage("import needs --attempt-receipt <file>"))?,
    })
}

fn parse(rest: &[String]) -> Result<Operation<'_>> {
    let (verb, arguments) = rest
        .split_first()
        .ok_or_else(|| crate::commands::usage(USAGE))?;
    match verb.as_str() {
        "status" => match arguments.first() {
            Some(other) => Err(crate::commands::usage(format!("unknown argument: {other}"))),
            None => Ok(Operation::Status),
        },
        "search" => search(arguments),
        "show" => show(arguments),
        "import" => import(arguments),
        other => Err(crate::commands::usage(format!(
            "unknown docs corpus operation {other}; {USAGE}"
        ))),
    }
}

/// The invocation is fully checked before the adopted corpus changes the
/// working directory or an importer can create any state.
pub fn run(arguments: &[String]) -> Result<()> {
    let operation = parse(arguments)?;
    crate::commands::remember_invocation_dir()?;
    crate::commands::corpus::activate_configured_root()?;
    match operation {
        Operation::Status => {
            let sites = collect_sites()?;
            let output = sites
                .iter()
                .map(|site| {
                    json!({
                        "slug": site.slug,
                        "name": site.name,
                        "category": site.category,
                        "source_url": site.source_url,
                        "inventory_url_count": site.inventory_url_count,
                        "seen": site.seen,
                        "cumulative_ok": site.cumulative_ok,
                        "noise": site.noise,
                        "done": site.retrieval_status.as_deref() == Some("retrieval_complete"),
                        "retrieval_status": site.retrieval_status,
                        "attempt": site.attempt,
                        "attempt_id": site.attempt_id,
                    })
                })
                .collect::<Vec<_>>();
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
        Operation::Search { query, site, limit } => {
            let sites = collect_sites()?;
            let query_lower = std::cell::OnceCell::new();
            let mut hits = Vec::new();
            let mut failure = None;
            let scanned: usize = sites
                .iter()
                .filter(|candidate| !site.is_some_and(|slug| slug != candidate.slug))
                .filter_map(|candidate| {
                    candidate
                        .corpus_dir
                        .as_deref()
                        .map(|directory| (candidate, directory))
                })
                .map_while(|(candidate, directory)| {
                    if limit.is_some_and(|limit| hits.len() >= limit.get()) {
                        return None;
                    }
                    match scan_site(
                        candidate,
                        directory,
                        query_lower.get_or_init(|| query.to_lowercase()),
                        limit,
                        &mut hits,
                    ) {
                        Ok(scanned) => Some(scanned),
                        Err(error) => {
                            failure = Some(error);
                            None
                        }
                    }
                })
                .sum();
            if let Some(error) = failure {
                return Err(error);
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "hits": hits, "scanned": scanned, "limit": limit,
                }))?
            );
            Ok(())
        }
        Operation::Show { site, url } => {
            let selected = selected_corpora()?;
            let corpus = selected
                .get(site)
                .with_context(|| format!("no completed v2 retrieval corpus for {site}"))?;
            let mut reader = open_jsonl(&corpus.corpus_dir)?;
            while let Some(record) = read_corpus_record(&mut reader)? {
                if record.get("url").and_then(Value::as_str) == Some(url) {
                    println!("{record}");
                    return Ok(());
                }
            }
            bail!("url not found in the completed {site} retrieval corpus")
        }
        Operation::Import { receipt } => {
            let receipt = crate::commands::operator_path(receipt);
            let (report, _, _) = read_attempt_receipt(&receipt)?;
            let mut imported = import_worker_report(&report)?;
            imported["attempt_receipt"] = json!(receipt);
            println!("{}", serde_json::to_string_pretty(&imported)?);
            Ok(())
        }
    }
}
