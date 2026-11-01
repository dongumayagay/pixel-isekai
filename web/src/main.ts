// Pixel Isekai client: boots the simulation worker, streams terrain,
// assembles each frame (props, villagers, shadows, lights, weather) and
// drives input and UI.

import '@fontsource/pixelify-sans/400.css';
import '@fontsource/pixelify-sans/600.css';
import './ui/style.css';
import { Renderer } from './gl/renderer';
import { atmosphere } from './atmosphere';
import { Particles } from './particles';
import { UI } from './ui/ui';
import { EF, ENT_STRIDE, H, HEADER, Activity, type Cmd, type FromWorker, type ToWorker, type WorldInfo } from './protocol';

const canvas = document.getElementById('game') as HTMLCanvasElement;
const renderer = new Renderer(canvas);
const worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
const ui = new UI();
const fx = new Particles();

function send(m: ToWorker) {
  worker.postMessage(m);
}
function cmd(c: Cmd) {
  send({ t: 'cmd', cmd: c });
}

let info: WorldInfo | null = null;
let table: Float32Array = new Float32Array(0);
let props: Float32Array = new Float32Array(0);
let lights: Float32Array = new Float32Array(0);
let named: Record<string, number> = {};
// Static props bucketed by chunk for culling.
let buckets = new Map<number, number[]>();
const houseOverlay = new Map<number, number>();
const chimneys: { x: number; y: number; house: number; t: number }[] = [];
const housePos = new Map<number, [number, number]>();
let litHouses = new Set<number>();
const requested = new Set<string>();

interface Snap {
  data: Float32Array;
  at: number;
  index: Map<number, number>;
}
let prev: Snap | null = null;
let cur: Snap | null = null;
const TICK = 1000 / 30;

const cam = { x: 0, y: 0, ready: false };
let time = 0;
let lastFrame = performance.now();
let flash = 0;
let nextBolt = 5;

function bucketKey(cx: number, cy: number) {
  return cy * 1000 + cx;
}

function rebuildBuckets() {
  buckets = new Map();
  if (!info) return;
  const cp = info.chunkPx;
  for (let i = 0; i < props.length; i += 5) {
    const k = bucketKey(Math.floor(props[i] / cp), Math.floor(props[i + 1] / cp));
    let b = buckets.get(k);
    if (!b) buckets.set(k, (b = []));
    b.push(i);
  }
}

function indexSnap(data: Float32Array): Map<number, number> {
  const m = new Map<number, number>();
  const n = data[H.count];
  for (let i = 0; i < n; i++) m.set(data[HEADER + i * ENT_STRIDE], HEADER + i * ENT_STRIDE);
  return m;
}

worker.onmessage = (ev: MessageEvent<FromWorker>) => {
  const m = ev.data;
  switch (m.t) {
    case 'progress':
      ui.loading(m.label);
      break;
    case 'ready': {
      info = m.info;
      table = m.table;
      props = m.props;
      lights = m.lights;
      named = m.named;
      renderer.initAtlas(m.pages, info.pageSize);
      for (let i = 0; i < m.houses.length; i += 4) {
        const [idx, overlay, cx, cy] = [m.houses[i], m.houses[i + 1], m.houses[i + 2], m.houses[i + 3]];
        houseOverlay.set(idx, overlay);
        if (cx >= 0) chimneys.push({ x: cx, y: cy, house: idx, t: Math.random() });
      }
      rebuildBuckets();
      for (let i = 0; i < props.length; i += 5) {
        if ((props[i + 3] & EF.HOUSE) !== 0) housePos.set(props[i + 4], [props[i], props[i + 1]]);
      }
      ui.setMinimap(m.minimap, info.w, info.h);
      ui.ready(m.genMs);
      break;
    }
    case 'frame': {
      const snap: Snap = { data: m.snap, at: performance.now(), index: indexSnap(m.snap) };
      prev = cur;
      cur = snap;
      const lc = m.snap[H.litCount];
      const base = HEADER + m.snap[H.count] * ENT_STRIDE;
      litHouses = new Set();
      for (let i = 0; i < lc; i++) litHouses.add(m.snap[base + i]);
      ui.frame(m.snap, JSON.parse(m.ui), m.simMs);
      break;
    }
    case 'props':
      props = m.props;
      rebuildBuckets();
      break;
    case 'season':
      // Terrain repaints for the new season.
      requested.clear();
      break;
    case 'chunk':
      renderer.setChunk(m.cx, m.cy, m.season, info!.chunkPx, m.px);
      requested.delete(`${m.cx},${m.cy}`);
      break;
    case 'atlas':
      renderer.updateAtlas(m.page, m.x, m.y, m.w, m.h, m.px);
      break;
    case 'sprites':
      table = m.table;
      break;
    default:
      ui.message(m);
  }
};

