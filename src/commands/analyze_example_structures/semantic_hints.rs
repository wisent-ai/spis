use super::*;

pub(crate) fn semantic_hints(example: &Value, catalog: &str) -> Hints {
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
pub(crate) fn choose_boundary(
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

pub(crate) fn region(role: &str, position: &str, bounds: (f64, f64, f64, f64), evidence: &str) -> Value {
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

pub(crate) type Layout = (String, String, Vec<Value>, &'static str);
