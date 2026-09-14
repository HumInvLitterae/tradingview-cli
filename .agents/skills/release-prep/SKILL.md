---
name: release-prep
description: Prepare tv release versions, notes, archives, and validation when a release or distribution change is requested.
---

# Release preparation

This is a contributor skill, excluded from runtime archives. Use the
[release packaging contract](../../../docs/release-packaging.md) and the current
release work record. A notes-only edit does not require a new release plan.

Confirm the requested version/scope, current diff, relevant commits, and existing
notes before editing. Inspect remote CI/release status only when it matters to
the task; local preparation does not establish publication.

| Work | Completion evidence |
| --- | --- |
| Version preparation | Cargo metadata and versioned artifacts agree on the requested version. |
| Changelog and curated notes | Explain user-visible changes; omit a redundant top-level tag heading from the release body. |
| Archive changes | Explicit runtime allowlist, complete references, matching agent guides, no development skills. Stage locally and inspect the result. |
| Release validation | Use the [development gates](../../../docs/development.md#validation-baseline) required by the candidate; distinguish the full release baseline from focused notes/package checks. |

Keep feature work, CI fixes, and release preparation in coherent separate
changes. Use the [commit convention](../../../docs/development.md#commit-messages)
within the assigned commit authority. Reuse valid verification; explain new
candidate drift before choosing which checks need to run again.

The user owns publication. Do not create tags, push, or create remote releases
unless explicitly requested in the current turn. Prepare the concrete local
result before a remaining publication decision. Keep account-local identifiers,
raw live payloads, credentials, and machine paths out of public artifacts.
