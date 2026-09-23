use super::*;

pub const USER_AGENT: &str =
    concat!("Spis/", env!("CARGO_PKG_VERSION"), " (evidence-grade interface corpus; +https://spis.wisent.com/docs)");

/// Every object a crawl publishes lives under one namespace and one of its two
/// named roots.
///
/// Stado authorizes object traffic per namespace *prefix*
/// (`object_api.namespaces`, `ObjectPrefixPolicy`), and a prefix only matches
/// as a prefix when it ends in `/`; a key that begins with a per-run or
/// per-digest identifier can therefore be granted by nothing narrower than the
/// empty prefix, which is the whole namespace. Every namespace Stado declares
/// grants named first segments instead, so the crawl tree carries two: one
/// attempt root and one input root. They share a namespace deliberately — the
/// caller sends exactly one bearer per request and the service compares it
/// against the credential item of the namespace being addressed, so a second
/// namespace would mean a second bearer for the same coordinator run.
pub const CRAWL_NAMESPACE: &str = "stado://spis-crawls";

/// Immutable attempt trees: `{CRAWL_ATTEMPT_ROOT}/{run_id}/...`.
pub const CRAWL_ATTEMPT_ROOT: &str = "stado://spis-crawls/runs";

/// Immutable coordinator inputs, addressed by content digest. Today that is
/// the generated runtime-bindings document every worker re-downloads.
pub const CRAWL_INPUT_ROOT: &str = "stado://spis-crawls/inputs/runtime-bindings";

/// The immutable coordinate of one record inside one run.
pub fn crawl_record_base_uri(
    run_id: &str,
    catalog: &str,
    record: &str,
    record_key: &str,
) -> String {
    format!("{CRAWL_ATTEMPT_ROOT}/{run_id}/{catalog}/{record}/{record_key}")
}

/// The immutable coordinate of one attempt of one record. Every producer and
/// every verifier of an attempt URI derives it here, so a change of shape can
/// never leave one side of a digest comparison spelling it the old way.
pub fn crawl_attempt_base_uri(
    run_id: &str,
    catalog: &str,
    record: &str,
    record_key: &str,
    // The attempt number is a u32 in the runtime manifest, a u64 in the
    // durable state and a serde integer in a receipt. It renders identically
    // in all three, and accepting them by display keeps one derivation
    // instead of a cast at every call site.
    attempt: impl std::fmt::Display,
    attempt_id: &str,
) -> String {
    format!(
        "{}/attempts/{attempt}/{attempt_id}",
        crawl_record_base_uri(run_id, catalog, record, record_key)
    )
}

pub(crate) mod robots {
    use super::USER_AGENT;
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::time::Duration;

    #[derive(Clone)]
    struct Rule {
        pattern: regex::Regex,
        specificity: usize,
        allow: bool,
    }

    #[derive(Clone)]
    struct Rules {
        directives: Vec<Rule>,
    }

    impl Rules {
        fn allows(&self, path: &str) -> bool {
            self.directives.iter()
                .filter(|rule| rule.pattern.is_match(path))
                .max_by(|left, right| left.specificity.cmp(&right.specificity).then(left.allow.cmp(&right.allow)))
                .is_none_or(|rule| rule.allow)
        }
    }

    fn compute(origin: &str) -> Rules {
        let mut groups: Vec<(Vec<String>, Vec<Rule>)> = Vec::new();
        let url = format!("{origin}/robots.txt");
        let response = ureq::get(&url)
            .timeout(Duration::from_secs(15))
            .set("User-Agent", USER_AGENT)
            .call();
        let body = match response {
            Ok(response) => response.into_string().ok(),
            Err(ureq::Error::Status(status, _)) if status == 401 || status == 403 => {
                return Rules { directives: vec![Rule {
                    pattern: regex::Regex::new("^/").expect("fixed robots pattern"),
                    specificity: 1,
                    allow: false,
                }] };
            }
            Err(ureq::Error::Status(status, _)) if (400..500).contains(&status) => None,
            Err(_) => {
                return Rules { directives: vec![Rule {
                    pattern: regex::Regex::new("^/").expect("fixed robots pattern"),
                    specificity: 1,
                    allow: false,
                }] };
            }
        };
        if let Some(body) = body {
            let mut agents = Vec::new();
                let mut directives = Vec::new();
                for raw in body.lines() {
                    let line = raw.split('#').next().unwrap_or("").trim();
                    let Some((field, value)) = line.split_once(':') else { continue };
                    let field = field.trim();
                    let value = value.trim();
                    if field.eq_ignore_ascii_case("user-agent") {
                        if !directives.is_empty() {
                            groups.push((std::mem::take(&mut agents), std::mem::take(&mut directives)));
                        }
                        agents.push(value.to_ascii_lowercase());
                    } else if field.eq_ignore_ascii_case("allow") || field.eq_ignore_ascii_case("disallow") {
                        if !agents.is_empty() && !value.is_empty() {
                            let terminal = value.ends_with('$');
                            let source = value.strip_suffix('$').unwrap_or(value);
                            let expression = format!(
                                "^{}{}",
                                regex::escape(source).replace(r"\*", ".*"),
                                if terminal { "$" } else { "" }
                            );
                            if let Ok(pattern) = regex::Regex::new(&expression) {
                                directives.push(Rule {
                                    pattern,
                                    specificity: source.chars().filter(|character| *character != '*').count(),
                                    allow: field.eq_ignore_ascii_case("allow"),
                                });
                            }
                        }
                    }
                }
                if !agents.is_empty() {
                    groups.push((agents, directives));
                }
            }
        let user_agent = USER_AGENT.to_ascii_lowercase();
        let specificity = groups.iter().flat_map(|(agents, _)| agents)
            .filter_map(|agent| if agent == "*" { Some(0) } else if user_agent.starts_with(agent) { Some(agent.len()) } else { None })
            .max();
        let directives = specificity.map(|wanted| groups.into_iter()
            .filter(|(agents, _)| agents.iter().any(|agent| {
                (agent == "*" && wanted == 0) || (agent != "*" && agent.len() == wanted && user_agent.starts_with(agent))
            }))
            .flat_map(|(_, rules)| rules)
            .collect()).unwrap_or_default();
        Rules { directives }
    }

