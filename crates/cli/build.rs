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

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const UNKNOWN: &str = "UNKNOWN";

/// Working-tree paths that can change the built executable. Documentation-only
/// edits must not mark a build dirty, otherwise the marker becomes noise.
const BUILD_INPUT_PATHS: [&str; 5] = [
    "crates",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "rust-toolchain",
];

fn main() {
    let root = workspace_root();

    emit_rerun_directives(&root);

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
    println!(
        "cargo::rustc-env=TV_BUILD_HOST={}",
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

struct Stamp {
    /// Commit field of the `tv --version` line, including any `-dirty` marker.
    version_commit: String,
    /// Date field of the `tv --version` line.
    version_date: String,
    commit_hash: String,
    commit_date: String,
    built_at: String,
    /// `true`, `false`, or `UNKNOWN` when there is no commit to compare against.
    dirty: String,
}

impl Stamp {
    fn read(root: &Path) -> Self {
        let built_at = built_at(root);
        let build_date = built_at.get(..10).unwrap_or(UNKNOWN).to_string();

        let Some(short_commit) = git(root, &["rev-parse", "--short", "HEAD"]) else {
            return Self {
                version_commit: UNKNOWN.to_string(),
                version_date: UNKNOWN.to_string(),
                commit_hash: UNKNOWN.to_string(),
                commit_date: UNKNOWN.to_string(),
                built_at,
                dirty: UNKNOWN.to_string(),
            };
        };

        let commit_hash = git(root, &["rev-parse", "HEAD"]).unwrap_or_else(|| short_commit.clone());
        let commit_date = git(root, &["log", "-1", "--date=short", "--format=%cd"])
            .unwrap_or_else(|| UNKNOWN.to_string());
        let dirty = is_dirty(root);

        Self {
            version_commit: if dirty {
                format!("{short_commit}-dirty")
            } else {
                short_commit
            },
            // A dirty binary is no longer described by its commit date, so the
            // version line falls back to the day it was actually built.
            version_date: if dirty {
                build_date
            } else {
                commit_date.clone()
            },
            commit_hash,
            commit_date,
            built_at,
            dirty: dirty.to_string(),
        }
    }
}

/// Reports whether tracked or new executable sources differ from `HEAD`.
///
/// A `git status` failure is reported as dirty: the build cannot be attributed
/// to a commit, which is exactly what the marker means.
fn is_dirty(root: &Path) -> bool {
    let mut args = vec!["status", "--porcelain", "--"];
    args.extend_from_slice(&BUILD_INPUT_PATHS);
    git(root, &args).is_none_or(|status| !status.is_empty())
}

/// Build time as an RFC 3339 local timestamp such as
/// `2026-08-21T07:15:01+09:00`.
///
/// The wall clock is rendered in the machine's own time zone so it lines up
/// with the local dates `git` prints for commits. Without `git` the offset is
/// unknown and the timestamp falls back to UTC, which the printed `+00:00`
/// states explicitly, so the instant stays correct either way.
///
/// `SOURCE_DATE_EPOCH` overrides the clock for reproducible builds and is
/// rendered as UTC, as that convention requires.
fn built_at(root: &Path) -> String {
    let (epoch_seconds, offset_seconds) = match reproducible_epoch() {
        Some(epoch_seconds) => (epoch_seconds, 0),
        None => {
            let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
                return UNKNOWN.to_string();
            };
            (now.as_secs() as i64, local_utc_offset_seconds(root))
        }
    };

    let local_seconds = epoch_seconds + offset_seconds;
    let (year, month, day) = civil_from_days(local_seconds.div_euclid(86_400));
    let second_of_day = local_seconds.rem_euclid(86_400);
    let (hour, minute, second) = (
        second_of_day / 3600,
        (second_of_day % 3600) / 60,
        second_of_day % 60,
    );

    let sign = if offset_seconds < 0 { '-' } else { '+' };
    let offset_minutes = offset_seconds.abs() / 60;

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{sign}{:02}:{:02}",
        offset_minutes / 60,
        offset_minutes % 60
    )
}

/// Honors the `SOURCE_DATE_EPOCH` reproducible-build convention.
fn reproducible_epoch() -> Option<i64> {
    env::var("SOURCE_DATE_EPOCH").ok()?.trim().parse().ok()
}

/// Local UTC offset in seconds, taken from `git` so the build date and commit
/// dates use the same time-zone source. Falls back to UTC when unavailable.
fn local_utc_offset_seconds(root: &Path) -> i64 {
    let current = git(root, &["var", "GIT_COMMITTER_IDENT"]);
    let head = || git(root, &["log", "-1", "--date=raw", "--format=%cd"]);

    current
        .or_else(head)
        .and_then(|ident| parse_utc_offset(ident.split_whitespace().next_back()?))
        .unwrap_or(0)
}

/// Parses a `git` time-zone token such as `+0900` into seconds.
fn parse_utc_offset(token: &str) -> Option<i64> {
    let (sign, digits) = match token.split_at_checked(1)? {
        ("+", digits) => (1, digits),
        ("-", digits) => (-1, digits),
        _ => return None,
    };
    if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let hours: i64 = digits[..2].parse().ok()?;
    let minutes: i64 = digits[2..].parse().ok()?;
    Some(sign * (hours * 3600 + minutes * 60))
}

/// Converts days since the Unix epoch into a proleptic Gregorian date.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_position + 2) / 5 + 1;
    let month = if month_position < 10 {
        month_position + 3
    } else {
        month_position - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Watches the inputs that can change the stamp: executable sources for the
/// dirty marker, and the `git` refs that change on commit or checkout.
fn emit_rerun_directives(root: &Path) {
    println!("cargo::rerun-if-env-changed=SOURCE_DATE_EPOCH");

    for path in BUILD_INPUT_PATHS {
        watch(&root.join(path));
    }

    for git_path in ["HEAD", "packed-refs"] {
        if let Some(resolved) = git(root, &["rev-parse", "--git-path", git_path]) {
            watch(&root.join(resolved));
        }
    }

    if let Some(head_ref) = git(root, &["symbolic-ref", "--quiet", "HEAD"])
        && let Some(resolved) = git(root, &["rev-parse", "--git-path", &head_ref])
    {
        watch(&root.join(resolved));
    }
}

/// Cargo treats a missing watched path as always-changed, so only existing
/// paths are declared.
fn watch(path: &Path) {
    if path.exists() {
        println!("cargo::rerun-if-changed={}", path.display());
    }
}

/// Runs `git` in the workspace root and returns trimmed stdout on success.
fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
}
