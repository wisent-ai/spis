use super::*;

// ---------------------------------------------------------------------------
// Catalog loading
// ---------------------------------------------------------------------------

pub(crate) fn load_full_references(slug: &str, examples: &[Value]) -> Result<Value> {
    let catalog_dir = PathBuf::from(slug);
    let index_path = catalog_dir.join("references.json");
    let index_path_str = index_path.to_string_lossy().to_string();
    let mut index: Value = lib::read_json(index_path.to_str().unwrap())?;

    require_nonempty(
        &index,
        &["schema", "catalog", "reference_count", "references"],
        &index_path_str,
    )?;
    if index.get("schema").and_then(Value::as_str) != Some(INDEX_SCHEMA) {
        bail!(
            "{index_path_str}: expected schema '{INDEX_SCHEMA}', found {}",
            python_repr(index.get("schema"))
        );
    }
    if index.get("catalog").and_then(Value::as_str) != Some(slug) {
        bail!("{index_path_str}: catalog must equal directory name");
    }

    let records: Vec<Value> = index
        .get("references")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if records.len() != examples.len() {
        bail!(
            "{index_path_str}: {} references for {} curated examples",
            records.len(),
            examples.len()
        );
    }
    if index.get("reference_count").and_then(Value::as_u64) != Some(records.len() as u64) {
        bail!("{index_path_str}: reference_count does not match the reference list");
    }

    let mut provenance: BTreeMap<String, usize> = BTreeMap::new();
    let mut statuses: BTreeMap<String, usize> = BTreeMap::new();
    let mut gap_total = 0usize;

    for (position, (entry, example)) in records.iter().zip(examples.iter()).enumerate() {
        let position = position + 1;
        require_nonempty(
            entry,
            &["index", "name", "path", "evidence_status"],
            &index_path_str,
        )?;
        if entry.get("index").and_then(Value::as_u64) != Some(position as u64)
            || entry.get("name") != example.get("name")
        {
            bail!("{index_path_str}: reference {position} does not match sources.json");
        }
        let entry_status = entry
            .get("evidence_status")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !EVIDENCE_STATUSES.contains(&entry_status) {
            bail!("{index_path_str}: reference {position} has status '{entry_status}'");
        }

        let entry_path = entry.get("path").and_then(Value::as_str).unwrap_or("");
        let record_path = resolve_evidence_path(&catalog_dir, entry_path, &index_path_str)?;
        let record_path_str = record_path.to_string_lossy().to_string();
        let record: Value = lib::read_json(record_path.to_str().unwrap())?;
        if record.get("schema").and_then(Value::as_str) != Some(RECORD_SCHEMA) {
            bail!("{record_path_str}: expected schema '{RECORD_SCHEMA}'");
        }
        if record.get("name") != example.get("name")
            || record.get("product_url") != example.get("source_url")
        {
            bail!("{record_path_str}: product identity differs from sources.json");
        }

        let Some(gaps) = record.get("evidence_gaps").and_then(Value::as_array) else {
            bail!("{record_path_str}: evidence_gaps must be a list, empty when nothing is missing");
        };
        let expected = if gaps.is_empty() {
            "complete"
        } else {
            "partial"
        };
        if evidence_status_of(&record) != Some(expected) {
            bail!(
                "{record_path_str}: status {} contradicts {} recorded gaps",
                python_repr(record.get("evidence_status")),
                gaps.len()
            );
        }
        if entry_status != expected
            || entry.get("evidence_gap_count").and_then(Value::as_u64) != Some(gaps.len() as u64)
        {
            bail!("{index_path_str}: reference {position} disagrees with its record");
        }

        let reference_dir = record_path.parent().unwrap_or(Path::new("."));
        let verified_provenance = VerifiedProvenanceSet::verify_record(&record, reference_dir);
        for class in validate_motion(
            &record,
            &record_path_str,
            reference_dir,
            &verified_provenance,
        )? {
            *provenance.entry(class).or_insert(0) += 1;
        }
        validate_states(
            &record,
            &record_path_str,
            reference_dir,
            &verified_provenance,
        )?;
        validate_behaviour(
            &record,
            &record_path_str,
            reference_dir,
            &verified_provenance,
        )?;

        *statuses.entry(expected.to_string()).or_insert(0) += 1;
        gap_total += gaps.len();
    }

    let obj = index.as_object_mut().expect("index is an object");
    obj.insert(
        "measured_provenance".into(),
        provenance
            .iter()
            .map(|(k, v)| (k.clone(), json!(v)))
            .collect::<Map<String, Value>>()
            .into(),
    );
    obj.insert("measured_gap_total".into(), json!(gap_total));
    obj.insert(
        "complete_count".into(),
        json!(statuses.get("complete").copied().unwrap_or(0)),
    );
    obj.insert(
        "partial_count".into(),
        json!(statuses.get("partial").copied().unwrap_or(0)),
    );
    let status_count = statuses.values().sum::<usize>();
    if status_count != records.len() {
        bail!(
            "{index_path_str}: complete and partial counts cover {status_count} of {} references",
            records.len()
        );
    }
    obj.insert(
        "locally_driven_count".into(),
        json!(provenance
            .iter()
            .filter(|(name, _)| LOCAL_PROVENANCE.contains(&name.as_str()))
            .map(|(_, count)| count)
            .sum::<usize>()),
    );
    Ok(index)
}

/// Missing-key list formatted the way Python formats lists of strings.
pub(crate) fn missing_list(fields: &[&str]) -> String {
    let quoted: Vec<String> = fields.iter().map(|f| format!("'{f}'")).collect();
    format!("[{}]", quoted.join(", "))
}
