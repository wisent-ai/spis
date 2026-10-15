//! What a crawler may press, judged by a model through Brama rather than
//! guessed from words: one typed decision per screen, one `choice` per
//! control on it, every one answered. A crawler presses only navigation; the
//! rest is withheld with the class the model chose, and a screen nobody
//! could judge is refused with Brama's own cause, because pressing an
//! unjudged control could buy, delete or send something.

use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};

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

/// The class a control is asked about: every safety and what it means.
const CLASSES: &[(Safety, &str)] = &[
    (Safety::Navigation, "pressing it only goes back, cancels, closes or dismisses without changing anything"),
    (Safety::Consent, "it grants or refuses an operating system permission or notification"),
    (Safety::Destructive, "it deletes, removes, buys, pays, sends, publishes, posts, signs out or closes an account"),
    (Safety::Inert, "it is not something a person presses: text, an image, a group"),
    (Safety::Other, "any other control"),
];
/// Each control is asked under this prefix and its position on the screen.
const CONTROL: &str = "control-";

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
    let options: Vec<(String, String)> = CLASSES
        .iter()
        .map(|(class, meaning)| (class.name().to_string(), (*meaning).to_string()))
        .collect();
    let questions: Map<String, Value> = controls
        .iter()
        .enumerate()
        .map(|(index, (role, label))| {
            let instructions = format!(
                "The state lists the controls of one application screen an automated crawler \
                 may press, each with its accessibility role and visible label, in any language. \
                 What does pressing the {role} control at index {index}, labelled {label:?}, do? \
                 Judge what pressing it does, not single words."
            );
            (format!("{CONTROL}{index}"), super::brama::choice(&instructions, &options))
        })
        .collect();
    let answers = super::brama::decide(purpose, &Value::Array(listed).to_string(), &questions)?;
    controls
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let (name, _) = super::brama::chosen(purpose, &answers[&format!("{CONTROL}{index}")], &options)?;
            match CLASSES.iter().find(|(class, _)| class.name() == name) {
                Some((class, _)) => Ok(*class),
                None => bail!("{purpose}: Brama chose the class {name:?}, which is not a declared safety"),
            }
        })
        .collect()
}
