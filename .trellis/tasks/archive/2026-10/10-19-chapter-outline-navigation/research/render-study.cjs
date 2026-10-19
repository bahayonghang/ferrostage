// Review artifact only. No browser or HTML code is part of the native product.
const fs = require('fs');
const path = require('path');
const {pathToFileURL} = require('url');
const {chromium} = require('C:/Users/lyh/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const dir = __dirname;
const data = JSON.parse(fs.readFileSync(path.join(dir, 'pdf-outline.json'), 'utf8'));
const escape = value => String(value).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const rows = data.retained.map(row => `<button class="row depth-${row.depth}${row.content_number === 23 ? ' current' : ''}" id="entry-${row.source_id}" aria-label="${escape(row.title)}, 内容页 ${row.content_number}, PDF ${row.pdf_page}"><span class="title">${escape(row.title)}</span>${row.content_number === 23 ? '<span class="badge">当前章节</span>' : ''}<span class="pages">${row.content_number}<span class="dot">·</span>${row.pdf_page}</span></button>`).join('\n');
const html = `<!doctype html><html lang="zh-CN"><meta charset="utf-8"><title>FerroStage 章节目录布局研究</title>
<style>
:root{color-scheme:dark;--canvas:#181715;--soft:#1f1e1b;--card:#252320;--ink:#faf9f5;--body:#e6dfd8;--muted:#b9b3a8;--border:#91897c;--focus:#e5a38b;--primary:#a34f35}
*{box-sizing:border-box}body{margin:0;background:var(--canvas);color:var(--body);font:16px/1.5 'Segoe UI','Microsoft YaHei UI',sans-serif}button,input{font:inherit;color:inherit}button{cursor:pointer;min-height:44px;border:1px solid transparent;border-radius:8px;background:transparent;padding:8px 16px}button:hover{background:var(--soft)}button:active{transform:translateY(1px)}button:focus-visible,input:focus-visible{outline:2px solid var(--focus);outline-offset:2px}.context{width:880px;margin:24px auto 16px;display:flex;justify-content:space-between;font-size:12px;color:var(--muted)}.panel{width:880px;margin:auto;background:var(--card);border:1px solid var(--border);border-radius:12px;padding:16px;display:flex;flex-direction:column;gap:8px;height:812px}.heading{display:flex;align-items:center;justify-content:space-between}.heading h1{font-size:28px;line-height:1.3;font-weight:600;color:var(--ink);margin:0}.status{font-size:12px;color:var(--muted);margin:2px 0 6px}.utility{border-color:var(--border);background:var(--soft)}.line{display:flex;align-items:center;gap:8px;min-height:44px}.label{color:var(--muted);margin-right:4px}.segment{display:inline-flex;gap:4px;padding:2px;background:var(--soft);border-radius:8px}.segment button[aria-selected=true]{background:var(--card);color:var(--ink)}.check{color:var(--focus);margin-right:6px}.number{width:120px;height:44px;background:var(--soft);border:1px solid var(--border);border-radius:8px;padding:6px 12px;font-size:24px}.primary{background:var(--primary);color:white}.primary:hover{background:#93452f}.helper{font-size:12px;color:var(--muted);min-height:24px}.tabs{border-bottom:1px solid var(--border);display:flex;gap:8px;padding-bottom:8px}.tabs button{min-width:120px}.tabs [aria-selected=true]{background:var(--soft);color:var(--ink);font-weight:600}.columns{display:flex;justify-content:space-between;font-size:12px;color:var(--muted);padding:6px 12px}.list{flex:1;min-height:0;overflow-y:auto;scrollbar-color:var(--border) var(--soft);scrollbar-width:thin;padding:4px 8px 4px 4px}.row{display:flex;align-items:center;text-align:left;gap:16px;width:100%;border-radius:8px;padding:10px 12px;min-height:48px}.row .title{flex:1;min-width:0;overflow-wrap:anywhere}.depth-1{font-weight:600;margin-top:12px}.depth-2 .title{padding-left:24px}.row .pages{width:94px;flex-shrink:0;text-align:right;font-variant-numeric:tabular-nums;font-weight:400;color:var(--muted)}.dot{padding:0 12px}.current{background:var(--soft)}.badge{font-size:12px;font-weight:400;white-space:nowrap;color:var(--focus)}.footer{display:flex;justify-content:space-between;border-top:1px solid var(--border);padding-top:12px;color:var(--muted);font-size:12px;min-height:34px}.note{width:880px;margin:14px auto;color:var(--muted);font-size:12px;display:flex;justify-content:space-between}.empty{height:100%;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px}.empty strong{font-size:16px;font-weight:400;color:var(--body)}.empty span{font-size:12px;color:var(--muted)}
</style>
<div class="context"><span>FerroStage · defense.pdf</span><span>章节目录设计预览 · 非原生实现</span></div>
<main class="panel"><div class="heading"><h1>跳转页面</h1><button class="utility" aria-label="关闭跳转页面">关闭</button></div><div class="status">逻辑页 23/95 · 步进 1/1 · 物理页 45/190</div>
<div class="line"><span class="label">页码方式</span><div class="segment"><button aria-selected="true"><span class="check">✓</span>内容页</button><button aria-selected="false">PDF 原页码</button></div></div>
<div class="line"><label class="label" for="page">内容页码</label><input class="number" id="page" value="23" readonly><span>/ 95</span><button class="primary">跳转</button></div><div class="helper">第 23 页 · PDF 45</div>
<div class="tabs" role="tablist" aria-label="跳页视图"><button role="tab" aria-selected="false">缩略图</button><button role="tab" aria-selected="true">✓ 章节目录</button></div>
<div class="columns"><span>章节标题</span><span>内容页 · PDF</span></div><div class="list" id="directory">${rows}</div><div class="footer"><span id="count">39 个目录项 · 最多两级</span><span>点击标题直接跳页</span></div></main>
<div class="note"><span>页码与目录文字均读取自示例 PDF，未补号或改写标题。</span><span>布局研究稿，操作结果不代表 GPUI 验收。</span></div>
<script>
const list=document.getElementById('directory');const state=new URLSearchParams(location.search).get('state');
if(state==='empty'){list.innerHTML='<div class="empty"><strong>此 PDF 没有可用的章节目录</strong><span>可继续使用上方页码或切换缩略图跳页</span></div>';document.getElementById('count').textContent='0 个目录项';document.querySelector('.context span').textContent='FerroStage · 无书签状态示意';document.querySelector('.note span').textContent='本状态用于定义无书签时的界面，示例 PDF 本身有书签。'}
else{const target=state==='long'?Array.from(list.children).find(e=>e.textContent.startsWith('第6章')):list.querySelector('.current');list.scrollTop=target.offsetTop-list.offsetTop-100;}
</script></html>`;
fs.writeFileSync(path.join(dir, 'directory-study.html'), html);
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true});
 const page=await browser.newPage({viewport:{width:1100,height:960},deviceScaleFactor:1});
 const metrics=[];
 for(const state of ['current','long','empty']){
  await page.goto(pathToFileURL(path.join(dir,'directory-study.html')).href+'?state='+state);
  await page.screenshot({path:path.join(dir,'directory-'+state+'.png')});
  metrics.push(await page.evaluate(state=>({state,title:document.title,rows:document.querySelectorAll('.row').length,overflow:document.documentElement.scrollWidth>innerWidth,panel:document.querySelector('.panel').getBoundingClientRect().toJSON()}),state));
 }
 fs.writeFileSync(path.join(dir,'layout-metrics.json'),JSON.stringify(metrics,null,2)+'\n');
 await browser.close();
 console.log(JSON.stringify(metrics));
})().catch(error=>{console.error(error);process.exitCode=1});
