// Prismal web player: renders the frame descriptions of the presentation kernel (PK-12.1)
// as SVG, MathML and HTML controls, and forwards the learner's gestures to the kernel
// compiled to WebAssembly (crates/prismal-web). All model logic stays in the kernel; this
// file only draws and routes input.

import init, { WebPlayer, examples as loadExamples } from './pkg/prismal_web.js';

const SVGNS = 'http://www.w3.org/2000/svg';
const $ = (s) => document.querySelector(s);

const st = {
  examples: [],
  player: null,
  catalogue: null,
  layout: null,
  lesson: null,
  frame: null,
  p: 0,
  playing: false,
  speed: 1,
  lastTick: null,
  views: {},       // per view id: { kind, svg, panel, vb, base }
  gesture: null,   // a drag or pan in progress: { action, view, ids (pointers taking part), resume }
  focus: null,     // { rep } of the focused draggable, restored after each render
  items: new Map(), // keyed HTML items (controls, formulas, labels) by rep id
  // Narration sound (D-053): off, synthesized speech, or recordings by cue name; the cues
  // sounding in the current play-through.
  voice: { mode: 'off', files: new Map(), notes: [], sounding: new Map(), checks: 0 }
};

// ------------------------------------------------------------------ utilities

function fmt(x) {
  if (!Number.isFinite(x)) return String(x);
  if (x === 0) return '0';
  const mag = Math.floor(Math.log10(Math.abs(x)));
  if (mag < -4 || mag >= 6) return x.toExponential(4).replace(/\.?0+e/, 'e');
  return x.toFixed(Math.max(0, 5 - mag)).replace(/(\.\d*?)0+$/, '$1').replace(/\.$/, '');
}

function svg(tag, attrs = {}, parent) {
  const e = document.createElementNS(SVGNS, tag);
  for (const [k, v] of Object.entries(attrs)) {
    // Inline style, so that style sheets cannot override sizes given in view units.
    if (k === 'font-size') e.style.fontSize = `${v}px`;
    else e.setAttribute(k, v);
  }
  if (parent) parent.appendChild(e);
  return e;
}

function el(tag, attrs = {}, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (k === 'class') e.className = v;
    else if (k === 'text') e.textContent = v;
    else e.setAttribute(k, v);
  }
  for (const c of children) e.append(c);
  return e;
}

function status(msg, bad = false) {
  const s = $('#status');
  s.textContent = msg || '';
  s.classList.toggle('bad', bad);
}

function announce(msg) {
  const a = $('#announcer');
  a.textContent = '';
  requestAnimationFrame(() => { a.textContent = msg; });
}

function parseErr(e) {
  try { return JSON.parse(String(e)); } catch { return [{ code: 'error', message: String(e) }]; }
}

/// A "nice" step (1, 2 or 5 times a power of ten) near `raw`.
function niceStep(raw) {
  const p = Math.pow(10, Math.floor(Math.log10(raw)));
  const m = raw / p;
  return (m < 1.5 ? 1 : m < 3.5 ? 2 : m < 7.5 ? 5 : 10) * p;
}

function allReps(frame) {
  const out = [];
  for (const v of frame.views) for (const r of v.reps) out.push([v.id, r]);
  for (const r of frame.overlay || []) out.push([null, r]);
  return out;
}

// ------------------------------------------------------------------ loading

async function main() {
  await init();
  st.examples = JSON.parse(loadExamples());
  const sel = $('#program');
  for (const ex of st.examples) sel.append(el('option', { value: ex.key, text: ex.title }));
  sel.append(el('option', { value: '', text: 'Edited source' }));
  st.want = parseAddress();
  sel.value = st.examples.some((e) => e.key === st.want.key) ? st.want.key : st.examples[0].key;
  sel.addEventListener('change', () => {
    if (!sel.value) return;
    location.hash = sel.value;
    loadExample(sel.value);
  });
  $('#presentation').addEventListener('change', () => openPresentation());
  $('#medium').addEventListener('change', () => openPresentation());
  $('#compile').addEventListener('click', compileSource);
  $('#format').addEventListener('click', () => {
    // Formatting prints from the IR: the program must compile. Comments that are not
    // notes before a declaration are not kept (D-036).
    try {
      const p = new WebPlayer($('#source').value);
      $('#source').value = p.formatted();
      p.free();
      $('#program').value = '';
      status('Formatted. Only comments directly before a declaration are kept.');
    } catch (e) {
      compileSource();
    }
  });
  $('#source').addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) { e.preventDefault(); compileSource(); }
  });
  $('#source').addEventListener('input', () => { $('#program').value = ''; });
  setupTabs();
  setupTransport();
  $('#undo').addEventListener('click', () => { st.player.undo(); status(''); afterSessionAction(); });
  $('#redo').addEventListener('click', () => { st.player.redo(); status(''); afterSessionAction(); });
  $('#run-cases').addEventListener('click', runCases);
  window.addEventListener('resize', () => { if (st.frame) render(st.frame); });
  loadExample(sel.value);
}

/// The address `#key`, `#key/Presentation` or `#key/Presentation@seconds` opens that
/// program, presentation and instant (presentation time in a lesson, simulation time in a
/// session).
function parseAddress() {
  const m = decodeURIComponent(location.hash.slice(1)).match(/^([^/@]*)(?:\/([^@]*))?(?:@([0-9.]+))?$/);
  if (!m) return { key: '' };
  return { key: m[1], presentation: m[2] || null, time: m[3] != null ? parseFloat(m[3]) : null };
}

function loadExample(key) {
  const ex = st.examples.find((e) => e.key === key);
  $('#source').value = ex.source;
  compileSource();
}

function compileSource() {
  const src = $('#source').value;
  const list = $('#diagnostics');
  list.replaceChildren();
  stop();
  try {
    st.player = new WebPlayer(src);
  } catch (e) {
    st.player = null;
    const diags = parseErr(e);
    for (const d of diags) {
      const where = d.line ? `${d.line}:${d.col}` : '';
      const b = el('button', { type: 'button' }, el('code', { text: `${d.code} ${where}` }), ' ', d.message);
      b.addEventListener('click', () => selectSource(d.line, d.col, d.end - d.start));
      list.append(el('li', {}, b));
    }
    clearStage();
    status(`${diags.length} diagnostic${diags.length === 1 ? '' : 's'}: the program does not compile.`, true);
    selectTab('source');
    return;
  }
  st.catalogue = JSON.parse(st.player.catalogue());
  const psel = $('#presentation');
  psel.replaceChildren();
  for (const p of st.catalogue.presentations) {
    const note = p.kind === 'observations' ? ' (no views)' : p.kind === 'lesson' ? ' (lesson)' : '';
    psel.append(el('option', { value: p.name, text: p.name + note }));
  }
  const first = st.catalogue.presentations.find((p) => p.kind !== 'observations') || st.catalogue.presentations[0];
  if (first) psel.value = first.name;
  const asked = st.want && st.want.presentation;
  if (asked && st.catalogue.presentations.some((p) => p.name === asked)) psel.value = asked;
  $('#cases').replaceChildren();
  $('#cases-summary').textContent = `${st.catalogue.cases.length} case${st.catalogue.cases.length === 1 ? '' : 's'}`;
  openPresentation();
}

function selectSource(line, col, len) {
  const ta = $('#source');
  const lines = ta.value.split('\n');
  let idx = 0;
  for (let i = 0; i < line - 1 && i < lines.length; i++) idx += lines[i].length + 1;
  idx += Math.max(0, col - 1);
  selectTab('source');
  ta.focus();
  ta.setSelectionRange(idx, idx + Math.max(1, len || 1));
  const lh = parseFloat(getComputedStyle(ta).lineHeight) || 18;
  ta.scrollTop = Math.max(0, (line - 4) * lh);
}

