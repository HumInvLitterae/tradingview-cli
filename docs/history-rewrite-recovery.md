# Canonical history rewrite recovery

Status: canonical rewrite and post-rewrite documentation closeout are complete.
Fresh clones now receive the rewritten history. The maintainer primary clone has
also been realigned after backup and exact remote-manifest verification. Do not
run the optional existing-clone realignment commands below without first
confirming a clean worktree, a private backup, and the current canonical
manifest.

The `v0.26.0` preparation rewrote canonical `main` and release tags to remove a
machine-specific path from reachable Git history. File content at the new tip
is unchanged, and existing GitHub Release objects and assets remain in place.
Commit and tag-target identifiers changed because rewriting an old commit
changes every descendant identifier.

The safest recovery is a fresh clone. Do not pull or merge rewritten history
into an old clone. Preserve the old clone until you have audited and recovered
all local-only work.

## Inspect and back up the old clone

Run these commands in the old clone before changing it:

```bash
git status --short --branch
git branch -vv
git tag --list
git worktree list
git submodule status --recursive
git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}'
git log --oneline --decorate '@{upstream}..HEAD'
git ls-files --others --exclude-standard
```

Expected safe state:

- `git status` identifies every staged, unstaged, and untracked change.
- The upstream command succeeds and the log lists every unpushed commit. If no
  upstream is configured, treat the entire branch as local-only.
- `git worktree list` and the submodule command identify other checkouts that
  may still reference old commit IDs.

Create backups outside the clone. A bundle preserves committed refs, while the
two patches preserve staged and unstaged tracked-file changes:

```bash
git bundle create ../tradingview-cli-before-rewrite.bundle --all
git bundle verify ../tradingview-cli-before-rewrite.bundle
git diff --binary > ../tradingview-cli-before-rewrite-worktree.patch
git diff --cached --binary > ../tradingview-cli-before-rewrite-index.patch
```

Copy any required untracked files separately. Keep the bundle private: it
contains the old history that the canonical repository is removing.

## Recommended fresh-clone recovery

Save the canonical URL while still in the old clone, then clone into a new
sibling directory:

```bash
CANONICAL_URL="$(git remote get-url origin)"
cd ..
git clone "$CANONICAL_URL" tradingview-cli-fresh
cd tradingview-cli-fresh
git status --short --branch
python scripts/check-public-hygiene.py
```

Expected result: the fresh clone is on `main`, the worktree is clean, and the
public hygiene check passes. Keep using this clone as the canonical checkout.
Do not add the old clone as a remote and do not merge or pull its old `main`.

Move local work by content, not by joining the two histories. For an unpushed
commit, export and inspect one patch at a time from the old clone:

```bash
OLD_COMMIT=replace-with-reviewed-old-commit-id
git show --binary --format=email "$OLD_COMMIT" > ../candidate.patch
```

Then, in the fresh clone, inspect and apply it:

```bash
git apply --stat ../candidate.patch
git apply --check ../candidate.patch
git apply ../candidate.patch
python scripts/check-public-hygiene.py
git diff --check
```

Review the resulting diff before committing. Repeat only for work that remains
necessary. A direct cherry-pick is possible but should be used only after
inspecting the old commit, because it can restore content intentionally removed
by the rewrite.

Old commit URLs, review links, bookmarks, local branches, linked worktrees, and
submodule pins may still name old identifiers. Update them deliberately. A
submodule consumer must receive a normal commit that points to a rewritten
submodule identifier; do not silently reset another repository's submodule pin.

## Optional existing-clone realignment

Fresh clone recovery is preferred. Use this alternative only when the old clone
must be retained and its worktree is clean. The commands intentionally separate
fetch, inspection, and destructive reset.

First verify the worktree and create another local backup:

```bash
test -z "$(git status --porcelain=v1)"
test "$(git branch --show-current)" = main
git bundle create ../tradingview-cli-realignment-backup.bundle --all
git bundle verify ../tradingview-cli-realignment-backup.bundle
```

Expected result: both `test` commands return success and bundle verification
reports a valid bundle. Stop if either test fails.

Fetch rewritten refs into a temporary namespace without overwriting local
branches or tags:

```bash
TAG_MANIFEST=../tradingview-cli-rewritten-tags.txt
git ls-remote --tags --refs origin | LC_ALL=C sort -k2 > "$TAG_MANIFEST"
git fetch --no-tags origin \
  +refs/heads/main:refs/history-rewrite/main
while read -r expected_oid tag_ref; do
  git fetch --no-tags origin \
    "+${tag_ref}:refs/history-rewrite/${tag_ref}"
done < "$TAG_MANIFEST"
git log --oneline --decorate -5 refs/history-rewrite/main
git for-each-ref --format='%(objectname) %(refname)' \
  refs/history-rewrite/refs/tags
remote_tag_count="$(wc -l < "$TAG_MANIFEST" | tr -d ' ')"
temporary_tag_count="$(git for-each-ref --format='%(refname)' \
  refs/history-rewrite/refs/tags | wc -l | tr -d ' ')"
test "$temporary_tag_count" = "$remote_tag_count"
```

Expected result: the log shows the announced rewritten tip, and the temporary
tag count equals the line count in `TAG_MANIFEST`. Inspect these results before
continuing.

The following block changes local `main`, `origin/main`, and only the canonical
tag names listed by the remote manifest. Run it only after a separate manual
confirmation that the backup and fetched refs are correct:

```bash
git branch history-rewrite-backup-main main
git reset --hard refs/history-rewrite/main
git update-ref refs/remotes/origin/main \
  "$(git rev-parse refs/history-rewrite/main)"
while read -r expected_oid tag_ref; do
  rewritten_oid="$(git rev-parse "refs/history-rewrite/${tag_ref}")"
  test "$rewritten_oid" = "$expected_oid"
  git update-ref "$tag_ref" "$rewritten_oid"
done < "$TAG_MANIFEST"
git status --short --branch
python scripts/check-public-hygiene.py
```

Expected result: `main` is clean and agrees with `origin/main`, every canonical
tag equals its manifest OID, and the hygiene guard passes. Local-only branches
and tags are not deleted. Keep the backup branch and bundle until all recovered
work, worktrees, and submodule consumers have been checked.

## Limits

The rewrite changes canonical reachability. It cannot remove old objects from
third-party clones, forks, quoted logs, external caches, or old links. Do not
interpret a missing old identifier as loss of release assets; check the GitHub
Release page by tag name instead.
