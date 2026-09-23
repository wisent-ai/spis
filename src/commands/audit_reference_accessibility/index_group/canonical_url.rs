use super::*;

pub(crate) fn canonical_url(value: &Value, record_id: &str, field: &str) -> Result<String> {
    let raw = value
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow!("{record_id}: {field}: expected a non-empty http(s) URL"))?;
    let (scheme, rest) = raw
        .split_once("://")
        .ok_or_else(|| anyhow!("{record_id}: {field}: expected an http(s) URL, got {raw:?}"))?;
    if scheme != "http" && scheme != "https" {
        bail!("{record_id}: {field}: expected an http(s) URL, got {raw:?}");
    }
    let netloc_end = rest
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(rest.len());
    let netloc = &rest[..netloc_end];
    if netloc.is_empty() {
        bail!("{record_id}: {field}: expected an http(s) URL, got {raw:?}");
    }
    let tail = &rest[netloc_end..];
    let (before_fragment, fragment) = match tail.split_once('#') {
        Some((head, frag)) => (head, Some(frag)),
        None => (tail, None),
    };
    let (path, query) = match before_fragment.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (before_fragment, None),
    };
    let path = if path.is_empty() { "/" } else { path };
    let mut out = format!("{scheme}://{netloc}{path}");
    if let Some(query) = query {
        out.push('?');
        out.push_str(query);
    }
    if let Some(fragment) = fragment {
        out.push('#');
        out.push_str(fragment);
    }
    Ok(out)
}

pub(crate) fn parse_record_selection(raw: Option<&String>) -> Result<Option<HashSet<usize>>> {
    let Some(raw) = raw else { return Ok(None) };
    let mut selected: HashSet<usize> = HashSet::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            bail!("--records: empty item in the comma-separated selection");
        }
        let invalid = || anyhow!("--records: {token:?} is not a positive record number or range");
        let (first_s, last_s) = match token.split_once('-') {
            Some((a, b)) => (a, Some(b)),
            None => (token, None),
        };
        let parse_num = |value: &str| -> Result<usize> {
            if value.is_empty()
                || !value.bytes().all(|b| b.is_ascii_digit())
                || value.starts_with('0')
            {
                return Err(invalid());
            }
            value.parse::<usize>().map_err(|_| invalid())
        };
        let first = parse_num(first_s)?;
        let last = match last_s {
            Some(value) => parse_num(value)?,
            None => first,
        };
        if last < first {
            bail!("--records: descending range {token:?} is not allowed");
        }
        for number in first..=last {
            selected.insert(number);
        }
    }
    Ok(Some(selected))
}

pub(crate) fn normalize_catalog(value: &str) -> Result<String> {
    let mut catalog = value.trim().to_string();
    if catalog.is_empty() {
        bail!("--catalog: catalog name must not be empty");
    }
    if !catalog.ends_with("-examples") {
        catalog.push_str("-examples");
    }
    // fullmatch r"[a-z0-9][a-z0-9.-]*-examples"
    let stem = catalog.strip_suffix("-examples").unwrap_or(&catalog);
    let valid = !stem.is_empty()
        && stem
            .chars()
            .enumerate()
            .all(|(i, c)| c.is_ascii_lowercase() || c.is_ascii_digit() || (i > 0 && c == '-'))
        && stem.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit());
    if !valid {
        bail!("--catalog: invalid catalog name {value:?}");
    }
    Ok(catalog)
}

// ---------------------------------------------------------------------------
// References

#[derive(Clone)]
pub(crate) struct Reference {
    pub(crate) catalog: String,
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) path: PathBuf,
    pub(crate) source_url: String,
}

impl Reference {
    pub(crate) fn id(&self) -> String {
        format!("{}/{}", self.catalog, self.slug)
    }

    pub(crate) fn action(&self, batch: &str) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("batch".into(), Value::String(batch.to_string()));
        map.insert("site_slug".into(), Value::String(self.slug.clone()));
        map.insert("source_url".into(), Value::String(self.source_url.clone()));
        map.insert(
            "viewport".into(),
            serde_json::json!({"width": 1440, "height": 1000, "device_scale_factor": 1}),
        );
        map.insert(
            "artifact_prefix".into(),
            Value::String(format!(
                "{NAMESPACE}{batch}/{}/{}/accessibility/",
                self.catalog, self.slug
            )),
        );
        map
    }
}