function clearStage() {
  $('#views').replaceChildren();
  $('#overlay').replaceChildren();
  $('#overlay').hidden = true;
  $('#captions').replaceChildren();
  $('#transport').hidden = true;
  $('#interactive-bar').hidden = true;
  $('#medium-label').hidden = true;
  $('#observations').replaceChildren();
  $('#description').replaceChildren();
  st.items.clear();
  st.views = {};
  st.layout = null;
  st.frame = null;
}

function openPresentation() {
  stop();
  clearStage();
  if (!st.player) return;
  const name = $('#presentation').value;
  const info = st.catalogue.presentations.find((p) => p.name === name);
  if (!info) { status('The program declares no presentation.'); return; }
  $('#medium-label').hidden = info.kind !== 'lesson';
  if (info.kind === 'observations') {
    status(`${name} has no views: its observations and expectations run headless. See the Cases tab.`);
    selectTab('cases');
    return;
  }
  try {
    st.layout = JSON.parse(st.player.open(name, $('#medium').value === 'video'));
  } catch (e) {
    const diags = parseErr(e);
    status(diags.map((d) => `${d.code}: ${d.message}`).join('; '), true);
    return;
  }
  buildViews();
  st.p = 0;
  st.sess = null;
  $('#beats').hidden = false;
  $('#restart').textContent = 'Restart';
  $('#restart').title = 'Play the lesson again without your inputs';
  $('#voice-label').hidden = st.layout.mode !== 'lesson' || !(st.layout.lesson.captions || []).length;
  if (st.layout.mode === 'lesson') {
    st.lesson = st.layout.lesson;
    checkVoice();
    $('#transport').hidden = false;
    const controls = st.layout.permits.includes('timeline_controls');
    $('#scrub').disabled = !controls;
    $('#speed').disabled = !controls;
    buildBeats();
    status(lessonNotes());
  } else {
    st.lesson = null;
    st.sess = st.layout.session;
    st.p = st.sess.t;
    $('#interactive-bar').hidden = false;
    if (st.sess.dynamic) {
      // A running model: the transport plays the session's run (RC section 12).
      $('#transport').hidden = false;
      $('#scrub').disabled = false;
      $('#speed').disabled = false;
      $('#beats').hidden = true;
      $('#restart').textContent = 'Reset';
      $('#restart').title = 'A new run with no interventions, from the start';
    }
    status('');
  }
  if (st.want && st.want.time != null && clocked()) st.p = Math.max(clockStart(), Math.min(clockEnd(), st.want.time));
  st.want = null;
  refresh();
}

function lessonNotes() {
  const l = st.lesson;
  const notes = [];
  if (l.medium === 'video') notes.push('Video medium: explore beats play their fallbacks (PK-9.10).');
  for (const n of st.voice.notes) notes.push(n);
  for (const d of l.diagnostics) notes.push(d);
  for (const u of l.unsupported) notes.push(u);
  return notes.join(' ');
}

// ------------------------------------------------------------------ views

function buildViews() {
  const host = $('#views');
  // The author's title of the presentation (D-076).
  if (st.layout.title) host.append(el('h2', { class: 'pres-title', text: st.layout.title }));
  for (const v of st.layout.views) {
    const fig = el('figure', { class: 'view', 'aria-label': `view ${v.title || v.name}` });
    // A view's caption: the author's `title(...)`, or its name and kind (D-076).
    const cap = el('figcaption', {}, el('span', { text: v.title || `${v.name} (${v.kind})` }));
    fig.append(cap);
    const entry = { ...v, fig, panel: el('div', { class: 'view-panel' }) };
    if (v.kind === 'spatial') {
      // The engine frames the view (D-047); each frame gives the box it shows.
      entry.svg = svg('svg', { role: 'group', 'aria-label': `${v.name}: spatial view` });
      // A wide scene spans the row; a small one sits beside the other views.
      if (v.extent[2] - v.extent[0] + 80 > 480) fig.classList.add('wide');
    } else if (v.kind === 'plot') {
      entry.W = 560; entry.H = 360; entry.m = 44;
      entry.svg = svg('svg', { viewBox: `0 0 ${entry.W} ${entry.H}`, role: 'group', 'aria-label': `${v.name}: plot` });
    }
    // Plots zoom and pan like spatial views (D-070); views of three dimensions turn (D-104).
    if (entry.svg && ['zoom', 'pan', 'orbit'].some((w) => st.layout.permits.includes(w))) {
      const reset = el('button', { type: 'button', class: 'hint', text: 'Reset view' });
      reset.addEventListener('click', () => { st.player.view_reset(v.id); refresh(); });
      cap.append(reset);
    }
    if (entry.svg) {
      fig.append(entry.svg);
      setupPointer(entry);
    }
    fig.append(entry.panel);
    st.views[v.id] = entry;
  }
  // The author's page layout (PK-7.4a, D-063): rows and columns of views, a row turned into
  // a column on a narrow screen. Without one, the views fill a grid in order.
  host.classList.toggle('paged', !!st.layout.page);
  const place = (node) => {
    if (node.view) return st.views[node.view]?.fig;
    const kids = (node.row || node.column || []).map(place).filter(Boolean);
    return el('div', { class: node.row ? 'page-row' : 'page-column' }, ...kids);
  };
  if (st.layout.page) host.append(place(st.layout.page));
  else for (const v of st.layout.views) host.append(st.views[v.id].fig);
}

/// View coordinates of a plot view to SVG coordinates: the frame's box (the plot's ranges)
/// fills the drawn area less its margin (D-047).
function plotMap(v) {
  const [x0, y0, bw, bh] = v.box || [v.x[0], v.y[0], v.x[1] - v.x[0], v.y[1] - v.y[0]];
  const sx = (v.W - 2 * v.m) / bw, sy = (v.H - 2 * v.m) / bh;
  return { to: ([x, y]) => [v.m + (x - x0) * sx, v.H - v.m - (y - y0) * sy] };
}

function mapFor(v) {
  if (v.kind === 'plot') return plotMap(v);
  return { to: (p) => p };
}

/// SVG user units per screen pixel, to keep strokes and handles a constant size. The box is
/// fitted into the element, so the larger ratio applies.
function unitsPerPx(v) {
  const r = v.svg.getBoundingClientRect();
  const w = r.width || 600;
  if (v.kind === 'spatial') return Math.max(v.box[2] / w, r.height ? v.box[3] / r.height : 0);
  return v.W / w;
}

