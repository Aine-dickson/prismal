// Plays the guide's programs of chance, fields, solved equations and constraints in a
// headless browser and checks what the renderer draws: heatmaps as one image per frame and
// the frame rate of a ripple tank while it plays, field graphs, histograms of one run and of
// many, readings as points, an unknown plotted against time, and a drag that lands on a
// constraint. With OUT set to a directory, a screenshot of each program is saved there.
//
// Needs Node 22 or later, the player built (`./web/build.sh`) and Microsoft Edge or Chrome
// (path in BROWSER, extra flags in BROWSER_ARGS), as `check-input.mjs`.
//
//   node web/check-content.mjs
//
// MIN_FPS (default 15) is the lowest frame rate accepted for the ripple tank.

import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { readFileSync, statSync, mkdtempSync, writeFileSync } from 'node:fs';
import { join, extname, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const web = dirname(fileURLToPath(import.meta.url));
const browser = process.env.BROWSER || 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm' };
const server = createServer((req, res) => {
  const path = join(web, decodeURIComponent(new URL(req.url, 'http://x').pathname));
  try {
    const file = statSync(path).isDirectory() ? join(path, 'index.html') : path;
    res.writeHead(200, { 'content-type': types[extname(file)] || 'application/octet-stream' });
    res.end(readFileSync(file));
  } catch {
    res.writeHead(404);
    res.end();
  }
});
await new Promise((ok) => server.listen(0, '127.0.0.1', ok));
const port = server.address().port;
const devtools = 9338;
const edge = spawn(browser, [...(process.env.BROWSER_ARGS || '').split(' ').filter(Boolean), '--headless=new', '--no-first-run', `--remote-debugging-port=${devtools}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), 'prismal-content-'))}`, '--window-size=1300,900', 'about:blank']);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let target;
for (let i = 0; i < 150 && !target; i++) {
  await sleep(200);
  try {
    target = (await (await fetch(`http://127.0.0.1:${devtools}/json`)).json()).find((t) => t.type === 'page');
  } catch {}
}
if (!target) {
  console.error(`the browser at ${browser} did not open its DevTools port`);
  edge.kill();
  process.exit(1);
}
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener('open', r));
let id = 0;
const pending = new Map();
ws.addEventListener('message', (m) => {
  const d = JSON.parse(m.data);
  if (pending.has(d.id)) {
    pending.get(d.id)(d);
    pending.delete(d.id);
  }
});
const cdp = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
await cdp('Emulation.setDeviceMetricsOverride', { width: 1300, height: 900, deviceScaleFactor: 1, mobile: false });
const ev = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true })).result.result.value;
const open = async (hash) => {
  await cdp('Page.navigate', { url: `http://127.0.0.1:${port}/?${Math.random()}#${hash}` });
  for (let i = 0; i < 100; i++) {
    await sleep(200);
    if (await ev(`!!document.querySelector('#views svg g')`)) break;
  }
  await sleep(500);
};
const shot = async (name) => {
  if (!process.env.OUT) return;
  const { result } = await cdp('Page.captureScreenshot', { format: 'png' });
  writeFileSync(join(process.env.OUT, `${name}.png`), Buffer.from(result.data, 'base64'));
};
const center = (sel) => ev(`(() => { const r = document.querySelector(${JSON.stringify(sel)}).getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
const mouse = (type, x, y, extra = {}) => cdp('Input.dispatchMouseEvent', { type, x, y, button: 'left', buttons: type === 'mouseReleased' ? 0 : 1, clickCount: 1, ...extra });
const count = (sel) => ev(`document.querySelectorAll(${JSON.stringify(sel)}).length`);
let failed = 0;
const check = (name, cond, detail) => {
  console.log(`${cond ? 'ok  ' : 'FAIL'} ${name}${detail ? `: ${detail}` : ''}`);
  if (!cond) failed++;
};
// Frames drawn while playing for `ms`: each frame replaces the raster's image.
const playFor = async (ms) => {
  await ev(`window.__frames = 0; new MutationObserver(() => window.__frames++).observe(document.querySelector('#views'), { childList: true, subtree: true, attributes: true, attributeFilter: ['href', 'points'] }); true`);
  await ev(`document.querySelector('#play').click(); true`);
  await sleep(ms);
  await ev(`document.querySelector('#play').click(); true`);
  return ev('window.__frames');
};

// Fields (D-091): the ripple tank's heatmap, its frame rate while playing.
await open('g16-tank/TankLab');
check('a heatmap is one image', (await count('#views image.raster')) === 1 && (await count('#views rect:not(.frame)')) < 50, `${await count('#views image.raster')} images, ${await count('#views rect')} rects`);
// A stone dropped by a click (`set h(p) = ...`, D-091) makes waves.
const [tx, ty] = await center('#views image.raster');
await mouse('mouseMoved', tx - 40, ty - 30, { buttons: 0 });
await mouse('mousePressed', tx - 40, ty - 30);
await mouse('mouseReleased', tx - 40, ty - 30);
await sleep(300);
const frames = await playFor(3000);
const minFps = Number(process.env.MIN_FPS || 15);
check(`the ripple tank plays at ${minFps} frames per second or more`, frames / 3 >= minFps, `${(frames / 3).toFixed(1)} frames per second`);
// The image's pixels: not all one colour once the stone has fallen.
const colours = await ev(`new Promise((ok) => { const im = new Image(); im.onload = () => { const c = document.createElement('canvas'); c.width = im.width; c.height = im.height; const x = c.getContext('2d'); x.drawImage(im, 0, 0); const d = x.getImageData(0, 0, im.width, im.height).data; const set = new Set(); for (let k = 0; k < d.length; k += 4) set.add(d[k] * 65536 + d[k + 1] * 256 + d[k + 2]); ok(set.size); }; im.src = document.querySelector('#views image.raster').getAttribute('href'); })`);
check('the waves show in many colours', colours > 20, `${colours} colours`);
await shot('g16-tank');
await open('g16-rod/RodLab');
check('a field on a line is drawn as a graph', (await count('#views polyline')) >= 1);
await shot('g16-rod');
await open('g16-string/StringLab');
await shot('g16-string');

// Chance (D-083 to D-090): histograms, readings as points, a Brownian trace.
for (const [key, pres] of [['g15-sample2', 'ManyRuns@20'], ['g15-walk', 'WalkLab@20'], ['g15-counter', 'CounterLab@20'], ['g15-geiger', 'GeigerLab@40'], ['g15-pollen', 'PollenLab@5'], ['g15-drop', 'DropLab@5'], ['g15-atoms', 'AtomsLab@5'], ['g15-isomer', 'IsomerLab@10']]) {
  await open(`${key}/${pres}`);
  const drawn = await count('#views [data-rep]');
  check(`${key} draws its representations`, drawn > 0, `${drawn}`);
  await shot(key.replace(/@.*/, ''));
}

// Solved equations (D-092): the current, an unknown, plotted as it falls.
await open('g17-charging/ChargingLab@4');
const ys = await ev(`[...document.querySelectorAll('#views polyline')].map((p) => p.getAttribute('points').split(' ').length)`);
check('the unknown current is sampled, not a staircase', ys.every((n) => n > 20), JSON.stringify(ys));
await shot('g17-charging');

// Constraints (D-093): the bob dragged off its circle lands on it.
await open('g18-rodpendulum/RodLab');
const bob = '[data-rep="RodLab.view.scene.ball"] circle.marker';
const [bx, by] = await center(bob);
await mouse('mouseMoved', bx, by, { buttons: 0 });
await mouse('mousePressed', bx, by);
await mouse('mouseMoved', bx + 60, by - 20);
await mouse('mouseMoved', bx + 120, by - 40);
await mouse('mouseReleased', bx + 120, by - 40);
await sleep(300);
const text = await ev(`document.querySelector('[data-rep="RodLab.view.scene.ball"] title').textContent`);
const m = text.match(/x = ([-0-9.e]+) m, y = ([-0-9.e]+) m/);
const r = m ? Math.hypot(+m[1], +m[2]) : NaN;
check('a dragged bob lands on its circle', Math.abs(r - 1) < 1e-5, text);
await shot('g18-rodpendulum');
for (const [key, pres] of [['g18-doublependulum', 'DoubleLab@6'], ['g18-bead', 'BeadLab@2'], ['g18-unitcircle', 'CircleLab']]) {
  await open(`${key}/${pres}`);
  await shot(key);
}

// A view of three dimensions (D-100): projected axes and floor, the path of a spiral.
await open('g20-cyclotron/Spiral@30');
const axes = await count('#views line.axis');
const floor = await count('#views line.gridline');
check('a view of three dimensions draws its three axes and its floor', axes >= 3 && floor > 4, `${axes} axes, ${floor} grid lines`);
await shot('g20-cyclotron');

// An algorithm (D-101): bars as segments in a plot view; after 3 s, four comparisons made,
// two bars drawn as the pair compared.
await open('g21-sorting/SortLab@3');
const bars = await count('#views line.segment');
const compared = await ev(`(() => { const c = [...document.querySelectorAll('#views line.segment')].map((l) => getComputedStyle(l).stroke); const n = {}; c.forEach((x) => n[x] = (n[x] || 0) + 1); return Math.min(...Object.values(n)); })()`);
check('a sorting algorithm draws one bar per value', bars === 8 && compared === 2, `${bars} bars, ${compared} in another colour`);
await shot('g21-sorting');

// Optics (D-103): the rays of the refraction, lens and prism labs are drawn.
for (const [key, pres, want] of [['g22-refraction', 'RefractionLab', 4], ['g22-lens', 'LensLab', 8], ['g22-prism', 'PrismLab', 6]]) {
  await open(`${key}/${pres}`);
  const segs = await count('#views line.segment');
  check(`${key} draws its rays`, segs >= want, `${segs} segments`);
  await shot(key);
}

// The spoon example (D-094): a select chooses the material; the choice survives a restart.
// Line ends normalized: a checkout on Windows may write \r\n.
const md = readFileSync(join(web, '..', 'examples', 'heat.md'), 'utf8').replace(/\r\n/g, '\n');
const spoon = [...md.matchAll(/```text\n([\s\S]*?)```/g)].map((m) => m[1]).join('\n');
await open('rp01');
await ev(`document.querySelector('#source').value = ${JSON.stringify(spoon)}; document.querySelector('#compile').click(); true`);
for (let i = 0; i < 50 && !(await ev(`!!document.querySelector('#views select')`)); i++) await sleep(200);
const choices = await ev(`[...document.querySelectorAll('#views select option')].map((o) => o.textContent)`);
check('a select lists the cases', JSON.stringify(choices) === '["copper","steel","glass"]', JSON.stringify(choices));
await ev(`(() => { const s = document.querySelector('#views select'); s.value = '1'; s.dispatchEvent(new Event('change')); return true; })()`);
await sleep(300);
await ev(`[...document.querySelectorAll('#views button')].find((b) => b.textContent === 'Into the water again').click(); true`);
await sleep(300);
const chosen = await ev(`document.querySelector('#views select').getAttribute('aria-label')`);
check('the choice is kept by a restart', chosen.startsWith('select for material = steel'), chosen);
await shot('heat');

ws.close();
edge.kill();
server.close();
console.log(failed ? `${failed} checks failed` : 'all checks pass');
process.exit(failed ? 1 : 0);
