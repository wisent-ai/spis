use super::*;

impl RobotsSnapshot {
    /// The deny-all snapshot every robots failure path falls back to. Kept as one
    /// constructor so the serialized shape cannot drift between failure paths; it
    /// is persisted in `DurableState.robots` and feeds `inventory_sha256`.
    pub(crate) fn deny_all() -> Self {
        Self {
            directives: vec![RobotsRule {
                pattern: "^/".into(),
                specificity: 1,
                allow: false,
            }],
        }
    }
}

/// `RobotsSnapshot` is the persisted wire form; this is the matcher. Patterns are
/// compiled exactly once per run instead of once per rule per URL.
/// Compilation is fallible here and never ignored, so a rule can no longer be
/// silently dropped — dropping a `Disallow` would have turned it into an
/// allow, the one fail-open path in this file.
pub(crate) struct CompiledRobots {
    pub(crate) rules: Vec<CompiledRobotsRule>,
}

pub(crate) struct CompiledRobotsRule {
    pub(crate) pattern: regex::Regex,
    pub(crate) specificity: usize,
    pub(crate) allow: bool,
}

impl CompiledRobots {
    pub(crate) fn compile(snapshot: &RobotsSnapshot) -> Result<Self> {
        let rules = snapshot
            .directives
            .iter()
            .map(|rule| {
                Ok(CompiledRobotsRule {
                    pattern: compile_robots_pattern(&rule.pattern)?,
                    specificity: rule.specificity,
                    allow: rule.allow,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { rules })
    }

    pub(crate) fn allows(&self, url: &Url) -> bool {
        let mut path = url.path().to_string();
        if let Some(query) = url.query() {
            path.push('?');
            path.push_str(query);
        }
        self.rules
            .iter()
            .filter(|rule| rule.pattern.is_match(&path))
            .max_by(|left, right| {
                left.specificity
                    .cmp(&right.specificity)
                    .then(left.allow.cmp(&right.allow))
            })
            .is_none_or(|rule| rule.allow)
    }
}

/// Compile one robots pattern under an explicit program-size ceiling, so the
/// bound is ours rather than whatever `regex` defaults to.
pub(crate) fn compile_robots_pattern(pattern: &str) -> Result<regex::Regex> {
    regex::RegexBuilder::new(pattern)
        .size_limit(MAX_ROBOTS_PROGRAM_BYTES)
        .build()
        .with_context(|| format!("robots pattern {pattern:?} does not compile"))
}

/// Every in-scope page a site's inventory names, with what reading it cost.
/// No page is left out for want of room: a corpus holds the whole site.
pub(crate) struct InventoryResolution {
    pub(crate) pages: Vec<(String, Option<String>)>,
    pub(crate) diagnostics: Vec<CrawlDiagnostic>,
    pub(crate) robots: RobotsSnapshot,
    pub(crate) downloaded_bytes: u64,
}

pub(crate) fn push_inventory_diagnostic(
    diagnostics: &mut Vec<CrawlDiagnostic>,
    code: &str,
    message: impl Into<String>,
    url: impl Into<String>,
) {
    if diagnostics.len() < MAX_INVENTORY_DIAGNOSTICS {
        diagnostics.push(CrawlDiagnostic {
            code: code.into(),
            message: message.into(),
            url: url.into(),
        });
    } else if diagnostics.len() == MAX_INVENTORY_DIAGNOSTICS {
        diagnostics.push(CrawlDiagnostic {
            code: "inventory_diagnostics_truncated".into(),
            message: format!(
                "inventory diagnostics exceeded the {MAX_INVENTORY_DIAGNOSTICS}-entry limit"
            ),
            url: String::new(),
        });
    }
}
