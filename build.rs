use std::process::Command;

fn main() {
    let pkg_ver = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.2.0-preview".into());
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".into());
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".into());

    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/main");

    let git_sha = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "local".to_string());

    let version_label = format!("v{pkg_ver}");
    let short_tag = format!("v{pkg_ver} · {git_sha}");
    let full_tag = format!("v{pkg_ver} · {git_sha} · {target_os}/{target_arch}");

    println!("cargo:rustc-env=BIFROST_VERSION={version_label}");
    println!("cargo:rustc-env=BIFROST_GIT_SHA={git_sha}");
    println!("cargo:rustc-env=BIFROST_SHORT_TAG={short_tag}");
    println!("cargo:rustc-env=BIFROST_BUILD_TAG={full_tag}");
}
