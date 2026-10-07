use super::*;

mod robots;

pub(crate) fn resolve_urls(
    meta: &SiteMeta,
    rules: &SiteRules,
    policy: &UrlPolicy,
) -> Result<InventoryResolution> {
    let mut diagnostics = Vec::new();
    let total_inventory_bytes = AtomicU64::new(0);
    let inventory_counter = Some(&total_inventory_bytes);
    let (robots, compiled_robots, discovered_sitemaps) =
        robots::fetch_robots(policy, inventory_counter, &mut diagnostics)?;

    let mut pages = vec![(policy.source_url.as_str().to_string(), None)];
    let mut queue = VecDeque::<Url>::new();
    let seeds = if rules.sitemaps.is_empty() && rules.llms.is_empty() {
        let mut seeds = discovered_sitemaps;
        seeds.push(policy.source_url.join("/sitemap.xml")?.to_string());
        seeds
    } else {
        rules.sitemaps.clone()
    };
    for raw in seeds.into_iter().chain(rules.llms.iter().cloned()) {
        match policy.canonical(&raw, Some(&policy.source_url), "inventory source") {
            Ok(url) => queue.push_back(url),
            Err(error) => push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_rejected",
                format!("{error:#}"),
                raw,
            ),
        }
    }

    // Every inventory source is read once; the visited set is what stops a
    // sitemap index that names itself, not a count of sources.
    let mut visited = HashSet::new();
    while let Some(source) = queue.pop_front() {
        if !visited.insert(source.as_str().to_string()) {
            continue;
        }
        if !compiled_robots.allows(&source) {
            push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_robots_disallowed",
                "robots.txt disallows this inventory source",
                source.as_str(),
            );
            continue;
        }
        let response = match http_get(
            &source,
            policy,
            "documentation inventory source",
            inventory_counter,
        ) {
            Ok(response) if (200..300).contains(&response.status) => response,
            Ok(response) => {
                push_inventory_diagnostic(
                    &mut diagnostics,
                    "inventory_source_http_status",
                    format!("inventory source returned HTTP {}", response.status),
                    source.as_str(),
                );
                continue;
            }
            Err(error) => {
                push_inventory_diagnostic(
                    &mut diagnostics,
                    error.code,
                    error.to_string(),
                    source.as_str(),
                );
                continue;
            }
        };
        let payload = match std::str::from_utf8(&response.body) {
            Ok(payload) => payload,
            Err(error) => {
                push_inventory_diagnostic(
                    &mut diagnostics,
                    "inventory_source_non_utf8",
                    format!(
                        "inventory source is not valid UTF-8 at byte {}",
                        error.valid_up_to()
                    ),
                    source.as_str(),
                );
                continue;
            }
        };
        if payload.contains("<urlset") || payload.contains("<sitemapindex") {
            let (children, discovered_pages) = lib::parse_sitemap(&response.body);
            for child in children {
                match policy.canonical(&child, Some(&response.final_url), "child sitemap") {
                    Ok(url) => queue.push_back(url),
                    Err(error) => push_inventory_diagnostic(
                        &mut diagnostics,
                        "child_sitemap_rejected",
                        format!("{error:#}"),
                        child,
                    ),
                }
            }
            for (raw, lastmod) in discovered_pages {
                match policy.canonical(&raw, Some(&response.final_url), "sitemap page") {
                    Ok(url) if path_is_in_scope(&url, &rules.prefixes)? => {
                        pages.push((url.to_string(), lastmod));
                    }
                    Ok(_) => {}
                    Err(error) => push_inventory_diagnostic(
                        &mut diagnostics,
                        "sitemap_page_rejected",
                        format!("{error:#}"),
                        raw,
                    ),
                }
            }
        } else if source.path().ends_with(".txt") && source.path().contains("llms") {
            for line in payload.lines() {
                let trimmed = line.trim();
                let Some(rest) = trimmed.strip_prefix("- [") else {
                    continue;
                };
                let Some(close) = rest.find("](") else {
                    continue;
                };
                let after = &rest[close + 2..];
                let Some(end) = after.find(')') else {
                    continue;
                };
                let raw = &after[..end];
                match policy.canonical(raw, Some(&response.final_url), "llms.txt page") {
                    Ok(url) if path_is_in_scope(&url, &rules.prefixes)? => {
                        pages.push((url.to_string(), None));
                    }
                    Ok(_) => {}
                    Err(error) => push_inventory_diagnostic(
                        &mut diagnostics,
                        "llms_page_rejected",
                        format!("{error:#}"),
                        raw,
                    ),
                }
            }
        } else {
            push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_unrecognized",
                "inventory source was neither sitemap XML nor llms.txt",
                source.as_str(),
            );
        }
    }

    if pages.len() == 1 && meta.inventory_source.starts_with("landing-nav") {
        for item in &meta.landing_nav {
            match policy.canonical(
                &item.path,
                Some(&policy.source_url),
                "landing navigation page",
            ) {
                Ok(url) if path_is_in_scope(&url, &rules.prefixes)? => {
                    pages.push((url.to_string(), None));
                }
                Ok(_) => {}
                Err(error) => push_inventory_diagnostic(
                    &mut diagnostics,
                    "landing_page_rejected",
                    format!("{error:#}"),
                    item.path.clone(),
                ),
            }
        }
    }
    if pages.len() == 1 {
        push_inventory_diagnostic(
            &mut diagnostics,
            "inventory_degraded",
            "inventory resolution produced no canonical targets beyond exact source_url",
            policy.source_url.as_str(),
        );
    }
    Ok(InventoryResolution {
        pages,
        diagnostics,
        robots,
        downloaded_bytes: total_inventory_bytes.load(Ordering::SeqCst),
    })
}
