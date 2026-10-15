# Task06 primary media field contract

Reviewed the cached **Adobe PDF Reference 1.7** and **Adobe Supplement, BaseVersion 1.7 / ExtensionLevel 3 (June 2008)**, not another viewer's implementation. Source PDFs and hashes are in `fixture-provenance.json`; parent `native-research-resume.md` records acquisition.

- Adobe reference: https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf
- Adobe supplement, archived original linked by the PDF Association specification archive: https://web.archive.org/web/20220306152229id_/https://www.adobe.com/content/dam/acom/en/devnet/pdf/adobe_supplement_iso32000.pdf

| Subset | Exact path / interpretation | Primary reference |
| --- | --- | --- |
| Movie source | annotation `/Movie/F`: file specification string or dictionary; `/UF` before `/F`; `/EF/UF` or `/EF/F` decoded embedded stream | PDF1.7 table9.30; section3.10 |
| Movie times | annotation `/A/Start`, `/A/Duration`: nonnegative integer or 8-byte signed big-endian integer; `[value, positive integer scale]` permits seconds conversion. Bare values retain `Unscaled(ticks)` and diagnostic. No inferred movie time scale. | table9.31 |
| Movie repeat | `/A/Mode`: Repeat -> Forever, Once/Open -> Count(1); Palindrome and unknown modes unresolved | table9.31 |
| Movie activation | page `/AA/O` or any same-page annotation `/AA/PO` or `/AA/PV` with `/S/Movie`, unique `/Annotation` reference or unique Movie `/T` title, absent Operation or `/Operation/Play`. Action timing/mode override `/A` fields. Chains/multiple matching actions are diagnosed. | tables8.44,8.45,8.59 |
| Screen action | own `/AA/PO` or `/AA/PV`, else `/A`; `/S/Rendition`, `/OP 0`, `/AN` points to this annotation. JS/Next unsupported. Click `/A` alone leaves autostart unresolved. | table8.64 |
| Screen source/start | action `/R/S/MR`, `/R/C/S/MCD`, `/R/C/D` full Filespec; alternatively one `/R/C/S/MCS` wrapper with `/MH` or `/BE` `/B << /S/T /T << /S/S /V seconds >> >>`, nested `/D` is MCD. Nested sections, end offsets, frame/marker offsets unsupported. | tables9.6,9.9,9.12,9.13,9.20,9.21,9.24 |
| Screen duration/repeat/play | rendition `/P/MH` or `/P/BE`: `/D << /S/T /T << /S/S /V seconds >> >>`, `/RC` nonnegative number (zero forever), `/A` boolean play-on-activation. MH wins per key. Missing explicit values are not presented as recovered values. | tables9.14-16,9.24 |
| RichMedia selection | annotation `/RichMediaSettings/Activation/Configuration`, which must occur in `/RichMediaContent/Configurations`; if absent select first configuration per specification | Extension3 tables9.50a,9.51 |
| RichMedia source | selected configuration `/Instances` has exactly one `/Subtype/Video` or `/Sound` dictionary; `/Asset` must also occur in `/RichMediaContent/Assets` name tree, then file spec decoding as above | tables9.51a,9.51b |
| RichMedia activation | `/RichMediaSettings/Activation/Condition`: PO/PV true, XA false; missing/unknown unresolved. Timing/repeat remain None with unsupported diagnostic. | table9.50a |
| RichMedia exclusion | `/Params` and `/FlashVars` are Flash-specific, not generic Video start/duration/loop fields. Flash/3D, scripts, animations and scene aggregates are not supported. | table9.51b/c |

All annotation `/Rect` values start in unrotated PDF user space, bottom-left origin. The page's inherited MediaBox and CropBox are intersected; inherited Rotate is normalized modulo360. Rectangles are clipped, converted to top-left coordinates, rotated clockwise and divided by visible size. Document then clips/rescales to the selected slide crop. Zero/outside/nonfinite rectangles are diagnostic; missing/cyclic/invalid page geometry is a typed error. Non-default page UserUnit is a typed unsupported error. Native metadata and structure geometry must agree within0.05pt, with matching rotation and contiguous physical IDs.

Sources are metadata only. No external media is opened, scripts/actions are never executed, and no user PDF is written. Embedded decoding has a64MiB per-asset limit; decoding failure is diagnostic. This does not validate codec playability, security policy, movie duration, every PDF producer, or the media backend. Original PDF Annots ordinals are diagnostic locations; document media ordinals are local slot positions, not persistent cross-generation identities.

Independent review tightened diagnostics within this subset: a malformed/unresolved Filespec `/EF` does not fall back to an external filename (PDF null still means absent), and unsupported Movie `/Rate` or `/FWScale` fields are diagnosed in selected activation actions as well as annotation `/A`. No new media field is interpreted or executed.

Actual Beamer3.72 movie output has `/Start [256 128]`, `/Duration [1024 128]`, `/Mode/Repeat` and a separate Widget `/AA/PO` Movie action targeting `mmdefaultlabel1`. That title is unique in the fixture. Native and structure tests recover2s/8s/Forever/autostart true. The emitted fields were inspected before adding that supported activation path; no producer implementation source was read.
