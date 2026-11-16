# Authorized delivery and CI repair

On 2026-10-18 the user expanded this session to: make automatic CI PR-only, split-commit all changes, archive the task, push, manually run CI, repair current remote failures, and close related issues. The user subsequently approved creating a dev -> main PR and merging after repairs. GitHub account/repository identity was verified as bahayonghang / bahayonghang/ferrostage; main is default, dev tracks origin/dev. No issue or PR was open at initial inspection.

## Confirmed failure baseline

Run 39122532405 at 8c53c8c (push/dev) failed in three jobs:

- Linux default clippy: ten dead_code errors from unconditional platform friendly module whose production callers are Windows-only.
- Windows media: actual_helper_video_seek_repeat_pause_audio_and_reap times out; four later native tests are poisoned-lock cascades. Root cause must be established by actual helper/runtime evidence, not timeout inflation or poison recovery.
- Windows full-feature audit: RUSTSEC-2026-0370/0371/0372, three unmaintained Linux D-Bus dependencies. No vulnerabilities were identified by these three notices, but existing policy rejects them.

## Policy decision

The user explicitly chose expanding maintained Linux dependencies and migrating the chain. They did not approve an advisory exception. Keep the full dependency graph, vulnerability and license rules, existing GPUI source family, and Windows AccessKit repair. Upstream metadata and isolated Cargo resolution prove a simple lockfile update is insufficient; exact source/API migrations and provenance are owned by the migration workers and will be reviewed.

## Delivery order

1. Commit reviewed audience input feature plus its tests/specs as one atomic feature commit (a24e4c3).
2. Commit PR/manual workflow trigger policy and reproducible locked commands separately.
3. Commit platform target-gating fix and media root-cause fix separately with relevant verification.
4. Commit maintained Linux D-Bus migration coherently, with source provenance, dependency graph/audit and Linux compile evidence; no retired-source rename to evade audit.
5. Commit task artifacts, archive with branch dev/base main matching the subsequently approved PR target, then record session. No destructive branch cleanup/rewrite.
6. Push dev, create/attach dev -> main PR; inspect actual PR CI and repair any new failures in follow-up commits. Merge only once required checks are green. Then manually dispatch the CI now published on default main and inspect its actual completion.
7. Re-read issue state; close only matching resolved reports with evidence. An empty issue list is not a reason to invent or close unrelated items. Historical failed runs remain history.

## Scope/verification boundaries

Only real execution establishes Linux migrated-code compatibility; a Windows audit or build alone does not. Native desktop QA for the earlier audience feature remains in verification.md. Any unavailable/native-only qualification is recorded separately. Raw remote logs are downloaded into the OS temporary directory; only bounded diagnostic evidence is retained here. No private security advisory API access is needed or used.
