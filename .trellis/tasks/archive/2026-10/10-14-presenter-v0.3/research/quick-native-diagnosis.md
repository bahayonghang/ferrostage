# Quick native observation diagnosis

Read-only independent check after all four GUI runs closed; no product changes, build or desktop input.

- Current image/media/grid extend below the default Presenter scroll viewport. workbench_view.rs calculates a minimum fixed main-content height from viewport minus360; media and thumbnails follow it. presenter.rs explicitly provides overflow_y_scroll. Screenshots establish below-fold clipping, not absent/unreachable controls. Full size/layout matrix remains user-deferred; no confirmed routing regression from these samples.
- Grid button dispatches Intent::Grid and Shell only toggles grid. Numeric Jump is a separate intent and modal. The observed Jump screenshot cannot establish a Grid routing failure, nor prove grid navigation passed. Keep NOT_VERIFIED.
- Video pause-settled screenshot retains a black-background frame with white ball; native poll retains the existing frame when no new frame arrives. No confirmed pause-clears-frame defect. Control label changed to Play at3/8s; extended frame-retention/loop/timing checks were not run.

Minimum source locations reviewed: crates/ferrostage-ui/src/presenter/workbench_view.rs, presenter.rs, shell.rs; crates/ferrostage-media native controller. No source repair is justified by this bounded evidence alone.
