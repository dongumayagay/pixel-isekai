// Character creator / dev viewer: drives the Rust generator directly
// (no world, main thread) to preview every direction and animation, run
// the visual QA linter live, browse random galleries and export sheets.

import './creator.css';
import init, { creator_info, render_frame, random_look, lint_look, look_options, normalize_look, cast_list, profession_list, dress_look } from './pkg/sim.js';
import wasmUrl from './pkg/sim_bg.wasm?url';
import { $, parseJson, randomSeed } from './util';

/** Mirrors `Look` in sim/src/art/people.rs; fields Rust defaults may be missing from old links. */
interface Look {
  skin: number; hair: number; hair_style: number; primary: number; secondary: number; outfit: number;
  hat: number; accent: number; female: boolean; age: number; eyes: number;
  mature: boolean; race: number; beast: number; held: number; glasses: boolean;
  figure: number; build: number; height?: number; facial: number; cut?: number;
}
/** What each outfit's primary, secondary and accent dyes colour ('' if unused). */
type DyeSlots = [string, string, string];
type Dyes = [number, number, number];
/** Per sex, women first. */
type BySex<T> = [T, T];

interface Info {
  fw: number; fh: number; pivot: [number, number]; framesPerDir: number; sheetFrames: number;
  races: string[]; hairStyles: string[]; outfits: string[]; outfitSlots: BySex<DyeSlots>[]; outfitDefaults: BySex<Dyes>[];
  hats: string[]; held: string[]; beasts: string[];
  figures: string[]; heights: string[]; facial: string[]; cuts: string[]; buildsF: string[]; buildsM: string[];
  swatch: { skin: number[]; demon: number[]; hair: number[]; cloth: number[]; eyes: number[] };
}
/** Per option, which values the Rust lock table allows with the rest of the look (false = greyed out). */
interface Options { race: boolean[]; sex: BySex<boolean>; hairStyle: boolean[]; outfit: boolean[]; hat: boolean[]; held: boolean[]; beast: boolean[] }
interface LintIssue { kind: string; dir: number; frame: number; detail: string }
interface CastMember { name: string; look: Look }

// Race indices into info.races, as RACE_* in sim/src/art/people/mod.rs.
const RACE_BEAST = 3;
const RACE_DEMON = 4;

// Sheet directions, in render_frame's `dir` order; side frames face right.
const DIR_NAMES = ['down', 'up', 'side'];
const VIEWS = [
  { name: 'Down', dir: 0, flip: false },
  { name: 'Up', dir: 1, flip: false },
  { name: 'Left', dir: 2, flip: true },
  { name: 'Right', dir: 2, flip: false },
];
const numbered = (name: string, n: number) => Array.from({ length: n }, (_, i) => `${name} ${i + 1}`);
/** Pose names within one direction. */
const POSES = [...numbered('idle', 4), ...numbered('walk', 6), ...numbered('work', 2)];
// Frame sequences (indices into a direction's 12 poses; 12 = blink, down
// only) with per-frame hold times in ms, matching the in-game rates.
const BLINK = 12;
const ANIMS: Record<string, { frames: number[]; ms: number[] }> = {
  Idle: { frames: [0, 1, 2, 3, 0, 1, 2, 3, 0, BLINK], ms: [400, 400, 400, 400, 400, 400, 400, 400, 300, 120] },
  Walk: { frames: [4, 5, 6, 7, 8, 9], ms: [100, 100, 100, 100, 100, 100] },
  Work: { frames: [10, 11], ms: [330, 330] },
  Blink: { frames: [0, BLINK], ms: [700, 140] },
};
const ANIM_NAMES = Object.keys(ANIMS);
const BGS: Record<string, string> = { Grass: '#5f8f52', Dark: '#1a1524', Light: '#d8d0c0', Cobble: '#6e6878' };
const BG_NAMES = Object.keys(BGS);
const ZOOMS = [2, 3, 4, 6];
const SPEEDS = [0.5, 1, 2];

