// SPDX-License-Identifier: AGPL-3.0

use std::path::PathBuf;

fn main() {
    // ── Probe for each library via pkg-config ──────────────────────────
    // pkg-config discovers the correct include/library paths regardless of
    // distro (Debian multi-arch, Fedora lib64, Arch lib, Homebrew, etc.).
    //
    // The probe() call automatically emits cargo:rustc-link-lib,
    // cargo:rustc-link-search, and cargo:include directives to stdout.
    //
    // We collect the include paths to pass them to cc::Build.
    let mut include_paths: Vec<PathBuf> = Vec::new();
    let mut pkg_ok = true;

    for lib in &["freerdp3", "freerdp-client3", "winpr3"] {
        match pkg_config::Config::new().probe(lib) {
            Ok(info) => {
                include_paths.extend(info.include_paths);
            }
            Err(e) => {
                eprintln!(
                    "cargo:warning=pkg-config probe for '{}' failed: {}. \
                     Falling back to default Debian paths.",
                    lib, e
                );
                pkg_ok = false;
                break;
            }
        }
    }

    if !pkg_ok {
        fallback_build();
        return;
    }

    // Deduplicate (same path may appear from multiple .pc files)
    include_paths.sort();
    include_paths.dedup();

    // ── Compile the C bridge ────────────────────────────────────────────
    let mut build = cc::Build::new();
    build.file("rdp_bridge.c");

    for path in &include_paths {
        build.include(path);
    }

    build
        .flag("-Wall")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-deprecated-declarations")
        .compile("rdp_bridge");
}

/// Fallback for systems where pkg-config is not available or FreeRDP/WinPR
/// .pc files are not installed. Uses the Debian/Ubuntu paths.
fn fallback_build() {
    cc::Build::new()
        .file("rdp_bridge.c")
        .include("/usr/include/freerdp3")
        .include("/usr/include/winpr3")
        .flag("-Wall")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-deprecated-declarations")
        .compile("rdp_bridge");

    println!("cargo:rustc-link-lib=freerdp3");
    println!("cargo:rustc-link-lib=freerdp-client3");
    println!("cargo:rustc-link-lib=winpr3");
    println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu");
}
