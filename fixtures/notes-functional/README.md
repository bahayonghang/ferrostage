# Real Beamer notes functional fixture

This folder is the user-requested nonempty TeX/PDF acceptance input. The original Chinese/English deck introduces Pympress's dual-screen workflow and uses Beamer's standard `show notes on second screen=right` plus `note` commands. It does not contain copied or translated Pympress implementation code or assets. Pympress usage is a format reference, not a linked dependency or installed comparison app.

References checked2026-10-15:

- [Pympress public usage documentation](https://pympress.github.io/README.html#functionalities): second-screen notes, independently labelled notes and presenter/audience separation.
- [Official Beamer user guide](https://tug.ctan.org/macros/latex/contrib/beamer/doc/beameruserguide.pdf): sections19.3 and22.1 describe `note` and second-screen placement.

## Build

From this folder, using the existing TeX Live/XeLaTeX installation:

```powershell
New-Item -ItemType Directory -Force build | Out-Null
xelatex -interaction=nonstopmode -halt-on-error -no-shell-escape -output-directory=build pympress-notes.tex
xelatex -interaction=nonstopmode -halt-on-error -no-shell-escape -output-directory=build pympress-notes.tex
Copy-Item build/pympress-notes.pdf pympress-notes.pdf
```

The reviewed deliverable is `pympress-notes.pdf`; `build/` contains compiler receipts and `preview/` contains rendered validation pages. Source uses Windows CJK fonts through ctex; font availability is part of the build environment, not a product dependency.

## Expected mapping

| Physical PDF page | Logical frame / overlay | Left public marker | Right private marker |
| --- | --- | --- | --- |
|1|1 /1|FS-SLIDE-01|FS-NOTE-ONLY-01|
|2|2 /1|FS-SLIDE-02|FS-NOTE-ONLY-02|
|3|3 /1|FS-SLIDE-03-STEP-1|FS-NOTE-ONLY-03A|
|4|3 /2|FS-SLIDE-03-STEP-2|FS-NOTE-ONLY-03B|
|5|4 /1|FS-SLIDE-04|FS-NOTE-ONLY-04|

Each physical page is32:9: public16:9 slide on the left, notes on the right. Every page has unique content. Private markers and the final Chinese canary must be absent from audience crops. Test forward/backward overlay navigation, correct notes crop, and audience isolation separately.

XeLaTeX/pgfpages in this installed toolchain initially emitted labels for virtual slide and note pages, yielding incorrect physical labels1,1,2,2,3. This fixture disables those automatic labels and declares the five physical labels1,2,3,3,4 directly through xdvipdfmx's PDF catalog special in the TeX source. The PDF itself is never post-processed. This deterministic test mapping is not a claim that arbitrary third-party Beamer PDFs have correct labels. `verify_fixture.py` validates the compiled labels and side-specific text canaries using pypdf/pdfplumber; verification.json records identities and limitations.

## Acceptance boundaries

Recorded2026-10-15: compilation, all5 rendered pages, labels1/2/3/3/4 and side-specific text canaries PASS. The actual product PDFium integration test `crates/ferrostage-document/tests/notes_native.rs` also passes: Right layout,5physical/4logical, overlay grouping/navigation, full left/right pixel crop comparison and distinct notes. Evidence: `.trellis/tasks/10-14-07-gpui-shell/research/product-v15-document-isolated.json` and matching.log, pinned PDF SHA `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb`.

File compilation/visual review and document/PDFium crop tests prove the input and lower layers. They do not prove presenter UI notes display. Task07 currently renders page placeholders; real PDF/notes workbench is task08 and remains behind Gate B. Use this actual PDF for future native runs and record unavailable UI notes as NOT_IMPLEMENTED / NOT_RUN until implemented and visually tested. Do not turn the existing placeholder into a notes PASS.
