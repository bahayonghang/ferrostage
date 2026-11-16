"""Capture the reviewed source set and final independent integrity index."""
from pathlib import Path
import datetime
import difflib
import hashlib
import json
import shutil
import subprocess
import sys
import tomllib

ROOT = Path.cwd()
R = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'
C = ROOT / 'cache/gpui-maintenance-candidate'
REG = Path('C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f')
ZED = Path('C:/Users/lyh/.cargo/git/checkouts/zed-a70e2ad075855582/96a1b1b')
RETAIN = R / 'maintenance-independent-retained'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
def read(path):
    data = path.read_bytes()
    return data.decode('utf-16' if data.startswith((b'\xff\xfe', b'\xfe\xff')) else 'utf-8-sig')
def write(path, item):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(item, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
def record(path):
    return {'path':str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path), 'bytes':path.stat().st_size, 'sha256':sha(path)}
files = {p.relative_to(C).as_posix():sha(p) for folder in ('vendor','validation','src','tests','assets','maintenance-fixtures') for p in (C/folder).rglob('*') if p.is_file()}
files |= {name:sha(C/name) for name in ('Cargo.toml','Cargo.lock')}
if sys.argv[1] == 'source':
    write(R/'maintenance-independent-final-source.json', {'candidate':str(C), 'files':files, 'file_count':len(files)})
    for folder in ('vendor','validation'):
        for lock in (C/folder).rglob('Cargo.lock'):
            dest=RETAIN/'final-locks'/lock.relative_to(C)
            dest.parent.mkdir(parents=True,exist_ok=True)
            shutil.copy2(lock,dest)
    (RETAIN/'final-locks').mkdir(parents=True,exist_ok=True)
    shutil.copy2(C/'Cargo.lock',RETAIN/'final-locks/Cargo.lock')
    for name in ('Cargo.toml','src/main.rs','vendor/gpui/Cargo.toml','vendor/pulp/Cargo.toml','vendor/metal/Cargo.toml','vendor/cosmic-text/Cargo.toml','vendor/harfrust/Cargo.toml','validation/fontdb/Cargo.toml','validation/wgpu-consumer/Cargo.toml'):
        dest=RETAIN/'final-manifests'/name
        dest.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(C/name,dest)
    patch=[]
    comparisons=[]
    for name,source in [('gpui',ZED/'crates/gpui'),('pulp',REG/'pulp-0.22.3'),('metal',REG/'metal-0.33.0'),('cosmic-text',REG/'cosmic-text-0.19.0'),('harfrust',REG/'harfrust-0.12.0')]:
        dest=C/'vendor'/name
        changed=[]
        for original in source.rglob('*'):
            if original.is_file():
                rel=original.relative_to(source)
                if sha(original)!=sha(dest/rel):
                    changed.append(rel.as_posix())
                    if rel.as_posix()!='Cargo.lock':
                        patch.extend(difflib.unified_diff(read(original).splitlines(True),read(dest/rel).splitlines(True),fromfile=f'a/{name}/{rel.as_posix()}',tofile=f'b/{name}/{rel.as_posix()}'))
        extras=[p.relative_to(dest).as_posix() for p in dest.rglob('*') if p.is_file() and not (source/p.relative_to(dest)).exists()]
        comparisons.append({'package':name,'source':str(source),'changed_original_paths':changed,'added_paths':extras})
        for notice in dest.iterdir():
            if notice.is_file() and (notice.name.startswith('LICENSE') or notice.name=='FERROSTAGE-MAINTENANCE.txt'):
                target=RETAIN/'notices'/name/notice.name
                target.parent.mkdir(parents=True,exist_ok=True)
                shutil.copy2(notice,target)
    (R/'maintenance-independent-final-source.patch').write_text(''.join(patch),encoding='utf-8',newline='\n')
    write(R/'maintenance-independent-final-package-comparison.json',comparisons)
    locks=[]
    for rel in ('Cargo.lock','vendor/gpui/Cargo.lock','validation/wgpu-consumer/Cargo.lock'):
        old=tomllib.loads(read(R/'maintenance-retained'/rel))['package']
        new=tomllib.loads(read(C/rel))['package']
        by_key=lambda ps:{(p['name'],p['version']):p for p in ps}
        a,b=by_key(old),by_key(new)
        locks.append({'path':rel,'removed':sorted(a.keys()-b.keys()),'added':sorted(b.keys()-a.keys()),'changed':[{'package':key,'old':a[key],'new':b[key]} for key in a.keys()&b.keys() if a[key]!=b[key]]})
    write(R/'maintenance-independent-final-lock-comparison.json',locks)
    proof=ROOT/'cache/gpui-maintenance-cmap-proof'
    for rel in ('Cargo.toml','Cargo.lock','src/main.rs'):
        target=RETAIN/'final-proof'/rel
        target.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(proof/rel,target)
    print(json.dumps({'source_files':len(files),'package_comparisons':comparisons,'lock_changes':[{'path':x['path'],'added':x['added'],'removed':x['removed'],'changed_count':len(x['changed'])} for x in locks]}))
