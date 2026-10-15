"""Seal Root's V16 core-native observations without modifying old evidence."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import re
import tomllib

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
native = research / 'native'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def save(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write('\n')


freeze_path = research / 'product-native-freeze-v16.json'
freeze = load(freeze_path)
mismatches = [name for name, digest in freeze['source_files'].items() if sha(root / name) != digest]
assert not mismatches, mismatches
artifacts = {name: sha(Path(freeze[name]['path'])) for name in ('binary', 'pdfium', 'fixture')}
assert all(artifacts[name] == freeze[name]['sha256'] for name in artifacts)
runs = {}
for tag in ('v16-core-main', 'v16-core-relaunch'):
    launch = load(native / f'{tag}.launch.json')
    exit_record = load(native / f'{tag}.exit.json')
    inventory = load(native / f'{tag}-post-exit-windows.json')
    logs = {kind: (native / f'{tag}.{kind}.log').read_text(encoding='utf-8', errors='replace')
            for kind in ('stdout', 'stderr')}
    diagnostics = {kind: [line for line in text.splitlines() if re.search(
        r'(?i)panicked|\bERROR\b|failed|window not found|invalid window|cannot unwind', line)]
        for kind, text in logs.items()}
    placements = [line for line in logs['stdout'].splitlines() if 'window placement observation' in line]
    for line in placements:
        observed = re.search(r'requested_display=(\d+).*?requested_fullscreen=(true|false).*?actual_display=Some\((\d+)\).*?actual_fullscreen=(true|false)', line)
        assert observed and observed[1] == observed[3] and observed[2] == observed[4], line
    assert exit_record['exit_code'] == 0 and not inventory['windows']
    assert not any(diagnostics.values()) and launch['rust_backtrace'] == 'full'
    runs[tag] = {'launch': launch, 'exit': exit_record, 'post_exit': inventory,
                 'log_lines': {kind: len(text.splitlines()) for kind, text in logs.items()},
                 'diagnostics': diagnostics, 'placements': placements,
                 'session_observations': [line for line in logs['stdout'].splitlines()
                                          if 'window transaction committed' in line or 'display settings opened' in line]}

main_observations = runs['v16-core-main']['session_observations']
no_play = [line for line in main_observations if 'stage=Ready page=2 elapsed_ms=191785' in line]
assert len(no_play) == 2, no_play
no_play_times = [datetime.fromisoformat(line.split()[0]) for line in no_play]
no_play_gap = (no_play_times[1] - no_play_times[0]).total_seconds()
assert no_play_gap > 20
saved = tomllib.loads((native / 'v16-no-play-saved.toml').read_text())
assert [m['role'] for m in saved['display']['monitors']] == ['presenter', 'none', 'none']
assert saved['display']['single_monitor_mode'] == 'presenter_only'
assert (native / 'v16-core-relaunch.config-before.toml').read_bytes() == (native / 'v16-core-main.config-after.toml').read_bytes()
final = tomllib.loads((native / 'v16-core-relaunch.config-after.toml').read_text())
assert [m['role'] for m in final['display']['monitors']] == ['presenter', 'none', 'audience']
cancel = load(native / 'v16-cancel-config-readback.json')
assert cancel['bytes_equal'] and cancel['launch_semantic_equal']
hidden = load(research / 'product-v16-root-hidden-cache-access.json')
assert hidden['exit_code'] == 0
hidden_log = (research / 'product-v16-root-hidden-cache-access.log').read_text()
assert hidden_log.count('native destroy then four retained-context COM errors returned normally') == 16
audit = {'captured_utc': datetime.now(timezone.utc).isoformat(), 'freeze_sha256': sha(freeze_path),
         'source_count': len(freeze['source_files']), 'source_sha256': freeze['source_snapshot_sha256'],
         'source_mismatches': mismatches, 'artifact_sha256': artifacts, 'runs': runs,
         'no_play_frozen_elapsed_ms': 191785, 'no_play_observation_gap_seconds': no_play_gap,
         'cancel_config': cancel, 'final_display_config': final['display'],
         'hidden_owned_windows': 16, 'post_destruction_com_queries': 64,
         'limits': ['Bounded native runs, no claim of complete lifetime/race or HWND-reuse safety',
                    'Page placeholders only; actual PDF/notes UI not implemented by07',
                    'Physical topology and real single/zero displays NOT_RUN; size matrix deferred; Gate B INCOMPLETE']}
save(research / 'native-v16-audit.json', audit)
files = sorted(p for p in native.glob('v16-*') if p.is_file())
files += [research / name for name in ('native-v16-audit.json', 'native-v16-core-results.md',
           'native-v16-plan.md', 'seal-native-v16.py', 'product-v16-root-hidden.json',
           'product-v16-root-hidden.log', 'product-v16-root-hidden-cache-access.json',
           'product-v16-root-hidden-cache-access.log')]
manifest = {p.relative_to(root).as_posix(): {'bytes': p.stat().st_size, 'sha256': sha(p)} for p in files}
save(research / 'native-v16-evidence-manifest.json', manifest)
print(json.dumps({'files': len(manifest), 'original_images': sum(p.suffix in ('.jpg', '.png') for p in files),
                  'manifest_sha256': sha(research / 'native-v16-evidence-manifest.json'),
                  'no_play_gap_seconds': no_play_gap, 'source_count': len(freeze['source_files'])}))
