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

/// Render with whichever interpreter on this host has Pillow, exactly as the
/// former script did before re-execing itself.
pub(crate) fn pillow_python() -> Result<PathBuf> {
    static FOUND: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
        for cand in [
            "/usr/bin/python3",
            "/opt/homebrew/bin/python3",
            "/usr/local/bin/python3",
        ] {
            if !Path::new(cand).exists() {
                continue;
            }
            let probe = Command::new(cand).arg("-c").arg("import PIL").output();
            if probe.is_ok_and(|o| o.status.success()) {
                return Some(PathBuf::from(cand));
            }
        }
        None
    });
    FOUND
        .clone()
        .ok_or_else(|| anyhow!("Pillow is required to render the state PNGs and was not found."))
}

pub(crate) const RENDER_SCRIPT: &str = r#"
import json, os, re, sys
from PIL import Image, ImageDraw, ImageFont

COLS = 100
ROWS = 32
FONT_CANDIDATES = ("/System/Library/Fonts/Menlo.ttc", "/System/Library/Fonts/SFNSMono.ttf")
FONT_PX = 15
ANSI = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]")

events = json.load(open(sys.argv[1]))
path = sys.argv[2]

def strip_ansi(text):
    return ANSI.sub("", text).replace("\x07", "")

def visible_lines(text):
    out = []
    for line in strip_ansi(text).split("\n"):
        if line.endswith("\r"):
            line = line[:-1]
        if "\r" in line:
            line = line.split("\r")[-1]
        out.append(line.replace("\t", "    "))
    return out

def wrapped(lines, width):
    out = []
    for line in lines:
        if not line:
            out.append("")
            continue
        while len(line) > width:
            out.append(line[:width])
            line = line[width:]
        out.append(line)
    return out

text = "".join(e[2] for e in events)
rows = wrapped(visible_lines(text), COLS)
rows = rows[-ROWS:]
while len(rows) < ROWS:
    rows.append("")
font = None
for candidate in FONT_CANDIDATES:
    if os.path.exists(candidate):
        try:
            font = ImageFont.truetype(candidate, FONT_PX)
            break
        except OSError:
            continue
if font is None:
    font = ImageFont.load_default()
advance = font.getlength("M") if hasattr(font, "getlength") else FONT_PX * 0.6
cell_w = max(1, int(round(advance)))
cell_h = int(round(FONT_PX * 1.45))
pad = 12
size = (COLS * cell_w + 2 * pad, ROWS * cell_h + 2 * pad)
image = Image.new("RGB", size, (13, 17, 23))
draw = ImageDraw.Draw(image)
for index, row in enumerate(rows):
    draw.text((pad, pad + index * cell_h), row, font=font, fill=(222, 228, 234))
image.save(str(path), format="PNG", optimize=True)
print(image.size[0], image.size[1])
"#;

/// Deterministic PNG of the cast's own text, replayed to one event. Returns
/// (width, height) of the written image.
pub(crate) fn render_state(path: &Path, events: &[(f64, String)], cutoff_index: usize) -> Result<(u64, u64)> {
    let interpreter = pillow_python()?;
    std::fs::create_dir_all(scratch_root())?;
    let tmp = scratch_root().join(".render-events.json");
    let sliced: Vec<Value> = events
        .iter()
        .take(cutoff_index + 1)
        .map(|(t, text)| json!([t, "o", text]))
        .collect();
    std::fs::write(&tmp, serde_json::to_vec(&sliced)?)?;
    let output = Command::new(&interpreter)
        .args(["-c", RENDER_SCRIPT])
        .arg(&tmp)
        .arg(path)
        .output()
        .with_context(|| format!("run {}", interpreter.display()))?;
    let _ = std::fs::remove_file(&tmp);
    if !output.status.success() {
        bail!(
            "state render failed ({}, {})",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let dims = String::from_utf8_lossy(&output.stdout);
    let mut it = dims.split_whitespace();
    let w = it.next().and_then(|v| v.parse().ok()).unwrap_or(0u64);
    let h = it.next().and_then(|v| v.parse().ok()).unwrap_or(0u64);
    Ok((w, h))
}

pub(crate) fn write_cast(path: &Path, events: &[(f64, String)], title: &str, wall_start: u64) -> Result<()> {
    let header = format!(
        "{{\"version\":2,\"width\":{COLS},\"height\":{ROWS},\"timestamp\":{wall_start},\
         \"env\":{{\"SHELL\":\"{SHELL}\",\"TERM\":\"xterm-256color\"}},\"title\":{}}}\n",
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
                "Deterministic Pillow render of media/session.cast replayed to the end of the \
                 '{label}' step (event {index}, t={} s): the cast's own ANSI-stripped text, wrapped \
                 at {COLS} columns, last {ROWS} rows, Menlo {FONT_PX}px. It is a render of the cast \
                 at that named point, not a separate capture, and re-rendering the same cast \
                 produces the same bytes.",
                g(ts)
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
    PRODUCTS.iter().find(|p| p.name == name)
}
