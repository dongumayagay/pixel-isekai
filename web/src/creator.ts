// Character creator / dev viewer: drives the Rust generator directly
// (no world, main thread) to preview every direction and animation, run
// the visual QA linter live, browse random galleries and export sheets.

import './creator.css';
import init, { creator_info, render_frame, random_look, lint_look, cast_list } from './pkg/sim.js';
import wasmUrl from './pkg/sim_bg.wasm?url';

interface Look {
  skin: number; hair: number; hair_style: number; top: number; bottom: number; outfit: number;
  hat: number; hat_color: number; female: boolean; age: number; beard: boolean; eyes: number;
  mature: boolean; race: number; beast: number; held: number; glasses: boolean;
  figure: number; build: number; height: number; facial: number;
}
interface Info {
  fw: number; fh: number; pivot: [number, number]; framesPerDir: number; sheetFrames: number;
  races: string[]; hairStyles: number; figures: string[]; heights: string[]; facial: string[]; buildsF: string[]; buildsM: string[];
  swatch: { skin: number[]; demon: number[]; hair: number[]; cloth: number[]; eyes: number[] };
}

const OUTFITS = ['Tunic', 'Dress', 'Robe', 'Apron', 'Armor', 'Cloak', 'Vest', 'Dirndl', 'Guild', 'Smith', 'Gothic gown', 'Long coat'];
const HATS = ['None', 'Straw hat', 'Bandana', 'Helmet', 'Hood', 'Cap', 'Witch hat', 'Flower', 'Chef', 'Ribbon', 'Maid band'];
const HELD = ['None', 'Staff', 'Hammer', 'Mug', 'Book', 'Sword', 'Basket'];
const STYLES = ['Short', 'Long', 'Ponytail', 'Bun', 'Spiky', 'Bob', 'Twin tails', 'Side-swept'];
const BEASTS = ['Cat', 'Fox', 'Wolf', 'Rabbit'];
// No child characters for now; age 1 = adult, 2 = elder.
const AGES = ['Adult', 'Elder'];
const DIRS = [
  { name: 'Down', dir: 0, flip: false },
  { name: 'Up', dir: 1, flip: false },
  { name: 'Left', dir: 2, flip: true },
  { name: 'Right', dir: 2, flip: false },
];
// Frame sequences (indices into a direction's 12 poses; 12 = blink, down
// only) with per-frame hold times in ms, matching the in-game rates.
const BLINK = 12;
const ANIMS: Record<string, { frames: number[]; ms: number[] }> = {
  Idle: { frames: [0, 1, 2, 3, 0, 1, 2, 3, 0, BLINK], ms: [400, 400, 400, 400, 400, 400, 400, 400, 300, 120] },
  Walk: { frames: [4, 5, 6, 7, 8, 9], ms: [100, 100, 100, 100, 100, 100] },
  Work: { frames: [10, 11], ms: [330, 330] },
  Blink: { frames: [0, BLINK], ms: [700, 140] },
};
const BGS: Record<string, string> = { Grass: '#5f8f52', Dark: '#1a1524', Light: '#d8d0c0', Cobble: '#6e6878' };

const $ = <T extends HTMLElement = HTMLElement>(s: string) => document.querySelector(s) as T;
const hex = (n: number) => '#' + n.toString(16).padStart(6, '0');

let info: Info;
let look: Look;
// View settings survive the hot-reload loop.
const saved = JSON.parse(localStorage.getItem('creator-view') ?? '{}') as { anim?: string; zoom?: number; bg?: string; speed?: number };
let anim = saved.anim ?? 'Walk';
let zoom = saved.zoom ?? 4;
let bg = saved.bg ?? 'Grass';
let speed = saved.speed ?? 1;
const saveView = () => localStorage.setItem('creator-view', JSON.stringify({ anim, zoom, bg, speed }));
const cache = new Map<string, HTMLCanvasElement>();

function frameCanvas(l: Look, dir: number, f: number): HTMLCanvasElement {
  const key = JSON.stringify(l) + '|' + dir + '|' + f;
  let c = cache.get(key);
  if (c) return c;
  const px = render_frame(JSON.stringify(l), dir, f);
  c = document.createElement('canvas');
  c.width = info.fw;
  c.height = info.fh;
  c.getContext('2d')!.putImageData(new ImageData(new Uint8ClampedArray(px), info.fw, info.fh), 0, 0);
  if (cache.size > 4000) cache.clear();
  cache.set(key, c);
  return c;
}

