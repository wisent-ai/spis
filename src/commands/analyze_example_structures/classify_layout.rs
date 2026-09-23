use super::*;

pub(crate) fn classify_layout(
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