ui.onSend = send;
ui.onCmd = cmd;
ui.onStart = (seed: number) => {
  renderer.resize();
  send({ t: 'init', seed });
};

// ---------------------------------------------------------------- frame assembly

function requestChunks(season: number) {
  if (!info) return;
  const cp = info.chunkPx;
  const x0 = Math.floor((cam.x - cp * 0.5) / cp);
  const y0 = Math.floor((cam.y - cp * 0.5) / cp);
  const x1 = Math.floor((cam.x + renderer.viewW + cp * 0.5) / cp);
  const y1 = Math.floor((cam.y + renderer.viewH + cp * 0.5) / cp);
  const list: [number, number][] = [];
  const maxC = info.w / info.chunk;
  const maxR = info.h / info.chunk;
  // Nearest chunks first.
  const ccx = (cam.x + renderer.viewW / 2) / cp;
  const ccy = (cam.y + renderer.viewH / 2) / cp;
  const want: [number, number, number][] = [];
  for (let cy = y0; cy <= y1; cy++) {
    for (let cx = x0; cx <= x1; cx++) {
      if (cx < 0 || cy < 0 || cx >= maxC || cy >= maxR) continue;
      const key = `${cx},${cy}`;
      if (renderer.hasChunk(cx, cy, season) || requested.has(key)) continue;
      want.push([cx, cy, (cx + 0.5 - ccx) ** 2 + (cy + 0.5 - ccy) ** 2]);
    }
  }
  want.sort((a, b) => a[2] - b[2]);
  for (const [cx, cy] of want) {
    requested.add(`${cx},${cy}`);
    list.push([cx, cy]);
  }
  if (list.length) send({ t: 'chunks', list });
}

interface Drawable {
  y: number;
  x: number;
  sid: number;
  flags: number;
  alpha: number;
  layerY: number;
}

const draw: Drawable[] = [];

function pushSprite(d: Drawable) {
  const s = d.sid * 7;
  if (s + 6 >= table.length) return;
  const o = renderer.sprites.slot();
  const a = renderer.sprites.data;
  a[o] = d.x;
  a[o + 1] = d.y;
  a[o + 2] = table[s + 1];
  a[o + 3] = table[s + 2];
  a[o + 4] = table[s + 3];
  a[o + 5] = table[s + 4];
  a[o + 6] = table[s + 5];
  a[o + 7] = table[s + 6];
  a[o + 8] = table[s];
  a[o + 9] = d.flags;
  a[o + 10] = d.alpha;
}

function spriteW(sid: number) {
  return table[sid * 7 + 3] || 16;
}