function draw(target: HTMLCanvasElement, l: Look, dir: number, f: number, flip: boolean, scale: number, back = true) {
  const ctx = target.getContext('2d')!;
  ctx.imageSmoothingEnabled = false;
  if (back) {
    ctx.fillStyle = BGS[bg];
    ctx.fillRect(0, 0, target.width, target.height);
    // Contact shadow at the pivot, as in game.
    ctx.fillStyle = 'rgba(20,16,40,0.28)';
    ctx.beginPath();
    ctx.ellipse(info.pivot[0] * scale, (info.pivot[1] - 1) * scale, 13 * scale, 5 * scale, 0, 0, Math.PI * 2);
    ctx.fill();
  } else ctx.clearRect(0, 0, target.width, target.height);
  const src = frameCanvas(l, dir, f);
  ctx.save();
  if (flip) {
    ctx.translate(target.width, 0);
    ctx.scale(-1, 1);
  }
  ctx.drawImage(src, 0, 0, info.fw * scale, info.fh * scale);
  ctx.restore();
}

// ---------------------------------------------------------------- UI

function buttons<T>(host: HTMLElement, items: T[], label: (t: T, i: number) => string, isOn: (i: number) => boolean, pick: (i: number) => void) {
  host.innerHTML = '';
  items.forEach((t, i) => {
    const b = document.createElement('button');
    b.textContent = label(t, i);
    if (isOn(i)) b.classList.add('on');
    b.onclick = () => pick(i);
    host.appendChild(b);
  });
}

function swatches(host: HTMLElement, cols: number[], isOn: (i: number) => boolean, pick: (i: number) => void) {
  host.innerHTML = '';
  cols.forEach((c, i) => {
    const b = document.createElement('button');
    b.className = 'sw' + (isOn(i) ? ' on' : '');
    b.style.background = hex(c);
    b.title = String(i);
    b.onclick = () => pick(i);
    host.appendChild(b);
  });
}

function set(patch: Partial<Look>) {
  look = { ...look, ...patch };
  // Same rules as Look::normalized in Rust: beards only on adult men.
  if (look.age === 0) look.age = 1;
  look.mature = look.age > 0;
  if (look.female || look.age === 0) {
    look.beard = false;
    look.facial = 0;
  }
  if (look.race === 4) look.skin %= info.swatch.demon.length;
  else look.skin %= info.swatch.skin.length;
  location.hash = encodeURIComponent(JSON.stringify(look));
  renderControls();
  renderSheet();
  runQa();
}

