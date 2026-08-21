//! Build-time provenance stamp for `tv --version`.
//!
//! The stamp follows the `cargo`/`rustc` shape `tv <version> (<commit> <date>)`.
//! A clean build reports the short commit hash and that commit's date, so the
//! same commit always produces the same string. A build that contains
//! uncommitted changes to executable sources reports `<commit>-dirty` and the
//! build date, because the commit date no longer describes the binary.
//!
//! `TV_VERSION_*` holds the two fields of that line. `TV_BUILD_*` holds the
//! unreduced fields behind it, which `tv --version --verbose` prints.
//!
//! Commit provenance is derived from `git`. When `git` or the repository is
//! unavailable, those fields fall back to `UNKNOWN` instead of failing the
//! build.
//!
//! The derivation lives in `build/provenance.rs` so that
//! `crates/cli/tests/build_provenance.rs` can exercise it against real
//! repositories.

include!("build/provenance.rs");

fn main() {
    let root = workspace_root();

    println!("cargo::rerun-if-env-changed=SOURCE_DATE_EPOCH");
    for path in rerun_paths(&root) {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    let stamp = Stamp::read(&root);
    println!(
        "cargo::rustc-env=TV_VERSION_COMMIT={}",
        stamp.version_commit
    );
    println!("cargo::rustc-env=TV_VERSION_DATE={}", stamp.version_date);
    println!(
        "cargo::rustc-env=TV_BUILD_COMMIT_HASH={}",
        stamp.commit_hash
    );
    println!(
        "cargo::rustc-env=TV_BUILD_COMMIT_DATE={}",
        stamp.commit_date
    );
    println!("cargo::rustc-env=TV_BUILD_BUILT_AT={}", stamp.built_at);
    println!("cargo::rustc-env=TV_BUILD_DIRTY={}", stamp.dirty);
    // Cargo's `TARGET` is the platform the produced binary runs on. Cargo's
    // `HOST`, the host platform of the Rust compiler running this build, would
    // be a different field and is deliberately not reported.
    println!(
        "cargo::rustc-env=TV_BUILD_TARGET={}",
        env::var("TARGET").unwrap_or_else(|_| UNKNOWN.to_string())
    );
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set for build scripts"),
    );
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}
