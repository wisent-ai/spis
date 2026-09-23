use super::*;

/// Build the persisted snapshot *and* its compiled matcher together, so a
/// snapshot can never reach the crawl loop without every pattern having compiled
/// successfully. Every recoverable robots problem yields the deny-all policy plus
/// a durable diagnostic; only a failure to compile our own deny-all constant is
/// an error, and that is still fail-closed because it aborts the run.
pub(crate) fn parse_robots(
    body: &[u8],
    diagnostics: &mut Vec<CrawlDiagnostic>,
    robots_url: &Url,
) -> Result<(RobotsSnapshot, CompiledRobots, Vec<String>)> {
    let deny_all = |diagnostics: &mut Vec<CrawlDiagnostic>,
                    code: &str,
                    message: String|
     -> Result<(RobotsSnapshot, CompiledRobots, Vec<String>)> {
        push_inventory_diagnostic(diagnostics, code, message, robots_url.as_str());
        let snapshot = RobotsSnapshot::deny_all();
        let compiled = CompiledRobots::compile(&snapshot)?;
        Ok((snapshot, compiled, Vec::new()))
    };
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return deny_all(
                diagnostics,
                "robots_non_utf8",
                format!("robots.txt is not valid UTF-8 at byte {}", error.valid_up_to()),
            );
        }
    };
    let mut groups: Vec<(Vec<String>, Vec<RobotsRule>)> = Vec::new();
    let mut agents = Vec::new();
    let mut directives = Vec::new();
    let mut sitemaps = Vec::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        let field = field.trim();
        let value = value.trim();
        if field.eq_ignore_ascii_case("sitemap") {
            if sitemaps.len() < MAX_INVENTORY_SOURCES {
                sitemaps.push(value.to_string());
            }
            continue;
        }
        if field.eq_ignore_ascii_case("user-agent") {
            if !directives.is_empty() {
                groups.push((
                    std::mem::take(&mut agents),
                    std::mem::take(&mut directives),
                ));
            }
            agents.push(value.to_ascii_lowercase());
        } else if (field.eq_ignore_ascii_case("allow")
            || field.eq_ignore_ascii_case("disallow"))
            && !agents.is_empty()
            && !value.is_empty()
        {
            if directives.len() >= MAX_ROBOTS_RULES {
                push_inventory_diagnostic(
                    diagnostics,
                    "robots_rules_truncated",
                    format!("robots directives exceeded the {MAX_ROBOTS_RULES}-rule limit"),
                    robots_url.as_str(),
                );
                continue;
            }
            let terminal = value.ends_with('$');
            let source = value.strip_suffix('$').unwrap_or(value);
            // A robots.txt line is otherwise bounded only by MAX_ROBOTS_BYTES, so
            // an origin could serve `Disallow: /` plus ~250 000 `*` characters and
            // push the compiled program past any sane ceiling. Refuse the whole
            // file fail-closed rather than dropping the offending rule, because a
            // dropped `Disallow` reads as an allow.
            if source.len() > MAX_ROBOTS_PATTERN_BYTES {
                return deny_all(
                    diagnostics,
                    "robots_rule_too_long",
                    format!(
                        "robots directive exceeds the {MAX_ROBOTS_PATTERN_BYTES}-byte pattern limit"
                    ),
                );
            }
            directives.push(RobotsRule {
                pattern: format!(
                    "^{}{}",
                    regex::escape(source).replace(r"\*", ".*"),
                    if terminal { "$" } else { "" }
                ),
                specificity: source.chars().filter(|character| *character != '*').count(),
                allow: field.eq_ignore_ascii_case("allow"),
            });
        }
    }
    if !agents.is_empty() {
        groups.push((agents, directives));
    }
    let user_agent = lib::USER_AGENT.to_ascii_lowercase();
    let specificity = groups
        .iter()
        .flat_map(|(agents, _)| agents)
        .filter_map(|agent| {
            if agent == "*" {
                Some(0)
            } else if user_agent.starts_with(agent) {
                Some(agent.len())
            } else {
                None
            }
        })
        .max();
    let directives = specificity
        .map(|wanted| {
            groups
                .into_iter()
                .filter(|(agents, _)| {
                    agents.iter().any(|agent| {
                        (agent == "*" && wanted == 0)
                            || (agent != "*"
                                && agent.len() == wanted
                                && user_agent.starts_with(agent))
                    })
                })
                .flat_map(|(_, rules)| rules)
                .take(MAX_ROBOTS_RULES)
                .collect()
        })
        .unwrap_or_default();
    let snapshot = RobotsSnapshot { directives };
    match CompiledRobots::compile(&snapshot) {
        Ok(compiled) => Ok((snapshot, compiled, sitemaps)),
        // A served rule that will not compile must deny, never allow.
        Err(error) => deny_all(
            diagnostics,
            "robots_rule_uncompilable",
            format!("robots directive could not be compiled: {error:#}"),
        ),
    }
}

pub(crate) fn path_is_in_scope(url: &Url, prefixes: &[String]) -> Result<bool> {
    if prefixes.is_empty() {
        return Ok(true);
    }
    for prefix in prefixes {
        if !prefix.starts_with('/') || prefix.contains(['?', '#']) {
            bail!("documentation path prefix {prefix:?} is not an absolute path prefix");
        }
        let normalized = prefix.trim_end_matches('/');
        if normalized.is_empty()
            || url.path() == normalized
            || url
                .path()
                .strip_prefix(normalized)
                .is_some_and(|suffix| suffix.starts_with('/'))
        {
            return Ok(true);
        }
    }
    Ok(false)
}
