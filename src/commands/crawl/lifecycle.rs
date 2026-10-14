//! The states a crawl record attempt passes through, and the one rule by which
//! a catalog's state follows from its records' and a run's from its catalogs'.
//! Every reader names a state through [`State`]; a run file holding a state
//! not declared here gets no aggregate state, and a diagnostic naming the
//! value it held instead.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// One record attempt's state, or the state a catalog or a run aggregates to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum State {
    Planned,
    Preflighting,
    PreflightPassed,
    PreflightFailed,
    Unavailable,
    Submitting,
    SubmissionFailed,
    Queued,
    Running,
    CancelPending,
    PendingReview,
    Cancelled,
    Lost,
    Failed,
    Partial,
    Completed,
    Uploaded,
    Imported,
}

impl State {
    /// The spelling the run file stores.
    pub(crate) fn name(self) -> String {
        match serde_json::to_value(self) {
            Ok(Value::String(name)) => name,
            other => unreachable!("a unit variant serializes as a string, not {other:?}"),
        }
    }

    /// The state `value` names, refused with the value when no state is
    /// called that.
    pub(crate) fn of(value: &Value) -> Result<State> {
        serde_json::from_value(value.clone())
            .with_context(|| format!("crawl state {value} is not a declared attempt state"))
    }

    /// The attempt ended without delivering what it was asked for. Such an
    /// attempt is history: resumption plans a fresh one instead.
    pub(crate) fn ended_undelivered(self) -> bool {
        matches!(
            self,
            State::Failed
                | State::Cancelled
                | State::Lost
                | State::SubmissionFailed
                | State::PreflightFailed
        )
    }

    /// The attempt is counted against its catalog: it ended without its full
    /// result, or was never placed.
    pub(crate) fn counts_as_failure(self) -> bool {
        self.ended_undelivered() || matches!(self, State::Unavailable | State::Partial)
    }

    /// The attempt's artifact exists: the worker finished it, uploaded it or
    /// it was imported.
    pub(crate) fn delivered(self) -> bool {
        matches!(self, State::Completed | State::Uploaded | State::Imported)
    }

    /// The attempt has not reached a worker yet.
    fn planning(self) -> bool {
        matches!(self, State::Planned | State::Preflighting)
    }

    /// The worker delivered the attempt's artifact and it waits for import.
    pub(crate) fn awaits_import(self) -> bool {
        matches!(self, State::Completed | State::Uploaded)
    }

    /// The attempt is accepted and waits for a worker.
    fn waiting(self) -> bool {
        matches!(
            self,
            State::Queued | State::Submitting | State::PreflightPassed
        )
    }
}

/// The state a set of attempts aggregates to: work still moving wins, in the
/// order running, cancel pending, pending review, queued, planned; then all
/// imported, all delivered, something delivered (a partial result included),
/// and otherwise failed. An empty set has delivered nothing and is failed.
pub(crate) fn aggregate(states: &[State]) -> State {
    let any = |test: fn(State) -> bool| states.iter().copied().any(test);
    if any(|state| state == State::Running) {
        State::Running
    } else if any(|state| state == State::CancelPending) {
        State::CancelPending
    } else if any(|state| state == State::PendingReview) {
        State::PendingReview
    } else if any(State::waiting) {
        State::Queued
    } else if any(State::planning) {
        State::Planned
    } else if states.is_empty() {
        State::Failed
    } else if states.iter().all(|state| *state == State::Imported) {
        State::Imported
    } else if states.iter().copied().all(State::delivered) {
        State::Completed
    } else if any(|state| state.delivered() || state == State::Partial) {
        State::Partial
    } else {
        State::Failed
    }
}

/// The states held under `key` in every entry of `container[list]`. A missing
/// list, or an entry without its state, is refused: read as no entries the
/// catalog was failed with nothing planned, and an entry left out let the
/// rest aggregate to imported while it had not been.
fn states_in(container: &Value, list: &str) -> Result<Vec<State>> {
    container
        .get(list)
        .and_then(Value::as_array)
        .with_context(|| format!("it records no {list} list"))?
        .iter()
        .map(|entry| {
            entry
                .get("state")
                .with_context(|| format!("an entry of {list} records no state: {entry}"))
                .and_then(State::of)
        })
        .collect()
}

/// Write `container`'s aggregate state, or, when one of its states is not
/// declared, no state and the diagnostic naming the value.
fn write_or_refuse(container: &mut Value, written: Result<()>) {
    if let Err(error) = written {
        container["state"] = Value::Null;
        container["diagnostic"] = json!({
            "code": "undeclared_crawl_state",
            "message": format!("{error:#}"),
        });
    }
}

/// Write a catalog entry's state from its records': the aggregate, whether
/// any record failed, how many failed in each state, and a diagnostic when
/// the catalog holds no record attempt at all.
pub(crate) fn aggregate_catalog_entry(entry: &mut Value) {
    let written = (|| -> Result<()> {
        let states = states_in(entry, "records")?;
        let mut failures: BTreeMap<String, usize> = BTreeMap::new();
        for failed in states
            .iter()
            .copied()
            .filter(|state| state.counts_as_failure())
        {
            let count = states.iter().filter(|state| **state == failed).count();
            failures.insert(failed.name(), count);
        }
        entry["state"] = json!(aggregate(&states).name());
        entry["partial"] = json!(!failures.is_empty());
        entry["failure_counts"] = serde_json::to_value(failures)?;
        // An empty plan is not a failed crawl. Expose the missing record
        // attempts through the same typed diagnostic shape as record-level
        // refusals.
        if states.is_empty() {
            entry["diagnostic"] = json!({
                "code": "no_planned_records",
                "message": "catalog carries no record attempts; nothing was planned, submitted or imported for it",
            });
        }
        Ok(())
    })();
    write_or_refuse(entry, written);
}

/// Write a run's state from its catalogs', and whether any catalog is partial.
pub(crate) fn update_run_state(run: &mut Value) {
    let written = states_in(run, "catalogs").map(|states| {
        run["state"] = json!(aggregate(&states).name());
        run["partial"] = json!(states.contains(&State::Partial));
    });
    write_or_refuse(run, written);
}
