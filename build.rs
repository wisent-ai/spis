use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Stamps the exact source revision and derives the checked-in Weles bridge
/// digest. Both jobs must live in this one build script, and both depend on
/// Cargo re-running it:
///
/// * `SPIS_GIT_REVISION`/`SPIS_GIT_DIRTY` gate exact-revision crawl submission
///   in `commands::crawl::build_revision`, so a stale `SPIS_GIT_DIRTY=false`
///   would let a dirty tree submit work. Their input is the whole worktree.
/// * `SPIS_BRIDGE_SCRIPT_SHA256` is the constant `weles_provenance` compares the
///   checked-in bridge against, so a stale pin would reintroduce script drift.
///   Its input is `weles-bridge/spis-weles-bridge.mjs`.
///
/// This script therefore emits no `cargo:rerun-if-changed` line. The first such
/// line would replace Cargo's "re-run when any file in the package changes"
/// default with exactly the paths named, and no bounded path list covers the
/// worktree that `git status` inspects. Both inputs above are package files, so
/// the default already re-runs this script for either of them; narrowing to the
/// bridge script alone would keep the pin fresh while silently freezing the
/// dirty gate. Do not add one without replacing the dirty gate's input first.
fn main() {
    let revision = git(&["rev-parse", "HEAD"]).filter(|value| {
        value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
    });
    let dirty = git(&["status", "--porcelain"])
        .map(|status| !status.is_empty())
        .unwrap_or(true);

    println!("cargo:rustc-env=SPIS_GIT_REVISION={}", revision.as_deref().unwrap_or("unknown"));
    println!("cargo:rustc-env=SPIS_GIT_DIRTY={dirty}");

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is not set; build scripts must run under Cargo");
    let script_path = PathBuf::from(manifest_dir)
        .join("weles-bridge")
        .join("spis-weles-bridge.mjs");

    let script_bytes = std::fs::read(&script_path).unwrap_or_else(|error| {
        panic!(
            "checked-in Weles bridge could not be read at {}: {error}; \
             the bridge script must exist in the source tree so its SHA-256 pin can be derived",
            script_path.display()
        )
    });

    let digest = <sha2::Sha256 as sha2::Digest>::digest(&script_bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }

    println!("cargo:rustc-env=SPIS_BRIDGE_SCRIPT_SHA256={hex}");
}
