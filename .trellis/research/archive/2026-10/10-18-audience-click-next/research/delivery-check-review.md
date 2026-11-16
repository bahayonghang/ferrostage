# Independent delivery review

Reviewed the full `origin/main` to current working-tree product/workflow scope on 2026-10-18, including the inherited home/workbench/settings/display changes, audience input, remote platform/media repairs and authorized Linux D-Bus maintenance. No commit, push, merge, archive or desktop launch was performed by this reviewer.

## Findings (fixed)

- File: `vendor/ashpd/examples/screen_cast_pw.rs`
  - Issue: the D-Bus Value-to-Variant migration also changed `pw::spa::pod::Value::Object`, an unrelated PipeWire API, to `Variant::Object`.
  - Fix: restored the original PipeWire `Value` path. This is a source/API correction; the optional native PipeWire example was not compiled on this Windows host.
- File: `vendor/oo7/src/dbus/api/secret.rs`
  - Issue: `DBusSecret::serialize` discarded the first `serde::ser::SerializeTuple::serialize_element` result after migration, potentially swallowing a serialization error.
  - Fix: restored `?` propagation, matching the unchanged upstream serializer contract. The focused Linux-target library check passed afterward. The services worker regenerated the complete source diff/provenance receipts from these final files.

## Findings (not fixed)

- Direct standalone oo7 strict Linux clippy reports three `manual_filter` diagnostics in `src/file/api/mod.rs:55`, `:57`, `:60`. The same expressions exist verbatim in the original registry archive; the migration did not change them. They are outside this repair's source scope, so no unrelated cleanup or lint exemption was added. This extra vendor-only lint is FAIL; the required workspace and product strict lint gates both PASS. See `delivery-check-oo7-lint.log` and the successful separate type check in `delivery-check-oo7-type.log`.
- Actual Ubuntu GPUI consumer compilation and the Linux-only oo7 GVariant golden execution are CI-owned qualification gates. Cross-target checks on Windows establish Rust compatibility, not Linux desktop behavior. Live portal/keyring/notification/accessibility integration and the native PipeWire example remain NOT_RUN here. No missing runtime evidence is labeled PASS.
- Previously recorded audience first-inactive physical click and physical single-monitor native cases remain NOT_RUN as described in `verification.md`; inherited UI qualification was not expanded into a size/DPI matrix.

## Source and data-flow review

- Settings schema/parser/merge/control/runtime paths preserve optional-v1 defaults, unknown keys, future-schema read-only behavior and Unsaved/Retry. Audience input remains a payload-free outgoing event, deferred past its window lease, with committed handle/state/settings/modal/busy/blank checks before existing Next dispatch.
- Inherited font input captures paste text before deferred dispatch; display actual observations, configured allocation and pending draft remain distinct. Cancel affects allocation draft without undoing separately saved ordinary preferences. Existing Next, media invalidation, audience isolation and window lifecycle paths remain shared.
- Media repair adds the selected OpenH264 software closure rather than weakening frame/seek/pause/repeat/reap tests. Missing decoder messages now return Unsupported instead of accepting audio-only success for video. Independent inventory confirms all 49 required runtime destinations equal the manifest, every staged file matches its recorded size/SHA256, and 17 notice files exist. The media worker's 8 unit plus 11 native-test receipt remains the separate execution evidence.
- Compared migrated AccessKit/AT-SPI/lockstep consumer deltas with original registry sources, including maintained executor/task ownership, deferred builder errors, insertion return semantics and wire types. Services original archive hashes, original-file hashes and final maintained-file hashes were independently verified. Root lock contains no `zvariant`, `zvariant_derive` or `zbus_names`; deny policy was not weakened.
- Workflow automatically triggers only for pull requests and also exposes explicit workflow_dispatch. Cargo config arguments parse correctly; AT-SPI and oo7 filtered checks assert 1141 and 1 executed tests respectively. The portal wire test uses its locked standalone production-module fixture. Actionlint passes.

## Verification

| Gate | Result | Receipt |
| --- | --- | --- |
| `just ci` (fmt, strict workspace clippy, workspace tests, default deny) | PASS | `delivery-check-ci.log` |
| `just check-shell` (product build, strict product clippy, GPUI tests) | PASS; 143 tests, none ignored | `delivery-check-shell.log` |
| `cargo deny --all-features --locked check` | PASS; advisories, bans, licenses, sources | `delivery-check-deny-all.log` (compact receipt) |
| Focused oo7 Linux-target library check after serializer fix | PASS | `delivery-check-oo7-type.log` |
| Extra standalone oo7 strict Linux-target clippy | FAIL; 3 original upstream style diagnostics | `delivery-check-oo7-lint.log` |
| `actionlint -no-color .github/workflows/ci.yml` | PASS | Executed after final workflow changes |
| `git diff --check` | PASS | Executed after review fixes; line-ending conversion notices only |
| Services archive/original/final file hashes; media closure hashes | PASS | Independent archive and filesystem comparisons |

The first sandboxed CI attempt failed before compilation because mbx-managed cache/session paths were not writable. The authorized retry used the existing caches and process-scoped PowerShell execution-policy preference, then passed. No global execution-policy setting changed. The optional unused `atspi-connection` patch warning is documented: it retains the vendored atspi default/P2P API and is not selected by the current GPUI proxies-only feature graph.