function render(now: number) {
  requestAnimationFrame(render);
  const dt = Math.min(0.05, (now - lastFrame) / 1000);
  lastFrame = now;
  if (!info || !cur) return;
  time += dt;
  const snap = cur.data;
  const minute = snap[H.minute];
  const season = snap[H.season];
  const weather = snap[H.weather];
  const intensity = snap[H.intensity];
  const wind = snap[H.wind];
  const at = atmosphere(minute, weather, intensity, season);

  // Interpolation between the last two snapshots.
  const alpha = prev ? Math.min(1, Math.max(0, (now - cur.at) / TICK)) : 1;
  const lerpPos = (id: number, o: number): [number, number] => {
    const x = snap[o + 1];
    const y = snap[o + 2];
    if (!prev) return [x, y];
    const po = prev.index.get(id);
    if (po === undefined) return [x, y];
    const px = prev.data[po + 1];
    const py = prev.data[po + 2];
    if (Math.abs(px - x) > 64 || Math.abs(py - y) > 64) return [x, y];
    return [px + (x - px) * alpha, py + (y - py) * alpha];
  };

  // Camera follows the player smoothly.
  const po = cur.index.get(0);
  if (po !== undefined) {
    const [px, py] = lerpPos(0, po);
    const tx = px - renderer.viewW / 2;
    const ty = py - 24 - renderer.viewH / 2;
    if (!cam.ready) {
      cam.x = tx;
      cam.y = ty;
      cam.ready = true;
    }
    const k = 1 - Math.exp(-dt * 7);
    cam.x += (tx - cam.x) * k;
    cam.y += (ty - cam.y) * k;
    const W = info.w * info.tile;
    const Hh = info.h * info.tile;
    cam.x = Math.max(0, Math.min(W - renderer.viewW, cam.x));
    cam.y = Math.max(0, Math.min(Hh - renderer.viewH, cam.y));
    ui.player(px, py);
  }
  requestChunks(season);

  const r = renderer;
  r.sprites.reset();
  r.overlay.reset();
  r.shadows.reset();
  r.lights.reset();
  r.particles.reset();
  r.particlesTop.reset();
  draw.length = 0;

  // Visible static props (generous vertical margin: trees and houses are tall).
  const cp = info.chunkPx;
  const vx0 = cam.x - 120;
  const vx1 = cam.x + r.viewW + 120;
  const vy0 = cam.y - 40;
  const vy1 = cam.y + r.viewH + 200;
  for (let cy = Math.floor(vy0 / cp); cy <= Math.floor(vy1 / cp); cy++) {
    for (let cx = Math.floor(vx0 / cp); cx <= Math.floor(vx1 / cp); cx++) {
      const b = buckets.get(bucketKey(cx, cy));
      if (!b) continue;
      for (const i of b) {
        const x = props[i];
        const y = props[i + 1];
        if (x < vx0 || x > vx1 || y < vy0 || y > vy1) continue;
        const sid = props[i + 2];
        const f = props[i + 3];
        const pidx = props[i + 4];
        if (f & EF.GROUND) {
          draw.push({ x, y, sid, flags: f, alpha: 1, layerY: -1e9 });
          continue;
        }
        draw.push({ x, y, sid, flags: f, alpha: 1, layerY: y });
        const w = spriteW(sid);
        if (f & EF.BIG_SHADOW) r.shadows.push(x + 2, y - 1, Math.min(30, w * 0.3), Math.min(11, w * 0.11), 0.34);
        else if (f & EF.SHADOW) r.shadows.push(x, y - 1, Math.max(5, Math.min(22, w * 0.36)), Math.max(2, Math.min(8, w * 0.14)), 0.3);
        if (f & EF.HOUSE) {
          r.shadows.push(x + w / 2 - 6, y + 2, w * 0.5, 7, 0.32);
          if (litHouses.has(pidx) && at.night > 0.05) {
            const ov = houseOverlay.get(pidx);
            if (ov !== undefined) draw.push({ x, y, sid: ov, flags: 0, alpha: Math.min(1, at.night * 1.6), layerY: y + 0.01 });
          }
        }
      }
    }
  }

  // Villagers and the player.
  const n = snap[H.count];
  const emotes: [number, number, number][] = [];
  for (let k = 0; k < n; k++) {
    const o = HEADER + k * ENT_STRIDE;
    const id = snap[o];
    const [x, y] = lerpPos(id, o);
    if (x < vx0 || x > vx1 || y < vy0 || y > vy1) continue;
    const sid = snap[o + 3];
    const f = snap[o + 4];
    draw.push({ x, y, sid, flags: f & (EF.FLIP | EF.FLASH), alpha: 1, layerY: y });
    r.shadows.push(x, y - 1, 8, 3, 0.36);
    const emote = snap[o + 5];
    if (emote > 0) emotes.push([x, y, emote]);
    const act = snap[o + 6] & 255;
    workFx(id, x, y, act, f, dt);
    if (f & EF.PLAYER && at.night > 0.2) r.lights.push(x, y - 20, 84, 1, 0.9, 0.75, 0.5 * at.night);
  }

  draw.sort((a, b) => a.layerY - b.layerY);
  for (const d of draw) pushSprite(d);
  for (const [x, y, e] of emotes) {
    const sid = named[`emote_${e}`];
    if (sid === undefined) continue;
    const bob = Math.round(Math.sin(time * 4 + x) * 1.5);
    const s = sid * 7;
    r.overlay.push(x, y - 50 + bob, table[s + 1], table[s + 2], table[s + 3], table[s + 4], table[s + 5], table[s + 6], table[s], 0, 1);
  }

  // Lights: lamps, windows of lit houses, fires.
  if (at.night > 0.02) {
    for (let i = 0; i < lights.length; i += 7) {
      const x = lights[i];
      const y = lights[i + 1];
      const rad = lights[i + 2];
      if (x < cam.x - rad || x > cam.x + r.viewW + rad || y < cam.y - rad || y > cam.y + r.viewH + rad) continue;
      const house = lights[i + 6];
      if (house >= 0 && !litHouses.has(house)) continue;
      const flicker = 0.92 + Math.sin(time * 9 + x) * 0.04 + Math.sin(time * 23 + y) * 0.03;
      r.lights.push(x, y, rad, lights[i + 3], lights[i + 4], lights[i + 5], at.night * flicker);
    }
  }

  ambientFx(dt, at, season, wind, minute);
  emitParticles();

  // Lightning
  if (at.storm) {
    nextBolt -= dt;
    if (nextBolt < 0) {
      flash = 1;
      nextBolt = 4 + Math.random() * 9;
      ui.thunder();
    }
  }
  flash = Math.max(0, flash - dt * 3.5);

  r.frame(
    {
      camX: cam.x,
      camY: cam.y,
      time,
      ambient: at.ambient,
      night: at.night,
      wind,
      clouds: at.clouds,
      fog: at.fog,
      fogColor: at.fogColor,
      flash: flash > 0.5 ? flash * 0.6 : flash * 0.15,
      saturation: at.saturation,
      tint: at.tint,
      vignette: 0.55 + at.night * 0.35,
      windDir: [0.8 + wind, 0.3],
    },
    info.chunkPx,
  );
}