pub(crate) fn load_references(
    catalogs: &[String],
    selection: Option<&HashSet<usize>>,
) -> Result<Vec<Reference>> {
    let mut references: Vec<Reference> = Vec::new();
    for catalog in catalogs {
        let catalog_dir = PathBuf::from(catalog);
        let catalog_path = catalog_dir.join("references.json");
        if !catalog_path.is_file() {
            bail!("{catalog}: references.json: catalog does not exist");
        }
        let text = std::fs::read_to_string(&catalog_path)
            .with_context(|| format!("{catalog}: references.json: unreadable"))?;
        let payload = strict_json(&text, &format!("{catalog}: references.json"))?;
        let pointers = payload
            .get("references")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("{catalog}: references.json: references must be an array"))?;
        let mut known: HashSet<usize> = HashSet::new();
        for (position, pointer) in pointers.iter().enumerate() {
            let position = position + 1;
            let pointer_id = format!("{catalog}/record-{position}");
            let pointer = pointer
                .as_object()
                .ok_or_else(|| anyhow!("{pointer_id}: catalog entry: expected an object"))?;
            let index = pointer
                .get("index")
                .and_then(Value::as_u64)
                .filter(|v| (1..=usize::MAX as u64).contains(v))
                .ok_or_else(|| anyhow!("{pointer_id}: index: expected a positive integer"))?
                as usize;
            if known.contains(&index) {
                bail!("{catalog}/{index}: index: duplicate catalog record");
            }
            known.insert(index);
            if let Some(selection) = selection {
                if !selection.contains(&index) {
                    continue;
                }
            }
            let relative = pointer
                .get("path")
                .and_then(Value::as_str)
                .filter(|p| !p.is_empty())
                .ok_or_else(|| anyhow!("{catalog}/{index:02}: path: expected a reference path"))?;
            let record_id = format!("{catalog}/{index:02}");
            let joined = catalog_dir.join(relative);
            let resolved = lex_norm(&joined);
            let catalog_resolved = lex_norm(&catalog_dir);
            if !is_relative_to(&resolved, &catalog_resolved)
                || resolved.file_name().and_then(|n| n.to_str()) != Some("reference.json")
            {
                bail!("{record_id}: path: {relative:?} is outside the catalog reference layout");
            }
            if !resolved.is_file() {
                bail!("{record_id}: path: {relative:?} does not exist");
            }
            let document_text = std::fs::read_to_string(&resolved)
                .with_context(|| format!("{record_id}: reference.json: unreadable"))?;
            let document = strict_json(&document_text, &format!("{record_id}: reference.json"))?;
            if !document.is_object() {
                bail!("{record_id}: reference.json: expected an object");
            }
            let slug = resolved
                .parent()
                .and_then(Path::file_name)
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            if !looks_like_slug(&slug) {
                bail!("{record_id}: site_slug: directory name {slug:?} is not a Weles slug");
            }
            let name = document
                .get("name")
                .and_then(Value::as_str)
                .filter(|n| !n.is_empty())
                .ok_or_else(|| anyhow!("{record_id}: name: expected a non-empty string"))?
                .to_string();
            let source_field = if document.get("product_url").map(Value::is_null) == Some(false) {
                "product_url"
            } else {
                "source_url"
            };
            let source_url = canonical_url(
                document.get(source_field).unwrap_or(&Value::Null),
                &record_id,
                source_field,
            )?;
            references.push(Reference {
                catalog: catalog.clone(),
                index,
                name,
                slug,
                path: resolved,
                source_url,
            });
        }
        if let Some(selection) = selection {
            let mut missing: Vec<usize> = selection.difference(&known).copied().collect();
            missing.sort_unstable();
            if !missing.is_empty() {
                let joined = missing
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                bail!("{catalog}: --records: record(s) {joined} do not exist");
            }
        }
    }
    Ok(references)
}
