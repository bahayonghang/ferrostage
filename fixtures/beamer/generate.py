"""Self-authored deterministic structural fixtures, MIT OR Apache-2.0.
This generator writes only named artifacts in its own fixture directory.
Actual overlays.pdf and notes-right.pdf are separate unmodified TeX outputs.
"""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
class PDF:
    def __init__(self): self.objects=[]
    def obj(self,data): self.objects.append(data.encode('ascii') if isinstance(data,str) else data); return len(self.objects)
    def save(self,name):
        out=bytearray(b'%PDF-1.7\n'); offsets=[0]
        for i,data in enumerate(self.objects,1): offsets.append(len(out)); out+=f'{i} 0 obj\n'.encode()+data+b'\nendobj\n'
        xref=len(out); out+=f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode()
        for offset in offsets[1:]:out+=f'{offset:010d} 00000 n \n'.encode()
        out+=f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode(); (ROOT/name).write_bytes(out)

def simple(name,labels,sizes,rotation=0):
    p=PDF();p.obj('');p.obj('');kids=[];nums=[]
    for i,label in enumerate(labels):
        w,h=sizes[i%len(sizes)]; data=f'1 0 0 rg 0 0 {w/2} {h} re f 0 0 1 rg {w/2} 0 {w/2} {h} re f'.encode()
        stream=p.obj(f'<< /Length {len(data)} >>\nstream\n'.encode()+data+b'\nendstream')
        page=p.obj(f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents {stream} 0 R >>');kids.append(page)
        # Empty prefix explicitly encodes an empty original label (no numeric style).
        if label is not None:nums.append(f'{i} << /P ({label}) >>')
        else: nums.append(f'{i} << /P () >>')
    p.objects[0]=f'<< /Type /Catalog /Pages 2 0 R /PageLabels << /Nums [{" ".join(nums)}] >> >>'.encode()
    p.objects[1]=f'<< /Type /Pages /Count {len(kids)} /Kids [{" ".join(f"{k} 0 R" for k in kids)}] /Rotate {rotation} >>'.encode();p.save(name)
labels=['notes:Intro','Intro','notes:Intro','notes:Intro','Intro','notes:Intro','B','notes:Intro','Intro','01','notes:1','1','',None]
simple('independent.pdf',labels,[(320,180)])
for name,size,rotate in [('bottom-wide',(320,360),0),('bottom-classic',(240,360),0),('top-rotated',(360,320),90),('wide-counterexample',(640,180),0),('portrait-counterexample',(240,360),0),('ordinary',(320,180),0),('tall-ordinary',(180,320),0)]:simple(name+'.pdf',['A','B'],[size],rotate)
simple('mixed.pdf',['A','B'],[(640,180),(320,180)])
# Inherited CropBox/rotation; media Rect is in unrotated PDF bottom-origin user coordinates.
p=PDF();p.obj('<< /Type /Catalog /Pages 2 0 R >>');p.obj('<< /Type /Pages /Count 1 /Kids [3 0 R] /MediaBox [0 0 400 300] /CropBox [20 30 380 270] /Rotate 90 >>')
p.obj('<< /Type /Page /Parent 2 0 R /Annots [4 0 R 5 0 R 6 0 R 7 0 R 8 0 R] /AA << /O 9 0 R >> >>')
p.obj('<< /Type /Annot /Subtype /Movie /Rect [20 30 200 150] /Movie << /F (external-demo.mp4) >> /A << /Start [2000 1000] /Duration [8000 1000] /Mode /Repeat >> >>')
p.obj('<< /Type /Annot /Subtype /Screen /P 3 0 R /Rect [20 30 200 150] /A 10 0 R /AA << /PO 10 0 R >> >>')
p.obj('<< /Type /Annot /Subtype /RichMedia /Rect [20 30 200 150] /RichMediaSettings << /Activation << /Condition /PV >> >> /RichMediaContent << /Assets << /Names [(clip.mp4) 12 0 R] >> /Configurations [13 0 R] >> >>')
p.obj('<< /Type /Annot /Subtype /Movie /Rect [200 150 380 270] /Movie << /F (unscaled.mp4) >> /A << /Start 120 /Duration [8 0] /Mode /Palindrome >> >>')
p.obj('<< /Type /Annot /Subtype /RichMedia /Rect [20 30 200 150] /RichMediaContent << /Configurations [15 0 R] >> >>')
p.obj('<< /S /Movie /Annotation 4 0 R /Operation /Play >>')
p.obj('<< /S /Rendition /OP 0 /AN 5 0 R /R << /S /MR /C << /S /MCS /MH << /B << /S /T /T << /S /S /V 2 >> >> >> /D << /S /MCD /D 12 0 R /CT (video/mp4) >> >> /P << /MH << /D << /S /T /T << /S /S /V 8 >> >> /RC 0 /A true >> >> >> >>')
p.obj(b'<< /Type /EmbeddedFile /Length 16 >>\nstream\nfixture-not-video\nendstream')
p.obj('<< /Type /Filespec /F (clip.mp4) /UF (clip.mp4) /EF << /F 11 0 R >> >>')
p.obj('<< /Type /RichMediaConfiguration /Subtype /Video /Instances [14 0 R] >>')
p.obj('<< /Type /RichMediaInstance /Subtype /Video /Asset 12 0 R >>')
p.obj('<< /Type /RichMediaConfiguration /Subtype /Flash /Instances [16 0 R] >>')
p.obj('<< /Type /RichMediaInstance /Subtype /Flash /Asset 12 0 R /Params << /FlashVars (start=2&loop=true) >> >>')
p.save('media.pdf')
expected={'independent':{'labels':labels,'steps':[1,4,6,8,9,11,12,13],'groups':[[1,4],[6],[8],[9],[11],[12],[13]],'notes':{'1':[2,3],'4':[5]},'orphans':[0,7,10]},'media':{'visible_size':[240,360],'rotation':90,'first_rect':[0,0,0.5,0.5],'movie_start_seconds':2,'movie_duration_seconds':8,'screen_start_seconds':2,'screen_duration_seconds':8,'embedded_ascii':'fixture-not-video','unscaled_ticks':120},'limitations':['Embedded bytes are metadata test data, not a playable video.','No user PDF is modified.','Synthetic labels are explicitly authored PageLabels prefixes, not claims about Beamer output.']}
expected['actual_beamer']={'overlays':{'frames':2,'labels':['1','1','2'],'groups':[[0,1],[2]],'layout':'None'},'notes-right':{'frames':2,'labels':['1','1'],'groups':[[0,1]],'physical_pages_groups':[[0],[1]],'layout':'Right','warning':'Producer repeats labels; PDF kept unmodified'},'movie':{'start_seconds':2,'duration_seconds':8,'repeat':'Forever','autostart':True,'activation':'Widget/AA/PO -> Movie/T unique same-page title'}}
expected['geometry']={'bottom-wide':'Bottom','bottom-classic':'Bottom','top-rotated':'Bottom candidate; Top override is authoritative','wide-counterexample':'Right candidate; None override recovers full page','portrait-counterexample':'Bottom candidate; None override recovers full page','ordinary':'None','tall-ordinary':'None','mixed':'None with MixedGeometry'}
(ROOT/'expected.json').write_text(json.dumps(expected,indent=2)+'\n')
