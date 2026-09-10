// build.rs — compile the Zig kernels into a static library when `zig`
// is on the PATH (and the default-on `zig` feature is enabled);
// otherwise the crate builds with the scalar fallback. The link flags
// and the `qdecorators_zig` cfg are emitted here, once.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    // the cfg is conditionally emitted — declare it so rustc does not
    // flag it as unexpected when the kernels are absent
    println!("cargo:rustc-check-cfg=cfg(qdecorators_zig)");
    println!("cargo:rerun-if-changed=zig/kernels.zig");
    println!("cargo:rerun-if-env-changed=PATH");
    println!("cargo:rerun-if-env-changed=TARGET");

    if std::env::var("CARGO_FEATURE_ZIG").is_err() {
        // the lane is opt-out by contract
        return;
    }
    let Some(zig) = find_zig() else {
        println!("cargo:warning=zig not found on PATH — building without the Zig kernels");
        return;
    };

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let target = std::env::var("TARGET").unwrap_or_default();
    let zig_target = zig_target(&target);

    // Apple's ld64 demands 8-byte-aligned archive members; zig's own
    // archive writer pads members correctly, so the path is
    // build-obj → `zig ar`
    let obj_path = out.join("zigkern.o");
    let obj_arg = format!("-femit-bin={}", obj_path.display());
    let compiled = Command::new(&zig)
        .args([
            "build-obj",
            "zig/kernels.zig",
            "-O",
            "ReleaseSafe",
            "-fPIC",
            "-target",
            &zig_target,
        ])
        .arg(&obj_arg)
        .output()
        .expect("run zig build-obj");

    if !compiled.status.success() {
        let stderr = String::from_utf8_lossy(&compiled.stderr);
        let first: Vec<&str> = stderr.lines().take(8).collect();
        println!("cargo:warning=zig build-obj failed — {}", first.join(" | "));
        return;
    }

    let lib_path = out.join("libzigkern.a");
    let archived = Command::new(&zig)
        .args(["ar", "rcs"])
        .arg(&lib_path.display().to_string())
        .arg(&obj_path)
        .status()
        .expect("run zig ar");

    if !archived.success() {
        println!("cargo:warning=zig ar failed — building without the Zig kernels");
        return;
    }

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=zigkern");
    println!("cargo:rustc-cfg=qdecorators_zig");
}

fn find_zig() -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join("zig");
        if cand.is_file() {
            return Some(cand.to_string_lossy().to_string());
        }
    }
    None
}

/// map the Rust target triple to Zig's -target spelling; unknown triples
/// fall back to the host ("native")
fn zig_target(target: &str) -> String {
    let (arch, rest) = target.split_once('-').unwrap_or((target, ""));
    let arch = match arch {
        "aarch64" | "arm64" => "aarch64",
        "x86_64" | "amd64" => "x86_64",
        other => {
            eprintln!("cargo:warning=zig target mapping for {other} — using native");
            return "native".into();
        }
    };
    let os = if rest.starts_with("apple-darwin") {
        "macos"
    } else if rest.starts_with("unknown-linux-gnu") {
        "linux-gnu"
    } else if rest.starts_with("unknown-linux-musl") {
        "linux-musl"
    } else {
        eprintln!("cargo:warning=zig os mapping for {rest} — using native");
        return "native".into();
    };
    format!("{arch}-{os}")
}