elif sys.argv[1] == 'final':
    expected=json.loads(read(R/'maintenance-independent-final-source.json'))['files']
    assert files==expected
    originals=json.loads(read(R/'maintenance-evidence-manifest.json'))
    reviewed_sources=[]
    for name,relative in [('read-fonts-0.41.0','src/tables/cmap.rs'),('read-fonts-0.41.0','generated/generated_cmap.rs'),('read-fonts-0.41.0','src/font_data.rs'),('skrifa-0.44.0','src/charmap.rs'),('ttf-parser-0.25.1','src/lib.rs'),('ttf-parser-0.25.1','src/tables/cmap/mod.rs'),('ttf-parser-0.25.1','src/tables/cmap/format2.rs'),('rustybuzz-0.20.1','src/hb/face.rs'),('harfrust-0.12.0','src/hb/charmap.rs'),('harfrust-0.12.0','src/hb/tables.rs'),('harfrust-0.12.0','src/hb/face.rs'),('harfrust-0.12.0','src/hb/font_funcs.rs')]:
        reviewed_sources.append(record(REG/name/relative))
    generated=[]
    for old in originals['generated_cache_artifacts']:
        p=ROOT/old['path']
        if p.is_file(): generated.append(record(p))
    for p in (ROOT/'cache/gpui-maintenance-cmap-proof/target/debug/deps').glob('ferrostage_maintenance_cmap_proof-*.exe'):
        generated.append(record(p))
    artifacts=[record(p) for p in R.rglob('maintenance-independent-*') if p.is_file() and p.name not in ('maintenance-independent-manifest.json','maintenance-independent-manifest.sha256')]
    for folder in R.glob('maintenance-independent-*'):
        if folder.is_dir(): artifacts.extend(record(p) for p in folder.rglob('*') if p.is_file() and p.name not in ('maintenance-independent-manifest.json','maintenance-independent-manifest.sha256'))
    artifacts=list({x['path']:x for x in artifacts}.values())
    versions={command:subprocess.run(command.split(),capture_output=True,text=True).stdout.strip() for command in ('rustc -Vv','cargo -V','cargo deny --version','rustfmt --version','python --version')}
    data={'captured_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'REVIEWED CANDIDATE; adoption/license exceptions unapproved; upstream direct lint baseline failure retained',
          'entrypoint':str((R/'maintenance-independent-prepare-final.py').relative_to(ROOT)),
          'frozen_handoff_manifest_sha256':sha(R/'maintenance-evidence-manifest.json'),'frozen_handoff_patch_sha256':sha(R/'maintenance-source.patch'),
          'final_source_snapshot':record(R/'maintenance-independent-final-source.json'),'final_source_patch':record(R/'maintenance-independent-final-source.patch'),
          'candidate_source_files':[record(C/p) for p in sorted(files)],'review_artifacts':sorted(artifacts,key=lambda x:x['path']),
          'reviewed_primary_source_files':reviewed_sources,'generated_cache_artifacts':generated,'toolchain':versions,
          'product_policy':record(ROOT/'deny.toml'),'task06_baseline_manifest':record(ROOT/'.trellis/tasks/10-14-06-beamer-document-model/research/independent-check-manifest.json')}
    path=R/'maintenance-independent-manifest.json'
    write(path,data)
    (R/'maintenance-independent-manifest.sha256').write_text(sha(path)+'  maintenance-independent-manifest.json\n',encoding='utf-8')
    print(json.dumps({'manifest_sha256':sha(path),'source_files':len(files),'review_artifacts':len(artifacts),'patch_sha256':sha(R/'maintenance-independent-final-source.patch')}))
