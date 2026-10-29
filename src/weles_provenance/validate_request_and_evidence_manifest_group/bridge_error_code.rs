use super::*;

/// The error code the bridge states on stderr as `{"code": "..."}`, a
/// non-empty run of ASCII letters, digits, `-` and `_`. A bridge that states
/// none fails as the generic `bridge-error`, and why no code was read is said.
pub(crate) fn bridge_error_code(stderr: &[u8]) -> String {
    match stated_code(stderr) {
        Ok(code) => code,
        Err(why) => {
            eprintln!("spis: the Weles bridge failed without a usable code: {why}");
            "bridge-error".to_string()
        }
    }
}

fn stated_code(stderr: &[u8]) -> Result<String, String> {
    let value = serde_json::from_slice::<Value>(stderr)
        .map_err(|error| format!("its stderr is not JSON ({error})"))?;
    let code = value
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("its stderr states no code: {value}"))?;
    let valid = !code.is_empty()
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(code.to_string())
    } else {
        Err(format!(
            "its stated code {code:?} is not letters, digits, - and _"
        ))
    }
}

pub(crate) fn resolve_retained_file(base: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty() || relative.contains('\\') {
        return Err("retained path must be a portable relative path".to_string());
    }
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("retained path escapes the record directory".to_string());
    }
    let canonical_base = fs::canonicalize(base).map_err(|error| {
        format!(
            "record directory {} could not be resolved: {error}",
            base.display()
        )
    })?;
    let joined = canonical_base.join(relative_path);
    let link_metadata = fs::symlink_metadata(&joined)
        .map_err(|error| format!("retained file {} cannot be read: {error}", joined.display()))?;
    if link_metadata.file_type().is_symlink() || !link_metadata.is_file() {
        return Err("retained path is not a regular non-symlink file".to_string());
    }
    let canonical_file = fs::canonicalize(&joined).map_err(|error| {
        format!(
            "retained file {} could not be resolved: {error}",
            joined.display()
        )
    })?;
    if !canonical_file.starts_with(&canonical_base) {
        return Err("retained file resolves outside the record directory".to_string());
    }
    Ok(canonical_file)
}

/// Everything `reader` yields; a checked-in file, a document the reference pins by
/// digest, or the bridge's own output is as long as it is.
pub(crate) fn read_stream(mut reader: impl Read, label: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{label} could not be read: {error}"))?;
    Ok(bytes)
}

/// A retained file read up to the byte count its signed inventory or receipt states;
/// more is refused, because the signature covers exactly that many bytes.
pub(crate) fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|error| {
        format!(
            "retained file {} could not be opened: {error}",
            path.display()
        )
    })?;
    let mut bytes = Vec::new();
    let mut taken = file.take(limit);
    taken.read_to_end(&mut bytes).map_err(|error| {
        format!(
            "retained file {} could not be read: {error}",
            path.display()
        )
    })?;
    // One byte past the signed count means the file is longer than signed.
    match taken.into_inner().bytes().next() {
        None => Ok(bytes),
        Some(Ok(_)) => Err("retained file is longer than its signed byte count".to_string()),
        Some(Err(error)) => Err(format!(
            "retained file {} could not be read past its signed byte count: {error}",
            path.display()
        )),
    }
}

/// A retained file read whole; its digest is then checked against the reference.
pub(crate) fn read_whole(path: &Path) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|error| {
        format!(
            "retained file {} could not be opened: {error}",
            path.display()
        )
    })?;
    read_stream(file, "retained JSON document")
}

pub(crate) fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| {
        format!(
            "retained artifact {} could not be opened: {error}",
            path.display()
        )
    })?;
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut DigestWriter(&mut hash)).map_err(|error| {
        format!(
            "retained artifact {} could not be hashed: {error}",
            path.display()
        )
    })?;
    Ok(hex::encode(hash.finalize()))
}

pub(crate) struct DigestWriter<'a, D>(pub(crate) &'a mut D);

impl<D: Digest> Write for DigestWriter<'_, D> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// A git object name: a SHA-1 digest of twenty bytes, hex encoded:
/// https://git-scm.com/docs/hash-function-transition (SHA-1 object names)
const GIT_SHA1_BYTES: usize = 20;

/// Lowercase hex encoding exactly `bytes` bytes.
fn lower_hex_of(value: &str, bytes: usize) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        && hex::decode(value).is_ok_and(|decoded| decoded.len() == bytes)
}

/// A SHA-256 digest in lowercase hex, its length that of the digest itself.
pub(crate) fn is_sha256(value: &str) -> bool {
    lower_hex_of(value, <Sha256 as Digest>::output_size())
}

pub(crate) fn is_git_revision(value: &str) -> bool {
    lower_hex_of(value, GIT_SHA1_BYTES)
}

pub(crate) fn is_sha256_id(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(is_sha256)
}

pub(crate) fn update_framed(hash: &mut Sha256, label: &str, value: &str) {
    hash.update(label.len().to_string().as_bytes());
    hash.update(b":");
    hash.update(label.as_bytes());
    hash.update(value.len().to_string().as_bytes());
    hash.update(b":");
    hash.update(value.as_bytes());
}