function drawSpatialChrome(v, g, reps, u) {
  const [x, y, w, h] = v.box;
  const step = niceStep((60 * u) / v.px_per_m) * v.px_per_m; // at least ~60 screen px
  const metres = step / v.px_per_m;
  // In a view of three dimensions the axes and grid come drawn as projected lines (D-100).
  const hasGrid = reps.some((r) => r.shape === 'grid');
  const hasAxes = reps.some((r) => r.shape === 'axes');
  if (hasGrid) {
    for (let gx = Math.ceil(x / step) * step; gx <= x + w; gx += step) svg('line', { class: 'gridline', x1: gx, y1: y, x2: gx, y2: y + h, 'stroke-width': u }, g);
    for (let gy = Math.ceil(y / step) * step; gy <= y + h; gy += step) svg('line', { class: 'gridline', x1: x, y1: gy, x2: x + w, y2: gy, 'stroke-width': u }, g);
  }
  if (hasAxes) {
    svg('line', { class: 'axis', x1: x, y1: 0, x2: x + w, y2: 0, 'stroke-width': u }, g);
    svg('line', { class: 'axis', x1: 0, y1: y, x2: 0, y2: y + h, 'stroke-width': u }, g);
    const fs = 10 * u;
    for (let gx = Math.ceil(x / step) * step; gx <= x + w; gx += step) {
      if (Math.abs(gx) < 1e-9) continue;
      svg('line', { class: 'axis', x1: gx, y1: -3 * u, x2: gx, y2: 3 * u, 'stroke-width': u }, g);
      const t = svg('text', { class: 'tick', x: gx, y: 14 * u, 'text-anchor': 'middle', 'font-size': fs }, g);
      t.textContent = fmt(gx / v.px_per_m);
    }
    const up = v.y_up ? -1 : 1;
    for (let gy = Math.ceil(y / step) * step; gy <= y + h; gy += step) {
      if (Math.abs(gy) < 1e-9) continue;
      svg('line', { class: 'axis', x1: -3 * u, y1: gy, x2: 3 * u, y2: gy, 'stroke-width': u }, g);
      const t = svg('text', { class: 'tick', x: -6 * u, y: gy + 3 * u, 'text-anchor': 'end', 'font-size': fs }, g);
      t.textContent = fmt((up * gy) / v.px_per_m);
    }
    const axes = v.axes && v.axes.length >= 2 ? v.axes : ['x', 'y'];
    const tx = svg('text', { class: 'tick', x: x + w - 6 * u, y: -6 * u, 'text-anchor': 'end', 'font-size': fs }, g);
    tx.textContent = `${axes[0]} (m)`;
    const ty = svg('text', { class: 'tick', x: 6 * u, y: (v.y_up ? y : y + h) + 14 * u, 'font-size': fs }, g);
    ty.textContent = `${axes[1]} (m), grid ${fmt(metres)} m`;
  }
}

function drawPlotChrome(v, g) {
  const pm = plotMap(v);
  // The ranges shown (the frame's box: followed data, zoom and pan) in each axis's display
  // unit (D-070); ticks are placed in those units.
  const [kx, ky] = v.unit_scale || [1, 1];
  const [bx, by, bw, bh] = v.box || [v.x[0], v.y[0], v.x[1] - v.x[0], v.y[1] - v.y[0]];
  const [x0, x1] = [bx / kx, (bx + bw) / kx], [y0, y1] = [by / ky, (by + bh) / ky];
  const m = { to: ([x, y]) => pm.to([x * kx, y * ky]) };
  svg('rect', { class: 'frame', x: v.m, y: v.m, width: v.W - 2 * v.m, height: v.H - 2 * v.m }, g);
  // About one tick per 70 px across and 32 px down.
  const xs = niceStep((x1 - x0) / Math.max(2, (v.W - 2 * v.m) / 70));
  const ys = niceStep((y1 - y0) / Math.max(2, (v.H - 2 * v.m) / 32));
  for (let x = Math.ceil(x0 / xs) * xs; x <= x1 + 1e-9; x += xs) {
    const [X] = m.to([x, y0]);
    svg('line', { class: 'gridline', x1: X, y1: v.m, x2: X, y2: v.H - v.m }, g);
    const t = svg('text', { class: 'tick', x: X, y: v.H - v.m + 16, 'text-anchor': 'middle', 'font-size': 12 }, g);
    t.textContent = fmt(Math.abs(x) < xs * 1e-9 ? 0 : x);
  }
  for (let y = Math.ceil(y0 / ys) * ys; y <= y1 + 1e-9; y += ys) {
    const [, Y] = m.to([x0, y]);
    svg('line', { class: 'gridline', x1: v.m, y1: Y, x2: v.W - v.m, y2: Y }, g);
    const t = svg('text', { class: 'tick', x: v.m - 6, y: Y + 4, 'text-anchor': 'end', 'font-size': 12 }, g);
    t.textContent = fmt(Math.abs(y) < ys * 1e-9 ? 0 : y);
  }
  if (x0 < 0 && x1 > 0) { const [X] = m.to([0, 0]); svg('line', { class: 'axis', x1: X, y1: v.m, x2: X, y2: v.H - v.m }, g); }
  if (y0 < 0 && y1 > 0) { const [, Y] = m.to([0, 0]); svg('line', { class: 'axis', x1: v.m, y1: Y, x2: v.W - v.m, y2: Y }, g); }
  const [ux, uy] = v.units || ['', ''];
  if (ux) {
    const t = svg('text', { class: 'tick', x: v.W - v.m, y: v.H - 8, 'text-anchor': 'end', 'font-size': 12 }, g);
    t.textContent = ux === 's' && v.x_elapsed !== false ? 'elapsed time (s)' : `(${ux})`;
  }
  if (uy) {
    const t = svg('text', { class: 'tick', x: v.m, y: v.m - 10, 'font-size': 12 }, g);
    t.textContent = `(${uy})`;
  }
}

function arrowHead(from, to, size) {
  const dx = to[0] - from[0], dy = to[1] - from[1];
  const len = Math.hypot(dx, dy);
  if (len < 1e-9) return null;
  const ux = dx / len, uy = dy / len;
  const s = Math.min(size, len * 0.6);
  const bx = to[0] - ux * s, by = to[1] - uy * s;
  return `${to[0]},${to[1]} ${bx - uy * s * 0.45},${by + ux * s * 0.45} ${bx + uy * s * 0.45},${by - ux * s * 0.45}`;
}

/// The path of an `ellipse` shape: arcs of at most a quarter turn each; a whole one closed.
function curvePath(r) {
  const [cx, cy] = r.center, [rx, ry] = r.radii;
  const s = Math.sin(r.rotation), c = Math.cos(r.rotation);
  const at = (a) => {
    const x = rx * Math.cos(a), y = ry * Math.sin(a);
    return `${cx + c * x - s * y} ${cy + s * x + c * y}`;
  };
  const pieces = Math.max(1, Math.ceil(Math.abs(r.sweep) / (Math.PI / 2)));
  let d = `M${at(r.start)}`;
  for (let k = 1; k <= pieces; k++) {
    d += ` A${rx} ${ry} ${(r.rotation * 180) / Math.PI} 0 ${r.sweep > 0 ? 1 : 0} ${at(r.start + (r.sweep * k) / pieces)}`;
  }
  return r.closed ? `${d} Z` : d;
}

/// The color a representation of this kind is drawn with when the author gives none.
function ownColor(r, colorIndex) {
  switch (r.shape) {
    case 'point': return 'var(--rep-2)';
    case 'arrow': return ['var(--rep)', 'var(--rep-2)', 'var(--rep-3)', 'var(--rep-4)'][colorIndex % 4];
    case 'segment': return 'var(--rep)';
    case 'polyline': return r.kind === 'trace' ? 'var(--muted)' : 'var(--accent)';
    default: return 'var(--accent)';
  }
}

// The palettes of frames (D-091), the same stops as prismal-svg: sequential (dark blue,
// green, yellow) and diverging about 0 (blue, near white, red).
const PALETTES = {
  sequential: [[68, 1, 84], [59, 82, 139], [33, 145, 140], [94, 201, 98], [253, 231, 37]],
  diverging: [[59, 76, 192], [242, 242, 242], [180, 4, 38]],
};
function paletteRgb(name, v) {
  const stops = PALETTES[name] || PALETTES.sequential;
  const x = Math.min(Math.max(v, 0), 1) * (stops.length - 1);
  const i = Math.min(Math.floor(x), stops.length - 2);
  const w = x - i;
  return [0, 1, 2].map((k) => Math.round(stops[i][k] + w * (stops[i + 1][k] - stops[i][k])));
}

