# Design token contrast — 2026-10-14

Source: root DESIGN.md, brand:claude adapted for FerroStage. WCAG sRGB relative luminance, (L1+0.05)/(L2+0.05). This is a token calculation, not a screenshot or native accessibility pass.

| Foreground | Background | Required | Ratio | Result |
| --- | --- | --- | --- | --- |
| ink | canvas | 4.5 | 17.498 | PASS |
| ink | surface-soft | 4.5 | 16.250 | PASS |
| ink | surface-card | 4.5 | 15.257 | PASS |
| body | canvas | 4.5 | 10.344 | PASS |
| body | surface-soft | 4.5 | 9.606 | PASS |
| body | surface-card | 4.5 | 9.019 | PASS |
| muted | canvas | 4.5 | 5.437 | PASS |
| muted | surface-soft | 4.5 | 5.049 | PASS |
| muted | surface-card | 4.5 | 4.740 | PASS |
| success | canvas | 4.5 | 6.952 | PASS |
| success | surface-soft | 4.5 | 6.456 | PASS |
| success | surface-card | 4.5 | 6.061 | PASS |
| warning | canvas | 4.5 | 6.235 | PASS |
| warning | surface-soft | 4.5 | 5.790 | PASS |
| warning | surface-card | 4.5 | 5.436 | PASS |
| error | canvas | 4.5 | 6.381 | PASS |
| error | surface-soft | 4.5 | 5.926 | PASS |
| error | surface-card | 4.5 | 5.564 | PASS |
| border | canvas | 3.0 | 3.722 | PASS |
| border | surface-soft | 3.0 | 3.456 | PASS |
| border | surface-card | 3.0 | 3.245 | PASS |
| focus | canvas | 3.0 | 5.348 | PASS |
| focus | surface-soft | 3.0 | 4.967 | PASS |
| focus | surface-card | 3.0 | 4.663 | PASS |
| ink-dark | canvas-dark | 4.5 | 17.005 | PASS |
| ink-dark | surface-soft-dark | 4.5 | 15.823 | PASS |
| ink-dark | surface-card-dark | 4.5 | 14.877 | PASS |
| body-dark | canvas-dark | 4.5 | 13.574 | PASS |
| body-dark | surface-soft-dark | 4.5 | 12.630 | PASS |
| body-dark | surface-card-dark | 4.5 | 11.875 | PASS |
| muted-dark | canvas-dark | 4.5 | 8.596 | PASS |
| muted-dark | surface-soft-dark | 4.5 | 7.998 | PASS |
| muted-dark | surface-card-dark | 4.5 | 7.520 | PASS |
| success-dark | canvas-dark | 4.5 | 9.784 | PASS |
| success-dark | surface-soft-dark | 4.5 | 9.104 | PASS |
| success-dark | surface-card-dark | 4.5 | 8.559 | PASS |
| warning-dark | canvas-dark | 4.5 | 9.774 | PASS |
| warning-dark | surface-soft-dark | 4.5 | 9.094 | PASS |
| warning-dark | surface-card-dark | 4.5 | 8.550 | PASS |
| error-dark | canvas-dark | 4.5 | 9.505 | PASS |
| error-dark | surface-soft-dark | 4.5 | 8.844 | PASS |
| error-dark | surface-card-dark | 4.5 | 8.315 | PASS |
| border-dark | canvas-dark | 3.0 | 5.181 | PASS |
| border-dark | surface-soft-dark | 3.0 | 4.821 | PASS |
| border-dark | surface-card-dark | 3.0 | 4.533 | PASS |
| focus-dark | canvas-dark | 3.0 | 8.482 | PASS |
| focus-dark | surface-soft-dark | 3.0 | 7.893 | PASS |
| focus-dark | surface-card-dark | 3.0 | 7.421 | PASS |
| on-primary | primary | 4.5 | 5.634 | PASS |
| on-primary | primary-hover | 4.5 | 6.703 | PASS |
| on-primary | primary-active | 4.5 | 7.992 | PASS |

Hairline is decorative only. Focus ring uses a surface gap around primary buttons. Dark primary controls use border-dark for a visible boundary. PDF/media pixels and content colors are not part of UI text token evaluation.

Native screenshots, actual font fallback, keyboard/focus behavior, hover/disabled/error/loading, system switching, narrow window and mixed-DPI checks: NOT_RUN. Owned by 07/08/09; no UI verified claim.
