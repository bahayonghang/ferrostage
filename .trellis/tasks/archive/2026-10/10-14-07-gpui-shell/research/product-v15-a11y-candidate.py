"""Research-only AccessKit candidate. Never changes the product dependency graph."""
from pathlib import Path
from datetime import datetime, timezone
import difflib
import hashlib
import json
import os
import shutil
import subprocess
import sys

research = Path(__file__).resolve().parent
upstream = Path(os.environ['USERPROFILE']) / '.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/accesskit_windows-0.34.0'
candidate = research / 'product-v15-accesskit-candidate'
mode = sys.argv[1]
if mode == 'prepare':
    candidate.mkdir(exist_ok=False)
    shutil.copytree(upstream / 'src', candidate / 'src')
    manifest = (upstream / 'Cargo.toml').read_text()
    manifest = manifest[:manifest.index('[dev-dependencies.once_cell]')]
    manifest = manifest.replace('[[example]]\nname = "hello_world"\npath = "examples/hello_world.rs"\n\n', '')
    (candidate / 'Cargo.toml').write_text(manifest + '\n[workspace]\n\n[features]\nroot-native-window = []\n')
    shutil.copy2(Path.cwd() / 'Cargo.lock', candidate / 'Cargo.lock')
    path = candidate / 'src/lib.rs'
    path.write_text(path.read_text().replace('mod tests;', 'mod v15_research_tests;'))
    (candidate / 'src/v15_research_tests.rs').write_text('''// Research harness: real Windows calls, no window creation or desktop mutation.
use crate::{util::client_top_left, window_handle::WindowHandle};
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::GetDesktopWindow};

#[test]
fn retired_hwnd_coordinate_query_does_not_panic() {
    let result = std::panic::catch_unwind(|| {
        // A permanently invalid native handle exercises the same Windows error
        // as the captured post-destruction ClientToScreen failure.
        let _ = client_top_left(WindowHandle(HWND(-1isize as *mut _)));
    });
    assert!(result.is_ok(), "retired HWND must return a provider error, not unwind");
}

#[test]
fn live_desktop_coordinate_query_still_succeeds() {
    let result = std::panic::catch_unwind(|| {
        let _ = client_top_left(WindowHandle(unsafe { GetDesktopWindow() }));
    });
    assert!(result.is_ok());
}
''')
    originals = {name: hashlib.sha256((upstream / name).read_bytes()).hexdigest()
                 for name in ['Cargo.toml', 'src/util.rs', 'src/context.rs', 'src/node.rs', 'src/text.rs']}
    (research / 'product-v15-a11y-upstream.json').write_text(json.dumps({
        'path': str(upstream), 'version': '0.34.0', 'license': 'MIT OR Apache-2.0',
        'registry_checksum': '106c2b961215864d1c2e703ee63269c25c4e80a577ffb2c1017b9c17dcdf83a1',
        'vcs': json.loads((upstream / '.cargo_vcs_info.json').read_text()), 'files': originals,
        'scope': 'isolated research only; no product patch or dependency adoption',
        'harness_changes': 'Original GUI tests are not run: test module redirects to two native leaf calls; dev dependencies/examples removed only in standalone research manifest.'
    }, indent=2) + '\n')