pub(crate) fn provenance_id(
    receipt: &RetainedReceipt,
    key_set_version: &str,
    artifact: &RetainedArtifact,
) -> Result<String, String> {
    let binding_value = serde_json::to_value(&receipt.spis_binding)
        .map_err(|error| format!("receipt spisBinding could not be canonicalized: {error}"))?;
    let binding_json = String::from_utf8(canonical_json_bytes(&binding_value)?)
        .map_err(|error| format!("canonical receipt spisBinding was not UTF-8: {error}"))?;
    let mut hash = Sha256::new();
    for (label, value) in [
        ("receipt.schema", receipt.schema.as_str()),
        ("receipt.keyId", receipt.key_id.as_str()),
        ("receipt.signedPayload", receipt.signed_payload.as_str()),
        ("receipt.signature", receipt.signature.as_str()),
        ("receipt.requestDigest", receipt.request_digest.as_str()),
        ("receipt.resultDigest", receipt.result_digest.as_str()),
        ("receipt.spisBinding", binding_json.as_str()),
        ("keySetVersion", key_set_version),
        ("artifact.path", artifact.path.as_str()),
        ("artifact.sha256", artifact.sha256.as_str()),
    ] {
        update_framed(&mut hash, label, value);
    }
    Ok(format!("sha256:{}", hex::encode(hash.finalize())))
}

pub(crate) fn strip_provenance(value: &Value) -> Value {
    match value {
        Value::Array(entries) => Value::Array(entries.iter().map(strip_provenance).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, _)| key.as_str() != "provenance")
                .map(|(key, entry)| (key.clone(), strip_provenance(entry)))
                .collect(),
        ),
        scalar => scalar.clone(),
    }
}

pub(crate) fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, String> {
    /// Number.MAX_SAFE_INTEGER, the largest integer JCS (RFC 8785) carries
    /// exactly: https://tc39.es/ecma262/#sec-number.max_safe_integer
    const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

    fn write_value(value: &Value, output: &mut String) -> Result<(), String> {
        match value {
            Value::Null => output.push_str("null"),
            Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            Value::Number(number) => {
                if let Some(value) = number.as_i64() {
                    if value.unsigned_abs() > MAX_SAFE_INTEGER {
                        return Err("JCS input integer exceeds the safe integer range".to_string());
                    }
                    output.push_str(&value.to_string());
                } else if let Some(value) = number.as_u64() {
                    if value > MAX_SAFE_INTEGER {
                        return Err("JCS input integer exceeds the safe integer range".to_string());
                    }
                    output.push_str(&value.to_string());
                } else {
                    // One declared canonicalization, one behavior: `JSON.parse("1.0")`
                    // yields the JS number 1 and the bridge emits `1`, so an integral
                    // double inside the safe-integer range canonicalizes to the same
                    // integer text here. Fractional and out-of-range numbers are
                    // rejected on both sides.
                    let float = number.as_f64().ok_or_else(|| {
                        "JCS input contains an unrepresentable number".to_string()
                    })?;
                    if !float.is_finite() || float.trunc() != float {
                        return Err("JCS input contains a fractional number".to_string());
                    }
                    if float.abs() > MAX_SAFE_INTEGER as f64 {
                        return Err("JCS input integer exceeds the safe integer range".to_string());
                    }
                    output.push_str(&(float as i64).to_string());
                }
            }
            Value::String(value) => {
                let serialized = serde_json::to_string(value)
                    .map_err(|error| format!("JCS string could not be serialized: {error}"))?;
                output.push_str(&serialized);
            }
            Value::Array(entries) => {
                output.push('[');
                let mut entries = entries.iter();
                if let Some(first) = entries.next() {
                    write_value(first, output)?;
                }
                for entry in entries {
                    output.push(',');
                    write_value(entry, output)?;
                }
                output.push(']');
            }
            Value::Object(object) => {
                let mut entries: Vec<_> = object.iter().collect();
                entries
                    .sort_by(|(left, _), (right, _)| left.encode_utf16().cmp(right.encode_utf16()));
                output.push('{');
                let mut entries = entries.into_iter();
                if let Some((key, entry)) = entries.next() {
                    write_member(key, entry, output)?;
                }
                for (key, entry) in entries {
                    output.push(',');
                    write_member(key, entry, output)?;
                }
                output.push('}');
            }
        }
        Ok(())
    }

    fn write_member(key: &str, entry: &Value, output: &mut String) -> Result<(), String> {
        let serialized_key = serde_json::to_string(key)
            .map_err(|error| format!("JCS object key could not be serialized: {error}"))?;
        output.push_str(&serialized_key);
        output.push(':');
        write_value(entry, output)
    }

    let mut output = String::new();
    write_value(value, &mut output)?;
    Ok(output.into_bytes())
}

pub(crate) fn canonical_json_sha256(value: &Value) -> Result<String, String> {
    Ok(sha256_bytes(&canonical_json_bytes(value)?))
}
