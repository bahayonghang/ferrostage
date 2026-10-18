# Verified tool-return records (not raw Cargo transcripts)

The following are exact returned summaries from commands already completed in this implementation turn. Earlier commands were not redirected, so these records explicitly distinguish the tool summary from a raw test log. Subsequent gates save raw command output and exit codes.

- `cargo fetch --manifest-path crates/ferrostage-pdf/Cargo.toml` (sandbox): exit 1, could not resolve index.crates.io. Approved retry: exit 0.
- `cargo check -p ferrostage-render --features pdfium --offline` (sandbox): exit 1, access denied opening target/debug/.cargo-build-lock.
- Approved compile retry: exit 1, E0308 i32 expected/u16 found at native.rs page lookup. Corrected to i32.
- Next approved compile: exit 1, E0507 owned Message moved in match guard. Corrected match body.
- `cargo test -p ferrostage-render --features pdfium --test native --offline -- --nocapture`: exit 0, tool output `cargo test: 1 passed (1 suite, 0.17s)`. Runtime cache/pdfium-7881-win-x64/bin/pdfium.dll.
- `cargo test -p ferrostage-pdf -p ferrostage-render --offline`: exit 0, tool output `cargo test: 11 passed (6 suites, 0.00s)`.

The initial native test source was subsequently expanded; its first pass is not evidence for later source changes. The final rerun logs and source hashes supersede these intermediate records.
