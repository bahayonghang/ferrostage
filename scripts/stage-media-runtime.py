#!/usr/bin/env python3
"""Extract only the reviewed runtime closure; verify archives and entries first."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import zipfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--archives', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    package = Path(__file__).resolve().parent.parent / 'packaging/media'
    manifest = json.loads((package / 'runtime.json').read_text())
    for archive in manifest['archives']:
        path = args.archives / archive['file']
        if path.stat().st_size != archive['bytes'] or digest(path.read_bytes()) != archive['sha256']:
            raise ValueError(f'Archive identity mismatch: {path.name}')
    allowed = {entry['destination'].lower() for entry in manifest['files']}
    if args.output.exists():
        for path in args.output.rglob('*'):
            if path.is_file() and path.suffix.lower() in {'.dll', '.exe'}:
                relative = path.relative_to(args.output).as_posix()
                if relative.lower() not in allowed:
                    raise ValueError(f'Refusing unselected runtime binary in destination: {relative}')
    for entry in manifest['files']:
        with zipfile.ZipFile(args.archives / entry['archive']) as archive:
            info = archive.getinfo(entry['entry'])
            if info.file_size != entry['bytes']:
                raise ValueError(f'Entry size mismatch: {entry["name"]}')
            data = archive.read(info)
        if digest(data) != entry['sha256']:
            raise ValueError(f'Entry hash mismatch: {entry["name"]}')
        relative = Path(entry['destination'])
        if relative.is_absolute() or '..' in relative.parts:
            raise ValueError('Unsafe manifest destination')
        target = args.output / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists() and target.read_bytes() != data:
            raise ValueError(f'Refusing to overwrite a different runtime: {target}')
        target.write_bytes(data)
    for name in ['notices', 'recipes']:
        shutil.copytree(package / name, args.output / name, dirs_exist_ok=True)
    for name in ['runtime.json', 'native-runtime-sources.md']:
        shutil.copyfile(package / name, args.output / name)
    print(f'Verified and staged {len(manifest["files"])} runtime files with notices and source recipes.')


if __name__ == '__main__':
    main()
