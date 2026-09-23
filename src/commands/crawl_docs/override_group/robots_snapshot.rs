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
/// compiled exactly once per run instead of once per rule per URL, which used to
/// cost up to `MAX_ROBOTS_RULES * MAX_TARGETS` compilations of identical
/// programs. Compilation is fallible here and never ignored, so a rule can no
/// longer be silently dropped — dropping a `Disallow` would have turned it into
/// an allow, the one fail-open path in this file.
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

/// What one record's inventory could not fit, and whether that number is
/// exact.
///
/// A documentation corpus holds at most [`MAX_TARGETS`] page records. Four
/// sites in the documentation family declare inventories far past it -- Google
/// Cloud at 216,092 canonical URLs, .NET at 201,460, Azure at 201,009 and MDN
/// at 54,594 -- so for those four the bound is not a safety margin, it decides
/// what the record can ever contain. Until this existed the bound was applied
/// and never stated: the inventory walk stopped at the 50,000th URL and the
/// run recorded one diagnostic naming the limit but no quantity, so nothing
/// downstream could say whether a record was missing ten pages or a hundred
/// and sixty thousand.
///
/// `exact` is false only when the excluded set itself hit
/// [`MAX_COUNTED_EXCLUDED_KEYS`], in which case `pages_outside_corpus` is a
/// floor and says so rather than a number pretending to be the total.
#[derive(Clone, Copy, Deserialize, Serialize)]
pub(crate) struct CorpusCapacity {
    pub(crate) pages_outside_corpus: u64,
    pub(crate) exact: bool,
}

pub(crate) struct InventoryResolution {
    pub(crate) pages: Vec<(String, Option<String>)>,
    pub(crate) diagnostics: Vec<CrawlDiagnostic>,
    pub(crate) robots: RobotsSnapshot,
    pub(crate) downloaded_bytes: u64,
    pub(crate) capacity: Option<CorpusCapacity>,
}

/// Record one in-scope URL that did not fit, by digest rather than by string.
///
/// Digests, not URLs, because the whole point of [`MAX_TARGETS`] is that this
/// process does not hold the excluded material: a set of digests counts the
/// distinct remainder without retaining what it names. The set is itself
/// bounded, so an inventory larger than anything this fleet has measured
/// degrades to a stated floor instead of unbounded memory.
pub(crate) fn note_excluded(
    excluded: &mut std::collections::HashSet<String>,
    exact: &mut bool,
    url: &str,
) {
    if excluded.len() >= MAX_COUNTED_EXCLUDED_KEYS {
        *exact = false;
        return;
    }
    excluded.insert(lib::sha256_hex(url.as_bytes()));
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
