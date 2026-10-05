use super::*;

use fontdue::{Font, FontSettings};
use image::{ImageFormat, Rgb, RgbImage};

/// Where macOS keeps the monospace faces a terminal frame is drawn in, tried
/// in order: Menlo, then SF Mono.
const FONT_CANDIDATES: [&str; 2] = ["/System/Library/Fonts/Menlo.ttc", "/System/Library/Fonts/SFNSMono.ttf"];

/// The monospace face the state PNGs are drawn in, refused before any capture
/// starts when none can be read.
pub(crate) fn font(px: f32) -> Result<Font> {
    for candidate in FONT_CANDIDATES {
        let Ok(bytes) = std::fs::read(candidate) else { continue };
        if let Ok(font) = Font::from_bytes(bytes, FontSettings { scale: px, ..FontSettings::default() }) {
            return Ok(font);
        }
    }
    bail!(
        "no monospace font to render the state PNGs: none of {} could be read as a font",
        FONT_CANDIDATES.join(", ")
    )
}

fn colour(hex: &str) -> Result<Rgb<u8>> {
    let digits = hex.strip_prefix('#').unwrap_or(hex);
    let channel = |range: std::ops::Range<usize>| {
        digits.get(range).and_then(|pair| u8::from_str_radix(pair, 16).ok())
    };
    match (channel(0..2), channel(2..4), channel(4..6), digits.len()) {
        (Some(r), Some(g), Some(b), 6) => Ok(Rgb([r, g, b])),
        _ => bail!("{hex:?} is not a #rrggbb colour"),
    }
}

/// The cast's text as the terminal showed it: control sequences removed, each
/// carriage return keeping only what was written after it, tabs as four
/// spaces.
fn visible_lines(text: &str) -> Vec<String> {
    static ANSI: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]")
            .expect("the control-sequence pattern compiles")
    });
    let stripped = ANSI.replace_all(text, "").replace('\x07', "");
    stripped
        .split('\n')
        .map(|line| {
            let line = line.strip_suffix('\r').unwrap_or(line);
            let line = line.rsplit('\r').next().unwrap_or(line);
            line.replace('\t', "    ")
        })
        .collect()
}

/// Lines wrapped at `width` characters, an empty line kept as one row.
fn wrapped(lines: &[String], width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for line in lines {
        let characters: Vec<char> = line.chars().collect();
        if characters.is_empty() {
            rows.push(String::new());
            continue;
        }
        for chunk in characters.chunks(width) {
            rows.push(chunk.iter().collect());
        }
    }
    rows
}

/// Deterministic PNG of the cast's own text, replayed to one event: the last
/// rows of the plan's terminal, each cell the font's own advance and line
/// height, no margin. Returns (width, height) of the written image.
pub(crate) fn render_state(path: &Path, events: &[(f64, String)], cutoff_index: usize) -> Result<(u64, u64)> {
    let terminal = terminal();
    let text: String = events.iter().take(cutoff_index + 1).map(|(_, chunk)| chunk.as_str()).collect();
    let mut rows = wrapped(&visible_lines(&text), terminal.columns);
    if rows.len() > terminal.rows {
        rows.drain(..rows.len() - terminal.rows);
    }
    rows.resize(terminal.rows, String::new());

    let px = terminal.font_px as f32;
    let font = font(px)?;
    let cell_w = font.metrics('M', px).advance_width.round().max(1.0) as u32;
    let lines = font
        .horizontal_line_metrics(px)
        .context("the font states no horizontal line metrics")?;
    let ascent = lines.ascent.round() as i64;
    let cell_h = (lines.ascent.round() - lines.descent.round()) as u32;
    let width = terminal.columns as u32 * cell_w;
    let height = terminal.rows as u32 * cell_h;
    let background = colour(&terminal.background)?;
    let foreground = colour(&terminal.foreground)?;
    let mut image = RgbImage::from_pixel(width, height, background);

    for (index, row) in rows.iter().enumerate() {
        let baseline = index as i64 * i64::from(cell_h) + ascent;
        let mut pen = 0.0f32;
        for character in row.chars() {
            let (metrics, coverage) = font.rasterize(character, px);
            let left = (pen + metrics.xmin as f32).round() as i64;
            let top = baseline - (metrics.height as i64 + i64::from(metrics.ymin));
            for (offset, alpha) in coverage.iter().enumerate() {
                let x = left + (offset % metrics.width.max(1)) as i64;
                let y = top + (offset / metrics.width.max(1)) as i64;
                if *alpha == 0 || x < 0 || y < 0 || x >= i64::from(width) || y >= i64::from(height) {
                    continue;
                }
                let pixel = image.get_pixel_mut(x as u32, y as u32);
                let a = u16::from(*alpha);
                for channel in 0..3 {
                    let under = u16::from(pixel.0[channel]);
                    let over = u16::from(foreground.0[channel]);
                    pixel.0[channel] = ((over * a + under * (255 - a) + 127) / 255) as u8;
                }
            }
            pen += metrics.advance_width;
        }
    }
    image
        .save_with_format(path, ImageFormat::Png)
        .with_context(|| format!("write {}", path.display()))?;
    Ok((u64::from(width), u64::from(height)))
}
