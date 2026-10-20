//! A typed question asked of any OpenAI-compatible model service instead of
//! Brama: on a machine without Brama, `SPIS_DECISION_CHAT_MODEL` names a
//! model of the service at `MODEL_ROUTER_URL`, and each question is one chat
//! completion whose answer is bound by a JSON schema to the options the
//! question declares, so the answer is one of them by construction, as
//! Brama's decision route answers.

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

use super::Setting;

pub(crate) const CHAT_MODEL: Setting = Setting {
    variable: "SPIS_DECISION_CHAT_MODEL",
    holds: "a model of the OpenAI-compatible service at MODEL_ROUTER_URL, asked through /v1/chat/completions instead of Brama's decision route",
    unset: "typed questions go to Brama's POST /v1/decisions under MODEL_ROUTER_MODEL",
};

/// The chat model the operator declared for typed questions; `None` when
/// the variable is unset or blank, which keeps the questions on Brama.
pub(crate) fn declared() -> Result<Option<String>> {
    match std::env::var(CHAT_MODEL.variable) {
        Ok(value) if !value.trim().is_empty() => Ok(Some(value.trim().to_string())),
        Ok(_) | Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => anyhow::bail!("{} cannot be read: {error}", CHAT_MODEL.variable),
    }
}

/// The schema of one `choice` answer: one of the question's options and a
/// confidence.
fn answer_schema(purpose: &str, key: &str, question: &Value) -> Result<Value> {
    let options: Vec<Value> = question["criteria"]
        .as_object()
        .with_context(|| format!("{purpose}: question `{key}` declares no options: {question}"))?
        .keys()
        .map(|option| json!(option))
        .collect();
    Ok(json!({
        "type": "object",
        "properties": {
            "choice": { "type": "string", "enum": options },
            "confidence": { "type": "number" }
        },
        "required": ["choice", "confidence"],
        "additionalProperties": false
    }))
}

/// The answer document `{"answers": {...}}` the model gave for `questions`
/// about `state`, the shape Brama's decision route answers with.
pub(crate) fn ask(
    purpose: &str,
    model: &str,
    state: &str,
    questions: &Map<String, Value>,
) -> Result<Value> {
    let mut properties = Map::new();
    for (key, question) in questions {
        properties.insert(key.clone(), answer_schema(purpose, key, question)?);
    }
    let required: Vec<&String> = questions.keys().collect();
    let schema = json!({
        "type": "object",
        "properties": {
            "answers": {
                "type": "object",
                "properties": properties,
                "required": required,
                "additionalProperties": false
            }
        },
        "required": ["answers"],
        "additionalProperties": false
    });
    let prompt = format!(
        "State:\n{state}\n\nAnswer every question by its key: choose one of the options its criteria name and give your confidence that the choice is right.\n{}",
        Value::Object(questions.clone())
    );
    let payload = json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }],
        "response_format": {
            "type": "json_schema",
            "json_schema": { "name": "decision", "strict": true, "schema": schema }
        }
    });
    let body = super::post(purpose, "chat/completions", &payload.to_string())?;
    let content = body
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice["message"]["content"].as_str())
        .with_context(|| {
            format!("{purpose}: the model service's completion carries no message content: {body}")
        })?;
    serde_json::from_str(content).with_context(|| {
        format!("{purpose}: model {model} answered with content that is not the decision JSON: {content}")
    })
}
