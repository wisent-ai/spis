use super::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CATALOG_SCHEMA: &str = "wisent.example-catalog.v2";

pub const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";

pub const INDEX_SCHEMA: &str = "wisent.full-reference-catalog.v2";

/// The value of type `T` spelled `name`, or nothing when `T` has none called
/// that. Every vocabulary below is read through its declared type this way.
pub fn named<T: DeserializeOwned>(name: &str) -> Option<T> {
    serde_json::from_value(Value::String(name.to_string())).ok()
}

/// How a declared value of a vocabulary below is spelled in a record.
pub fn spelling<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(name)) => name,
        other => unreachable!("a unit variant serializes as a string, not {other:?}"),
    }
}

/// Every interface family of the public corpus. Extra or missing families
/// are refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Family {
    IosAppExamples,
    AndroidAppExamples,
    MacosAppExamples,
    DesktopAppExamples,
    WebAppExamples,
    DashboardConsoleExamples,
    TuiExamples,
    CliExamples,
    OnboardingAuthExamples,
    DocumentationSiteExamples,
    AppStoreListingExamples,
    DesignSystemExamples,
    ReportEvidenceExamples,
    PricingPageExamples,
    LandingPageExamples,
}

impl Family {
    /// Every family, in the order the corpus lists them.
    pub const ALL: &'static [Family] = &[
        Family::IosAppExamples,
        Family::AndroidAppExamples,
        Family::MacosAppExamples,
        Family::DesktopAppExamples,
        Family::WebAppExamples,
        Family::DashboardConsoleExamples,
        Family::TuiExamples,
        Family::CliExamples,
        Family::OnboardingAuthExamples,
        Family::DocumentationSiteExamples,
        Family::AppStoreListingExamples,
        Family::DesignSystemExamples,
        Family::ReportEvidenceExamples,
        Family::PricingPageExamples,
        Family::LandingPageExamples,
    ];

    /// The catalog directory name of this family.
    pub fn name(self) -> String {
        spelling(&self)
    }

    /// The completeness profile a record of this family is measured under.
    pub fn profile(self) -> &'static str {
        match self {
            Family::CliExamples | Family::TuiExamples => "terminal-product",
            Family::DocumentationSiteExamples
            | Family::AppStoreListingExamples
            | Family::PricingPageExamples
            | Family::LandingPageExamples => "document-navigation",
            _ => "interactive-product",
        }
    }
}

/// What a motion asset is, measured from its container, never from its file
/// name or from what a record says.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MotionKind {
    VideoMp4,
    VideoWebm,
    AnimatedGif,
    AnimatedWebp,
    TerminalCast,
}

/// A media file reduced to one frame.
pub const STILL_KIND: &str = "still-image";

/// The canonical motion kind a record declares, or nothing when it declares
/// none or one Spis does not know.
pub fn canonical_motion_kind(declared: Option<&str>) -> Option<String> {
    let kind: MotionKind = named(declared?.trim())?;
    Some(spelling(&kind))
}

/// What ffprobe's `format_name` says a file is
/// (https://ffmpeg.org/ffprobe.html, `-show_format`).
#[derive(Clone, Copy, Debug, Deserialize)]
enum Container {
    #[serde(rename = "mov,mp4,m4a,3gp,3g2,mj2")]
    Mp4,
    #[serde(rename = "matroska,webm")]
    Webm,
    #[serde(rename = "gif")]
    Gif,
    #[serde(rename = "webp_pipe", alias = "webp")]
    Webp,
    #[serde(rename = "image2", alias = "png_pipe", alias = "mjpeg")]
    Still,
}

/// The kind of media ffprobe's `format_name` names: a motion kind or
/// [`STILL_KIND`], or nothing for a container Spis does not read.
pub fn container_kind(container: &str) -> Option<&'static str> {
    Some(match named::<Container>(container)? {
        Container::Mp4 => MotionKind::VideoMp4.as_str(),
        Container::Webm => MotionKind::VideoWebm.as_str(),
        Container::Gif => MotionKind::AnimatedGif.as_str(),
        Container::Webp => MotionKind::AnimatedWebp.as_str(),
        Container::Still => STILL_KIND,
    })
}

impl MotionKind {
    /// The spelling a record stores, for readers that keep it borrowed.
    pub fn as_str(self) -> &'static str {
        match self {
            MotionKind::VideoMp4 => "video-mp4",
            MotionKind::VideoWebm => "video-webm",
            MotionKind::AnimatedGif => "animated-gif",
            MotionKind::AnimatedWebp => "animated-webp",
            MotionKind::TerminalCast => "terminal-cast",
        }
    }
}

/// The extension of a retained media file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Extension {
    Gif,
    Webp,
    Mp4,
    Webm,
    Cast,
    Png,
    Jpg,
    Jpeg,
}

impl Extension {
    /// The extension of `path`, in any case, or nothing for one no media
    /// file of the corpus has.
    pub fn of(path: &std::path::Path) -> Option<Extension> {
        named(&path.extension()?.to_str()?.to_ascii_lowercase())
    }

    /// A motion asset may be stored under this extension.
    pub fn holds_motion(self) -> bool {
        matches!(
            self,
            Extension::Gif | Extension::Webp | Extension::Mp4 | Extension::Webm | Extension::Cast
        )
    }

    /// A state image may be stored under this extension.
    pub fn holds_state(self) -> bool {
        matches!(
            self,
            Extension::Png | Extension::Webp | Extension::Jpg | Extension::Jpeg
        )
    }
}

/// How motion was obtained. Cryptographically verified Weles evidence is a
/// distinct class and is never inferred from capture-method prose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provenance {
    LocalProductRun,
    WelesSignedBrowserEvidence,
    UpstreamOwnerMedia,
    UnverifiedSourceMedia,
}

impl Provenance {
    /// The phrase a catalog summary counts this provenance under.
    pub fn label(self) -> &'static str {
        match self {
            Provenance::LocalProductRun => "verified product run here",
            Provenance::WelesSignedBrowserEvidence => "Weles-signed browser evidence",
            Provenance::UpstreamOwnerMedia => "verified product-owner media",
            Provenance::UnverifiedSourceMedia => "unverified source media",
        }
    }

    /// The motion was produced by running the product here.
    pub fn local(self) -> bool {
        self == Provenance::LocalProductRun
    }
}

/// How long a motion's behaviour takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TimingClass {
    Instant,
    SubSecond,
    OneToThreeSeconds,
    MultiSecond,
    Continuous,
}

impl TimingClass {
    /// Every timing class, for messages that list them.
    pub const ALL: &'static [TimingClass] = &[
        TimingClass::Instant,
        TimingClass::SubSecond,
        TimingClass::OneToThreeSeconds,
        TimingClass::MultiSecond,
        TimingClass::Continuous,
    ];
}

/// Whether a record's evidence is complete or lists what it still misses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceStatus {
    /// `evidence_gaps` is empty.
    Complete,
    /// `evidence_gaps` lists what is missing.
    Partial,
}

impl EvidenceStatus {
    /// The status `record` declares, or nothing when it declares none Spis
    /// knows.
    pub fn of(record: &Value) -> Option<EvidenceStatus> {
        named(record.get("evidence_status")?.as_str()?)
    }
}
