use super::*;

pub(crate) fn nonempty_observation(value: &Value) -> bool {
    value
        .get("observation")
        .and_then(Value::as_str)
        .is_some_and(|text| !text.trim().is_empty())
}

pub(crate) fn is_null_or_empty_string_or_list(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) => s.is_empty(),
        Some(Value::Array(a)) => a.is_empty(),
        _ => false,
    }
}

/// Refuses `record` when any of `fields` is absent, null, empty text or an
/// empty list, naming every such field.
pub(crate) fn require_nonempty(record: &Value, fields: &[String], context: &str) -> Result<()> {
    let missing: Vec<&String> = fields
        .iter()
        .filter(|field| is_null_or_empty_string_or_list(record.get(field.as_str())))
        .collect();
    if !missing.is_empty() {
        bail!("{context}: missing {:?}", missing);
    }
    Ok(())
}

/// The text `record` states under `field`. A field that is absent, not text
/// or empty is refused naming it, never read as an empty string that a later
/// check reports as some other defect.
pub(crate) fn required_text<'a>(record: &'a Value, field: &str, context: &str) -> Result<&'a str> {
    match record.get(field) {
        Some(Value::String(text)) if !text.is_empty() => Ok(text),
        Some(other) => bail!("{context}: {field} must be non-empty text, found {other}"),
        None => bail!("{context}: {field} is missing"),
    }
}

pub(crate) fn python_repr(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "None".into(),
        Some(Value::Bool(true)) => "True".into(),
        Some(Value::Bool(false)) => "False".into(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::String(s)) => format!("'{s}'"),
        Some(other) => other.to_string(),
    }
}

/// urlparse-based web URL validity: the scheme is a web scheme, the netloc non-empty.
pub(crate) fn url_ok(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if crate::WebScheme::named(&scheme.to_ascii_lowercase()).is_none() {
        return false;
    }
    let Some(authority) = rest.split(['/', '?', '#']).next() else {
        return false;
    };
    // Strip userinfo; netloc itself must be non-empty.
    let Some(host) = authority.rsplit('@').next() else {
        return false;
    };
    !authority.is_empty() && !host.is_empty()
}

/// Resolve `relative` under `base`, refuse escapes and non-files. Lexical
/// normalization stands in for pathlib.resolve(); symlinks inside a catalog
/// pointing outside are not followed here.
pub(crate) fn resolve_evidence_path(base: &Path, relative: &str, context: &str) -> Result<PathBuf> {
    let joined = base.join(relative);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    bail!("{context}: unavailable local evidence {relative:?}");
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.starts_with(base) || !normalized.is_file() {
        bail!("{context}: unavailable local evidence {relative:?}");
    }
    Ok(normalized)
}

pub(crate) fn validate_file_metadata(path: &Path, record: &Value, context: &str) -> Result<()> {
    let payload = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let expected_bytes = record.get("bytes").and_then(Value::as_u64);
    if Some(payload.len() as u64) != expected_bytes {
        bail!("{context}: byte count differs from the file");
    }
    let expected_sha = record.get("sha256").and_then(Value::as_str);
    if Some(lib::sha256_hex(&payload).as_str()) != expected_sha {
        bail!("{context}: SHA-256 differs from the file");
    }
    Ok(())
}