function drawRep(v, g, r, u, colorIndex) {
  const m = mapFor(v);
  const cls = ['rep'];
  if (r.drag) cls.push('draggable');
  else if (r.click) cls.push('clickable');
  if (r.valid === false) cls.push('invalid');
  // The author's color (D-061); an invalid preview keeps its own. During a color
  // animation (D-075), a mix from the author's color, or the kind's own, to the next.
  const colored = (r.color || r.color_to) && r.valid !== false;
  if (colored) cls.push('styled');
  const grp = svg('g', { class: cls.join(' '), 'data-rep': r.id }, g);
  if (colored) {
    const from = r.color ? `var(--color-${r.color})` : ownColor(r, colorIndex);
    const c = r.color_to ? `color-mix(in srgb, ${from} ${Math.round((1 - r.color_mix) * 1000) / 10}%, var(--color-${r.color_to}))` : from;
    grp.style.setProperty('--c', c);
  }
  // An animated size (D-075): the dot and the arrow head grow with it.
  const grow = r.scale ?? 1;
  if (r.opacity != null) grp.setAttribute('opacity', r.opacity);
  const title = svg('title', {}, grp);
  title.textContent = r.text;
  let focusable = null;
  switch (r.shape) {
    case 'lines': {
      // Axes and grid of a view of three dimensions, projected by the engine (D-100).
      const cls = r.kind === 'axes' ? 'axis' : 'gridline';
      for (const [a, b] of r.lines) {
        const [A, B] = [m.to(a), m.to(b)];
        svg('line', { class: cls, x1: A[0], y1: A[1], x2: B[0], y2: B[1], 'stroke-width': u }, grp);
      }
      for (const l of r.labels) {
        const [X, Y] = m.to(l.at);
        const t = svg('text', { class: 'tick', x: X, y: Y, 'font-size': 10 * u, 'text-anchor': 'middle' }, grp);
        t.textContent = l.text;
      }
      break;
    }
    case 'point': {
      const [X, Y] = m.to(r.at);
      if (r.highlighted) svg('circle', { class: 'ring', cx: X, cy: Y, r: 13 * u, 'stroke-width': 3 * u }, grp);
      const c = svg('circle', { class: 'marker', cx: X, cy: Y, r: (r.drag || r.click ? 8 : 6) * u * grow, 'stroke-width': 1.5 * u }, grp);
      if (r.label) {
        const t = svg('text', { class: 'rep-label', x: X + 10 * u, y: Y - 10 * u, 'font-size': 12 * u }, grp);
        t.textContent = r.label;
      }
      if (r.drag || r.click) focusable = c;
      break;
    }
    case 'arrow': {
      const a = m.to(r.from), b = m.to(r.to);
      grp.classList.add('arrow', `c${colorIndex % 4}`);
      svg('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1], 'stroke-width': 2 * u }, grp);
      const head = arrowHead(a, b, 11 * u * grow);
      if (head) svg('polygon', { points: head }, grp);
      if (r.highlighted) svg('circle', { class: 'ring', cx: b[0], cy: b[1], r: 13 * u, 'stroke-width': 3 * u }, grp);
      const len = Math.hypot(b[0] - a[0], b[1] - a[1]);
      if (r.label && len > 24 * u) {
        // At the middle, offset to the left of the direction of the arrow.
        const nx = -(b[1] - a[1]) / len, ny = (b[0] - a[0]) / len;
        const t = svg('text', { class: 'rep-label', x: (a[0] + b[0]) / 2 + nx * 12 * u, y: (a[1] + b[1]) / 2 + ny * 12 * u + 4 * u, 'text-anchor': 'middle', 'font-size': 12 * u }, grp);
        t.textContent = r.label;
      }
      if (r.drag === 'head') {
        focusable = svg('circle', { class: 'handle', cx: b[0], cy: b[1], r: 7 * u, 'stroke-width': 2 * u, 'data-part': 'head' }, grp);
      }
      break;
    }
    case 'note': {
      // An author's text at a point (D-076).
      const [X, Y] = m.to(r.at);
      const anchor = { center: 'middle', right: 'end' }[r.align] || 'start';
      const t = svg('text', { class: 'note', x: X, y: Y, 'font-size': 13 * u, 'text-anchor': anchor }, grp);
      t.textContent = r.value;
      break;
    }
    case 'segment': {
      const a = m.to(r.from), b = m.to(r.to);
      svg('line', { class: 'segment', x1: a[0], y1: a[1], x2: b[0], y2: b[1], 'stroke-width': 2 * u }, grp);
      break;
    }
    case 'polyline': {
      const trace = r.kind === 'trace';
      // A sampled path breaks where its value jumped (a ball thrown again).
      const cuts = [0, ...(r.breaks || []), r.points.length];
      for (let k = 0; k + 1 < cuts.length; k++) {
        const pts = r.points.slice(cuts[k], cuts[k + 1]).map((p) => m.to(p).join(',')).join(' ');
        svg('polyline', { class: trace ? 'trace' : 'graph', points: pts, 'stroke-width': (trace ? 1.5 : 2) * u }, grp);
      }
      break;
    }
    case 'raster': {
      // Coloured cells of a field in the plane (D-091), with the colours prismal-svg gives
      // them. One image of one pixel per cell, scaled without smoothing: a ripple tank of
      // 60 by 60 cells would otherwise be 3600 elements redrawn every frame.
      const [fx, fy] = m.to(r.from);
      const [tx, ty] = m.to(r.to);
      const canvas = document.createElement('canvas');
      canvas.width = r.cols;
      canvas.height = r.rows;
      const ctx = canvas.getContext('2d');
      const img = ctx.createImageData(r.cols, r.rows);
      // Cell k is column k % cols from `from` towards `to`, row k / cols likewise; the image's
      // rows run downwards on the page.
      const flipX = tx < fx, flipY = ty < fy;
      r.values.forEach((v, k) => {
        const i = k % r.cols, j = Math.floor(k / r.cols);
        const px = flipX ? r.cols - 1 - i : i, py = flipY ? r.rows - 1 - j : j;
        const c = paletteRgb(r.palette, v);
        const o = 4 * (py * r.cols + px);
        img.data[o] = c[0];
        img.data[o + 1] = c[1];
        img.data[o + 2] = c[2];
        img.data[o + 3] = 255;
      });
      ctx.putImageData(img, 0, 0);
      svg('image', { class: 'raster', href: canvas.toDataURL(), x: Math.min(fx, tx), y: Math.min(fy, ty), width: Math.abs(tx - fx), height: Math.abs(ty - fy), preserveAspectRatio: 'none', style: 'image-rendering: pixelated' }, grp);
      break;
    }
    case 'polygon': {
      const pts = r.points.map((p) => m.to(p).join(',')).join(' ');
      // An opaque body (D-104) hides what is behind it: the surface under its tint.
      if (r.opaque) svg('polygon', { class: 'solid', points: pts }, grp);
      svg('polygon', { class: r.opaque ? 'shape opaque' : 'shape', points: pts, 'stroke-width': 2 * u }, grp);
      break;
    }
    case 'ellipse': {
      // A circle, ellipse or arc (PK-6.3c), exact in view coordinates.
      if (r.opaque && r.closed) svg('path', { class: 'solid', d: curvePath(r) }, grp);
      svg('path', { class: r.closed ? (r.opaque ? 'shape opaque' : 'shape') : 'curve', d: curvePath(r), 'stroke-width': 2 * u }, grp);
      break;
    }
    case 'group': {
      // Members are placed by the kernel; the group carries opacity and highlight (D-043).
      if (r.highlighted) grp.classList.add('highlighted');
      let arrows = 0;
      for (const mem of r.members) drawRep(v, grp, mem, u, mem.shape === 'arrow' ? colorIndex + arrows++ : 0);
      break;
    }
    default:
      grp.remove();
      return;
  }
  if (r.width) {
    // The author's line width (D-095), a multiple of each stroke's own.
    for (const e of grp.querySelectorAll('line, polyline, polygon, path')) {
      if (e.parentNode.classList.contains('arrow') && e.tagName === 'polygon') continue;
      const w = parseFloat(e.getAttribute('stroke-width'));
      if (w) e.setAttribute('stroke-width', w * r.width);
    }
  }
  if (r.line && r.drawn == null && r.valid !== false) {
    // The author's line style (D-061), as prismal-svg draws it.
    const dash = r.line === 'solid' ? 'none' : r.line === 'dashed' ? `${6 * u} ${4 * u}` : `${0.01 * u} ${4 * u}`;
    for (const e of grp.querySelectorAll('line, polyline, polygon, path')) {
      if (e.parentNode.classList.contains('arrow') && e.tagName === 'polygon') continue;
      e.style.strokeDasharray = dash;
      if (r.line === 'dotted') e.style.strokeLinecap = 'round';
    }
  }
  if (r.drawn != null) {
    // `reveal draw`: every stroke is drawn up to the fraction reached (D-042).
    for (const e of grp.querySelectorAll('line, polyline, polygon, path')) {
      if (e.parentNode.classList.contains('arrow') && e.tagName === 'polygon') {
        e.style.opacity = r.drawn > 0.95 ? 1 : 0;
        continue;
      }
      e.setAttribute('pathLength', '1');
      e.style.strokeDasharray = '1';
      e.style.strokeDashoffset = String(1 - r.drawn);
    }
  }
  // A representation clicked to request an event takes focus by its drawn shape (D-059).
  if (!focusable && r.click) focusable = grp.querySelector('line, polyline, polygon, path');
  if (focusable) {
    const how = [r.drag ? 'Arrow keys move it.' : '', r.click ? 'Enter activates it.' : ''].filter(Boolean).join(' ');
    focusable.setAttribute('tabindex', '0');
    focusable.setAttribute('role', 'button');
    focusable.setAttribute('aria-label', `${r.text}. ${how}`);
    focusable.classList.add('rep');
    focusable.dataset.rep = r.id;
    // The browser moves focus (Tab), for assistive technology; the engine is told, and
    // keys are forwarded to it (D-047).
    focusable.addEventListener('keydown', onRepKey);
    focusable.addEventListener('focus', () => {
      st.focus = { rep: r.id };
      st.player.focus(r.id, st.p);
    });
    focusable.addEventListener('blur', () => {
      // Re-rendering replaces the element; its focus is restored after the render.
      if (!st.rendering && st.focus && st.focus.rep === r.id) st.focus = null;
    });
  }
}

// ------------------------------------------------------------------ HTML items (panel reps)

function item(id, make) {
  let it = st.items.get(id);
  if (!it) {
    it = make();
    st.items.set(id, it);
  }
  it.seen = true;
  return it;
}

function renderItem(host, r) {
  const it = item(r.id, () => {
    const root = el('div', { class: `item ${r.shape}` });
    return { root, kind: r.shape };
  });
  it.root.classList.toggle('highlighted', !!r.highlighted);
  it.root.style.opacity = r.opacity != null ? r.opacity : '';
  it.root.title = r.text;
  if (it.root.parentElement !== host) host.append(it.root);
  switch (r.shape) {
    case 'formula': {
      if (it.mathml !== r.mathml) {
        it.root.innerHTML = r.mathml || '';
        it.values = el('div', { class: 'values' });
        it.root.append(it.values);
        it.root.setAttribute('aria-label', r.text);
        it.mathml = r.mathml;
      }
      const vals = r.symbols.filter((s) => s.value != null).map((s) => `${s.symbol} = ${s.value}`).join(',  ');
      it.values.textContent = vals;
      break;
    }
    case 'equation': {
      if (it.mathml !== r.mathml) {
        it.root.innerHTML = r.mathml || '';
        it.values = el('div', { class: 'values' });
        it.root.prepend(el('span', { class: 'what', text: `equation ${r.name}` }));
        it.root.append(it.values);
        it.root.setAttribute('aria-label', r.text);
        it.mathml = r.mathml;
      }
      it.values.textContent = r.symbols.filter((s) => s.value != null).map((s) => `${s.symbol} = ${s.value}`).join(',  ');
      break;
    }
    case 'button': {
      if (!it.button) {
        it.button = el('button', { type: 'button', text: r.label });
        it.button.addEventListener('click', () => {
          if (st.lesson) {
            // An explore beat's button (D-069): a learner input at the instant shown.
            try {
              st.lesson = JSON.parse(st.player.lesson_press(st.p, r.id));
              afterLessonInput();
            } catch (e) {
              status(String(e), true);
            }
            return;
          }
          report(JSON.parse(st.player.press(r.id)));
          afterSessionAction();
        });
        it.root.replaceChildren(it.button);
      }
      it.button.setAttribute('aria-label', r.text);
      break;
    }
    case 'table': {
      const table = el('table', { class: 'data' });
      table.append(el('thead', {}, el('tr', {}, ...r.columns.map((c) => el('th', { text: c })))));
      const body = el('tbody');
      // The latest rows, most recent last.
      for (const row of r.rows.slice(-12)) body.append(el('tr', {}, ...row.map((c) => el('td', { text: c }))));
      table.append(body);
      it.root.replaceChildren(el('span', { class: 'what', text: `${r.rows.length} rows` }), table);
      break;
    }
    case 'control':
      renderControl(it, r);
      break;
    case 'text':
      // A label's `name = value`, or an author's text (D-076).
      it.root.replaceChildren(el('span', { class: r.kind === 'text' ? 'author-text' : 'label-value', text: r.text }));
      break;
    case 'status':
      it.root.replaceChildren(el('span', { class: 'status-value', text: r.text }));
      break;
    default:
      it.root.replaceChildren(el('span', { class: 'hint', text: r.text }));
  }
}

function controlDisplay(r, value) {
  if (r.control === 'toggle') return value ? 'true' : 'false';
  if (r.options) return r.options[value] ?? '';
  if (r.display_unit) return `${fmt(value / r.display_unit.scale)} ${r.display_unit.text}`;
  return r.unit ? `${fmt(value)} ${r.unit}` : fmt(value);
}

function renderControl(it, r) {
  if (!it.input) {
    const label = el('label', { class: 'control' });
    const name = el('span', { class: 'what', text: `${r.symbol}` });
    let input;
    if (r.control === 'slider') {
      input = el('input', { type: 'range' });
      input.min = r.min ?? -10;
      input.max = r.max ?? 10;
      // Without a declared step the slider is continuous; keyboard steps are the kernel's
      // (PK-11.2a).
      input.step = r.step ?? 'any';
      input.addEventListener('keydown', (e) => onControlKey(e, r, it));
    } else if (r.control === 'toggle') {
      input = el('input', { type: 'checkbox' });
    } else if (r.control === 'select') {
      // The cases of an enumeration (D-094); the value is the chosen case's number.
      input = el('select');
      r.options.forEach((o, k) => input.append(el('option', { value: String(k), text: o })));
    } else {
      input = el('input', { type: 'number', step: r.step ?? 'any' });
      if (r.min != null) input.min = r.min;
      if (r.max != null) input.max = r.max;
    }
    const out = el('output');
    // Words at a slider's two ends (D-077).
    if (r.ends) {
      label.append(name, el('span', { class: 'end', text: r.ends[0] }), input, el('span', { class: 'end', text: r.ends[1] }), out);
    } else {
      label.append(name, input, out);
    }
    it.root.replaceChildren(label);
    it.input = input;
    it.out = out;
    const read = () => (r.control === 'toggle' ? (input.checked ? 1 : 0) : parseFloat(input.value));
    input.addEventListener('input', () => { it.out.textContent = controlDisplay(r, read()); it.editing = true; });
    input.addEventListener('change', () => { it.editing = false; commitControl(r.id, read()); });
    input.addEventListener('blur', () => { it.editing = false; });
  }
  it.input.setAttribute('aria-label', r.text);
  it.input.disabled = !!r.disabled;
  if (!it.editing) {
    if (r.control === 'toggle') it.input.checked = !!r.value;
    else it.input.value = r.value;
    it.out.textContent = controlDisplay(r, r.value);
  }
}

function onControlKey(e, r, it) {
  const keys = { ArrowLeft: 'left', ArrowDown: 'left', ArrowRight: 'right', ArrowUp: 'right' };
  const k = keys[e.key];
  if (!k) return;
  e.preventDefault();
  it.editing = false;
  if (st.lesson) {
    // An explore control: one step of 1/100 of the range, or the declared step.
    const step = r.step ?? ((r.max - r.min) / 100);
    const v = Math.min(r.max, Math.max(r.min, parseFloat(it.input.value) + (k === 'right' ? step : -step)));
    commitControl(r.id, v);
  } else {
    report(JSON.parse(st.player.key(r.id, k)));
    afterSessionAction();
  }
}

function commitControl(id, value) {
  if (st.lesson) {
    try {
      st.lesson = JSON.parse(st.player.lesson_set_control(st.p, id, value));
      afterLessonInput();
    } catch (e) {
      status(String(e), true);
    }
  } else {
    const res = JSON.parse(st.player.set_control(id, value));
    report(res);
    afterSessionAction();
  }
}

/// The run changes after an intervention: its span and diagnostics are read again.
function afterSessionAction() {
  if (st.sess) {
    const wasPaused = st.sess.status === 'paused';
    st.sess = JSON.parse(st.player.session());
    const d = st.sess.diagnostics;
    if (st.sess.status === 'paused') pausedNote();
    else if (wasPaused) status('The run continues: the change cleared the failure.');
    else if (d.length && !$('#status').classList.contains('bad')) status(`Run: ${d[0]}${d.length > 1 ? ` (and ${d.length - 1} more)` : ''}`, true);
  }
  refresh();
}

/// A failed interactive run pauses at its last committed state (RC-10.3): say why, and how
/// to go on.
function pausedNote() {
  const f = st.sess.failure;
  status(`Paused at t = ${fmt(f.t)} s: ${f.message}. Change a value (a slider, a drag, a button) to continue from here, or Reset.`, true);
}

function report(res) {
  if (res.ok) status('');
  else status(`${res.why === 'refused' ? 'Refused' : 'Rejected'}: ${res.message}`, true);
}

// ------------------------------------------------------------------ rendering

function render(frame) {
  st.rendering = true;
  try {
    renderFrame(frame);
  } finally {
    st.rendering = false;
  }
}

function renderFrame(frame) {
  st.frame = frame;
  for (const it of st.items.values()) it.seen = false;
  for (const vf of frame.views) {
    const v = st.views[vf.id];
    if (!v) continue;
    if (v.svg) {
      // The engine gives the box of view coordinates each view shows: its framing, the
      // learner's zoom and pan, the timeline's camera (D-047).
      v.box = vf.box;
      // A view that requests an event when an empty point is clicked (D-060).
      v.click = vf.click || null;
      if (v.kind === 'spatial') {
        v.svg.setAttribute('viewBox', v.box.join(' '));
      } else {
        v.m = vf.margin;
        // A plot is drawn at its rendered size, so that text and margins stay readable.
        const w = Math.max(240, Math.round(v.svg.clientWidth || 560));
        if (w !== v.W) {
          v.W = w;
          v.H = Math.round(Math.min(w * 0.62, 420));
          v.svg.setAttribute('viewBox', `0 0 ${v.W} ${v.H}`);
        }
      }
      const u = unitsPerPx(v);
      v.svg.replaceChildren();
      const g = svg('g', {}, v.svg);
      if (v.kind === 'spatial') drawSpatialChrome(v, g, vf.reps, u);
      else drawPlotChrome(v, g);
      if (v.kind === 'plot') {
        const clip = svg('clipPath', { id: `clip-${cssId(v.id)}` }, v.svg);
        svg('rect', { x: v.m, y: v.m, width: v.W - 2 * v.m, height: v.H - 2 * v.m }, clip);
      }
      const reps = svg('g', v.kind === 'plot' ? { 'clip-path': `url(#clip-${cssId(v.id)})` } : {}, v.svg);
      const top = svg('g', {}, v.svg);
      let arrows = 0;
      for (const r of vf.reps) {
        if (['lines', 'point', 'arrow', 'segment', 'polyline', 'polygon', 'ellipse', 'group', 'note', 'raster'].includes(r.shape)) {
          // Draggable and clickable representations are drawn last and outside the plot's clip.
          drawRep(v, r.drag || r.click ? top : reps, r, u, r.shape === 'arrow' ? arrows++ : 0);
        } else if (!['axes', 'grid', 'title'].includes(r.shape)) {
          renderItem(v.panel, r);
        }
      }
    } else {
      // A title is the panel's caption, not an item (D-076).
      for (const r of vf.reps) if (r.shape !== 'title') renderItem(v.panel, r);
    }
  }
  const overlay = $('#overlay');
  const lessonExplore = st.lesson && exploreAt(st.p);
  for (const r of frame.overlay || []) {
    if (r.shape === 'control' && st.lesson && !lessonExplore) r.disabled = true;
    renderItem(overlay, r);
  }
  for (const [id, it] of st.items) {
    if (!it.seen) { it.root.remove(); st.items.delete(id); }
  }
  overlay.hidden = overlay.children.length === 0;
  const caps = $('#captions');
  caps.replaceChildren(...(frame.captions || []).map((c) => el('span', { text: c })));
  for (const a of frame.announcements || []) announce(`${a} at ${fmt(frame.t)} s`);
  // The engine's focus (a grabbed representation takes it, D-047), else the browser's.
  if (frame.focus) st.focus = { rep: frame.focus };
  if (st.focus) {
    const f = document.querySelector(`[tabindex][data-rep="${CSS.escape(st.focus.rep)}"]`);
    if (f && document.activeElement !== f) f.focus({ preventScroll: true });
  }
  renderDescription(frame);
}

function cssId(id) {
  return id.replace(/[^a-zA-Z0-9_-]/g, '_');
}

let lastDescription = 0;
function renderDescription(frame) {
  const now = performance.now();
  if (st.playing && now - lastDescription < 500) return;
  lastDescription = now;
  const list = $('#description');
  const lines = [`${frame.run} run, simulation time ${fmt(frame.t)} s`];
  for (const [, r] of allReps(frame)) if (r.kind !== 'axes' && r.kind !== 'grid') lines.push(r.text);
  list.replaceChildren(...lines.map((t) => el('li', { text: t })));
}

function refresh() {
  if (!st.player || !st.layout) return;
  if (st.sess) st.p = JSON.parse(st.player.seek(st.p)).t;
  const frame = JSON.parse(st.player.frame(st.p, 0));
  render(frame);
  renderObservations();
  if (clocked()) updateTransport();
}

/// Whether the open presentation has a clock: a lesson, or a session of a dynamic model.
function clocked() {
  return !!(st.lesson || (st.sess && st.sess.dynamic));
}

function clockStart() {
  return st.lesson ? 0 : st.sess.t0;
}

function clockEnd() {
  return st.lesson ? st.lesson.end : st.sess.end;
}

function renderObservations() {
  const obs = JSON.parse(st.player.observations());
  const dl = $('#observations');
  dl.replaceChildren();
  for (const [name, lines] of Object.entries(obs)) {
    const dd = el('dd');
    if (lines.length === 0) dd.append(el('span', { class: 'hint', text: 'none' }));
    for (const l of lines.slice(-50)) dd.append(el('div', { text: l }));
    dl.append(el('dt', { text: name }), dd);
  }
  if (!dl.children.length) dl.append(el('dt', { text: '-' }), el('dd', { class: 'hint', text: 'No observations for this mode.' }));
}

// ------------------------------------------------------------------ pointer and keyboard

// The player catches the browser's events and forwards them to the engine, which targets
// them, runs drags, pans and zooms, and keeps keyboard focus (D-047, HI-4.5). Positions are
// pixels of the view as drawn: screen pixels in a spatial view, whose box the engine fits
// into the element, and the plot's own drawing units in a plot.
function drawnPoint(v, e) {
  if (v.kind === 'plot') {
    const pt = v.svg.createSVGPoint();
    pt.x = e.clientX; pt.y = e.clientY;
    const q = pt.matrixTransform(v.svg.getScreenCTM().inverse());
    return { x: q.x, y: q.y, width: v.W, height: v.H };
  }
  const r = v.svg.getBoundingClientRect();
  return { x: e.clientX - r.left, y: e.clientY - r.top, width: r.width, height: r.height };
}

function forward(v, phase, e) {
  const p = drawnPoint(v, e);
  const id = Number.isInteger(e.pointerId) ? e.pointerId : -1;
  return JSON.parse(st.player.pointer(phase, v.id, p.x, p.y, p.width, p.height, e.pointerType || 'mouse', id, st.p));
}

function setupPointer(v) {
  v.svg.addEventListener('pointerdown', (e) => {
    // A second touch during a pan pinches the view (HI-4.5).
    if (st.gesture && st.gesture.action === 'pan' && st.gesture.view === v && e.pointerType === 'touch') {
      const res = forward(v, 'down', e);
      if (res.action === 'pinch') {
        e.preventDefault();
        st.gesture.action = 'pinch';
        st.gesture.ids.push(e.pointerId);
        v.svg.setPointerCapture(e.pointerId);
      }
      return;
    }
    if (st.gesture) return;
    const res = forward(v, 'down', e);
    if (!res.handled) return;
    e.preventDefault();
    if (res.action === 'drag' && !res.ok) { report(res); return; }
    // Drag mode `hold`: the run pauses while dragging and resumes after the commit (PK-10.9).
    // Drag mode `live`: the run keeps going and the drag steers it (D-071).
    const hold = res.action === 'drag' && !res.live;
    st.gesture = { action: res.action, view: v, ids: [e.pointerId], resume: hold && st.playing };
    if (hold) stop();
    v.svg.setPointerCapture(e.pointerId);
  });
  v.svg.addEventListener('pointermove', (e) => {
    const g = st.gesture;
    if (g && !g.ids.includes(e.pointerId)) return;
    const res = forward(v, 'move', e);
    if (!g) {
      // Hovering: show what can be grabbed, or clicked (D-059); an empty point of a view
      // that takes clicks shows a crosshair (D-060).
      v.svg.style.cursor = res.hover ? (res.hover.drag ? 'grab' : 'pointer') : (v.click && !st.lesson ? 'crosshair' : '');
      return;
    }
    if (res.action === 'drag') status(res.ok ? '' : `Not valid here: ${res.message || 'rejected'}`, !res.ok);
    render(JSON.parse(st.player.frame(st.p, 0)));
  });
  v.svg.addEventListener('pointerup', (e) => {
    const g = st.gesture;
    if (!g || !g.ids.includes(e.pointerId)) return;
    st.gesture = null;
    const res = forward(v, 'up', e);
    if (res.action === 'click') {
      report(res);
      afterSessionAction();
      if (g.resume) togglePlay();
      return;
    }
    if (res.action !== 'drag') { refresh(); return; }
    if (!res.ok) report(res);
    else if (!res.committed) status('Nothing committed: no valid position during the drag.', true);
    else status('');
    afterSessionAction();
    if (g.resume) togglePlay();
  });
  v.svg.addEventListener('pointercancel', (e) => {
    if (!st.gesture) return;
    st.gesture = null;
    forward(v, 'cancel', e);
    refresh();
  });
  v.svg.addEventListener('wheel', (e) => {
    const p = drawnPoint(v, e);
    const res = JSON.parse(st.player.wheel(v.id, p.x, p.y, p.width, p.height, e.deltaY, st.p));
    if (!res.handled) return;
    e.preventDefault();
    refresh();
  }, { passive: false });
}

window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && st.gesture) {
    st.gesture = null;
    JSON.parse(st.player.key_down('Escape', false, st.p));
    status('Drag cancelled.');
    refresh();
  }
  if (e.key === ' ' && clocked() && !['INPUT', 'TEXTAREA', 'SELECT', 'BUTTON'].includes(document.activeElement.tagName) && !document.activeElement.dataset.rep) {
    e.preventDefault();
    togglePlay();
  }
});

