# Component Guidelines

A "component" is a GPUI view or element, not a React function component. There are no props objects and no JSX.

## Rules

- One view owns one window role (`WindowRole`).
- The audience view's render input is `AudienceProjectionSnapshot`.
- The presenter view may also read `ferrostage_document::SlideNotes`.
- Video is a `ferrostage_media::SharedFrame` drawn as a BGRA image. Do not embed a child window.
- Stable element ids, once GPUI is linked. Do not rebuild identity from a translated label.

## Not yet in the tree

GPUI types (`Render`, `Context`, `Window`) are intentionally absent so this crate builds without Zed. The first UI task adds `gpui-kit` 0.7.3 (or a git dependency on `zed-industries/zed` tag `v1.23.3`) and updates ADR-001.

## Forbidden

- React, `dioxus`, `egui`, or a webview for the slide surface.
- crates.io `gpui` 0.2.x.
- Reading the PDF file from a view. Ask `ferrostage-pdf` / `ferrostage-render`.
