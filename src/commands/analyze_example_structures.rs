//! `spis analyze-example-structures` — measure panel geometry and record
//! interface anatomy for example images.
//!
//! For every example in a catalog's sources.json this decodes the overview
//! image, detects dominant separators on a grayscale downscale, classifies a
//! layout model with semantic hints from the example metadata, and stores an
//! `interface_structure` record back into sources.json. Failures land in
//! structure-analysis-failures.json. Rust port of the former
//! `analyze-example-structures.py`; the separator statistics mirror the numpy
//! pipeline (zero-padded moving average, linear-interpolated percentiles,
//! population standard deviation, banker's rounding).
//!
//! Requires the `image` crate (reported to the integrator for Cargo.toml).

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::Path;

const CATALOGS: &[&str] = &[
    "ios-app-examples",
    "android-app-examples",
    "macos-app-examples",
    "desktop-app-examples",
    "web-app-examples",
    "dashboard-console-examples",
    "tui-examples",
    "cli-examples",
    "onboarding-auth-examples",
    "documentation-site-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
    "landing-page-examples",
    "wisent-product-examples",
];

/// Python's round(): halves go to the nearest even value.
fn py_round(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits);
    let scaled = value * factor;
    let floor = scaled.floor();
    let fract = scaled - floor;
    let rounded = if (fract - 0.5).abs() < 1e-9 {
        if floor % 2.0 == 0.0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        scaled.round()
    };
    rounded / factor
}

struct Gray {
    data: Vec<f32>,
    width: usize,
    height: usize,
}

impl Gray {
    fn at(&self, y: usize, x: usize) -> f32 {
        self.data[y * self.width + x]
    }
}

/// np.convolve(values, kernel, mode="same") with a uniform kernel: zero-padded
/// at both ends, each output divided by the full kernel width.
fn moving_average(values: &[f32], radius: usize) -> Vec<f32> {
    let width = radius * 2 + 1;
    if values.len() < width {
        return values.to_vec();
    }
    values
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let mut sum = 0.0f32;
            for k in 0..width {
                let j = i + k;
                if j >= radius && j - radius < values.len() {
                    sum += values[j - radius];
                }
            }
            sum / width as f32
        })
        .collect()
}

/// numpy.percentile(values, q) with linear interpolation.
fn percentile_linear(mut values: Vec<f32>, q: f64) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    let position = (n - 1) as f64 * q;
    let lower = position.floor() as usize;
    let upper = std::cmp::min(lower + 1, n - 1);
    values[lower] as f64 + (values[upper] - values[lower]) as f64 * (position - lower as f64)
}

