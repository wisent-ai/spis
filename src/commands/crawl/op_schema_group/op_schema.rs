use super::*;

pub(crate) const OP_SCHEMA: &str = "wisent.crawl-operation.v1";

pub(crate) const RUN_SCHEMA: &str = "wisent.crawl-run.v1";

pub(crate) const SUBMISSION_SCHEMA: &str = "wisent.crawl-submission.v1";

pub(crate) const HOST_PROBE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub(crate) struct CommandTimedOut {
    pub(crate) operation: String,
    pub(crate) timeout: Duration,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

impl std::fmt::Display for CommandTimedOut {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} exceeded hard timeout {:?}; stdout={:?}; stderr={:?}",
            self.operation,
            self.timeout,
            String::from_utf8_lossy(&self.stdout),
            String::from_utf8_lossy(&self.stderr)
        )
    }
}

impl std::error::Error for CommandTimedOut {}

/// Every product family and the engine that crawls it.
///
/// Public because the generated documentation enumerates it: `docs_site`
/// reads this table rather than restating a count in prose, which is how the
/// published documentation came to claim thirteen families.
pub(crate) const CATALOGS: &[(&str, &str)] = &[
    ("ios-app-examples", "mobile"),
    ("android-app-examples", "mobile"),
    ("macos-app-examples", "desktop"),
    ("desktop-app-examples", "desktop"),
    ("web-app-examples", "web"),
    ("dashboard-console-examples", "web"),
    ("tui-examples", "tui"),
    ("cli-examples", "cli"),
    ("onboarding-auth-examples", "web"),
    ("documentation-site-examples", "docs"),
    ("app-store-listing-examples", "web"),
    ("design-system-examples", "web"),
    ("report-evidence-examples", "web"),
    ("pricing-page-examples", "web"),
    ("landing-page-examples", "web"),
];

pub(crate) const RUNTIME_MANIFEST_SCHEMA: &str = "wisent.crawl-runtime-manifest.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RuntimeSurfaceIdentity {
    pub family: String,
    pub exact_url: String,
    pub origin: String,
    pub path: String,
    pub allowed_origins: Vec<String>,
    pub allowed_actions: Vec<String>,
    pub terminal_outcomes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeProduct {
    pub kind: String,
    /// The identity committed in `reference.json`, never rewritten.
    ///
    /// `record_preflight` resolves a display name into a bundle id and a slug
    /// into a binary name, so `identifier` legitimately changes mid-flight. The
    /// declared value stays fixed, which lets `decode_runtime_manifest` compare
    /// every engine against the committed record instead of exempting the three
    /// engines that resolve.
    pub declared_identifier: String,
    pub identifier: String,
    pub product_url: String,
    pub identity_source: String,
    pub surface: Option<RuntimeSurfaceIdentity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeAccount {
    pub mode: String,
    pub account_id: Option<String>,
    #[serde(default)]
    pub credential_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeConstraints {
    pub no_first_run_consent: bool,
    pub no_system_permission_prompts: bool,
    pub no_notifications: bool,
    pub no_purchase: bool,
    pub no_final_destructive_action: bool,
    pub headless: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimePreparedProof {
    pub schema: String,
    pub product_identifier: String,
    pub device_id: Option<String>,
    pub observed_by: String,
    pub product_version: String,
    pub executable_sha256: String,
    pub observed_at: String,
    pub evidence_uri: String,
    pub evidence_sha256: String,
    pub installed: bool,
    pub first_run_completed: bool,
    pub pending_permission_prompts: u32,
    pub pending_notification_prompts: u32,
    pub notification_delivery_disabled: bool,
    pub permission_prompt_invocation_disabled: bool,
    pub notification_prompt_invocation_disabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeDelivery {
    pub kind: String,
    #[serde(default)]
    pub secret_env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct RuntimeBinding {
    pub(crate) account: RuntimeAccount,
    pub(crate) constraints: RuntimeConstraints,
    pub(crate) prepared_proof: Option<RuntimePreparedProof>,
    pub(crate) delivery: RuntimeDelivery,
    pub(crate) surface: Option<RuntimeSurfaceIdentity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeServiceIdentity {
    pub name: String,
    pub generation: u64,
    pub consumer: String,
    pub capability: String,
    pub active_host: String,
    pub endpoint: String,
    pub action: String,
    /// Exact deployed Weles worker release advertised by the Stado service
    /// directory and re-confirmed against `{endpoint}/version` before a task is
    /// submitted. It is signed into the receipt through `spisBinding.service`.
    pub release_id: String,
    /// Exact Weles source revision behind `release_id`, from the same two
    /// independent observations.
    pub source_revision: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeExecutionIdentity {
    pub host: String,
    pub observed_hostname: String,
    pub platform: String,
    pub device_id: Option<String>,
    pub resolved_product_identifier: String,
    pub device_name: Option<String>,
    #[serde(default)]
    pub executable_path: Option<String>,
    #[serde(default)]
    pub product_version: Option<String>,
    #[serde(default)]
    pub executable_sha256: Option<String>,
    #[serde(default)]
    pub effective_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeManifest {
    pub schema: String,
    pub run_id: String,
    pub catalog: String,
    pub attempt: u32,
    pub attempt_id: String,
    pub record: String,
    pub engine: String,
    pub source_revision: String,
    pub source_input_sha256: String,
    pub reference_sha256: String,
    pub catalog_key: String,
    pub record_key: String,
    pub correlation_id: String,
    pub stado_run_id: String,
    pub artifact_uri: String,
    pub output_uri: String,
    pub runtime_product: RuntimeProduct,
    pub account: RuntimeAccount,
    pub constraints: RuntimeConstraints,
    pub docs_structure_sha256: Option<String>,
    pub bindings_file_sha256: String,
    pub bindings_source: String,
    pub bindings_sha256: String,
    pub bindings_uri: String,
    pub delivery: RuntimeDelivery,
    pub prepared_proof: Option<RuntimePreparedProof>,
    pub execution_identity: Option<RuntimeExecutionIdentity>,
    pub resource_lease: Option<String>,
    pub service_identity: Option<RuntimeServiceIdentity>,
}

impl RuntimeManifest {
    pub(crate) fn encoded(&self) -> Result<String> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        Ok(STANDARD.encode(serde_json::to_vec(self)?))
    }
}