/// Keys on a focused representation in a view go to the engine, which steps what has focus
/// (PK-11.2); the browser keeps moving focus with Tab.
function onRepKey(e) {
  if (e.key === 'Tab') return;
  const res = JSON.parse(st.player.key_down(e.key, e.shiftKey, st.p));
  if (!res.handled) return;
  e.preventDefault();
  if (res.ok === false) report(res);
  else status('');
  afterSessionAction();
}

// ------------------------------------------------------------------ lesson transport

function setupTransport() {
  $('#play').addEventListener('click', togglePlay);
  $('#restart').addEventListener('click', () => {
    stop();
    if (st.lesson) {
      st.layout = JSON.parse(st.player.lesson_restart());
      st.lesson = st.layout.lesson;
      st.p = 0;
      buildBeats();
      status(lessonNotes());
    } else {
      st.layout = JSON.parse(st.player.reset());
      st.sess = st.layout.session;
      st.p = st.sess.t;
      status('');
    }
    refresh();
  });
  $('#continue').addEventListener('click', learnerContinues);
  $('#voice').addEventListener('change', () => {
    silence();
    if ($('#voice').value === 'files') $('#voice-files').click();
    else { st.voice.mode = $('#voice').value; checkVoice(); status(lessonNotes()); }
  });
  $('#voice-files').addEventListener('cancel', () => { $('#voice').value = st.voice.mode; });
  $('#voice-files').addEventListener('change', () => {
    for (const u of st.voice.files.values()) URL.revokeObjectURL(u);
    st.voice.files = new Map([...$('#voice-files').files].map((f) => [f.name.replace(/\.[^.]*$/, ''), URL.createObjectURL(f)]));
    st.voice.mode = st.voice.files.size ? 'files' : 'off';
    $('#voice').value = st.voice.mode;
    checkVoice();
    status(lessonNotes());
  });
  $('#scrub').addEventListener('input', () => {
    silence();
    st.p = parseFloat($('#scrub').value);
    refresh();
  });
  $('#speed').addEventListener('change', () => {
    st.speed = parseFloat($('#speed').value);
    for (const s of st.voice.sounding.values()) if (s instanceof HTMLAudioElement) s.playbackRate = st.speed;
  });
}

