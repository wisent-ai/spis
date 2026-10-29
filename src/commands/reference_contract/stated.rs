//! What a reference record states, read as each evidence field means it.
//! A measured flag is the boolean true, never a non-empty string such as
//! "no" or a non-zero number. A stated field is any value but null, blank
//! text and an empty list or object, so a stated zero or false is a value the
//! record holds, not a gap. The generator and the verifier both read records
//! through these two, so they cannot disagree about one record.

use serde_json::Value;

/// Whether `value` says the thing was measured: the boolean true.
pub fn measured(value: Option<&Value>) -> bool {
    value == Some(&Value::Bool(true))
}

/// Whether `value` states something: present, not null, not blank text and
/// not an empty list or object.
pub fn stated(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::String(text)) => !text.trim().is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(fields)) => !fields.is_empty(),
        Some(Value::Bool(_)) | Some(Value::Number(_)) => true,
    }
}
