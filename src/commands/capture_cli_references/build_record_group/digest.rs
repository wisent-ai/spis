use super::*;

// -------------------------------------------------------------- media output

pub(crate) fn digest(path: &Path) -> Result<(u64, String)> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    let mut buf = vec![0u8; 1 << 20];
    loop {
        use std::io::Read;
        let n = file
            .read(&mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        if n == 0 {
            break;
        }
        size += n as u64;
        hasher.update(&buf[..n]);
    }
    Ok((size, format!("{:x}", hasher.finalize())))
}

pub(crate) fn write_cast(
    path: &Path,
    events: &[(f64, String)],
    title: &str,
    wall_start: u64,
) -> Result<()> {
    let header = format!(
        "{{\"version\":2,\"width\":{},\"height\":{},\"timestamp\":{wall_start},\
         \"env\":{{\"SHELL\":\"{SHELL}\",\"TERM\":\"xterm-256color\"}},\"title\":{}}}\n",
        terminal().columns,
        terminal().rows,
        json_str(title),
    );
    let mut out = String::from(&header);
    for (stamp, text) in events {
        out.push_str(&format!(
            "[{}, \"o\", {}]\n",
            g(round_n(*stamp, 6)),
            json_str(text)
        ));
    }
    std::fs::write(path, out).with_context(|| format!("write {}", path.display()))
}

pub(crate) fn write_media(run: &Run, ref_dir: &Path) -> Result<Value> {
    let media_dir = ref_dir.join("media");
    std::fs::create_dir_all(&media_dir)?;
    for entry in std::fs::read_dir(&media_dir)?.flatten() {
        if entry.path().is_file() {
            std::fs::remove_file(entry.path())?;
        }
    }

    let events = &run.events;
    let cast_path = media_dir.join("session.cast");
    let title = format!(
        "{} — real local first-look on {}",
        run.product.name,
        host_sentence()
    );
    write_cast(&cast_path, events, &title, run.wall_start)?;
    let (size, sha) = digest(&cast_path)?;
    let duration = round_n(events.iter().map(|(t, _)| *t).fold(0.0f64, f64::max), 3);

    let mut states = Vec::new();
    for (kind, filename, label) in STATE_PLAN {
        let step = &run.steps[*kind];
        let index = step.event_index;
        let path = media_dir.join(format!("{filename}.png"));
        let (width, height) = render_state(&path, events, index)?;
        let (st_size, st_sha) = digest(&path)?;
        let ts = events.get(index).map(|(t, _)| *t).unwrap_or(duration);
        states.push(json!({
            "label": format!("{}: {label}", run.product.name),
            "state_name": filename.split_once('-').map(|(_, rest)| rest).unwrap_or(filename),
            "local_path": format!("media/{filename}.png"),
            "event_index": index,
            "timestamp_seconds": ts,
            "width": width,
            "height": height,
            "bytes": st_size,
            "sha256": st_sha,
            "source_relationship": format!(
                "Deterministic render of media/session.cast replayed to the end of the \
                 '{label}' step (event {index}, t={} s): the cast's own ANSI-stripped text, wrapped \
                 at {} columns, last {} rows, Menlo {}px. It is a render of the cast \
                 at that named point, not a separate capture, and re-rendering the same cast \
                 produces the same bytes.",
                g(ts),
                terminal().columns,
                terminal().rows,
                terminal().font_px
            ),
        }));
    }

    Ok(json!({
        "cast": {
            "bytes": size,
            "sha256": sha,
            "duration_seconds": duration,
            "frame_count": events.len(),
        },
        "states": states,
        "captured_at": captured_at_now(),
    }))
}

pub(crate) fn write_reference(run: &Run, measured: &Value) -> Result<(PathBuf, Value)> {
    let ref_dir = catalog_dir()
        .join("references")
        .join(format!("{:02}-{}", run.index, run.product.slug));
    std::fs::create_dir_all(&ref_dir)?;
    let media = write_media(run, &ref_dir)?;
    let record = build_record(run, measured, &media);
    std::fs::write(
        ref_dir.join("reference.json"),
        serde_json::to_string_pretty(&record)? + "\n",
    )?;
    Ok((ref_dir, record))
}

// ------------------------------------------------------------ catalog files

pub(crate) fn load_records() -> Result<Vec<(PathBuf, Value)>> {
    let mut out = Vec::new();
    let refs_dir = catalog_dir().join("references");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&refs_dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("reference.json").is_file())
        .collect();
    dirs.sort();
    for dir in dirs {
        let path = dir.join("reference.json");
        let text = std::fs::read_to_string(&path)?;
        let value =
            serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        out.push((path, value));
    }
    Ok(out)
}

pub(crate) fn product_by_name(name: &str) -> Option<&'static Product> {
    products().iter().find(|p| p.name == name)
}
