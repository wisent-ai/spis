use super::*;

pub const USER_AGENT: &str = concat!(
    "Spis/",
    env!("CARGO_PKG_VERSION"),
    " (evidence-grade interface corpus; +https://spis.wisent.com/docs)"
);

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
            self.directives
                .iter()
                .filter(|rule| rule.pattern.is_match(path))
                .max_by(|left, right| {
                    left.specificity
                        .cmp(&right.specificity)
                        .then(left.allow.cmp(&right.allow))
                })
                .is_none_or(|rule| rule.allow)
        }
    }

    /// Every path refused: the robots.txt that disallows `/` to everyone,
    /// read the one way every robots.txt is. The answer to an origin whose
    /// robots.txt is withheld or could not be read whole, or that holds a
    /// rule that does not compile.
    fn deny_all() -> Rules {
        let (directives, _) = super::robots_directives("User-agent: *\nDisallow: /");
        Rules {
            directives: directives
                .into_iter()
                .map(|directive| Rule {
                    pattern: regex::Regex::new(&directive.pattern)
                        .expect("the deny-all rule compiles"),
                    specificity: directive.specificity,
                    allow: directive.allow,
                })
                .collect(),
        }
    }

    fn compute(origin: &str) -> Rules {
        let url = format!("{origin}/robots.txt");
        let response = ureq::get(&url).set("User-Agent", USER_AGENT).call();
        let body = match response {
            Ok(response) => match response.into_string() {
                Ok(body) => Some(body),
                Err(error) => {
                    eprintln!(
                        "robots.txt of {origin} could not be read, so every path is denied: {error}"
                    );
                    return deny_all();
                }
            },
            Err(ureq::Error::Status(status, _)) if status == 401 || status == 403 => {
                return deny_all();
            }
            Err(ureq::Error::Status(status, _)) if (400..500).contains(&status) => None,
            Err(_) => return deny_all(),
        };
        let Some(body) = body else {
            return Rules {
                directives: Vec::new(),
            };
        };
        let (directives, _) = super::robots_directives(&body);
        let compiled: Result<Vec<Rule>, regex::Error> = directives
            .into_iter()
            .map(|directive| {
                regex::Regex::new(&directive.pattern).map(|pattern| Rule {
                    pattern,
                    specificity: directive.specificity,
                    allow: directive.allow,
                })
            })
            .collect();
        match compiled {
            Ok(directives) => Rules { directives },
            // A served rule that does not compile denies, as crawl-docs does;
            // dropping it would allow the very paths it disallows.
            Err(error) => {
                eprintln!("robots.txt of {origin} holds a rule that does not compile, so every path is denied: {error}");
                deny_all()
            }
        }
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

pub fn robots_allows(url: &str) -> bool {
    robots::allows(url)
}

pub(crate) fn between<'a>(hay: &'a str, open: &str, close: &str) -> Option<String> {
    let s = hay.find(open)? + open.len();
    let e = hay[s..].find(close)? + s;
    Some(html_unescape(hay[s..e].trim()))
}
