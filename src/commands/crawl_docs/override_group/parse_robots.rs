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
                format!(
                    "robots.txt is not valid UTF-8 at byte {}",
                    error.valid_up_to()
                ),
            );
        }
    };
    // The one reading of robots.txt, shared with every library fetch.
    let (directives, sitemaps) = lib::robots_directives(text);
    let directives = directives
        .into_iter()
        .map(|directive| RobotsRule {
            pattern: directive.pattern,
            specificity: directive.specificity,
            allow: directive.allow,
        })
        .collect();
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