function renderControls() {
  const f = $('#controls');
  f.innerHTML = `
    <h3>Race</h3><div class="row" id="c-race"></div>
    <div id="c-beast-wrap"><h3>Beast kind</h3><div class="row" id="c-beast"></div></div>
    <h3>Body</h3><div class="row" id="c-sex"></div><div class="row" id="c-age" style="margin-top:5px"></div>
    <h3>Height</h3><div class="row" id="c-height"></div>
    <h3>Build</h3><div class="row" id="c-build"></div>
    <div id="c-fig-wrap"><h3>Figure</h3><div class="row" id="c-fig"></div></div>
    <h3>Skin</h3><div class="row" id="c-skin"></div>
    <h3>Eyes</h3><div class="row" id="c-eyes"></div>
    <h3>Hair style</h3><div class="row" id="c-style"></div>
    <h3>Hair colour</h3><div class="row" id="c-hair"></div>
    <h3>Outfit</h3><div class="row" id="c-outfit"></div>
    <h3>Top colour</h3><div class="row" id="c-top"></div>
    <h3>Bottom colour</h3><div class="row" id="c-bottom"></div>
    <h3>Headwear</h3><div class="row" id="c-hat"></div>
    <h3>Headwear colour</h3><div class="row" id="c-hatc"></div>
    <h3>Holding</h3><div class="row" id="c-held"></div>
    <div id="c-facial-wrap"><h3>Facial hair</h3><div class="row" id="c-facial"></div></div>
    <h3>Extras</h3><div class="row" id="c-extra"></div>`;
  buttons($('#c-race'), info.races, (r) => r, (i) => look.race === i, (i) => set({ race: i }));
  $('#c-beast-wrap').style.display = look.race === 3 ? '' : 'none';
  buttons($('#c-beast'), BEASTS, (b) => b, (i) => look.beast === i, (i) => set({ beast: i }));
  buttons($('#c-sex'), ['Female', 'Male'], (s) => s, (i) => look.female === (i === 0), (i) => set({ female: i === 0 }));
  buttons($('#c-age'), AGES, (a) => a, (i) => look.age === i + 1, (i) => set({ age: i + 1 }));
  buttons($('#c-height'), info.heights, (h) => h, (i) => (look.height ?? 1) === i, (i) => set({ height: i }));
  buttons($('#c-build'), look.female ? info.buildsF : info.buildsM, (b) => b, (i) => look.build === i, (i) => set({ build: i }));
  $('#c-fig-wrap').style.display = look.female && look.age > 0 ? '' : 'none';
  buttons($('#c-fig'), info.figures, (f) => f, (i) => look.figure === i, (i) => set({ figure: i }));
  swatches($('#c-skin'), look.race === 4 ? info.swatch.demon : info.swatch.skin, (i) => look.skin === i, (i) => set({ skin: i }));
  swatches($('#c-eyes'), info.swatch.eyes, (i) => look.eyes === i, (i) => set({ eyes: i }));
  buttons($('#c-style'), STYLES, (s) => s, (i) => look.hair_style === i, (i) => set({ hair_style: i }));
  swatches($('#c-hair'), info.swatch.hair, (i) => look.hair === i, (i) => set({ hair: i }));
  buttons($('#c-outfit'), OUTFITS, (s) => s, (i) => look.outfit === i, (i) => set({ outfit: i }));
  swatches($('#c-top'), info.swatch.cloth, (i) => look.top === i, (i) => set({ top: i }));
  swatches($('#c-bottom'), info.swatch.cloth, (i) => look.bottom === i, (i) => set({ bottom: i }));
  buttons($('#c-hat'), HATS, (s) => s, (i) => look.hat === i, (i) => set({ hat: i }));
  swatches($('#c-hatc'), info.swatch.cloth, (i) => look.hat_color === i, (i) => set({ hat_color: i }));
  buttons($('#c-held'), HELD, (s) => s, (i) => look.held === i, (i) => set({ held: i }));
  $('#c-facial-wrap').style.display = !look.female && look.age > 0 ? '' : 'none';
  // Legacy beard flag shows as a full beard (long on dwarves).
  const facial = look.facial || (look.beard ? (look.race === 2 ? 6 : 5) : 0);
  buttons($('#c-facial'), info.facial, (f) => f, (i) => facial === i, (i) => set({ facial: i, beard: false }));
  buttons($('#c-extra'), ['Glasses'], (s) => s, () => look.glasses, () => set({ glasses: !look.glasses }));
}

function renderSheet() {
  const host = $('#sheet');
  host.innerHTML = '';
  const names = ['idle 1', 'idle 2', 'idle 3', 'idle 4', 'walk 1', 'walk 2', 'walk 3', 'walk 4', 'walk 5', 'walk 6', 'work 1', 'work 2'];
  for (const d of [0, 1, 2]) {
    for (let f = 0; f < info.framesPerDir; f++) {
      const cell = document.createElement('div');
      cell.className = 'cell';
      const c = document.createElement('canvas');
      c.width = info.fw * 2;
      c.height = info.fh * 2;
      c.className = 'stagebox';
      draw(c, look, d, f, false, 2);
      cell.appendChild(c);
      const s = document.createElement('small');
      s.textContent = `${['down', 'up', 'side'][d]} · ${names[f]}`;
      cell.appendChild(s);
      host.appendChild(cell);
    }
  }
}