fn mean(values: &[f32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f32>() as f64 / values.len() as f64
}

/// Population standard deviation (numpy default ddof=0).
fn stddev(values: &[f32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mu = mean(values);
    let var = values.iter().map(|v| (*v as f64 - mu).powi(2)).sum::<f64>() / values.len() as f64;
    var.sqrt()
}

fn separator_positions(gray: &Gray, vertical: bool) -> Vec<Value> {
    // Per-coordinate mean absolute neighbor difference along the scan axis.
    let differences: Vec<f32> = if vertical {
        let mut diffs = Vec::with_capacity((gray.width - 1).max(0));
        for x in 0..gray.width.saturating_sub(1) {
            let s: f32 = (0..gray.height)
                .map(|y| (gray.at(y, x + 1) - gray.at(y, x)).abs())
                .sum();
            diffs.push(s / gray.height as f32);
        }
        diffs
    } else {
        let mut diffs = Vec::with_capacity((gray.height - 1).max(0));
        for y in 0..gray.height.saturating_sub(1) {
            let s: f32 = (0..gray.width)
                .map(|x| (gray.at(y + 1, x) - gray.at(y, x)).abs())
                .sum();
            diffs.push(s / gray.width as f32);
        }
        diffs
    };
    let extent = if vertical { gray.width } else { gray.height };

    let smoothed = moving_average(&differences, std::cmp::max(1, extent / 350));
    let low = ((extent as f64) * 0.06) as usize;
    let high = ((extent as f64) * 0.94) as usize;
    if low >= high || high > smoothed.len() {
        return Vec::new();
    }
    let segment = &smoothed[low..high];
    if segment.is_empty() {
        return Vec::new();
    }
    let threshold =
        percentile_linear(segment.to_vec(), 0.91).max(mean(segment) + stddev(segment) * 1.15);

    let group_gap = std::cmp::max(2, extent / 180);
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for index in low..high {
        if smoothed[index] < threshold as f32 {
            continue;
        }
        match groups.last_mut() {
            Some(group) if index - group[group.len() - 1] <= group_gap => group.push(index),
            _ => groups.push(vec![index]),
        }
    }

    // Strongest index per group (first maximal one on ties), then take up to 4
    // peaks by descending score, keeping peaks at least 7.5% of the extent apart.
    let mut peaks: Vec<(usize, f64)> = groups
        .iter()
        .map(|group| {
            let best = *group
                .iter()
                .max_by(|a, b| smoothed[**a].partial_cmp(&smoothed[**b]).unwrap())
                .unwrap();
            (best, smoothed[best] as f64)
        })
        .collect();
    peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let min_distance = (extent as f64) * 0.075;
    let mut selected: Vec<(usize, f64)> = Vec::new();
    for (index, score) in peaks {
        if selected
            .iter()
            .all(|(chosen, _)| (index as f64 - *chosen as f64).abs() >= min_distance)
        {
            selected.push((index, score));
        }
        if selected.len() == 4 {
            break;
        }
    }

    let maximum = selected
        .iter()
        .map(|(_, s)| *s)
        .fold(f64::NEG_INFINITY, f64::max);
    if !(maximum > 0.0) {
        return Vec::new();
    }
    selected.sort_by_key(|(index, _)| *index);
    selected
        .into_iter()
        .map(|(index, score)| {
            json!({
                "position": py_round(index as f64 / extent as f64, 3),
                "strength": py_round(score / maximum, 3),
            })
        })
        .collect()
}

fn contains_any(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

struct Hints {
    leading: bool,
    trailing: bool,
    table: bool,
    canvas: bool,
    command: bool,
    mobile: bool,
    request_response: bool,
}

fn semantic_hints(example: &Value, catalog: &str) -> Hints {
    let text = ["name", "category", "selection_note"]
        .iter()
        .map(|key| {
            example
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    Hints {
        leading: contains_any(
            &text,
            &[
                "sidebar",
                "navigation",
                "navigator",
                "channel",
                "workspace",
                "repository",
                "server",
                "folder",
                "object",
                "inbox",
                "project panel",
                "service switcher",
            ],
        ),
        trailing: contains_any(
            &text,
            &[
                "inspector",
                "detail panel",
                "side panel",
                "member list",
                "properties",
                "customer context",
                "context panel",
                "request-response",
                "detail view",
            ],
        ),
        table: contains_any(
            &text,
            &[
                "table",
                "grid",
                "result",
                "list",
                "stream",
                "inventory",
                "queue",
                "timeline",
            ],
        ),
        canvas: contains_any(
            &text,
            &[
                "canvas",
                "editor",
                "document",
                "map",
                "diagram",
                "chart",
                "dashboard",
                "workspace",
            ],
        ),
        command: matches!(catalog, "tui-examples" | "cli-examples"),
        mobile: matches!(
            catalog,
            "ios-app-examples" | "android-app-examples" | "app-store-listing-examples"
        ),
        request_response: contains_any(
            &text,
            &[
                "request-response",
                "request details",
                "response preview",
                "request and response",
            ],
        ),
    }
}

/// Strongest measured separator inside [lower, upper], else the fallback.
fn choose_boundary(
    separators: &[Value],
    lower: f64,
    upper: f64,
    fallback: Option<f64>,
) -> Option<f64> {
    separators
        .iter()
        .filter_map(|item| {
            let position = item.get("position")?.as_f64()?;
            let strength = item.get("strength")?.as_f64()?;
            (lower <= position && position <= upper).then_some((position, strength))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(position, _)| position)
        .or(fallback)
}

fn region(role: &str, position: &str, bounds: (f64, f64, f64, f64), evidence: &str) -> Value {
    json!({
        "role": role,
        "position": position,
        "bounds": {
            "x": py_round(bounds.0, 3),
            "y": py_round(bounds.1, 3),
            "width": py_round(bounds.2, 3),
            "height": py_round(bounds.3, 3),
        },
        "evidence": evidence,
    })
}

type Layout = (String, String, Vec<Value>, &'static str);

fn classify_layout(
    example: &Value,
    catalog: &str,
    vertical: &[Value],
    horizontal: &[Value],
) -> Layout {
    let hints = semantic_hints(example, catalog);
    let top = choose_boundary(
        horizontal,
        0.06,
        0.22,
        if !hints.command { Some(0.12) } else { None },
    );
    let bottom = choose_boundary(horizontal, 0.76, 0.94, None);
    let content_top = top.unwrap_or(0.0);
    let content_bottom = bottom.unwrap_or(1.0);
    let height = content_bottom - content_top;

    let leading = choose_boundary(
        vertical,
        0.14,
        0.42,
        if hints.leading { Some(0.26) } else { None },
    );
    let trailing = choose_boundary(
        vertical,
        0.58,
        0.88,
        if hints.trailing { Some(0.76) } else { None },
    );
    let mut regions: Vec<Value> = Vec::new();
    if let Some(top) = top {
        regions.push(region(
            "toolbar/header",
            "top",
            (0.0, 0.0, 1.0, top),
            "measured horizontal separator",
        ));
    }
    if let Some(bottom) = bottom {
        regions.push(region(
            "status/action bar",
            "bottom",
            (0.0, bottom, 1.0, 1.0 - bottom),
            "measured horizontal separator",
        ));
    }

    if hints.command {
        let split = choose_boundary(vertical, 0.25, 0.75, None);
        let horizontal_split = choose_boundary(horizontal, 0.28, 0.75, None);
        if let Some(split) = split {
            regions.extend([
                region(
                    "primary terminal pane",
                    "leading",
                    (0.0, content_top, split, height),
                    "measured separator in terminal image",
                ),
                region(
                    "secondary terminal pane",
                    "trailing",
                    (split, content_top, 1.0 - split, height),
                    "measured separator in terminal image",
                ),
            ]);
            return (
                "split-terminal".into(),
                "Two side-by-side terminal work areas with shared command context.".into(),
                regions,
                "medium",
            );
        }
        if let Some(horizontal_split) = horizontal_split {
            regions.extend([
                region(
                    "primary terminal pane",
                    "top",
                    (0.0, content_top, 1.0, horizontal_split - content_top),
                    "measured separator in terminal image",
                ),
                region(
                    "secondary terminal pane",
                    "bottom",
                    (
                        0.0,
                        horizontal_split,
                        1.0,
                        content_bottom - horizontal_split,
                    ),
                    "measured separator in terminal image",
                ),
            ]);
            return (
                "stacked-terminal".into(),
                "Two vertically stacked terminal work areas.".into(),
                regions,
                "medium",
            );
        }
        regions.push(region(
            "command and output flow",
            "center",
            (0.0, content_top, 1.0, height),
            "terminal-family catalog",
        ));
        return (
            "command-output".into(),
            "Single command-and-output flow without a stable secondary panel.".into(),
            regions,
            "medium",
        );
    }

    if hints.request_response {
        let navigation_end = leading.unwrap_or(0.0);
        let detail_top = choose_boundary(
            horizontal,
            (0.28f64).max(content_top + 0.12),
            (0.78f64).min(content_bottom - 0.12),
            Some(0.48),
        )
        .unwrap_or(0.48);
        let detail_split = choose_boundary(
            vertical,
            (0.42f64).max(navigation_end + 0.18),
            0.82,
            Some(0.62),
        )
        .unwrap_or(0.62);
        if let Some(leading) = leading {
            regions.push(region(
                "traffic source navigation",
                "leading",
                (0.0, content_top, leading, height),
                "semantic cue plus measured boundary",
            ));
        }
        regions.extend([
            region(
                "traffic request table",
                "upper center",
                (
                    navigation_end,
                    content_top,
                    1.0 - navigation_end,
                    detail_top - content_top,
                ),
                "measured horizontal separator",
            ),
            region(
                "request inspector",
                "lower center",
                (
                    navigation_end,
                    detail_top,
                    detail_split - navigation_end,
                    content_bottom - detail_top,
                ),
                "request-response semantic cue plus measured separators",
            ),
            region(
                "response inspector",
                "lower trailing",
                (
                    detail_split,
                    detail_top,
                    1.0 - detail_split,
                    content_bottom - detail_top,
                ),
                "request-response semantic cue plus measured separators",
            ),
        ]);
        return (
            "sidebar-table-request-response".into(),
            "Traffic-source navigation beside a request table, with paired request and response inspectors below.".into(),
            regions,
            "high",
        );
    }

    if hints.mobile {
        regions.push(region(
            "scrolling content or app screen",
            "center",
            (0.0, content_top, 1.0, height),
            "mobile-family catalog",
        ));
        if bottom.is_some() {
            for item in regions.iter_mut() {
                if item.get("role").and_then(Value::as_str) == Some("status/action bar") {
                    if let Some(obj) = item.as_object_mut() {
                        obj.insert("role".into(), json!("bottom navigation/action area"));
                    }
                }
            }
        }
        return (
            "mobile-single-column".into(),
            "Single-column mobile hierarchy with top context and a vertically scrolling primary surface.".into(),
            regions,
            "medium",
        );
    }

    let mut start = 0.0;
    let mut end = 1.0;
    if let Some(leading) = leading {
        regions.push(region(
            "navigation/list sidebar",
            "leading",
            (0.0, content_top, leading, height),
            "semantic cue plus measured or conventional boundary",
        ));
        start = leading;
    }
    if let Some(trailing) = trailing {
        if trailing > start + 0.18 {
            regions.push(region(
                "detail/inspector",
                "trailing",
                (trailing, content_top, 1.0 - trailing, height),
                "semantic cue plus measured or conventional boundary",
            ));
            end = trailing;
        }
    }
    let primary_role = if hints.table {
        "data table/list"
    } else {
        "primary canvas/content"
    };
    regions.push(region(
        primary_role,
        "center",
        (start, content_top, end - start, height),
        "dominant remaining region",
    ));

    if leading.is_some() && trailing.is_some() {
        let confidence = if hints.leading && hints.trailing {
            "high"
        } else {
            "medium"
        };
        return (
            "sidebar-content-inspector".into(),
            "Leading navigation, central work area, and trailing detail inspector.".into(),
            regions,
            confidence,
        );
    }
    if leading.is_some() {
        let confidence = if hints.leading { "high" } else { "medium" };
        return (
            "sidebar-content".into(),
            "Leading navigation or collection list beside a dominant content area.".into(),
            regions,
            confidence,
        );
    }
    if trailing.is_some() {
        let confidence = if hints.trailing { "high" } else { "medium" };
        return (
            "content-inspector".into(),
            "Dominant content area with a trailing detail or property inspector.".into(),
            regions,
            confidence,
        );
    }
    if hints.table {
        return (
            "table-or-list".into(),
            "Single dominant table or list with controls around its perimeter.".into(),
            regions,
            "medium",
        );
    }
    if hints.canvas {
        return (
            "canvas".into(),
            "Single dominant canvas or editor with peripheral controls.".into(),
            regions,
            "medium",
        );
    }
    (
        "single-surface".into(),
        "One dominant content surface without a stable secondary panel detected.".into(),
        regions,
        "low",
    )
}

fn analyze(example: &Value, catalog: &str, image_path: &Path) -> Result<Value> {
    use image::GenericImageView;
    let payload = std::fs::read(image_path)?;
    let image = image::open(image_path)?;
    let (orig_w, orig_h) = image.dimensions();

    // PIL thumbnail((900, 900)): shrink-only, aspect preserved, LANCZOS.
    let factor = (900.0 / orig_w as f64).min(900.0 / orig_h as f64).min(1.0);
    let working = if factor < 1.0 {
        let nw = std::cmp::max(1, (orig_w as f64 * factor).round() as u32);
        let nh = std::cmp::max(1, (orig_h as f64 * factor).round() as u32);
        image.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3)
    } else {
        image.clone()
    };
    let rgb = working.to_rgb8();
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);
    let mut gray = Gray {
        data: Vec::with_capacity(w * h),
        width: w,
        height: h,
    };
    for px in rgb.pixels() {
        gray.data
            .push((px[0] as f32 + px[1] as f32 + px[2] as f32) / 3.0);
    }

    let vertical = separator_positions(&gray, true);
    let horizontal = separator_positions(&gray, false);
    let (layout, summary, regions, confidence) =
        classify_layout(example, catalog, &vertical, &horizontal);

    let mut gradients: Vec<f32> = Vec::with_capacity(h * w.saturating_sub(1));
    for y in 0..h {
        for x in 0..w.saturating_sub(1) {
            gradients.push((gray.at(y, x + 1) - gray.at(y, x)).abs());
        }
    }
    let edge_density = if gradients.is_empty() {
        0.0
    } else {
        gradients.iter().filter(|&&g| g as f64 > 24.0).count() as f64 / gradients.len() as f64
    };
    let density = if edge_density >= 0.14 {
        "high"
    } else if edge_density >= 0.075 {
        "medium"
    } else {
        "low"
    };
    let orientation = if orig_w as f64 > orig_h as f64 * 1.1 {
        "landscape"
    } else if orig_h as f64 > orig_w as f64 * 1.1 {
        "portrait"
    } else {
        "square"
    };

    Ok(json!({
        "analysis_kind": "deterministic-image-layout-v1",
        "image_sha256": lib::sha256_hex(&payload),
        "orientation": orientation,
        "layout_model": layout,
        "panel_summary": summary,
        "regions": regions,
        "detected_separators": { "vertical": vertical, "horizontal": horizontal },
        "visual_density": density,
        "confidence": confidence,
    }))
}

fn analyze_catalog(slug: &str) -> Result<(usize, Vec<Value>)> {
    let source_path = Path::new(slug).join("sources.json");
    let mut catalog: Value = lib::read_json(source_path.to_str().unwrap())?;
    let mut failures: Vec<Value> = Vec::new();
    let mut count = 0usize;
    let example_count = catalog
        .get("examples")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    for position in 0..example_count {
        let index = position + 1;
        let (name, local_path) = {
            let example = &catalog["examples"][position];
            (
                example
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                example
                    .get("visual")
                    .and_then(|v| v.get("local_path"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
            )
        };
        match local_path {
            None => failures.push(json!({"index": index, "name": name, "error": "visual missing"})),
            Some(local_path) => {
                let image_path = Path::new(slug).join(&local_path);
                let example = catalog["examples"][position].clone();
                match analyze(&example, slug, &image_path) {
                    Ok(structure) => {
                        if let Some(obj) = catalog["examples"][position].as_object_mut() {
                            obj.insert("interface_structure".into(), structure);
                        }
                        count += 1;
                    }
                    Err(error) => failures
                        .push(json!({"index": index, "name": name, "error": format!("{error:#}")})),
                }
            }
        }
    }

    if failures.is_empty() && count == example_count {
        if let Some(obj) = catalog.as_object_mut() {
            obj.insert("schema".into(), json!("wisent.example-catalog.v2"));
            obj.insert("visual_count".into(), json!(count));
            obj.insert("structure_count".into(), json!(count));
        }
    }
    std::fs::write(&source_path, serde_json::to_string_pretty(&catalog)? + "\n")
        .with_context(|| format!("write {}", source_path.display()))?;
    let failure_path = Path::new(slug).join("structure-analysis-failures.json");
    if !failures.is_empty() {
        std::fs::write(
            &failure_path,
            serde_json::to_string_pretty(&failures)? + "\n",
        )?;
    } else {
        let _ = std::fs::remove_file(&failure_path);
    }
    println!("{slug}: analyzed={count} failures={}", failures.len());
    Ok((count, failures))
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut requested: Vec<String> = Vec::new();
    for arg in rest {
        if arg.starts_with('-') {
            bail!("unknown argument: {arg}");
        }
        requested.push(arg.clone());
    }
    let mut unknown: Vec<&String> = requested
        .iter()
        .filter(|c| !CATALOGS.contains(&c.as_str()))
        .collect();
    unknown.sort();
    if !unknown.is_empty() {
        let names: Vec<&str> = unknown.iter().map(|s| s.as_str()).collect();
        bail!("unknown catalog(s): {}", names.join(", "));
    }
    let selected: Vec<&str> = if requested.is_empty() {
        CATALOGS.to_vec()
    } else {
        requested.iter().map(String::as_str).collect()
    };

    let mut failures = 0usize;
    for slug in selected {
        let (_, catalog_failures) = analyze_catalog(slug)?;
        failures += catalog_failures.len();
    }
    if failures > 0 {
        bail!("structure analysis left {failures} unresolved entries");
    }
    Ok(())
}
