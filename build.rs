use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn main() {

    let revision = git(&["rev-parse", "HEAD"]).filter(|value| {
        value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
    });
    let dirty = git(&["status", "--porcelain"])
        .map(|status| !status.is_empty())
        .unwrap_or(true);

    println!("cargo:rustc-env=SPIS_GIT_REVISION={}", revision.as_deref().unwrap_or("unknown"));
    println!("cargo:rustc-env=SPIS_GIT_DIRTY={dirty}");
}