const lastFx = new Map<number, number>();

function workFx(id: number, x: number, y: number, act: number, flags: number, dt: number) {
  const t = (lastFx.get(id) ?? 0) - dt;
  if (t > 0) {
    lastFx.set(id, t);
    return;
  }
  const flip = flags & EF.FLIP ? -1 : 1;
  switch (act) {
    case Activity.Chop:
      for (let i = 0; i < 4; i++)
        fx.spawn({ x: x + flip * 6, y: y - 16, vx: (Math.random() - 0.5) * 60, vy: -40 - Math.random() * 30, life: 0.6, max: 0.6, size: 2, r: 0.7, g: 0.5, b: 0.3, kind: 0 });
      lastFx.set(id, 0.66);
      break;
    case Activity.Mine:
      for (let i = 0; i < 4; i++)
        fx.spawn({ x: x + flip * 6, y: y - 12, vx: (Math.random() - 0.5) * 70, vy: -50 - Math.random() * 30, life: 0.5, max: 0.5, size: 2, r: 0.62, g: 0.62, b: 0.7, kind: 0 });
      lastFx.set(id, 0.66);
      break;
    case Activity.Smith:
      for (let i = 0; i < 6; i++)
        fx.spawn({ x: x, y: y - 18, vx: (Math.random() - 0.5) * 80, vy: -30 - Math.random() * 50, life: 0.45, max: 0.45, size: 1, r: 1, g: 0.75, b: 0.3, kind: 0 });
      lastFx.set(id, 0.5);
      break;
    case Activity.Hoe:
      for (let i = 0; i < 3; i++)
        fx.spawn({ x: x + flip * 8, y: y - 2, vx: (Math.random() - 0.5) * 40, vy: -30 - Math.random() * 20, life: 0.4, max: 0.4, size: 2, r: 0.45, g: 0.32, b: 0.25, kind: 0 });
      lastFx.set(id, 0.7);
      break;
    case Activity.Pray:
      fx.spawn({ x: x + (Math.random() - 0.5) * 16, y: y - 30, vy: -12, life: 1.4, max: 1.4, size: 1, r: 1, g: 0.9, b: 0.5, kind: 1 });
      lastFx.set(id, 0.3);
      break;
    case Activity.Fish:
      fx.spawn({ x: x + flip * 20, y: y + 10, vx: 0, vy: 0, life: 0.8, max: 0.8, size: 3, r: 0.85, g: 0.95, b: 1, a: 0.6, kind: 7 });
      lastFx.set(id, 1.6 + Math.random() * 2);
      break;
    default:
      lastFx.set(id, 0.4);
  }
}

