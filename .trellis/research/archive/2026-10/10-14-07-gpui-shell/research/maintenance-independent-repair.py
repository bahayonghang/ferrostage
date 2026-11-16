"""Apply only the reviewed glyph-coverage repair after the frozen preparer."""
from pathlib import Path
import difflib
import hashlib
import json
import sys
import subprocess

ROOT = Path.cwd()
RESEARCH = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'
DEST = ROOT / (sys.argv[1] if len(sys.argv) > 1 else 'cache/gpui-maintenance-candidate')
assert DEST.resolve().is_relative_to((ROOT / 'cache').resolve())
assert DEST.name.startswith('gpui-maintenance-')
path = DEST / 'vendor/gpui/src/svg_renderer.rs'
old = path.read_text(encoding='utf-8')
if 'fn cmap2_glyph(' in old:
    raise SystemExit('Already repaired; run only against a fresh frozen reproduction.')
new = old.replace('use skrifa::MetadataProvider;', 'use skrifa::raw::TableProvider;')
start = new.index('fn font_has_char(')
end = new.index('\nfn select_emoji_font(', start)
new = new[:start] + (RESEARCH / 'maintenance-independent-font-helper.rs').read_text(encoding='utf-8') + '\n' + new[end:]
new += '\n' + (RESEARCH / 'maintenance-independent-cmap-tests.rs').read_text(encoding='utf-8')
path.write_text(new, encoding='utf-8', newline='\n')
subprocess.run(['rustfmt', '--edition', '2024', str(path)], check=True)
new = path.read_text(encoding='utf-8')
patch = ''.join(difflib.unified_diff(old.splitlines(True), new.splitlines(True),
                                  fromfile='a/gpui/src/svg_renderer.rs', tofile='b/gpui/src/svg_renderer.rs'))
patch_out = ROOT / sys.argv[2] if len(sys.argv) > 2 else RESEARCH
(patch_out / 'maintenance-independent-repair.patch').write_text(patch, encoding='utf-8', newline='\n')
print(json.dumps({'path':str(path), 'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}))
