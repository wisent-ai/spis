//! The engines that crawl a product family and the kinds of runtime product a
//! record names, each declared once with what follows from it. A manifest
//! naming an engine or a kind not declared here is refused with the value.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The engine that crawls one product family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Engine {
    /// Installed iOS or Android apps through Appium.
    Mobile,
    /// Installed macOS or desktop apps through Cua Driver.
    Desktop,
    /// Browser sessions through the Weles task API.
    Web,
    /// Terminal applications in a real PTY.
    Tui,
    /// Command-line tools.
    Cli,
    /// Documentation sites over bounded HTTP.
    Docs,
}

impl Engine {
    /// The engine `name` names, refused with the name when none is called that.
    pub(crate) fn of(name: &str) -> Result<Engine> {
        serde_json::from_value(Value::String(name.to_string()))
            .with_context(|| format!("{name:?} is not a declared crawl engine"))
    }

    /// The engine runs the installed product on the host itself, so its
    /// records hold the host exclusively while they run.
    pub(crate) fn runs_on_host(self) -> bool {
        matches!(
            self,
            Engine::Mobile | Engine::Desktop | Engine::Cli | Engine::Tui
        )
    }

    /// The record names its product by something the host resolves (a
    /// display name, a slug), so the resolved identifier differs from the
    /// declared one.
    pub(crate) fn resolves_identity(self) -> bool {
        matches!(self, Engine::Desktop | Engine::Cli | Engine::Tui)
    }

    /// The product needs a prepared runtime (a device or a launched app)
    /// before the crawl may start.
    pub(crate) fn prepares_runtime(self) -> bool {
        matches!(self, Engine::Mobile | Engine::Desktop)
    }

    /// The engine drives a terminal program.
    pub(crate) fn terminal(self) -> bool {
        matches!(self, Engine::Cli | Engine::Tui)
    }
}

/// What a record's runtime product identifier is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductKind {
    IosBundle,
    AndroidPackage,
    /// A desktop app named as people see it; preflight resolves it into a
    /// [`ProductKind::DesktopBundle`].
    DesktopDisplayName,
    DesktopBundle,
    CliBinary,
    /// A terminal app named by its slug; preflight resolves it into a
    /// [`ProductKind::TuiBinary`].
    TuiSlug,
    TuiBinary,
    Url,
}

impl ProductKind {
    /// The kind `name` names, refused with the name when none is called that.
    pub(crate) fn of(name: &str) -> Result<ProductKind> {
        serde_json::from_value(Value::String(name.to_string()))
            .with_context(|| format!("{name:?} is not a declared runtime product kind"))
    }

    /// The spelling a manifest stores.
    pub(crate) fn name(self) -> String {
        match serde_json::to_value(self) {
            Ok(Value::String(name)) => name,
            other => unreachable!("a unit variant serializes as a string, not {other:?}"),
        }
    }

    /// The product is an executable whose bytes preflight hashes.
    pub(crate) fn executable(self) -> bool {
        matches!(self, ProductKind::CliBinary | ProductKind::TuiBinary)
    }
}

/// Whose account a record's crawl runs under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AccountMode {
    /// No account: a public, read-only probe.
    AnonymousReadOnlyProbe,
    /// No account at all.
    None,
}

impl AccountMode {
    /// The mode `name` names, refused with the name when none is called that.
    pub(crate) fn of(name: &str) -> Result<AccountMode> {
        serde_json::from_value(Value::String(name.to_string()))
            .with_context(|| format!("{name:?} is not a declared crawl account mode"))
    }
}

/// How a record's worker receives its credentials.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DeliveryKind {
    /// Stado injects the Weles bearer and organization into the worker.
    WelesServiceEnv,
    /// The device or app is already signed in; nothing is injected.
    PreauthenticatedDevice,
    /// Stado injects the account's secret references as environment.
    StadoSecretEnv,
    /// Nothing is delivered.
    None,
}

impl DeliveryKind {
    /// The kind `name` names, refused with the name when none is called that.
    pub(crate) fn of(name: &str) -> Result<DeliveryKind> {
        serde_json::from_value(Value::String(name.to_string()))
            .with_context(|| format!("{name:?} is not a declared credential delivery"))
    }
}

/// How a Weles browser-evidence task may end, as Weles reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WebOutcome {
    Completed,
    Failed,
    Blocked,
}

impl WebOutcome {
    /// The outcome `name` names, or nothing when Weles has none called that.
    pub(crate) fn named(name: &str) -> Option<WebOutcome> {
        serde_json::from_value(Value::String(name.to_string())).ok()
    }
}
