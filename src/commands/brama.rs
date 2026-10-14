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

/// Ask Brama `user` under `system` and return the object it answers through
/// the one `answer` tool it is given. `purpose` finishes every refusal, so
/// the operator learns which command needed the setting.
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
    // The answer comes back as the arguments of one declared tool call, so
    // it is read from the reply's structure, never searched for in prose.
    let payload = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "tools": [{"type": "function", "function": {
            "name": ANSWER_TOOL,
            "description": purpose,
            "parameters": {"type": "object"},
        }}],
        "tool_choice": {"type": "function", "function": {"name": ANSWER_TOOL}},
    })
    .to_string();
    // Read before the request so a bad reference is named, never sent anonymously.
    let token = crate::commands::role_secret("MODEL_ROUTER_TOKEN_ROLE")
        .with_context(|| format!("{purpose}: the Brama bearer could not be read"))?;
    let mut request = ureq::post(&endpoint).set("Content-Type", "application/json");
    if let Some(token) = &token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = match request.send_string(&payload) {
        Ok(response) => response,
        // Brama's own refusal is kept: its status and the body it answered.
        Err(ureq::Error::Status(status, response)) => {
            let said = response.into_string().with_context(|| {
                format!(
                    "{purpose}: Brama at {endpoint} answered HTTP {status} with an unreadable body"
                )
            })?;
            anyhow::bail!(
                "{purpose}: Brama at {endpoint} answered HTTP {status}: {}",
                said.trim()
            );
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("{purpose}: Brama at {endpoint} could not be reached"));
        }
    };
    let mut body_bytes = Vec::new();
    response.into_reader().read_to_end(&mut body_bytes)?;
    let body: Value = serde_json::from_slice(&body_bytes)
        .with_context(|| format!("{purpose}: Brama answered with a body that is not JSON"))?;
    let calls: Vec<&Value> = body["choices"][0]["message"]["tool_calls"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|call| call["function"]["name"].as_str() == Some(ANSWER_TOOL))
        .collect();
    let [call] = calls.as_slice() else {
        anyhow::bail!(
            "{purpose}: Brama answered {} {ANSWER_TOOL} calls where exactly one is required: {body}",
            calls.len()
        );
    };
    let arguments = call["function"]["arguments"].as_str().with_context(|| {
        format!("{purpose}: Brama's {ANSWER_TOOL} call carries no arguments: {call}")
    })?;
    match serde_json::from_str::<Value>(arguments) {
        Ok(object @ Value::Object(_)) => Ok(object),
        Ok(other) => anyhow::bail!("{purpose}: Brama's answer is not an object: {other}"),
        Err(error) => Err(error)
            .with_context(|| format!("{purpose}: Brama's answer is not JSON: {arguments}")),
    }
}

/// The one tool Brama answers through.
const ANSWER_TOOL: &str = "answer";
