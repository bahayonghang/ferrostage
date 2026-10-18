# Research: Full-feature CI dependency remediation

- Query: Can RUSTSEC-2026-0370/0371/0372 be fixed by supported upstream updates without disabling functionality, shrinking audit scope, adding waivers, or expanding the approved maintained GPUI source set?
- Scope: mixed; read-only product manifests and sources, public upstream metadata, isolated resolver probe.
- Date: 2026-10-18

## Findings

**No compatible lock-only repair exists in the currently published upstream graph.** All three advisories are maintenance notices, not vulnerability findings, and list no patched versions. The supported replacement is zbus 6.0.0 (`wire`, `names`, derive macros); GVariant users need the separate zgvariant package. Existing upstream consumers still select zbus 5 and the retired packages, including the newest oo7 prerelease. This is a dependency/API migration boundary, not a CI YAML error.

### Files found and current reverse paths

- `Cargo.toml:3,33-34,70-79`: six excluded vendor packages, maintained GPUI path, exact Zed v1.23.3 platform pin and patches.
- `deny.toml:4-6`: advisory v2 policy; no advisory ignore.
- `.trellis/spec/backend/quality-guidelines.md:18`: full product dependency audit is required; advisory policy must not be broadened.
- `docs/adr/ADR-008-gpui-dependency-gate.md:5-15,65`: approved six-package maintenance boundary; additional upgrades/advisory waivers are not in the earlier bounded adoption approval.
- `research/check-deny-all-features.log:5051-5209`: all three actual failures and reverse edges.
- `research/dependency-upstream-metadata.json`: current public crates.io release metadata and exact dependencies, including prerelease; refresh using `research/dependency-probe.py`.
- `research/dependency-resolver-zbus6.log`: Cargo's actual semver rejection of zbus 6 for oo7 0.6.

The common root is `ferrostage-app -> ferrostage-ui -> gpui_platform -> gpui_linux` at Zed revision `96a1b1b5` (the checkout also contains an unrelated `322cd50`; it was not used for the final source conclusions):

1. `gpui_linux -> accesskit_unix 0.22.1 -> accesskit_atspi_common 0.19.1 / atspi 0.29.0 -> atspi-common 0.13.0 / atspi-proxies 0.13.0 -> zbus 5 / zvariant 5 / zbus_names 4`; `atspi-common` also brings `zbus-lockstep(-macros) 0.5.2 -> zbus_xml 5.2.1 -> zbus_names/zvariant`.
2. `gpui_linux -> oo7 0.6.0 -> ashpd 0.13.13 / zbus 5.19 / zbus_macros 5.19 / zvariant 5.15`. oo7 uses GVariant and serde_bytes, so substituting only zbus's D-Bus wire module is insufficient.
3. `gpui_linux -> notify-rust 4.18.2 -> zbus 5.19`.
4. `zvariant 5.15 -> zvariant_derive 5.15` covers the third advisory.

Exact cached upstream source patterns at `C:/Users/lyh/.cargo/git/checkouts/zed-a70e2ad075855582/96a1b1b/`:

- `Cargo.toml:525`: accesskit_unix `0.22`; `533`: ashpd `0.13`; `702`: notify-rust `4`.
- `crates/gpui_linux/Cargo.toml:55`: accesskit_unix is an unconditional Linux/FreeBSD dependency; `74-79`: oo7 `0.6` with async-std/native_crypto and notify-rust; `83`: optional portal dependency ashpd used by x11/wayland features.
- Cached registry `oo7-0.6.0/Cargo.toml:154,262-278`: ashpd `^0.13`, zbus `^5.12.0`, zbus_macros `^5.11` with gvariant, zvariant `^5.8` with gvariant/serde_bytes.
- Cached registry `accesskit_unix-0.22.1/Cargo.toml:53-57,71-74,101-104`: shared AccessKit 0.24.1 types, atspi 0.29 and zbus 5.5/async-io.

### Current published upstream candidates

Verified directly against public crates.io API on this date; exact dependency records are persisted in the JSON evidence.

