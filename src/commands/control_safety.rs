//! What a crawler may press, judged by a model through Brama rather than
//! guessed from words: one question per screen, every control on it
//! answered. A crawler presses only navigation; the rest is withheld with
//! the class the model gave it, and a screen nobody could judge is refused
//! with Brama's own cause, because pressing an unjudged control could buy,
//! delete or send something.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

/// What pressing one control does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Safety {
    /// Goes back, cancels, closes or dismisses without changing anything.
    Navigation,
    /// Grants or refuses an operating-system permission or notification.
    Consent,
    /// Deletes, removes, buys, pays, sends, publishes, posts, signs out or
    /// closes an account: changes something outside the screen.
    Destructive,
    /// Not something a person presses: text, an image, a group.
    Inert,
    /// Any other control.
    Other,
}

impl Safety {
    /// Why a control of this class is not pressed, or nothing for navigation.
    pub(crate) fn withheld(self) -> Option<&'static str> {
        match self {
            Safety::Navigation => None,
            Safety::Consent => Some("permission/notification-like control withheld before delivery"),
            Safety::Destructive => Some("destructive control withheld before delivery"),
            Safety::Inert => Some("element is not a control and was not pressed"),
            Safety::Other => Some(
                "control is not independently safe navigation or cancellation and was withheld before delivery",
            ),
        }
    }

    /// The name a crawl record stores.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Safety::Navigation => "navigation",
            Safety::Consent => "consent",
            Safety::Destructive => "destructive",
            Safety::Inert => "inert",
            Safety::Other => "other",
        }
    }
}

#[derive(Deserialize)]
struct Judged {
    index: usize,
    class: Safety,
}

#[derive(Deserialize)]
struct Answer {
    controls: Vec<Judged>,
}

/// The safety of every `(role, label)` on one screen, in order. `purpose`
/// names the crawler in every refusal.
pub(crate) fn judge(purpose: &str, controls: &[(String, String)]) -> Result<Vec<Safety>> {
    if controls.is_empty() {
        return Ok(Vec::new());
    }
    let listed: Vec<Value> = controls
        .iter()
        .enumerate()
        .map(|(index, (role, label))| json!({ "index": index, "role": role, "label": label }))
        .collect();
    let answer = super::brama::ask_json(
        purpose,
        "You decide which controls on one application screen an automated crawler may press. \
         For every control you are given (its accessibility role and its visible label, in any \
         language) answer one class: navigation when pressing it only goes back, cancels, closes \
         or dismisses without changing anything; consent when it grants or refuses an operating \
         system permission or notification; destructive when it deletes, removes, buys, pays, \
         sends, publishes, posts, signs out or closes an account; inert when it is not something \
         a person presses (text, an image, a group); other for any other control. Judge what \
         pressing it does, not single words. Answer only JSON: \
         {\"controls\": [{\"index\": <index>, \"class\": \"navigation|consent|destructive|inert|other\"}]}.",
        &Value::Array(listed).to_string(),
    )?;
    let answer: Answer = serde_json::from_value(answer)
        .with_context(|| format!("{purpose}: Brama's control classes are not the asked shape"))?;
    let mut classes: Vec<Option<Safety>> = controls.iter().map(|_| None).collect();
    for judged in answer.controls {
        let Some(slot) = classes.get_mut(judged.index) else {
            bail!(
                "{purpose}: Brama answered a control {} that was not asked",
                judged.index
            );
        };
        *slot = Some(judged.class);
    }
    classes
        .into_iter()
        .zip(controls)
        .map(|(class, (role, label))| {
            class.with_context(|| {
                format!("{purpose}: Brama left the {role} control {label:?} unjudged")
            })
        })
        .collect()
}