function runQa() {
  const issues = JSON.parse(lint_look(JSON.stringify(look))) as { kind: string; dir: number; frame: number; detail: string }[];
  const real = issues.filter((i) => i.kind !== 'palette');
  const el = $('#qa');
  el.classList.toggle('bad', real.length > 0);
  el.textContent = real.length
    ? `QA: ${real.length} defect(s) — ` + real.slice(0, 4).map((i) => `${i.kind} (${['down', 'up', 'side'][i.dir]} f${i.frame}: ${i.detail})`).join('; ')
    : `QA: clean — no clipping, debris, pinholes, floating feet, jitter or asymmetry across all ${info.sheetFrames} frames.`;
}

// ---------------------------------------------------------------- animation

const viewCanvases: HTMLCanvasElement[] = [];
let galleryLooks: Look[] = [];
const galleryCanvases: HTMLCanvasElement[] = [];

function buildViews() {
  const host = $('#views');
  host.innerHTML = '';
  viewCanvases.length = 0;
  for (const v of DIRS) {
    const w = document.createElement('div');
    w.className = 'view';
    const c = document.createElement('canvas');
    c.width = info.fw * zoom;
    c.height = info.fh * zoom;
    c.className = 'stagebox';
    w.appendChild(c);
    const s = document.createElement('small');
    s.textContent = v.name;
    w.appendChild(s);
    host.appendChild(w);
    viewCanvases.push(c);
  }
}

function frameAt(t: number, dir: number): number {
  const a = ANIMS[anim];
  const total = a.ms.reduce((x, y) => x + y, 0);
  let m = (t * speed) % total;
  for (let i = 0; i < a.frames.length; i++) {
    if (m < a.ms[i]) {
      const f = a.frames[i];
      return f === BLINK && dir !== 0 ? 0 : f;
    }
    m -= a.ms[i];
  }
  return a.frames[0];
}

function tick(t: number) {
  DIRS.forEach((v, i) => draw(viewCanvases[i], look, v.dir, frameAt(t, v.dir), v.flip, zoom));
  castLooks.forEach((l, i) => draw(castCanvases[i], l, 0, frameAt(t + i * 211, 0), false, 2));
  galleryLooks.forEach((l, i) => {
    const phase = t + i * 97;
    const f = ANIMS.Walk.frames[Math.floor((phase * speed) / 100) % 6];
    const d = Math.floor(phase / 2600) % 4;
    draw(galleryCanvases[i], l, DIRS[d].dir, f, DIRS[d].flip, 2);
  });
  requestAnimationFrame(tick);
}

const castCanvases: HTMLCanvasElement[] = [];
let castLooks: Look[] = [];
function buildCast() {
  const host = $('#cast');
  const list = JSON.parse(cast_list()) as { name: string; look: Look }[];
  castLooks = list.map((c) => c.look);
  list.forEach((c) => {
    const card = document.createElement('div');
    card.className = 'card';
    const cv = document.createElement('canvas');
    cv.width = info.fw * 2;
    cv.height = info.fh * 2;
    card.appendChild(cv);
    const s = document.createElement('small');
    s.textContent = c.name;
    card.appendChild(s);
    card.onclick = () => {
      set(c.look);
      document.querySelector('main')!.scrollTo({ top: 0, behavior: 'smooth' });
    };
    host.appendChild(card);
    castCanvases.push(cv);
  });
}

function buildGallery() {
  const host = $('#gallery');
  host.innerHTML = '';
  galleryCanvases.length = 0;
  const base = Math.floor(Math.random() * 1e9);
  galleryLooks = Array.from({ length: 24 }, (_, i) => JSON.parse(random_look(base + i)) as Look);
  galleryLooks.forEach((l) => {
    const card = document.createElement('div');
    card.className = 'card';
    const c = document.createElement('canvas');
    c.width = info.fw * 2;
    c.height = info.fh * 2;
    card.appendChild(c);
    const s = document.createElement('small');
    s.textContent = `${info.races[l.race]} · ${OUTFITS[l.outfit]}`;
    card.appendChild(s);
    card.onclick = () => {
      set(l);
      window.scrollTo({ top: 0, behavior: 'smooth' });
    };
    host.appendChild(card);
    galleryCanvases.push(c);
  });
}

// ---------------------------------------------------------------- export

function download(name: string, blob: Blob) {
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 2000);
}

/** Sheet: one row per direction (down, up, side-right) of 8 poses, then the
 *  blink frame on a fourth row. Metadata lists animations and timings. */