elif mode == 'patch':
    path = candidate / 'src/util.rs'
    s = path.read_text()
    s = s.replace('pub(crate) fn client_top_left(hwnd: WindowHandle) -> Point {', 'pub(crate) fn client_top_left(hwnd: WindowHandle) -> Result<Point> {')
    s = s.replace('''    // If ClientToScreen fails, that means the window is gone.
    // That's an unexpected condition, so we should fail loudly.
    unsafe { ClientToScreen(hwnd.0, &mut result) }.unwrap();
    Point::new(result.x.into(), result.y.into())''', '''    // A UIA call may already hold Context while the native owner is destroyed.
    // Check the conversion itself, not an earlier racy IsWindow observation.
    unsafe { ClientToScreen(hwnd.0, &mut result) }.ok().map_err(|error| {
        if error.code() == HRESULT::from_win32(ERROR_INVALID_WINDOW_HANDLE.0) {
            element_not_available()
        } else {
            error
        }
    })?;
    Ok(Point::new(result.x.into(), result.y.into()))''')
    path.write_text(s)
    path = candidate / 'src/context.rs'
    path.write_text(path.read_text().replace('fn client_top_left(&self) -> Point {', 'fn client_top_left(&self) -> windows::core::Result<Point> {'))
    path = candidate / 'src/node.rs'
    s = path.read_text().replace('let client_top_left = context.client_top_left();', 'let client_top_left = context.client_top_left()?;')
    s = s.replace('let rect = node.bounding_box().map_or(UiaRect::default(), |rect| {', 'let rect = if let Some(rect) = node.bounding_box() {')
    s = s.replace('''                    height: rect.height(),
                }
            });
            Ok(rect)''', '''                    height: rect.height(),
                }
            } else {
                UiaRect::default()
            };
            Ok(rect)''')
    path.write_text(s)
    path = candidate / 'src/text.rs'
    path.write_text(path.read_text().replace('let client_top_left = context.client_top_left();', 'let client_top_left = context.client_top_left()?;'))
    path = candidate / 'src/v15_research_tests.rs'
    s = path.read_text().replace('let _ = client_top_left(WindowHandle(unsafe { GetDesktopWindow() }));', 'let point = client_top_left(WindowHandle(unsafe { GetDesktopWindow() })).unwrap();\n        assert!(point.x.is_finite() && point.y.is_finite());')
    s += '''
#[test]
fn retired_hwnd_coordinate_query_returns_element_not_available() {
    let error = client_top_left(WindowHandle(HWND(-1isize as *mut _))).unwrap_err();
    assert_eq!(error.code(), crate::util::element_not_available().code());
}
'''
    path.write_text((research / 'product-v15-a11y-tests.rs').read_text())
    patch = ''
    for name in ['src/util.rs', 'src/context.rs', 'src/node.rs', 'src/text.rs']:
        patch += ''.join(difflib.unified_diff((upstream / name).read_text().splitlines(True), (candidate / name).read_text().splitlines(True), fromfile='a/' + name, tofile='b/' + name))
    (research / 'product-v15-accesskit-proposal.patch').write_text(patch)
elif mode in ('red', 'green', 'hidden-compile', 'root-hidden'):
    suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
    log = research / f'product-v15-a11y-{mode}{suffix}.log'
    receipt = log.with_suffix('.json')
    assert not log.exists() and not receipt.exists()
    command = ['cargo', 'test', '--manifest-path', str(candidate / 'Cargo.toml'), '--lib', '--offline', 'v15_research_tests', '--', '--test-threads=1', '--nocapture']
    if mode != 'red':
        command.insert(command.index('--lib'), '--locked')
    if mode == 'hidden-compile':
        command = command[:command.index('--')] + ['--features', 'root-native-window', '--no-run']
    elif mode == 'root-hidden':
        command = command[:command.index('v15_research_tests')] + ['--features', 'root-native-window', 'v15_research_tests::root_only_owned_hidden_hwnd_destroy_with_retained_context', '--', '--exact', '--test-threads=1', '--nocapture']
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(Path.cwd() / 'target')
    record = {'started_utc': datetime.now(timezone.utc).isoformat(), 'argv': command,
              'status': 'RUNNING', 'scope': ('ROOT ONLY: sixteen owned hidden-window destroy cycles, no product dependency change' if mode == 'root-hidden' else 'isolated upstream research only; no windows created, no GUI interaction, no product dependency changed')}
    receipt.write_text(json.dumps(record, indent=2) + '\n')
    with log.open('x', encoding='utf-8') as stream:
        result = subprocess.run(command, env=env, stdout=stream, stderr=subprocess.STDOUT)
    record.update(status='RETURNED', exit_code=result.returncode, ended_utc=datetime.now(timezone.utc).isoformat())
    receipt.write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps(record, indent=2))
    print('\n'.join(log.read_text(errors='replace').splitlines()[-45:]))
    sys.exit(result.returncode)
else:
    raise ValueError(mode)
