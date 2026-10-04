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

/// Compile one robots pattern. A pattern the regex engine refuses fails here,
/// and the caller then denies everything: a dropped `Disallow` would read as
/// an allow.
pub(crate) fn compile_robots_pattern(pattern: &str) -> Result<regex::Regex> {
    regex::Regex::new(pattern).with_context(|| format!("robots pattern {pattern:?} does not compile"))
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
    diagnostics.push(CrawlDiagnostic {
        code: code.into(),
        message: message.into(),
        url: url.into(),
    });
}