| Package | Latest stable / checked prerelease | Relevant dependency and outcome |
|---|---|---|
| zvariant / derive / zbus_names | 5.15.1 / 5.15.1 / 4.3.5 | Final deprecated releases; no patched versions |
| zbus | 6.0.0 | Replacement; MSRV 1.89, edition 2024 |
| ashpd | 0.13.13 | zbus `^5.13`; cannot eliminate advisories |
| oo7 | 0.6.0 | zbus `^5.12.0`, zvariant `^5.8`; cannot eliminate advisories |
| oo7 prerelease | 0.7.0-beta | Still zbus `^5.19`, ashpd `^0.13`, zgvariant `^1.0`; cannot eliminate advisories |
| notify-rust | 4.18.2 | zbus `^5`; alternative libdbus backend only addresses this one branch |
| accesskit_unix | 0.24.0 | Still zbus `^5.19`, accesskit_atspi_common `^0.21.0`, atspi `^0.29`; not a repair |
| accesskit_atspi_common | 0.21.0 | Still zvariant `^5.15`, atspi-common `^0.13`; not a repair |
| atspi / common / proxies | 0.30.0 / 0.14.0 / 0.14.0 | Still zbus 5; common retains zvariant 5, zbus_names 4, lockstep 0.5 |
| zbus-lockstep | 0.7.0 | Still zvariant `^5.13.0`, zbus_xml `^5.2.0` |
| zbus_xml | 6.0.0 | zbus 6, but lockstep's `^5.2.0` cannot select it |
| zgvariant | 2.0.0 | Maintained GVariant target; does not fix consumers still requesting zvariant/zbus5 |

### Resolver proof and minimal future implementation

An isolated manifest under `research/dependency-probe/` requests the existing oo7 0.6.0 configuration. After copying the root lock **only into that disposable directory**, this command fails with exit 1:

```powershell
cargo update --manifest-path .trellis/tasks/10-18-audience-click-next/research/dependency-probe/Cargo.toml --offline -p zbus:5.19.0 --precise 6.0.0
```

Cargo reports: requirement `zbus = "^5.12.0"`, candidate 6.0.0 does not match, required by oo7 0.6.0. The disposable copied lock was removed after the probe; root lock and manifests were not modified. This is an actual version-requirement rejection, not a network/cache inference.

There is no product file scope for a proven compliant immediate fix. A future complete source migration must cover every independent consumer above: accessibility/AT-SPI plus lockstep generation, portals/keyring including GVariant, and notifications. Bumping one root or adding zbus6 beside zbus5 leaves the advisory nodes reachable. Renaming retired packages or patching their names/versions does not constitute migration.

The smallest *potential* expansion would retain the current GPUI family and evaluate bounded downstream source patches to migrate all these consumers, documenting exact source identities, licenses and regressions. It is not yet a validated implementation proposal: additional maintained packages, changed feature/runtime APIs and Linux native testing are material scope. Do not adopt it automatically under a CI workflow edit. Otherwise wait for compatible upstream consumer releases and re-evaluate the same complete graph. No advisory exclusion or target restriction is a compliant substitute under current instructions.

MSRV is not the immediate blocker: checked latest candidates range through Rust 1.95, below project Rust 1.99. API/runtime risks are substantive: zbus wire/name/derive paths and data types change; GVariant separates into zgvariant2; async-io/async-std versus the zbus6 runtime feature model requires deliberate preservation; newer AccessKit versions use changed shared tree types and consumer versions. No compile/native claim follows from metadata resolution. Required future proof includes full all-feature audit, appropriate Linux build/lint, AT-SPI tree behavior, Secret Service/portal keyring behavior, X11/Wayland portals and notifications, with no removed functionality.

### Official external references

- [RustSec RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html), [0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html), [0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html): no patched versions; supported migration targets.
- [Upstream post-6 release tracking](https://github.com/z-galaxy/zbus/issues/2043): final deprecated releases, merge and GVariant separation.
- [zbus6 published manifest](https://docs.rs/crate/zbus/6.0.0/source/Cargo.toml) and [official 6.0.0 release notes](https://github.com/z-galaxy/zbus/releases/tag/zbus-6.0.0): MSRV, replacement package, runtime/proxy/service feature model; async-io is replaced by default-rt, blocking APIs/proxies are removed and Value becomes Variant.
- [oo7 stable published manifest](https://docs.rs/crate/oo7/0.6.0/source/Cargo.toml), [ashpd manifest](https://docs.rs/crate/ashpd/0.13.13/source/Cargo.toml), [notify-rust manifest](https://docs.rs/crate/notify-rust/4.18.2/source/Cargo.toml): exact current constraints.
- Public crates.io version/dependency API URLs for every candidate are embedded in `dependency-upstream-metadata.json`; these include newer prereleases absent from docs.rs latest-stable pages.

## Caveats / Not Found

- No compatible published update removes all three advisories; no complete source migration was compiled or product-tested. An unreviewed upstream branch is not evidence of an adopted solution.
- User scope decision is needed before expanding the approved maintained package set or changing advisory policy; policy exceptions are currently forbidden. The main session should keep this CI failure and any linked issue visibly unresolved until a real permitted remediation passes.
- Research cannot close the issue or mark the all-feature gate successful. Other remote failures belong to the separate platform/media investigations.
- Initial default sandbox network query was denied; read-only public release metadata subsequently succeeded with approved network access. The referenced RTK.md file does not exist in this environment.