const hex = (n: number) => '#' + n.toString(16).padStart(6, '0');
const capitalize = (s: string) => s && s[0].toUpperCase() + s.slice(1);
const sexIndex = (l: Look) => (l.female ? 0 : 1);
const parseLook = (json: string) => parseJson<Look>(json);
const randomLook = (seed: number) => parseLook(random_look(seed));

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

const skinSwatches = (l: Look) => (l.race === RACE_DEMON ? info.swatch.demon : info.swatch.skin);

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

function draw(target: HTMLCanvasElement, l: Look, dir: number, f: number, flip: boolean, scale: number) {
  const ctx = target.getContext('2d')!;
  ctx.imageSmoothingEnabled = false;
  ctx.fillStyle = BGS[bg];
  ctx.fillRect(0, 0, target.width, target.height);
  // Contact shadow at the pivot, as in game.
  ctx.fillStyle = 'rgba(20,16,40,0.28)';
  ctx.beginPath();
  ctx.ellipse(info.pivot[0] * scale, (info.pivot[1] - 1) * scale, 13 * scale, 5 * scale, 0, 0, Math.PI * 2);
  ctx.fill();
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

/** A row of buttons with the `selected` one highlighted and those `allowed` marks false locked. */
function choice(id: string, labels: readonly string[], selected: number, pick: (i: number) => void, allowed?: readonly boolean[]) {
  const host = $(id);
  host.innerHTML = '';
  labels.forEach((label, i) => {
    const b = document.createElement('button');
    b.textContent = label;
    if (i === selected) b.classList.add('on');
    if (allowed?.[i] === false) {
      b.disabled = true;
      b.title = "Doesn't go with the rest of this look";
    }
    b.onclick = () => pick(i);
    host.appendChild(b);
  });
}

function swatches(id: string, colors: number[], selected: number, pick: (i: number) => void) {
  const host = $(id);
  host.innerHTML = '';
  colors.forEach((c, i) => {
    const b = document.createElement('button');
    b.className = 'sw' + (i === selected ? ' on' : '');
    b.style.background = hex(c);
    b.title = String(i);
    b.onclick = () => pick(i);
    host.appendChild(b);
  });
}

/** A labelled canvas sized for one frame at `scale`, appended to `host` in a `cls` wrapper. */
function figure(host: HTMLElement, cls: string, label: string, scale: number): HTMLCanvasElement {
  const wrap = document.createElement('div');
  wrap.className = cls;
  const c = document.createElement('canvas');
  c.width = info.fw * scale;
  c.height = info.fh * scale;
  wrap.appendChild(c);
  const s = document.createElement('small');
  s.textContent = label;
  wrap.appendChild(s);
  host.appendChild(wrap);
  return c;
}

function stage(host: HTMLElement, cls: string, label: string, scale: number): HTMLCanvasElement {
  const c = figure(host, cls, label, scale);
  c.className = 'stagebox';
  return c;
}

function card(host: HTMLElement, label: string, onClick: () => void): HTMLCanvasElement {
  const c = figure(host, 'card', label, 2);
  c.parentElement!.onclick = onClick;
  return c;
}

/** Load a look into the editor and scroll back up to it. */
function edit(l: Look) {
  set(l);
  $('main').scrollTo({ top: 0, behavior: 'smooth' });
}

function set(patch: Partial<Look>) {
  look = { ...look, ...patch };
  // Everyone is a grown adult; looking older comes from grey hair, a bald
  // cut or a beard rather than an age switch.
  if (look.age === 0) look.age = 1;
  look.mature = true;
  // Rust owns every other rule (Look::normalized): clashing hats come off,
  // hair tucks under them, traits the body can't carry are dropped.
  look = parseLook(normalize_look(JSON.stringify(look)));
  location.hash = encodeURIComponent(JSON.stringify(look));
  renderControls();
  renderSheet();
  runQa();
}

const show = (id: string, visible: boolean) => ($(id).style.display = visible ? '' : 'none');

function renderControls() {
  $('#controls').innerHTML = `
    <h3>Race</h3><div class="row" id="c-race"></div>
    <div id="c-beast-wrap"><h3>Beast kind</h3><div class="row" id="c-beast"></div></div>
    <h3>Body</h3><div class="row" id="c-sex"></div>
    <h3>Height</h3><div class="row" id="c-height"></div>
    <h3>Build</h3><div class="row" id="c-build"></div>
    <div id="c-fig-wrap"><h3>Figure</h3><div class="row" id="c-fig"></div></div>
    <h3>Skin</h3><div class="row" id="c-skin"></div>
    <h3>Eyes</h3><div class="row" id="c-eyes"></div>
    <h3>Hair style</h3><div class="row" id="c-style"></div>
    <h3>Hair colour</h3><div class="row" id="c-hair"></div>
    <h3>Outfit</h3><div class="row" id="c-outfit"></div>
    <h3>Cut</h3><div class="row" id="c-cut"></div>
    <div id="c-p-wrap"><h3 id="c-p-label"></h3><div class="row" id="c-p"></div></div>
    <div id="c-s-wrap"><h3 id="c-s-label"></h3><div class="row" id="c-s"></div></div>
    <h3>Headwear</h3><div class="row" id="c-hat"></div>
    <div id="c-a-wrap"><h3 id="c-a-label"></h3><div class="row" id="c-a"></div></div>
    <h3>Holding</h3><div class="row" id="c-held"></div>
    <div id="c-facial-wrap"><h3>Facial hair</h3><div class="row" id="c-facial"></div></div>
    <h3>Extras</h3><div class="row" id="c-extra"></div>`;
  const sex = sexIndex(look);
  // Rust decides what clashes; options that would draw badly are locked.
  const ok = parseJson<Options>(look_options(JSON.stringify(look)));
  choice('#c-race', info.races, look.race, (race) => set({ race }), ok.race);
  show('#c-beast-wrap', look.race === RACE_BEAST);
  choice('#c-beast', info.beasts, look.beast, (beast) => set({ beast }), ok.beast);
  choice('#c-sex', ['Female', 'Male'], sex, (i) => set({ female: i === 0 }), ok.sex);
  choice('#c-height', info.heights, look.height ?? 1, (height) => set({ height }));
  choice('#c-build', look.female ? info.buildsF : info.buildsM, look.build, (build) => set({ build }));
  show('#c-fig-wrap', look.female && look.mature);
  choice('#c-fig', info.figures, look.figure, (figure) => set({ figure }));
  swatches('#c-skin', skinSwatches(look), look.skin, (skin) => set({ skin }));
  swatches('#c-eyes', info.swatch.eyes, look.eyes, (eyes) => set({ eyes }));
  choice('#c-style', info.hairStyles, look.hair_style, (hair_style) => set({ hair_style }), ok.hairStyle);
  swatches('#c-hair', info.swatch.hair, look.hair, (hair) => set({ hair }));
  // Picking an outfit dresses it in its best-known colours.
  choice('#c-outfit', info.outfits, look.outfit, (outfit) => {
    const [primary, secondary, accent] = info.outfitDefaults[outfit][sex];
    set({ outfit, primary, secondary, accent });
  }, ok.outfit);
  choice('#c-cut', info.cuts, look.cut ?? 0, (cut) => set({ cut }));
  // Each colour row is named after the piece it dyes on this outfit; the
  // accent also colours the headwear.
  const [primaryPiece, secondaryPiece, accentPiece] = info.outfitSlots[look.outfit][sex];
  const dyeRow = (key: 'p' | 's' | 'a', label: string, selected: number, pick: (i: number) => void) => {
    show(`#c-${key}-wrap`, label !== '');
    $(`#c-${key}-label`).textContent = label;
    swatches(`#c-${key}`, info.swatch.cloth, selected, pick);
  };
  dyeRow('p', primaryPiece ? `${primaryPiece} (primary)` : '', look.primary, (primary) => set({ primary }));
  dyeRow('s', secondaryPiece ? `${secondaryPiece} (secondary)` : '', look.secondary, (secondary) => set({ secondary }));
  choice('#c-hat', info.hats, look.hat, (hat) => set({ hat }), ok.hat);
  const accent = [accentPiece, look.hat ? 'headwear' : ''].filter(Boolean).join(' & ');
  dyeRow('a', accent ? `${capitalize(accent)} (accent)` : '', look.accent, (accent) => set({ accent }));
  choice('#c-held', info.held, look.held, (held) => set({ held }), ok.held);
  show('#c-facial-wrap', !look.female && look.mature);
  choice('#c-facial', info.facial, look.facial, (facial) => set({ facial }));
  choice('#c-extra', ['Glasses'], look.glasses ? 0 : -1, () => set({ glasses: !look.glasses }));
}

function renderSheet() {
  const host = $('#sheet');
  host.innerHTML = '';
  DIR_NAMES.forEach((dirName, d) => {
    for (let f = 0; f < info.framesPerDir; f++) draw(stage(host, 'cell', `${dirName} · ${POSES[f]}`, 2), look, d, f, false, 2);
  });
}

function runQa() {
  const issues = parseJson<LintIssue[]>(lint_look(JSON.stringify(look)));
  const real = issues.filter((i) => i.kind !== 'palette');
  const el = $('#qa');
  el.classList.toggle('bad', real.length > 0);
  el.textContent = real.length
    ? `QA: ${real.length} defect(s) — ` + real.slice(0, 4).map((i) => `${i.kind} (${DIR_NAMES[i.dir]} f${i.frame}: ${i.detail})`).join('; ')
    : `QA: clean — no clipping, debris, pinholes, floating feet, jitter or asymmetry across all ${info.sheetFrames} frames.`;
}

// ---------------------------------------------------------------- animation

interface Preview {
  look: Look;
  canvas: HTMLCanvasElement;
}

const viewCanvases: HTMLCanvasElement[] = [];
const cast: Preview[] = [];
const professions: Preview[] = [];
let gallery: Preview[] = [];

function buildViews() {
  const host = $('#views');
  host.innerHTML = '';
  viewCanvases.length = 0;
  for (const v of VIEWS) viewCanvases.push(stage(host, 'view', v.name, zoom));
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
  VIEWS.forEach((v, i) => draw(viewCanvases[i], look, v.dir, frameAt(t, v.dir), v.flip, zoom));
  // Offset phases so the lists don't animate in lockstep.
  cast.forEach((p, i) => draw(p.canvas, p.look, 0, frameAt(t + i * 211, 0), false, 2));
  professions.forEach((p, i) => draw(p.canvas, p.look, 0, frameAt(t + i * 173, 0), false, 2));
  // Gallery figures always walk, turning every 2.6 s.
  const walk = ANIMS.Walk;
  gallery.forEach((p, i) => {
    const phase = t + i * 97;
    const f = walk.frames[Math.floor((phase * speed) / walk.ms[0]) % walk.frames.length];
    const v = VIEWS[Math.floor(phase / 2600) % VIEWS.length];
    draw(p.canvas, p.look, v.dir, f, v.flip, 2);
  });
  requestAnimationFrame(tick);
}

function buildCast() {
  const host = $('#cast');
  for (const c of parseJson<CastMember[]>(cast_list())) {
    cast.push({ look: c.look, canvas: card(host, c.name, () => edit(c.look)) });
  }
}

function buildProfessions() {
  const host = $('#profs');
  parseJson<string[]>(profession_list()).forEach((name, job) => {
    // A plain adult human of alternating sex, dressed for the job.
    const person = { ...randomLook(9000 + job), race: 0, age: 1, female: job % 2 === 0, hat: 0, held: 0 };
    const dressed = parseLook(dress_look(JSON.stringify(person), job, 77 + job));
    professions.push({ look: dressed, canvas: card(host, name, () => edit(dressed)) });
  });
}

function buildGallery() {
  const host = $('#gallery');
  host.innerHTML = '';
  const base = randomSeed();
  gallery = Array.from({ length: 24 }, (_, i) => {
    const l = randomLook(base + i);
    const canvas = card(host, `${info.races[l.race]} · ${info.outfits[l.outfit]}`, () => {
      set(l);
      window.scrollTo({ top: 0, behavior: 'smooth' });
    });
    return { look: l, canvas };
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

/** Sheet: one row per direction (down, up, side-right) of every pose, then
 *  the blink frame on a fourth row. Metadata lists animations and timings. */
function exportSheet() {
  const c = document.createElement('canvas');
  c.width = info.fw * info.framesPerDir;
  c.height = info.fh * 4;
  const ctx = c.getContext('2d')!;
  DIR_NAMES.forEach((_, d) => {
    for (let f = 0; f < info.framesPerDir; f++) ctx.drawImage(frameCanvas(look, d, f), f * info.fw, d * info.fh);
  });
  ctx.drawImage(frameCanvas(look, 0, BLINK), 0, 3 * info.fh);
  c.toBlob((b) => b && download('character.png', b), 'image/png');
  const meta = {
    frame: { w: info.fw, h: info.fh },
    pivot: { x: info.pivot[0], y: info.pivot[1] },
    rows: ['down', 'up', 'side (faces right; mirror for left)', 'blink (down)'],
    poses: POSES.map((p) => p.replace(' ', '')),
    animations: Object.fromEntries(Object.entries(ANIMS).map(([k, v]) => [k.toLowerCase(), v])),
    look,
  };
  download('character.json', new Blob([JSON.stringify(meta, null, 2)], { type: 'application/json' }));
}

// ---------------------------------------------------------------- boot

/** The look encoded in the URL hash, if any. */
function lookFromHash(): Look | null {
  if (location.hash.length <= 1) return null;
  try {
    // Links from before the dyes were renamed carry top/bottom/hat_color
    // (Rust still accepts them as serde aliases).
    const raw = JSON.parse(decodeURIComponent(location.hash.slice(1))) as Record<string, number | boolean>;
    for (const [old, now] of [['top', 'primary'], ['bottom', 'secondary'], ['hat_color', 'accent']]) {
      if (old in raw) {
        raw[now] ??= raw[old];
        delete raw[old];
      }
    }
    return raw as unknown as Look;
  } catch {
    return null;
  }
}

function renderViewTabs() {
  saveView();
  choice('#anims', ANIM_NAMES, ANIM_NAMES.indexOf(anim), (i) => {
    anim = ANIM_NAMES[i];
    renderViewTabs();
  });
  choice('#zooms', ZOOMS.map((z) => `${z}×`), ZOOMS.indexOf(zoom), (i) => {
    zoom = ZOOMS[i];
    buildViews();
    renderViewTabs();
  });
  choice('#bgs', BG_NAMES, BG_NAMES.indexOf(bg), (i) => {
    bg = BG_NAMES[i];
    renderViewTabs();
    renderSheet();
  });
  choice('#speeds', SPEEDS.map((s) => `${s}×`), SPEEDS.indexOf(speed), (i) => {
    speed = SPEEDS[i];
    renderViewTabs();
  });
}

async function boot() {
  await init({ module_or_path: wasmUrl });
  info = parseJson<Info>(creator_info());
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
      <section><h2>Professions — signature looks</h2><div class="gallery" id="profs"></div></section>
      <section><h2>Every frame</h2><div class="sheet" id="sheet"></div></section>
      <section><h2>Gallery — click to edit <button id="reroll">Reroll</button></h2><div class="gallery" id="gallery"></div></section>
    </main>`;
  renderViewTabs();
  look = lookFromHash() ?? randomLook(randomSeed());
  const seed = $<HTMLInputElement>('#seed');
  $('#rand').onclick = () => {
    const s = Number(seed.value) || randomSeed();
    seed.value = '';
    set(randomLook(s));
  };
  $('#copy').onclick = () => navigator.clipboard.writeText(JSON.stringify(look));
  $('#export').onclick = exportSheet;
  $('#reroll').onclick = buildGallery;
  buildViews();
  set({});
  buildCast();
  buildProfessions();
  buildGallery();
  requestAnimationFrame(tick);
}

boot();
