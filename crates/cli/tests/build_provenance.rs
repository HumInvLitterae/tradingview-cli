//! Tests for the build-time provenance derivation in `crates/cli/build.rs`.
//!
//! A build script cannot be imported, so the derivation is included from the
//! same source the build script includes. `cli_contract.rs` checks that the
//! binary prints what the build script produced; these tests check that what
//! the build script produces is correct, against real repositories and known
//! dates.

#![allow(dead_code)]

use std::fs;

use tempfile::TempDir;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/build/provenance.rs"));

/// Commit timestamps are fixed so that the commit date can never coincide with
/// the build date, which is what separates a clean stamp from a dirty one.
const FIRST_COMMIT: &str = "2024-03-04T10:00:00+09:00";
const SECOND_COMMIT: &str = "2024-03-05T10:00:00+09:00";

fn run_git(root: &Path, args: &[&str]) -> String {
    git(root, args).unwrap_or_else(|| panic!("git {args:?} failed in {}", root.display()))
}

fn commit(root: &Path, timestamp: &str, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .env("GIT_AUTHOR_DATE", timestamp)
        .env("GIT_COMMITTER_DATE", timestamp)
        .args(args)
        .output()
        .expect("git is required for provenance tests");
    assert!(output.status.success(), "git {args:?} failed: {output:?}");
}

/// A repository shaped like this workspace: executable sources under `crates/`
/// and documentation beside them.
fn repository() -> TempDir {
    let repository = TempDir::new().expect("temp dir");
    let root = repository.path();

    run_git(root, &["init", "--quiet", "--initial-branch", "main"]);
    run_git(root, &["config", "user.name", "Provenance Test"]);
    run_git(
        root,
        &["config", "user.email", "provenance@example.invalid"],
    );
    run_git(root, &["config", "commit.gpgsign", "false"]);

    fs::create_dir_all(root.join("crates/cli/src")).expect("create source dirs");
    fs::write(root.join("crates/cli/src/main.rs"), "fn main() {}\n").expect("write source");
    fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("write manifest");
    fs::write(root.join("README.md"), "# docs\n").expect("write docs");

    run_git(root, &["add", "."]);
    commit(
        root,
        FIRST_COMMIT,
        &["commit", "--quiet", "--message", "initial"],
    );

    repository
}

#[test]
fn clean_repository_is_stamped_by_its_commit() {
    let repository = repository();
    let root = repository.path();

    let stamp = Stamp::read(root);

    assert_eq!(stamp.dirty, "false");
    assert_eq!(
        stamp.version_commit,
        run_git(root, &["rev-parse", "--short", "HEAD"])
    );
    assert_eq!(stamp.commit_hash, run_git(root, &["rev-parse", "HEAD"]));
    assert_eq!(
        stamp.commit_date,
        run_git(root, &["log", "-1", "--date=short", "--format=%cd"])
    );
    assert_eq!(stamp.commit_date, "2024-03-04");
    assert_eq!(stamp.version_date, stamp.commit_date);
}

#[test]
fn changed_source_marks_the_stamp_dirty_and_dates_it_by_the_build() {
    let repository = repository();
    let root = repository.path();

    fs::write(root.join("crates/cli/src/main.rs"), "fn main() { }\n").expect("edit source");
    let stamp = Stamp::read(root);

    assert_eq!(stamp.dirty, "true");
    assert_eq!(
        stamp.version_commit,
        format!("{}-dirty", run_git(root, &["rev-parse", "--short", "HEAD"]))
    );
    assert_eq!(stamp.version_date, stamp.built_at[..10]);
    assert_eq!(stamp.commit_date, "2024-03-04");
    assert_ne!(
        stamp.version_date, stamp.commit_date,
        "a dirty stamp is dated by its build, not by its commit"
    );
}

#[test]
fn untracked_source_marks_the_stamp_dirty() {
    let repository = repository();
    let root = repository.path();

    fs::write(root.join("crates/cli/src/added.rs"), "\n").expect("add source");

    assert_eq!(Stamp::read(root).dirty, "true");
}

#[test]
fn changed_documentation_leaves_the_stamp_clean() {
    let repository = repository();
    let root = repository.path();

    fs::write(root.join("README.md"), "# docs, edited\n").expect("edit docs");
    fs::write(root.join("NOTES.md"), "untracked\n").expect("add docs");

    let stamp = Stamp::read(root);

    assert_eq!(stamp.dirty, "false");
    assert_eq!(stamp.version_date, "2024-03-04");
}