function afterLessonInput() {
  buildBeats();
  const refusals = st.lesson.refusals || [];
  const last = refusals[refusals.length - 1];
  if (last && Math.abs(last.at - st.p) < 1e-9) status(`Refused: ${last.reason}`, true);
  else status(lessonNotes());
  refresh();
}

// The learner continues at the current instant: ends an explore beat or a continue point
// (D-067), and plays on.
function learnerContinues() {
  try {
    st.lesson = JSON.parse(st.player.lesson_continue(st.p));
    afterLessonInput();
    if (!st.playing) togglePlay();
  } catch (e) {
    status(String(e), true);
  }
}

// A continue point the learner has not passed yet, at instant p: playback stops there (D-067).
function pendingAt(p) {
  return (st.lesson?.waits || []).find((w) => w.pending && Math.abs(w.at - p) < 1e-9) || null;
}

function exploreAt(p) {
  return (st.lesson?.explore || []).find((x) => p >= x.start && p < x.end) || null;
}

function buildBeats() {
  const host = $('#beats');
  host.replaceChildren();
  const end = st.lesson.end || 1;
  for (const b of st.lesson.beats) {
    const w = Math.max(0, b.end - b.start);
    const btn = el('button', { type: 'button', role: 'listitem', title: `${b.scene} / ${b.beat}: ${fmt(b.start)} s to ${fmt(b.end)} s`, text: b.beat });
    btn.style.flex = `${Math.max(w, end * 0.012)} 1 0`;
    if (st.lesson.explore.some((x) => x.beat === b.beat)) btn.classList.add('explore');
    btn.addEventListener('click', () => { silence(); st.p = b.start; refresh(); });
    btn.dataset.beat = b.beat;
    host.append(btn);
  }
}