function exportSheet() {
  const c = document.createElement('canvas');
  c.width = info.fw * info.framesPerDir;
  c.height = info.fh * 4;
  const ctx = c.getContext('2d')!;
  for (const d of [0, 1, 2]) for (let f = 0; f < info.framesPerDir; f++) ctx.drawImage(frameCanvas(look, d, f), f * info.fw, d * info.fh);
  ctx.drawImage(frameCanvas(look, 0, BLINK), 0, 3 * info.fh);
  c.toBlob((b) => b && download('character.png', b), 'image/png');
  const meta = {
    frame: { w: info.fw, h: info.fh },
    pivot: { x: info.pivot[0], y: info.pivot[1] },
    rows: ['down', 'up', 'side (faces right; mirror for left)', 'blink (down)'],
    poses: ['idle1', 'idle2', 'idle3', 'idle4', 'walk1', 'walk2', 'walk3', 'walk4', 'walk5', 'walk6', 'work1', 'work2'],
    animations: Object.fromEntries(Object.entries(ANIMS).map(([k, v]) => [k.toLowerCase(), v])),
    look,
  };
  download('character.json', new Blob([JSON.stringify(meta, null, 2)], { type: 'application/json' }));
}

// ---------------------------------------------------------------- boot

async function boot() {
  await init({ module_or_path: wasmUrl });
  info = JSON.parse(creator_info()) as Info;
  $('#app').innerHTML = `
    <header>
      <h1>CHARACTER CREATOR</h1>
      <input id="seed" type="number" placeholder="seed" />
      <button id="rand" class="primary">Randomize</button>
      <span class="grow"></span>
      <button id="copy">Copy look JSON</button>
      <button id="export" class="primary">Export sheet + JSON</button>
    </header>
    <aside id="controls"></aside>
    <main>
      <div class="tabs" id="anims"></div>
      <div class="tabs">
        <span>Zoom</span><span class="row" id="zooms"></span>
        <span>Background</span><span class="row" id="bgs"></span>
        <span>Speed</span><span class="row" id="speeds"></span>
      </div>
      <div class="views" id="views"></div>
      <div id="qa"></div>
      <section><h2>Core cast</h2><div class="gallery" id="cast"></div></section>
      <section><h2>Every frame</h2><div class="sheet" id="sheet"></div></section>
      <section><h2>Gallery — click to edit <button id="reroll">Reroll</button></h2><div class="gallery" id="gallery"></div></section>
    </main>`;
  const tabs = () => {
    saveView();
    buttons($('#anims'), Object.keys(ANIMS), (a) => a, (i) => Object.keys(ANIMS)[i] === anim, (i) => { anim = Object.keys(ANIMS)[i]; tabs(); });
    buttons($('#zooms'), [2, 3, 4, 6], (z) => `${z}×`, (i) => [2, 3, 4, 6][i] === zoom, (i) => { zoom = [2, 3, 4, 6][i]; buildViews(); tabs(); });
    buttons($('#bgs'), Object.keys(BGS), (b) => b, (i) => Object.keys(BGS)[i] === bg, (i) => { bg = Object.keys(BGS)[i]; tabs(); renderSheet(); });
    buttons($('#speeds'), [0.5, 1, 2], (s) => `${s}×`, (i) => [0.5, 1, 2][i] === speed, (i) => { speed = [0.5, 1, 2][i]; tabs(); });
  };
  tabs();
  const fromHash = (() => {
    try {
      return location.hash.length > 1 ? (JSON.parse(decodeURIComponent(location.hash.slice(1))) as Look) : null;
    } catch {
      return null;
    }
  })();
  look = fromHash ?? (JSON.parse(random_look(Math.floor(Math.random() * 1e9))) as Look);
  $('#rand').onclick = () => {
    const s = Number(($('#seed') as HTMLInputElement).value) || Math.floor(Math.random() * 1e9);
    ($('#seed') as HTMLInputElement).value = '';
    set(JSON.parse(random_look(s)) as Look);
  };
  $('#copy').onclick = () => navigator.clipboard.writeText(JSON.stringify(look));
  $('#export').onclick = exportSheet;
  $('#reroll').onclick = buildGallery;
  buildViews();
  set({});
  buildCast();
  buildGallery();
  requestAnimationFrame(tick);
}

boot();
