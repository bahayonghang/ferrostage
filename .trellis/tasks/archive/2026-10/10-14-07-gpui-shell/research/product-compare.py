from pathlib import Path
import hashlib,json,difflib,datetime,re,tomllib
root=Path.cwd();r=root/'.trellis/tasks/10-14-07-gpui-shell/research'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
reviewed=root/'cache/gpui-maintenance-candidate/vendor'
changes=[];patch=[]
for family in ['gpui','pulp','metal','cosmic-text','harfrust']:
 for old in sorted((reviewed/family).rglob('*')):
  if not old.is_file():continue
  rel=old.relative_to(reviewed);new=root/'vendor'/rel
  if sha(old)!=sha(new):
   changes.append({'path':rel.as_posix(),'reviewed':sha(old),'product':sha(new)})
   patch.extend(difflib.unified_diff(old.read_text(encoding='utf-8').splitlines(True),new.read_text(encoding='utf-8').splitlines(True),fromfile='reviewed/'+rel.as_posix(),tofile='product/'+rel.as_posix()))
assert {i['path'] for i in changes}=={'gpui/src/svg_renderer.rs','harfrust/src/hb/charmap.rs'},changes
# Adaptation lies entirely in test modules, preserving reviewed runtime source.
for family,path in [('gpui','src/svg_renderer.rs'),('harfrust','src/hb/charmap.rs')]:
 old=(reviewed/family/path).read_text(encoding='utf-8');new=(root/'vendor'/family/path).read_text(encoding='utf-8')
 assert old.split('#[cfg(test)]',1)[0]==new.split('#[cfg(test)]',1)[0]
(r/'product-vendor-final-adaptation.patch').write_text(''.join(patch),encoding='utf-8')
(r/'product-vendor-final-comparison.json').write_text(json.dumps({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reviewed_final_manifest_sha256':sha(r/'maintenance-independent-manifest.json'),'runtime_source_identical_to_reviewed_final':True,'changes':changes,'final_files':{p.relative_to(root).as_posix():sha(p) for p in sorted((root/'vendor').rglob('*')) if p.is_file()}},indent=2)+'\n',encoding='utf-8')
base=json.loads((root/'.trellis/tasks/10-14-06-beamer-document-model/research/independent-check-manifest.json').read_text(encoding='utf-8'))['source_files']
delta=[{'path':name,'baseline':value,'current':sha(root/name)} for name,value in base.items() if sha(root/name)!=value]
(r/'product-baseline-comparison.json').write_text(json.dumps({'baseline_count':len(base),'unchanged':len(base)-len(delta),'changed':delta,'all_previous_core_platform_pdf_render_document_source_unchanged':all(sha(root/name)==value for name,value in base.items() if any(name.startswith('crates/ferrostage-'+p+'/') for p in ['core','platform','pdf','render','document']))},indent=2)+'\n',encoding='utf-8')
locales={lang:tomllib.loads((root/'assets/locales'/lang/'app.toml').read_text(encoding='utf-8')) for lang in ['en-US','zh-CN']}
used=set()
for p in (root/'crates/ferrostage-ui/src').glob('*.rs'):
 s=p.read_text(encoding='utf-8');used.update(re.findall(r'\.(?:text|format|error)\("([a-z][a-z-]+)"',s))
assert used <= locales['en-US'].keys();assert locales['en-US'].keys()==locales['zh-CN'].keys()
(r/'product-locale-contract.json').write_text(json.dumps({'keys':len(locales['en-US']),'direct_call_keys':sorted(used),'catalog_hashes':{lang:sha(root/'assets/locales'/lang/'app.toml') for lang in locales},'status':'PASS: identical keys, direct calls present; test verifies interpolation fields; runtime screenshots pending'},indent=2)+'\n',encoding='utf-8')
print(json.dumps({'vendor_adaptations':len(changes),'baseline_unchanged':len(base)-len(delta),'baseline_changed':[x['path'] for x in delta],'locale_keys':len(locales['en-US'])}))