function updateTransport() {
  const [t0, end] = [clockStart(), clockEnd()];
  const scrub = $('#scrub');
  scrub.min = t0;
  scrub.max = end;
  scrub.value = st.p;
  const label = (x) => fmt(Math.round(x * 10) / 10);
  let where = '';
  if (st.lesson) {
    const l = st.lesson;
    const beat = l.beats.find((b) => st.p >= b.start && st.p < b.end) || l.beats[l.beats.length - 1];
    for (const b of $('#beats').children) b.classList.toggle('current', beat && b.dataset.beat === beat.beat);
    const ex = exploreAt(st.p);
    const wait = !st.playing && pendingAt(st.p);
    $('#continue').hidden = !ex && !wait;
    if (ex) $('#continue').textContent = `Continue (explore ends in ${Math.ceil(ex.end - st.p)} s)`;
    else if (wait) $('#continue').textContent = 'Continue';
    if (beat) where = `, beat ${beat.beat}`;
  } else {
    $('#continue').hidden = true;
    where = `, ${st.sess.interventions} intervention${st.sess.interventions === 1 ? '' : 's'}`;
  }
  $('#clock').textContent = `${st.lesson ? '' : 't = '}${label(st.p)} s / ${label(end)} s${where}`;
  $('#play').textContent = st.playing ? 'Pause' : st.p >= end ? 'Replay' : 'Play';
  $('#play').setAttribute('aria-label', $('#play').textContent);
}