let leafT = 0;
let smokeT = 0;
function ambientFx(dt: number, at: ReturnType<typeof atmosphere>, season: number, wind: number, minute: number) {
  const r = renderer;
  const vx = cam.x;
  const vy = cam.y;
  // Rain
  const drops = Math.floor(at.rain * 16 * (60 * dt) + Math.random());
  for (let i = 0; i < drops; i++) {
    fx.spawn({ x: vx + Math.random() * (r.viewW + 80) - 40, y: vy - 10 + Math.random() * r.viewH * 0.4, vx: -60 - wind * 60, vy: 420, life: 0.35 + Math.random() * 0.6, max: 1, size: 1, r: 0.78, g: 0.86, b: 1, a: 0.75, kind: 4 });
  }
  if (at.rain > 0.1) {
    for (let i = 0; i < Math.ceil(at.rain * 5); i++)
      fx.spawn({ x: vx + Math.random() * r.viewW, y: vy + Math.random() * r.viewH, life: 0.3, max: 0.3, size: 2, r: 0.8, g: 0.9, b: 1, a: 0.5, kind: 7 });
  }
  // Snow
  const flakes = Math.floor(at.snow * 4 * (60 * dt) + Math.random());
  for (let i = 0; i < flakes; i++) {
    fx.spawn({ x: vx + Math.random() * (r.viewW + 60) - 30, y: vy - 8, vy: 22 + Math.random() * 18, life: 9, max: 9, size: Math.random() < 0.3 ? 2 : 1, r: 1, g: 1, b: 1, a: 0.9, kind: 5 });
  }
  // Falling leaves / petals
  leafT -= dt;
  if (leafT < 0 && (season === 2 || season === 0) && at.rain < 0.5) {
    leafT = 0.25;
    const autumn = season === 2;
    const c = autumn ? [[0.85, 0.42, 0.18], [0.92, 0.65, 0.25], [0.7, 0.25, 0.15]][Math.floor(Math.random() * 3)] : [0.97, 0.75, 0.82];
    fx.spawn({ x: vx + Math.random() * r.viewW, y: vy - 4, vy: 18 + Math.random() * 10, life: 10, max: 10, size: 2, r: c[0], g: c[1], b: c[2], kind: 6 });
  }
  // Fireflies on warm nights
  if (at.night > 0.5 && season < 2 && at.rain < 0.1 && Math.random() < dt * 3) {
    fx.spawn({ x: vx + Math.random() * r.viewW, y: vy + Math.random() * r.viewH, life: 6, max: 6, size: 1, r: 0.85, g: 1, b: 0.45, kind: 3 });
  }
  // Chimney smoke, thicker in the evening and winter.
  smokeT -= dt;
  if (smokeT < 0) {
    smokeT = 0.12;
    const busy = season === 3 || minute > 1020 || minute < 480;
    for (const c of chimneys) {
      if (c.x < vx - 40 || c.x > vx + r.viewW + 40 || c.y < vy - 80 || c.y > vy + r.viewH + 40) continue;
      if (!busy && !litHouses.has(c.house) && Math.random() < 0.7) continue;
      if (Math.random() < 0.5) continue;
      const g = 0.62 + Math.random() * 0.12;
      fx.spawn({ x: c.x + (Math.random() - 0.5) * 3, y: c.y - 2, vy: -16 - Math.random() * 6, vx: 0, life: 3.2, max: 3.2, size: 2, r: g, g: g, b: g + 0.04, a: 0.55, kind: 2 });
    }
  }
}

