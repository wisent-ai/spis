use std::path::PathBuf;
use std::process::Command;

/// What `git args` printed, or why it printed nothing usable: the spawn error, or
/// the exit status with git's own stderr.
fn git(args: &[&str]) -> Result<String, String> {
    let invocation = format!("git {}", args.join(" "));
    let output = stado_wait::output(Command::new("git").args(args))
        .map_err(|error| format!("{invocation} could not run: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{invocation} failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// One line a `cargo:rustc-env` directive can carry.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Stamps the exact source revision and derives the checked-in Weles bridge
/// digest. Both jobs must live in this one build script, and both depend on
/// Cargo re-running it:
///
/// * `SPIS_GIT_REVISION` is the commit a clean tree was built from, and
///   `SPIS_GIT_REVISION_REFUSAL` says why there is none (git could not answer,
///   or the tree was dirty). Exact-revision crawl submission in
///   `commands::crawl::build_revision` refuses with that sentence, so a stale
///   stamp would let a dirty tree submit work. Their input is the whole worktree.
/// * `SPIS_BRIDGE_SCRIPT_SHA256` is the constant `weles_provenance` compares the
///   checked-in bridge against, so a stale pin would reintroduce script drift.
///   Its input is `weles-bridge/spis-weles-bridge.mjs`; a bridge that cannot be
///   read fails the build naming it.
///
/// This script therefore emits no `cargo:rerun-if-changed` line. The first such
/// line would replace Cargo's "re-run when any file in the package changes"
/// default with exactly the paths named, and no bounded path list covers the
/// worktree that `git status` inspects. Both inputs above are package files, so
/// the default already re-runs this script for either of them; narrowing to the
/// bridge script alone would keep the pin fresh while silently freezing the
/// dirty gate. Do not add one without replacing the dirty gate's input first.
fn main() -> Result<(), String> {
    let stamp = git(&["rev-parse", "--verify", "HEAD"]).and_then(|revision| {
        let status = git(&["status", "--porcelain"])?;
        if status.is_empty() {
            Ok(revision)
        } else {
            Err(format!(
                "this Spis binary was built from a dirty source tree at {revision} ({})",
                one_line(&status)
            ))
        }
    });
    match stamp {
        Ok(revision) => {
            println!("cargo:rustc-env=SPIS_GIT_REVISION={revision}");
            println!("cargo:rustc-env=SPIS_GIT_REVISION_REFUSAL=");
        }
        Err(cause) => {
            println!("cargo:rustc-env=SPIS_GIT_REVISION=");
            println!(
                "cargo:rustc-env=SPIS_GIT_REVISION_REFUSAL={}",
                one_line(&cause)
            );
        }
    }

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .ok_or("CARGO_MANIFEST_DIR is not set; build scripts must run under Cargo")?;
    let script_path = PathBuf::from(manifest_dir)
        .join("weles-bridge")
        .join("spis-weles-bridge.mjs");
    let script_bytes = std::fs::read(&script_path).map_err(|error| {
        format!(
            "checked-in Weles bridge could not be read at {}: {error}; \
             the bridge script must exist in the source tree so its SHA-256 pin can be derived",
            script_path.display()
        )
    })?;

    let digest = <sha2::Sha256 as sha2::Digest>::digest(&script_bytes);
    println!("cargo:rustc-env=SPIS_BRIDGE_SCRIPT_SHA256={digest:x}");
    Ok(())
}
