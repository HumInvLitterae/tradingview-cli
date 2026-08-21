//! Build provenance reported by `tv --version` and `tv --version --verbose`.
//!
//! `crates/cli/build.rs` derives these values from `git` at build time. Fields
//! it cannot determine are `UNKNOWN` rather than absent, so the shape of the
//! output never depends on the build environment.

/// Version field of the short line, without the leading binary name that clap
/// prints: `<version> (<commit> <date>)`.
pub const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("TV_VERSION_COMMIT"),
    " ",
    env!("TV_VERSION_DATE"),
    ")"
);

/// Detailed provenance, in the `rustc --version --verbose` shape.
///
/// `commit-date` and `built-at` are both reported, unreduced: the short line
/// picks one of them depending on whether the build was dirty, and reduces
/// `built-at` to its date.
pub fn verbose_report() -> String {
    format!(
        "tv {VERSION}\n\
         binary: tv\n\
         release: {release}\n\
         commit-hash: {commit_hash}\n\
         commit-date: {commit_date}\n\
         built-at: {built_at}\n\
         dirty: {dirty}\n\
         target: {target}\n",
        release = env!("CARGO_PKG_VERSION"),
        commit_hash = env!("TV_BUILD_COMMIT_HASH"),
        commit_date = env!("TV_BUILD_COMMIT_DATE"),
        built_at = env!("TV_BUILD_BUILT_AT"),
        dirty = env!("TV_BUILD_DIRTY"),
        target = env!("TV_BUILD_TARGET"),
    )
}