#[test]
fn repository_without_commits_falls_back_to_unknown() {
    let repository = TempDir::new().expect("temp dir");
    let root = repository.path();
    run_git(root, &["init", "--quiet", "--initial-branch", "main"]);

    let stamp = Stamp::read(root);

    assert_eq!(stamp.version_commit, "UNKNOWN");
    assert_eq!(stamp.version_date, "UNKNOWN");
    assert_eq!(stamp.commit_hash, "UNKNOWN");
    assert_eq!(stamp.commit_date, "UNKNOWN");
    assert_eq!(stamp.dirty, "UNKNOWN");
    // The build time does not depend on the repository.
    assert_ne!(stamp.built_at, "UNKNOWN");
}

/// A commit on a branch whose ref is packed creates the loose ref for the first
/// time. Watching the branch ref by name would watch nothing while it is
/// packed, so the commit would leave a stale `-dirty` stamp behind.
#[test]
fn packed_branch_ref_is_watched_before_its_loose_ref_exists() {
    let repository = repository();
    let root = repository.path();

    fs::write(root.join("crates/cli/src/main.rs"), "fn main() { }\n").expect("edit source");
    run_git(root, &["pack-refs", "--all", "--prune"]);

    let loose_ref = root.join(".git/refs/heads/main");
    assert!(!loose_ref.exists(), "the branch ref should be packed");
    let watched = rerun_paths(root);
    let before = Stamp::read(root);
    assert_eq!(before.dirty, "true");

    commit(
        root,
        SECOND_COMMIT,
        &["commit", "--quiet", "--all", "--message", "second"],
    );

    assert!(loose_ref.exists(), "the commit should create the loose ref");
    assert!(
        watched.iter().any(|path| loose_ref.starts_with(path)),
        "the new ref is not under a watched path: {watched:?}"
    );

    let after = Stamp::read(root);
    assert_eq!(after.dirty, "false");
    assert_eq!(after.version_date, "2024-03-05");
    assert_ne!(after.version_commit, before.version_commit);
    assert_eq!(
        after.version_commit,
        run_git(root, &["rev-parse", "--short", "HEAD"])
    );
}

#[test]
fn staging_and_source_state_are_watched() {
    let repository = repository();
    let root = repository.path();
    let watched = rerun_paths(root);

    for expected in [".git/HEAD", ".git/refs", ".git/index"] {
        assert!(
            watched.iter().any(|path| path.ends_with(expected)),
            "{expected} is not watched: {watched:?}"
        );
    }
    assert!(watched.iter().any(|path| path.ends_with("crates")));
    assert!(
        !watched.iter().any(|path| path.ends_with("README.md")),
        "documentation must not be watched: {watched:?}"
    );
}

#[test]
fn civil_from_days_matches_known_dates() {
    for (days, expected) in [
        (-25_567, (1900, 1, 1)),
        (-1, (1969, 12, 31)),
        (0, (1970, 1, 1)),
        (11_016, (2000, 2, 29)),
        (18_321, (2020, 2, 29)),
        (20_686, (2026, 8, 21)),
        (24_855, (2038, 1, 19)),
        (47_541, (2100, 3, 1)),
    ] {
        assert_eq!(civil_from_days(days), expected, "day {days}");
    }
}

#[test]
fn timestamps_render_in_the_given_offset() {
    for (offset, expected) in [
        (0, "2023-11-14T22:13:20+00:00"),
        (9 * 3600, "2023-11-15T07:13:20+09:00"),
        (-34_200, "2023-11-14T12:43:20-09:30"),
        (20_700, "2023-11-15T03:58:20+05:45"),
        (14 * 3600, "2023-11-15T12:13:20+14:00"),
        (-12 * 3600, "2023-11-14T10:13:20-12:00"),
    ] {
        assert_eq!(format_local_timestamp(1_700_000_000, offset), expected);
    }

    assert_eq!(
        format_local_timestamp(0, 0),
        "1970-01-01T00:00:00+00:00",
        "the epoch itself"
    );
    assert_eq!(
        format_local_timestamp(-1, 0),
        "1969-12-31T23:59:59+00:00",
        "before the epoch"
    );
}

#[test]
fn utc_offsets_are_parsed_or_rejected() {
    assert_eq!(parse_utc_offset("+0900"), Some(9 * 3600));
    assert_eq!(parse_utc_offset("-0930"), Some(-34_200));
    assert_eq!(parse_utc_offset("+0000"), Some(0));
    assert_eq!(parse_utc_offset("-0000"), Some(0));
    assert_eq!(parse_utc_offset("+1400"), Some(14 * 3600));

    for rejected in ["0900", "+09:00", "+090", "+09000", "+090a", "", "+", "Z"] {
        assert_eq!(parse_utc_offset(rejected), None, "{rejected:?}");
    }
}
