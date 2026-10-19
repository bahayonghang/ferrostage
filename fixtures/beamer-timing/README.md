# Compiled Beamer overlay and timing fixture

Original FerroStage clean-room source, using standard Beamer commands only. No implementation or assets from another presenter. `overlays-timing.pdf` was compiled twice with pdfTeX (TeX Live 2025), Beamer 3.72; normal Rust tests use the checked-in PDF and do not require TeX. Embedded fonts are Computer Modern and AMS fonts distributed with TeX Live; no external artwork is used.

```powershell
New-Item -ItemType Directory -Force cache/beamer-timing-build | Out-Null
pdflatex -interaction=nonstopmode -halt-on-error -no-shell-escape -output-directory=cache/beamer-timing-build fixtures/beamer-timing/overlays-timing.tex
pdflatex -interaction=nonstopmode -halt-on-error -no-shell-escape -output-directory=cache/beamer-timing-build fixtures/beamer-timing/overlays-timing.tex
Copy-Item cache/beamer-timing-build/overlays-timing.pdf fixtures/beamer-timing/overlays-timing.pdf
```

The PDF has 33 physical pages: 16 content steps in 6 logical slides and 17 independent notes pages. `expected.json` records page labels, authored duration and extracted public/private canaries. Notes labels are assigned in the source and are never patched into the compiled PDF.

| Logical slide | One-based content pages | Coverage |
| --- | --- | --- |
| 1 | 1, 3 | pause; per-overlay notes; two notes after last step |
| 2 | 6, 8, 10 | only, uncover, visible, onslide, alt, temporal |
| 3 | 12, 14, 16 | incremental list |
| 4 | 18, 20, 22, 24 | built-in animate and animatevalue; zero dwell then manual |
| 5 | 26, 28, 30 | positive dwell 1.5 s; zero dwell with Fade 0.3 s; absent dwell |
| 6 | 32 | zero dwell on final content page, must stop |

Positive `/Dur` begins after the destination transition has settled (PDF Reference 1.4 Figure 8.1). The separate animate package's JavaScript/OCG engines are outside this fixture and are not executed. Known script/layer actions receive bounded diagnostics; detection is not claimed to be exhaustive.

The Rust fixture tests validate mapping, notes isolation, every step in both directions, duration metadata and PDFium raster differences. Native UI presentation and standing-distance legibility remain separate acceptance evidence.