function togglePlay() {
  if (!clocked()) return;
  if (st.playing) { stop(); updateTransport(); return; }
  if (st.lesson && pendingAt(st.p)) { learnerContinues(); return; }
  if (st.p >= clockEnd()) st.p = clockStart();
  st.playing = true;
  st.lastTick = null;
  requestAnimationFrame(tick);
  updateTransport();
}

function stop() {
  st.playing = false;
  silence();
}

// ------------------------------------------------------------------ narration sound

// Sound is the host's, not the program's (D-053): each caption cue is voiced by the
// recording named after it, or by synthesized speech. The timeline decides when a cue
// starts and how long it lasts; a recording can start part way through its cue, speech
// only at its start.

function voiceSync() {
  const v = st.voice;
  if (v.mode === 'off' || !st.lesson) return;
  for (const c of st.lesson.captions || []) {
    if (st.p < c.start || st.p >= c.end || v.sounding.has(c.cue)) continue;
    const url = v.mode === 'files' ? v.files.get(c.cue) : null;
    if (url) {
      const a = new Audio(url);
      a.currentTime = st.p - c.start;
      a.playbackRate = st.speed;
      a.play().catch(() => {});
      v.sounding.set(c.cue, a);
    } else if ('speechSynthesis' in window && st.p - c.start < 0.3) {
      const u = new SpeechSynthesisUtterance(c.text);
      u.rate = st.speed;
      speechSynthesis.speak(u);
      v.sounding.set(c.cue, u);
    } else {
      v.sounding.set(c.cue, null); // joined too late to speak; silent until the next cue
    }
  }
}

function silence() {
  for (const s of st.voice.sounding.values()) if (s instanceof HTMLAudioElement) s.pause();
  st.voice.sounding.clear();
  if ('speechSynthesis' in window) speechSynthesis.cancel();
}

/// Notes on the voice: cues without a recording, recordings longer than their cue.
function checkVoice() {
  const v = st.voice;
  v.notes = [];
  const check = ++v.checks; // lengths measured for an earlier choice are ignored
  if (v.mode !== 'files' || !st.lesson) return;
  const cues = st.lesson.captions || [];
  const missing = cues.filter((c) => !v.files.has(c.cue)).map((c) => c.cue);
  if (missing.length) v.notes.push(`No recording for ${missing.join(', ')}: synthesized instead.`);
  for (const c of cues) {
    const url = v.files.get(c.cue);
    if (!url) continue;
    const a = new Audio();
    a.preload = 'metadata';
    a.addEventListener('loadedmetadata', () => {
      if (check === v.checks && a.duration > c.end - c.start + 0.05) {
        v.notes.push(`Recording ${c.cue} lasts ${fmt(Math.round(a.duration * 10) / 10)} s, its cue ${fmt(c.end - c.start)} s: give the narration a longer \`for\`.`);
        status(lessonNotes());
      }
    });
    a.src = url;
  }
}

function tick(now) {
  if (!st.playing || !clocked()) return;
  const dt = st.lastTick == null ? 0 : Math.min(0.1, (now - st.lastTick) / 1000) * st.speed;
  st.lastTick = now;
  const end = clockEnd();
  const prev = st.p;
  st.p = Math.min(end, st.p + dt);
  // A continue point the learner has not passed: stop at it (D-067).
  const wait = st.lesson && (st.lesson.waits || []).find((w) => w.pending && w.at > prev && w.at <= st.p);
  if (wait) st.p = wait.at;
  if (st.sess) st.p = JSON.parse(st.player.seek(st.p)).t;
  const frame = JSON.parse(st.player.frame(st.p, dt));
  render(frame);
  voiceSync();
  if (wait) {
    stop();
    updateTransport();
    status(`${wait.beat}: Continue when ready.`);
    return;
  }
  updateTransport();
  if (st.p >= end) {
    stop();
    updateTransport();
    renderObservations();
    if (st.sess && st.sess.status === 'paused') pausedNote();
    return;
  }
  requestAnimationFrame(tick);
}

// ------------------------------------------------------------------ side panel

function setupTabs() {
  for (const b of document.querySelectorAll('.tabs button')) {
    b.addEventListener('click', () => selectTab(b.dataset.tab));
  }
}

function selectTab(name) {
  for (const b of document.querySelectorAll('.tabs button')) b.setAttribute('aria-selected', String(b.dataset.tab === name));
  for (const t of document.querySelectorAll('.tab')) t.hidden = t.id !== `tab-${name}`;
  if (name === 'observe' && st.player && st.layout) renderObservations();
}

function runCases() {
  if (!st.player) return;
  const host = $('#cases');
  host.replaceChildren(el('p', { class: 'hint', text: 'Running...' }));
  setTimeout(() => {
    const cases = JSON.parse(st.player.run_cases());
    host.replaceChildren();
    let pass = 0, fail = 0;
    for (const c of cases) {
      const box = el('div', { class: 'case' }, el('h3', { text: `run ${c.case}` }));
      const ul = el('ul');
      if (c.error) { ul.append(el('li', { class: 'fail', text: c.error })); fail++; }
      for (const b of c.beats || []) ul.append(el('li', { class: 'hint', text: `beat ${b.beat}: ${fmt(b.start)} s to ${fmt(b.end)} s` }));
      for (const r of c.results || []) {
        ul.append(el('li', { class: r.pass ? 'pass' : 'fail', text: r.pass ? `expect ${r.id}` : `expect ${r.id}: ${r.message}` }));
        if (r.pass) pass++; else fail++;
      }
      box.append(ul);
      host.append(box);
    }
    if (!cases.length) host.append(el('p', { class: 'hint', text: 'The program has no cases.' }));
    $('#cases-summary').textContent = `${pass} passed, ${fail} failed`;
  }, 10);
}

main().catch((e) => {
  status(`The player could not start: ${e}. Build the WebAssembly package first (web/README.md).`, true);
});
