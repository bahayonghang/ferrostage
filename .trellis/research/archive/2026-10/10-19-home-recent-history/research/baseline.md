# Home baseline and diagnosis

The user supplied a native home screenshot with three recent-file entries and the mixed-language message `无法读取 PDF: lopdf could not parse the buffer`. The existing home has no time metadata, column header or row separators; filename/path pairs read as undifferentiated text. Preserve its restrained warm palette, dedicated wordmark, user font choice and natural-width Open action.

Current source: `presenter/home_view.rs` composes first-three path rows from PresenterData, with full-path tooltip/focus text and shared activation. `Shell::await_import_read` parses in the background, wraps `PdfStructureError` into `std::io::Error::other`, then interpolates Display text through `read-error`. That loses the type boundary and exposes parser-internal English. The library's Unreadable variant is shared by invalid/truncated/unsupported-to-parser bytes; it cannot establish that a compiler is running.

Read-only bounded sampling of the user's recent `defense.pdf` found identical size/mtime in two samples 400ms apart and an EOF marker in the last 1024 bytes. No content, speaker notes or compiler logs were inspected. This describes current state only and neither proves full PDF validity nor reconstructs the screenshot's earlier failure.

## Live native visual baseline

Computer Use initialized successfully through the documented sky API. No FerroStage window existed initially, so root opened the installed executable and captured a normal native home with the real current settings. The returned image was 1103x792 including native frame, corresponding to the existing approximately 1100x760 logical client.

The helper attributes the launched process to its inherited parent AppUserModelID; root independently verified the returned HWND 9769390 belongs to the installed `ferrostage.exe` (PID199076), then used that exact returned window. This is the application's own native window, not the Codex UI. Accessibility text was empty; the screenshot itself is the visual oracle. The image shows the configured serif interface typography, a shared left edge around x176 for brand/content, and three filename/path row pairs around y315-460. No timestamp column exists in this baseline. Precise source-bound row/column measurements will come from GPUI regression geometry.

The baseline window was opened only for this task; no history entry was activated, file dialog opened or Windows display setting changed. Root owns subsequent native capture and cleanup. Native screenshot output is visible in the conversation; no screenshot payload was reprocessed for inspection.

## Verification boundary

User preference and current product specs prioritize core behavior and ordinary-window visual smoke. Reuse existing checks; do not introduce a size, DPI or performance matrix. History metadata must come from actual successful import, not filesystem timestamps or guessed migration times. Legacy unknown metadata must remain unknown.