function emitParticles() {
  const r = renderer;
  for (const p of fx.list) {
    const fade = Math.min(1, p.life / Math.min(p.max, 0.6));
    const a = p.a * fade;
    if (p.kind === 3) {
      const glow = 0.5 + 0.5 * Math.sin(p.phase * 4);
      r.particlesTop.push(p.x, p.y, 1, 1, p.r, p.g, p.b, glow * fade);
      r.lights.push(p.x, p.y, 14, 0.7, 1, 0.4, glow * fade * 0.8);
    } else if (p.kind === 4) {
      r.particlesTop.push(p.x, p.y, 1, 7, p.r, p.g, p.b, a);
      r.particlesTop.push(p.x - 1, p.y + 6, 1, 3, p.r, p.g, p.b, a * 0.5);
    } else if (p.kind === 7) {
      const s = Math.round((1 - p.life / p.max) * p.size * 2) + 1;
      r.particles.push(p.x - s, p.y, s * 2, 1, p.r, p.g, p.b, a);
    } else if (p.kind === 0) {
      r.particles.push(p.x, p.y, p.size, p.size, p.r, p.g, p.b, a);
    } else {
      const s = Math.round(p.size);
      r.particlesTop.push(p.x, p.y, s, s, p.r, p.g, p.b, a);
    }
  }
}

let simLast = performance.now();
setInterval(() => {
  const now = performance.now();
  const dt = Math.min(0.1, (now - simLast) / 1000);
  simLast = now;
  fx.update(dt, cur ? cur.data[H.wind] : 0.3);
}, 1000 / 60);

// ---------------------------------------------------------------- input

function toWorld(clientX: number, clientY: number): [number, number] {
  const k = renderer.cssToLogical();
  const rect = canvas.getBoundingClientRect();
  return [cam.x + (clientX - rect.left) * k, cam.y + (clientY - rect.top) * k];
}

let down: { x: number; y: number; t: number } | null = null;
canvas.addEventListener('pointerdown', (e) => {
  down = { x: e.clientX, y: e.clientY, t: performance.now() };
});
canvas.addEventListener('pointerup', (e) => {
  if (!down || !cam.ready) return;
  const moved = Math.hypot(e.clientX - down.x, e.clientY - down.y);
  down = null;
  if (moved > 12) return;
  const [x, y] = toWorld(e.clientX, e.clientY);
  cmd({ t: 'tap', x, y });
  ui.closeTransient();
  for (let i = 0; i < 8; i++) {
    const a = (i / 8) * Math.PI * 2;
    fx.spawn({ x, y, vx: Math.cos(a) * 30, vy: Math.sin(a) * 18, life: 0.35, max: 0.35, size: 1, r: 1, g: 0.95, b: 0.7, kind: 1 });
  }
});

const keys = new Set<string>();
function sendDir() {
  let x = 0;
  let y = 0;
  if (keys.has('a') || keys.has('arrowleft')) x -= 1;
  if (keys.has('d') || keys.has('arrowright')) x += 1;
  if (keys.has('w') || keys.has('arrowup')) y -= 1;
  if (keys.has('s') || keys.has('arrowdown')) y += 1;
  cmd({ t: 'dir', x, y });
}
window.addEventListener('keydown', (e) => {
  if (e.target instanceof HTMLInputElement) return;
  const k = e.key.toLowerCase();
  if (['w', 'a', 's', 'd', 'arrowleft', 'arrowright', 'arrowup', 'arrowdown'].includes(k)) {
    if (!keys.has(k)) {
      keys.add(k);
      sendDir();
    }
    e.preventDefault();
  } else ui.key(k);
});
window.addEventListener('keyup', (e) => {
  const k = e.key.toLowerCase();
  if (keys.delete(k)) sendDir();
});
window.addEventListener('blur', () => {
  keys.clear();
  sendDir();
});
window.addEventListener('resize', () => {
  if (info) renderer.resize();
});

requestAnimationFrame(render);

// Debug hooks for automated testing.
(window as any).__game = { cmd, cam, renderer, get snap() { return cur?.data; } };
