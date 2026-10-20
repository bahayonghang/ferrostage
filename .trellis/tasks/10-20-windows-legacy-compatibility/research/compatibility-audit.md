# Compatibility diagnosis

## Subsequent scope decision

The user dropped Windows 7 and requested Windows 10 and later. The current PRD/design supersede the legacy options below: qualify Windows 10 1903+ and Windows 11 x64 using the existing frontend; no legacy backend or 32-bit work. Original diagnosis remains evidence, not an active Win7 implementation plan.

## Result

The screenshot is explained by the explicit Windows 11 installer predicate. The requested Windows 7 support is a new runtime compatibility requirement: the existing release executable has mandatory imports absent from Windows 7, and the pinned GPUI Windows implementation requires newer graphics/text interfaces. Removing the predicate cannot produce a working Windows 7 release.

## Scope and evidence identity

- Repository inspected at `1317c23`, clean before task creation.
- Examined existing `target/dist/payload-1.1.0`, without rebuilding or installing it. This is local artifact evidence; identity with the user's downloaded Setup was not established.
- `ferrostage.exe` SHA256: `7e3d947f39541528587d5c8aa724bb26ae15fe9db1f0bf98ab20f63095152462`.
- `pdfium.dll` SHA256: `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`, matching `docs/pdfium-runtime.json:7`.
- Actual source dependency: Zed `96a1b1b5389efff473fc2306f7f8c690f2e1a417`; `gpui_windows` remains in the Cargo git checkout, separate from maintained `vendor/gpui`.
- Local host: Windows 11 x64, build 26300. Windows 7/8/8.1/10 runtime acceptance: NOT_RUN.

## Restriction matrix

| Boundary | Evidence | Consequence |
| --- | --- | --- |
| Setup | `packaging/windows/installer.nsi:123-125`, `AtLeastWin11` | Windows 10 and earlier are deliberately rejected; screenshot matches exact string. |
| Original product baseline | `docs/PROJECT_PLAN.md:56`, `docs/packaging.md:3` | Win11 was the original declared target; widening support changes the product baseline. |
| Rust | `rust-toolchain.toml:2`, Rust 1.99.0; [official baseline change](https://blog.rust-lang.org/2024/02/26/Windows-7/) | Regular `*-pc-windows-*` targets require Win10 starting at Rust 1.78. Edition 2024/project MSRV makes a casual downgrade inappropriate. |
| Legacy Rust route | [official Win7 MSVC target](https://doc.rust-lang.org/rustc/platform-support/win7-windows-msvc.html) | Tier 3 `x86_64-win7-windows-msvc` exists but has no distributed precompiled artifacts; self-built std/toolchain is only one prerequisite. |
| GPUI text initialization | Pinned `crates/gpui_windows/src/direct_write.rs:165-173`, mandatory `IDWriteFactory5` and in-memory font loader | [IDWriteFactory5 requires Windows 10 build 15063](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_3/nn-dwrite_3-idwritefactory5). Initialization cannot work unchanged on Win7/8/8.1. |
| GPUI device initialization | Pinned `crates/gpui_windows/src/directx_devices.rs:44-55,91-102`, mandatory `IDXGIFactory6` via `CreateDXGIFactory2` | [IDXGIFactory6](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_6/nn-dxgi1_6-idxgifactory6) is another later interface requirement; downgrading only font initialization is insufficient. |
| Mandatory executable imports | `ferrostage.exe-imports.txt`: `dcomp.dll`, `DCompositionCreateDevice`, `GetDpiForWindow`, `GetSystemMetricsForDpi`, `WaitOnAddress`, WinRT API sets, `icuuc.dll` | Win7 can fail in the OS loader before application fallback code runs. Dynamic lookup or separate compatible binaries are required wherever a newer import remains reachable. |
| Composition | Pinned `directx_renderer.rs:1294-1321`, composition creation and swap chain | [DCompositionCreateDevice requires Win8](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-dcompositioncreatedevice). Disabling composition at runtime alone does not remove its static import. |
| Window initialization | Pinned `window.rs:127`, plus `events.rs:796,995,1774-1781` | [GetDpiForWindow requires Win10 1607](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow). Multiple callers require an audit, not one patched call. |
| Upstream declared Windows support | Pinned `docs/src/installation.md:108-124`; [current upstream installation docs](https://zed.dev/docs/installation) | Win10 1903+ x64 is a reasonable qualification candidate for the current UI stack, not proof of FerroStage acceptance. |
| PDFium | Manifest version `151.0.7881.0`; inspected DLL imports only KERNEL32/ADVAPI32/GDI32/USER32 | No obvious post-Win7 blocker was established by the import list. DLL import names alone do not prove Win7 runtime compatibility, delay/dynamic API behavior, or CPU compatibility. Preserve this exact input pending tests; do not assume a downgrade is needed. |
| Deployment CRT | Existing EXE imports `VCRUNTIME140.dll` and UCRT API sets; packaging docs point to latest runtime | [Microsoft's current redistributable guidance](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist) limits latest v14 runtime to Win10/11 (and listed servers). Legacy work must pin and verify a compatible build/CRT combination or a reviewed static CRT arrangement. |
| Architecture | Current `windows-x64` payload, NSIS `RunningX64` gate | 32-bit Windows support is not implied by lowering the OS predicate; it is a separate target and native runtime qualification. |

## Same-pattern sweep

- Searched packaging, scripts, docs, workflows, Cargo manifests, maintained GPUI/AccessKit, and the exact pinned `gpui_windows` source.
- One explicit minimum-OS installer rejection found. `RunningX64` is a separate architecture restriction and remains relevant.
- Documentation baseline occurs in project plan and packaging docs; both must change with any approved support widening.
- Native restrictions span text initialization, DXGI factory creation, composition, display DPI, window creation/events, synchronization, WinRT/ICU and CRT. The listed examples prove incompatibility; this is not an exhaustive Win7 backport inventory.
- Optional media is absent from release features (`scripts/windows-distribution.ps1:152`, `gpui,pdfium`). It must not silently be advertised as Win7-qualified if enabled later.

## Options for the next scope decision

1. Qualify the existing frontend for Win10 1903+ x64 first, then work on a separate Win7 compatibility milestone. This can address the immediate Win10 rejection sooner, but the first delivery does not meet the user's full Win7+ requirement.
2. Treat Win7 as a required acceptance target before delivery. Prototype a legacy Rust target and compatible runtime closure, then compare a maintained `gpui_windows` backport with a separate compatible frontend. Preserve core/document/render semantics. This expands native maintenance beyond the current six-package Windows set and needs an explicit design before implementation.

Do not ship an installer labeled Win7-compatible until its actual payload passes old-system native acceptance. No product edit, build, test-suite run, installation or commit was performed during this planning audit. `git diff --check` passed for existing tracked changes; new planning files require their own final review.
