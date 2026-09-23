use super::*;

pub(crate) fn bridge_error_code(stderr: &[u8]) -> String {
    let Ok(value) = serde_json::from_slice::<Value>(stderr) else {
        return "bridge-error".to_string();
    };
    value
        .get("code")
        .and_then(Value::as_str)
        .filter(|code| {
            !code.is_empty()
                && code.len() <= 64
                && code
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
        .unwrap_or("bridge-error")
        .to_string()
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
    let canonical_base = fs::canonicalize(base)
        .map_err(|_| "record directory could not be resolved".to_string())?;
    let joined = canonical_base.join(relative_path);
    let link_metadata = fs::symlink_metadata(&joined)
        .map_err(|_| "retained file is absent".to_string())?;
    if link_metadata.file_type().is_symlink() || !link_metadata.is_file() {
        return Err("retained path is not a regular non-symlink file".to_string());
    }
    let canonical_file = fs::canonicalize(&joined)
        .map_err(|_| "retained file could not be resolved".to_string())?;
    if !canonical_file.starts_with(&canonical_base) {
        return Err("retained file resolves outside the record directory".to_string());
    }
    Ok(canonical_file)
}

pub(crate) fn read_stream_limited(
    reader: impl Read,
    limit: usize,
    label: &str,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| format!("{label} could not be read"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeded the size limit"));
    }
    Ok(bytes)
}

pub(crate) fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|_| "retained file could not be opened".to_string())?;
    read_stream_limited(file, limit as usize, "retained JSON document")
}

pub(crate) fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|_| "retained artifact could not be opened".to_string())?;
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut DigestWriter(&mut hash))
        .map_err(|_| "retained artifact could not be hashed".to_string())?;
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

pub(crate) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

pub(crate) fn is_git_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

pub(crate) fn is_sha256_id(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(is_sha256)
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
        .map_err(|_| "receipt spisBinding could not be canonicalized".to_string())?;
    let binding_json = String::from_utf8(canonical_json_bytes(&binding_value)?)
        .map_err(|_| "canonical receipt spisBinding was not UTF-8".to_string())?;
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
                    let float = number
                        .as_f64()
                        .ok_or_else(|| "JCS input contains an unrepresentable number".to_string())?;
                    if !float.is_finite() || float.fract() != 0.0 {
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
                    .map_err(|_| "JCS string could not be serialized".to_string())?;
                output.push_str(&serialized);
            }
            Value::Array(entries) => {
                output.push('[');
                for (index, entry) in entries.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    write_value(entry, output)?;
                }
                output.push(']');
            }
            Value::Object(object) => {
                let mut entries: Vec<_> = object.iter().collect();
                entries.sort_by(|(left, _), (right, _)| {
                    left.encode_utf16().cmp(right.encode_utf16())
                });
                output.push('{');
                for (index, (key, entry)) in entries.into_iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    let serialized_key = serde_json::to_string(key)
                        .map_err(|_| "JCS object key could not be serialized".to_string())?;
                    output.push_str(&serialized_key);
                    output.push(':');
                    write_value(entry, output)?;
                }
                output.push('}');
            }
        }
        Ok(())
    }

    let mut output = String::new();
    write_value(value, &mut output)?;
    Ok(output.into_bytes())
}

pub(crate) fn canonical_json_sha256(value: &Value) -> Result<String, String> {
    Ok(sha256_bytes(&canonical_json_bytes(value)?))
}