    static CACHE: Mutex<Option<HashMap<String, Rules>>> = Mutex::new(None);

    pub fn allows(url: &str) -> bool {
        let origin = super::origin_of(url);
        let path_and_query = url[origin.len()..].to_string();
        let rules = {
            let mut guard = CACHE.lock();
            guard
                .get_or_insert_with(HashMap::new)
                .entry(origin.to_string())
                .or_insert_with(|| compute(&origin))
                .clone()
        };
        rules.allows(&path_and_query)
    }
}

pub fn origin_of(url: &str) -> String {
    match url.split_once("://") {
        Some((scheme, rest)) => {
            format!("{scheme}://{}", rest.split('/').next().unwrap_or_default())
        }
        None => String::new(),
    }
}

pub fn split_url(url: &str) -> (String, String) {
    let (scheme, after_scheme) = match url.split_once("://") {
        Some((s, r)) => (s, r),
        None => ("http", url),
    };
    match after_scheme.find('/') {
        Some(slash) => {
            let host = format!("{scheme}://{}", &after_scheme[..slash]);
            let path_q = after_scheme[slash..].to_string();
            (host, path_q)
        }
        None => (format!("{scheme}://{after_scheme}"), "/".into()),
    }
}

/// Transparently decompress a gzipped payload (magic 1f 8b); passthrough otherwise.
pub fn maybe_gunzip(body: &[u8]) -> Vec<u8> {
    if body.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        let mut dec = flate2::read::MultiGzDecoder::new(body);
        if std::io::Read::read_to_end(&mut dec, &mut out).is_err() {
            return body.to_vec();
        }
        out
    } else {
        body.to_vec()
    }
}

pub fn http_get_with_retry(url: &str, tries: u32) -> Result<(u16, String)> {
    // Some servers stream bodies slower than any sane timeout; ureq's own
    // deadline does not always fire during body streaming, so each attempt
    // runs under a hard join-timeout. A timed-out attempt leaks its thread
    // until the server closes — accepted because such stragglers are rare.
    const ATTEMPT_DEADLINE: Duration = Duration::from_secs(45);

    for attempt in 1..=tries {
        let (tx, rx) = std::sync::mpsc::channel::<Result<(u16, String)>>();
        let url_owned = url.to_string();
        std::thread::spawn(move || {
            use std::io::Read as _;
            let resp = ureq::get(&url_owned)
                .timeout(ATTEMPT_DEADLINE)
                .set("User-Agent", USER_AGENT)
                .call();
            let result = match resp {
                Ok(r) => {
                    let status = r.status() as u16;
                    let mut body = Vec::new();
                    let read = r.into_reader().take(256 << 20).read_to_end(&mut body);
                    match read {
                        Ok(_) => Ok((status, String::from_utf8_lossy(&body).to_string())),
                        Err(e) => Err(anyhow::anyhow!("body read: {e}")),
                    }
                }
                Err(ureq::Error::Status(code, _)) => Err(anyhow::anyhow!("HTTP {code}")),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            };
            let _ = tx.send(result);
        });
        match rx.recv_timeout(ATTEMPT_DEADLINE) {
            Ok(Ok(pair)) => return Ok(pair),
            Ok(Err(e)) => {
                let msg = format!("{e:#}");
                let retryable = ["HTTP 429", "HTTP 500", "HTTP 502", "HTTP 503", "HTTP 504"]
                    .iter()
                    .any(|m| msg.contains(m));
                if retryable && attempt < tries {
                    std::thread::sleep(Duration::from_secs(2));
                    continue;
                }
                bail!("{msg}");
            }
            Err(_) => bail!(
                "attempt timed out after {}s: {url}",
                ATTEMPT_DEADLINE.as_secs()
            ),
        }
    }
    unreachable!("retry loop always returns or bails")
}

pub fn robots_allows(url: &str) -> bool {
    robots::allows(url)
}

pub(crate) fn between<'a>(hay: &'a str, open: &str, close: &str) -> Option<String> {
    let s = hay.find(open)? + open.len();
    let e = hay[s..].find(close)? + s;
    Some(html_unescape(hay[s..e].trim()))
}
