use super::USER_AGENT;

/// One rule of the robots.txt group addressed to [`USER_AGENT`]: its anchored
/// expression, how many literal characters it matches (the longer rule
/// wins) and whether it allows the path.
#[derive(Clone, Debug)]
pub struct RobotsDirective {
    pub pattern: String,
    pub specificity: usize,
    pub allow: bool,
}

/// How closely a group's agent token names [`USER_AGENT`]: the `*` group
/// answers anyone, a named token answers by its length, the longest winning.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AgentMatch {
    Anyone,
    Named(usize),
}

/// The rules a robots.txt addresses to [`USER_AGENT`] — the group whose agent
/// token it starts with, the longest such, else the `*` group — and its
/// sitemap lines. The one reading of robots.txt: crawl-docs keeps it as its
/// snapshot and compiles it into the matcher every documentation fetch obeys.
pub fn robots_directives(text: &str) -> (Vec<RobotsDirective>, Vec<String>) {
    let mut groups: Vec<(Vec<String>, Vec<RobotsDirective>)> = Vec::new();
    let mut agents = Vec::new();
    let mut directives = Vec::new();
    let mut sitemaps = Vec::new();
    for raw in text.lines() {
        let line = match raw.split_once('#') {
            Some((before, _)) => before,
            None => raw,
        }
        .trim();
        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        let field = field.trim();
        let value = value.trim();
        if field.eq_ignore_ascii_case("sitemap") {
            sitemaps.push(value.to_string());
            continue;
        }
        if field.eq_ignore_ascii_case("user-agent") {
            if !directives.is_empty() {
                groups.push((std::mem::take(&mut agents), std::mem::take(&mut directives)));
            }
            agents.push(value.to_ascii_lowercase());
            continue;
        }
        let allow = field.eq_ignore_ascii_case("allow");
        if (allow || field.eq_ignore_ascii_case("disallow"))
            && !agents.is_empty()
            && !value.is_empty()
        {
            let (source, anchor) = match value.strip_suffix('$') {
                Some(source) => (source, "$"),
                None => (value, ""),
            };
            directives.push(RobotsDirective {
                pattern: format!("^{}{anchor}", regex::escape(source).replace(r"\*", ".*")),
                specificity: source.chars().filter(|character| *character != '*').count(),
                allow,
            });
        }
    }
    if !agents.is_empty() {
        groups.push((agents, directives));
    }
    let user_agent = USER_AGENT.to_ascii_lowercase();
    let matching = |agent: &str| {
        if agent == "*" {
            Some(AgentMatch::Anyone)
        } else if user_agent.starts_with(agent) {
            Some(AgentMatch::Named(agent.len()))
        } else {
            None
        }
    };
    let wanted = groups
        .iter()
        .flat_map(|(agents, _)| agents)
        .filter_map(|agent| matching(agent))
        .max();
    let directives = match wanted {
        Some(wanted) => groups
            .into_iter()
            .filter(|(agents, _)| agents.iter().any(|agent| matching(agent) == Some(wanted)))
            .flat_map(|(_, rules)| rules)
            .collect(),
        None => Vec::new(),
    };
    (directives, sitemaps)
}
