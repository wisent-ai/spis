//! One question to Brama whose answer is a JSON object.
//!
//! Spis asks Brama what words and weights used to guess: which discovered
//! pages matter, which image on a page shows the product. The gateway is
//! `MODEL_ROUTER_URL`, the model the operator's router alias
//! `MODEL_ROUTER_MODEL`, and the bearer the vault role
//! `MODEL_ROUTER_TOKEN_ROLE` names; each missing one is refused with the
//! variable and the command that needed it.

use std::io::Read;

use anyhow::{Context, Result};
use serde_json::{json, Value};

/// Ask Brama `user` under `system` and return the first JSON object in its
/// answer. `purpose` finishes every refusal, so the operator learns which
/// command needed the setting.
pub(crate) fn ask_json(purpose: &str, system: &str, user: &str) -> Result<Value> {
    let router = std::env::var("MODEL_ROUTER_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("{purpose}: set MODEL_ROUTER_URL to the Brama address"))?;
    let endpoint = if router.contains("/v1") {
        format!("{}/chat/completions", router.trim_end_matches('/'))
    } else {
        format!("{}/v1/chat/completions", router.trim_end_matches('/'))
    };
    // The model is the router's alias chosen by the operator, never a provider
    // model written into the product (cli.md rule 14).
    let model = std::env::var("MODEL_ROUTER_MODEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("{purpose}: set MODEL_ROUTER_MODEL to a Brama model alias"))?;
    let payload = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
    })
    .to_string();
    // Read before the request so a bad reference is named, never sent anonymously.
    let token = crate::commands::role_secret("MODEL_ROUTER_TOKEN_ROLE")
        .with_context(|| format!("{purpose}: the Brama bearer could not be read"))?;
    let mut request = ureq::post(&endpoint).set("Content-Type", "application/json");
    if let Some(token) = &token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = request.send_string(&payload).with_context(|| {
        format!("{purpose}: Brama at {endpoint} refused or could not be reached")
    })?;
    let mut body_bytes = Vec::new();
    response.into_reader().read_to_end(&mut body_bytes)?;
    let body: Value = serde_json::from_slice(&body_bytes)
        .with_context(|| format!("{purpose}: Brama answered with a body that is not JSON"))?;
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .with_context(|| format!("{purpose}: Brama's answer carries no message content"))?;
    let open = content
        .find('{')
        .with_context(|| format!("{purpose}: Brama's answer holds no JSON object"))?;
    let close = content
        .rfind('}')
        .with_context(|| format!("{purpose}: Brama's answer holds no JSON object"))?
        + 1;
    serde_json::from_str(&content[open..close])
        .with_context(|| format!("{purpose}: Brama's answer is not valid JSON"))
}
