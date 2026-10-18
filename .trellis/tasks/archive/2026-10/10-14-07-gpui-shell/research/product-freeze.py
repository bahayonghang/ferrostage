from pathlib import Path
import hashlib,json,datetime,subprocess,sys
root=Path.cwd();r=root/'.trellis/tasks/10-14-07-gpui-shell/research'
version=sys.argv[1] if len(sys.argv)>1 else ''
freeze_name='product-native-freeze'+('-'+version if version else '')+'.json'
build_name='product-'+version+'-build.log' if version else 'product-freeze-build.log'
clippy_name='product-'+version+'-clippy.log' if version else 'product-freeze-clippy.log'
assert not (r/freeze_name).exists(), 'Preserve previous freeze; use a new version'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
files=[]
for folder in ['crates','vendor','assets/locales']:
 files.extend(p for p in (root/folder).rglob('*') if p.is_file())
for name in ['Cargo.toml','Cargo.lock','rust-toolchain.toml','deny.toml','justfile','.github/workflows/ci.yml','DESIGN.md','docs/packaging.md']:
 files.append(root/name)
source={p.relative_to(root).as_posix():sha(p) for p in sorted(set(files))}
source_hash=hashlib.sha256(json.dumps(source,sort_keys=True,separators=(',',':')).encode()).hexdigest()
exe=root/'target/debug/ferrostage.exe';dll=root/'cache/pdfium-7881-win-x64/bin/pdfium.dll'
record=lambda p:{'path':str(p),'resolved_path':str(p.resolve()),'bytes':p.stat().st_size,'sha256':sha(p)}
assert sha(dll)=='79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118'
freeze={'captured_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'FROZEN for Root native acceptance; no further source or executable changes until feedback','head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'source_algorithm':'SHA256 of UTF-8 JSON(source_files,sort_keys=True,separators=(comma,colon))','source_snapshot_sha256':source_hash,'source_files':source,'binary':record(exe),'pdfium':record(dll),'cargo_lock':record(root/'Cargo.lock'),'fixture':record(root/'fixtures/pdf/raster.pdf'),'final_build_log':record(r/build_name),'final_native_clippy_log':record(r/clippy_name),'native_runtime_status':'NOT_RUN by implementer; Root exclusively controls desktop','launch_argv':[str(exe),'--config',str(r/'native/a-light.toml'),'--pdfium-library',str(dll),str(root/'fixtures/pdf/raster.pdf')],'observations':['GPUI/platform display identity comparison: complete MonitorInfo and strict mapped MonitorId->GPUI display ID result','display role resolution warning: typed03 fallback/missing/duplicate warning','window created behind confirmation gate: handle, MonitorId, requested GPUI display ID, fullscreen, audience','window placement confirmed: WindowId, MonitorId, requested/actual display IDs, observed actual logical bounds, mode, bounds-event flag','window transaction committed: core stage and current presenter/audience handles','presenter activation confirmed: popup-owner WindowId']}
(r/freeze_name).write_text(json.dumps(freeze,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
print(json.dumps({'source_files':len(source),'source_sha256':source_hash,'binary':freeze['binary'],'lock_sha256':sha(root/'Cargo.lock'),'freeze_sha256':sha(r/freeze_name)},indent=2))
