use super::*;

pub(crate) fn resolve_urls(meta: &SiteMeta, rules: &SiteRules, policy: &UrlPolicy) -> Result<InventoryResolution> {
    let mut diagnostics = Vec::new();
    let total_inventory_bytes = AtomicU64::new(0);
    let inventory_budget = Some(ByteBudget {
        counter: &total_inventory_bytes,
        limit: MAX_TOTAL_INVENTORY_BYTES,
    });
    let robots_url = policy.source_url.join("/robots.txt")?;
    let (robots, compiled_robots, discovered_sitemaps) = match bounded_http_get(
        &robots_url,
        policy,
        MAX_ROBOTS_BYTES,
        "robots.txt",
        inventory_budget,
    ) {
        Ok(response) if (200..300).contains(&response.status) => {
            parse_robots(&response.body, &mut diagnostics, &robots_url)?
        }
        Ok(response) if matches!(response.status, 401 | 403) => {
            push_inventory_diagnostic(
                &mut diagnostics,
                "robots_access_denied",
                format!("robots.txt returned HTTP {}", response.status),
                robots_url.as_str(),
            );
            let snapshot = RobotsSnapshot::deny_all();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
        // An origin that answers "there is no policy here" is genuinely
        // unconstrained, and only these two statuses say that.
        Ok(response) if matches!(response.status, 404 | 410) => {
            let snapshot = RobotsSnapshot::default();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
        // `bounded_http_get` reports a served error status as `Ok`, so 429 and every 5xx
        // arrive here rather than in the transport-error branch below. An origin that
        // rate-limits or fails is an origin whose robots.txt was never observed: the
        // rules it does serve may forbid this sweep, so an unobserved policy denies
        // everything and leaves the same durable diagnostic as a transport failure.
        Ok(response) => {
            push_inventory_diagnostic(
                &mut diagnostics,
                "robots_unavailable",
                format!("robots.txt returned HTTP {}", response.status),
                robots_url.as_str(),
            );
            let snapshot = RobotsSnapshot::deny_all();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
        Err(error) => {
            push_inventory_diagnostic(
                &mut diagnostics,
                error.code,
                error.to_string(),
                robots_url.as_str(),
            );
            let snapshot = RobotsSnapshot::deny_all();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
    };

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
            Ok(url) if queue.len() < MAX_INVENTORY_SOURCES => queue.push_back(url),
            Ok(url) => push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_limit",
                format!("inventory sources exceeded the {MAX_INVENTORY_SOURCES}-source limit"),
                url.as_str(),
            ),
            Err(error) => push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_rejected",
                format!("{error:#}"),
                raw,
            ),
        }
    }

    let mut visited = HashSet::new();
    let mut excluded_keys = HashSet::<String>::new();
    let mut excluded_exact = true;
    while let Some(source) = queue.pop_front() {
        if !visited.insert(source.as_str().to_string()) {
            continue;
        }
        if visited.len() > MAX_INVENTORY_SOURCES {
            push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_limit",
                format!("inventory sources exceeded the {MAX_INVENTORY_SOURCES}-source limit"),
                source.as_str(),
            );
            break;
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
        let response = match bounded_http_get(
            &source,
            policy,
            MAX_INVENTORY_BYTES,
            "documentation inventory source",
            inventory_budget,
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
                if error.code == "total_download_byte_limit" {
                    break;
                }
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
                    Ok(url) if queue.len() + visited.len() < MAX_INVENTORY_SOURCES => {
                        queue.push_back(url)
                    }
                    Ok(url) => push_inventory_diagnostic(
                        &mut diagnostics,
                        "inventory_source_limit",
                        format!(
                            "inventory sources exceeded the {MAX_INVENTORY_SOURCES}-source limit"
                        ),
                        url.as_str(),
                    ),
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
                        // In scope, so it belongs in the material this record is
                        // supposed to hold. Whether it fits is the next
                        // question, and the answer is counted either way --
                        // before this, the walk stopped at the bound and the
                        // run could not say how much it had left behind.
                        if pages.len() < MAX_TARGETS {
                            pages.push((url.to_string(), lastmod));
                        } else {
                            note_excluded(
                                &mut excluded_keys,
                                &mut excluded_exact,
                                url.as_str(),
                            );
                        }
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
                        if pages.len() < MAX_TARGETS {
                            pages.push((url.to_string(), None));
                        } else {
                            note_excluded(
                                &mut excluded_keys,
                                &mut excluded_exact,
                                url.as_str(),
                            );
                        }
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
        std::thread::sleep(Duration::from_millis(250));
    }

    if pages.len() == 1 && meta.inventory_source.starts_with("landing-nav") {
        for item in &meta.landing_nav {
            match policy.canonical(&item.path, Some(&policy.source_url), "landing navigation page") {
                Ok(url) if path_is_in_scope(&url, &rules.prefixes)? => {
                    if pages.len() < MAX_TARGETS {
                        pages.push((url.to_string(), None));
                    } else {
                        note_excluded(&mut excluded_keys, &mut excluded_exact, url.as_str());
                    }
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
    // The bound, stated with its quantity, and stated where the inventory
    // digest already covers it. `target_limit` used to be pushed the moment
    // the walk hit the bound, naming the limit and nothing else; it is now
    // pushed once, at the end, when the whole in-scope inventory has been
    // counted, so the sentence carries the number an operator has to decide
    // about.
    let capacity = (!excluded_keys.is_empty()).then(|| CorpusCapacity {
        pages_outside_corpus: excluded_keys.len() as u64,
        exact: excluded_exact,
    });
    if let Some(capacity) = capacity {
        let qualifier = if capacity.exact { "" } else { "at least " };
        push_inventory_diagnostic(
            &mut diagnostics,
            "target_limit",
            format!(
                "this site declares more in-scope pages than one corpus holds: {} retrieved \
                 against the {MAX_TARGETS}-page corpus bound, leaving {qualifier}{} pages \
                 outside this record. The bound is per corpus and one site is one corpus, so \
                 the remainder cannot be delivered by this record at all",
                pages.len(),
                capacity.pages_outside_corpus
            ),
            policy.source_url.as_str(),
        );
    }
    Ok(InventoryResolution {
        pages,
        diagnostics,
        robots,
        downloaded_bytes: total_inventory_bytes.load(Ordering::SeqCst),
        capacity,
    })
}
