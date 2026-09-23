use super::*;

/// The site's robots.txt, compiled, with the sitemaps it names. Anything but a served policy or a
/// served "no policy here" (404/410) denies everything and leaves a durable diagnostic.
pub(super) fn fetch_robots(
    policy: &UrlPolicy,
    inventory_budget: Option<ByteBudget<'_>>,
    diagnostics: &mut Vec<CrawlDiagnostic>,
) -> Result<(RobotsSnapshot, CompiledRobots, Vec<String>)> {
    let robots_url = policy.source_url.join("/robots.txt")?;
    let fetched = match bounded_http_get(
        &robots_url,
        policy,
        MAX_ROBOTS_BYTES,
        "robots.txt",
        inventory_budget,
    ) {
        Ok(response) if (200..300).contains(&response.status) => {
            parse_robots(&response.body, diagnostics, &robots_url)?
        }
        Ok(response) if matches!(response.status, 401 | 403) => {
            push_inventory_diagnostic(
                diagnostics,
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
                diagnostics,
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
                diagnostics,
                error.code,
                error.to_string(),
                robots_url.as_str(),
            );
            let snapshot = RobotsSnapshot::deny_all();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
    };
    Ok(fetched)
}
