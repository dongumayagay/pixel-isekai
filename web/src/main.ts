// Pixel Isekai client: boots the simulation worker, streams terrain,
// assembles each frame (props, villagers, shadows, lights, weather) and
// drives input and UI.

import '@fontsource/pixelify-sans/400.css';
import '@fontsource/pixelify-sans/600.css';
import './ui/style.css';
import { Renderer, chunkKey, type Batch } from './gl/renderer';
import { atmosphere, type Atmos } from './atmosphere';
import { Kind, Particles } from './particles';
import { UI } from './ui/ui';
import { parseJson } from './util';
import {
  Activity,
  E,
  EF,
  ENT_STRIDE,
  H,
  HEADER,
  HOUSE_ROW,
  LIGHT_ROW,
  PROP_ROW,
  SPRITE_ROW,
  Season,
  type Cmd,
  type FromWorker,
  type ToWorker,
  type UiUpdate,
  type WorldInfo,
} from './protocol';

const canvas = document.getElementById('game') as HTMLCanvasElement;
const renderer = new Renderer(canvas);
renderer.resize();
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
const chimneys: { x: number; y: number; house: number }[] = [];
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
/** Height of a person's middle above their feet: the camera centres on it and the player's lantern hangs there. */
const BODY_MID = 46;

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
  for (let i = 0; i < props.length; i += PROP_ROW) {
    const k = bucketKey(Math.floor(props[i] / cp), Math.floor(props[i + 1] / cp));
    let b = buckets.get(k);
    if (!b) buckets.set(k, (b = []));
    b.push(i);
  }
}

/** Entity id -> offset of its record in the snapshot. */
function indexSnap(data: Float32Array): Map<number, number> {
  const m = new Map<number, number>();
  const n = data[H.count];
  for (let i = 0; i < n; i++) m.set(data[HEADER + i * ENT_STRIDE + E.id], HEADER + i * ENT_STRIDE);
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
      for (let i = 0; i < m.houses.length; i += HOUSE_ROW) {
        const [house, overlay, cx, cy] = m.houses.subarray(i, i + HOUSE_ROW);
        houseOverlay.set(house, overlay);
        if (cx >= 0) chimneys.push({ x: cx, y: cy, house });
      }
      rebuildBuckets();
      ui.setMinimap(m.minimap, info.w, info.h);
      ui.ready(m.genMs);
      break;
    }
    case 'frame': {
      prev = cur;
      cur = { data: m.snap, at: performance.now(), index: indexSnap(m.snap) };
      const lit = HEADER + m.snap[H.count] * ENT_STRIDE;
      litHouses = new Set(m.snap.subarray(lit, lit + m.snap[H.litCount]));
      ui.frame(m.snap, parseJson<UiUpdate>(m.ui), m.simMs);
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
      requested.delete(chunkKey(m.cx, m.cy));
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
  send({ t: 'init', seed });
};

// ---------------------------------------------------------------- frame assembly

/** Asks the worker for missing terrain chunks around the camera, nearest first. */
function requestChunks(season: number) {
  if (!info) return;
  const cp = info.chunkPx;
  const x0 = Math.floor((cam.x - cp * 0.5) / cp);
  const y0 = Math.floor((cam.y - cp * 0.5) / cp);
  const x1 = Math.floor((cam.x + renderer.viewW + cp * 0.5) / cp);
  const y1 = Math.floor((cam.y + renderer.viewH + cp * 0.5) / cp);
  const cols = info.w / info.chunk;
  const rows = info.h / info.chunk;
  const ccx = (cam.x + renderer.viewW / 2) / cp;
  const ccy = (cam.y + renderer.viewH / 2) / cp;
  const want: [number, number, number][] = [];
  for (let cy = y0; cy <= y1; cy++) {
    for (let cx = x0; cx <= x1; cx++) {
      if (cx < 0 || cy < 0 || cx >= cols || cy >= rows) continue;
      if (renderer.hasChunk(cx, cy, season) || requested.has(chunkKey(cx, cy))) continue;
      want.push([cx, cy, (cx + 0.5 - ccx) ** 2 + (cy + 0.5 - ccy) ** 2]);
    }
  }
  if (!want.length) return;
  want.sort((a, b) => a[2] - b[2]);
  const list = want.map(([cx, cy]): [number, number] => {
    requested.add(chunkKey(cx, cy));
    return [cx, cy];
  });
  send({ t: 'chunks', list });
}

interface Drawable {
  y: number;
  x: number;
  sid: number;
  flags: number;
  alpha: number;
  layerY: number;
}

/** World-space rectangle whose contents get drawn this frame. */
interface View {
  x0: number;
  x1: number;
  y0: number;
  y1: number;
}

type EntityPos = (id: number, offset: number) => [number, number];

const draw: Drawable[] = [];

/** Appends one atlas sprite instance; ids not yet in the streamed table are skipped. */
function writeSprite(batch: Batch, x: number, y: number, sid: number, flags: number, alpha: number) {
  const s = sid * SPRITE_ROW;
  if (s + SPRITE_ROW - 1 >= table.length) return;
  const o = batch.slot();
  const a = batch.data;
  a[o] = x;
  a[o + 1] = y;
  a[o + 2] = table[s + 1];
  a[o + 3] = table[s + 2];
  a[o + 4] = table[s + 3];
  a[o + 5] = table[s + 4];
  a[o + 6] = table[s + 5];
  a[o + 7] = table[s + 6];
  a[o + 8] = table[s];
  a[o + 9] = flags;
  a[o + 10] = alpha;
}

/** A sprite that also casts a projected ground shadow. */
function pushCaster(x: number, y: number, sid: number, flags: number) {
  writeSprite(renderer.casters, x, y, sid, flags & (EF.FLIP | EF.SWAY), 1);
}

/** Ground shadow projection from the sun (or moon) for this time of day. */
function sunParams(minute: number, weather: number, night: number): [number, number, number] {
  const t = Math.min(1, Math.max(0, (minute - 360) / 780));
  const dx = -1.25 + 2.5 * t + 0.2;
  const dy = 0.3 + Math.abs(t - 0.5) * 0.35;
  let k = night > 0.6 ? 0.35 : 1 - night * 0.55;
  if (night > 0.6) return [0.35, 0.28, k * [1, 0.5, 0.25, 0.15, 0.4, 0.2][weather]];
  k *= [1, 0.55, 0.32, 0.22, 0.5, 0.25][weather];
  return [dx, dy, k];
}

function spriteW(sid: number) {
  return table[sid * SPRITE_ROW + 3] || 16;
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
  const wind = snap[H.wind];
  const at = atmosphere(minute, weather, snap[H.intensity], season);

  // Interpolation between the last two snapshots.
  const alpha = prev ? Math.min(1, Math.max(0, (now - cur.at) / TICK)) : 1;
  const entityPos: EntityPos = (id, o) => {
    const x = snap[o + E.x];
    const y = snap[o + E.y];
    if (!prev) return [x, y];
    const po = prev.index.get(id);
    if (po === undefined) return [x, y];
    const px = prev.data[po + E.x];
    const py = prev.data[po + E.y];
    // Teleports and long jumps snap rather than slide.
    if (Math.abs(px - x) > 64 || Math.abs(py - y) > 64) return [x, y];
    return [px + (x - px) * alpha, py + (y - py) * alpha];
  };

  followPlayer(info, dt, entityPos);
  requestChunks(season);

  const r = renderer;
  r.sprites.reset();
  r.overlay.reset();
  r.shadows.reset();
  r.casters.reset();
  r.lights.reset();
  r.particles.reset();
  r.particlesTop.reset();
  draw.length = 0;

  // Generous vertical margin: trees and houses are tall.
  const view: View = { x0: cam.x - 120, x1: cam.x + r.viewW + 120, y0: cam.y - 46, y1: cam.y + r.viewH + 200 };
  collectProps(info.chunkPx, view, at);
  const emotes = collectActors(snap, view, at, entityPos, dt);
  draw.sort((a, b) => a.layerY - b.layerY);
  for (const d of draw) writeSprite(r.sprites, d.x, d.y, d.sid, d.flags, d.alpha);
  drawEmotes(emotes);
  if (at.night > 0.02) collectLights(at);

  ambientFx(dt, at, season, wind, minute);
  emitParticles();
  updateLightning(dt, at);

  r.frame(
    {
      sun: sunParams(minute, weather, at.night),
      shadowTint: at.night > 0.6 ? [0.78, 0.8, 0.9] : [0.7, 0.74, 0.9],
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
      vignette: 0.22 + at.night * 0.4,
      windDir: [0.8 + wind, 0.3],
    },
    info.chunkPx,
  );
}

/** Eases the camera toward the player, clamped to the world edges. */
function followPlayer(world: WorldInfo, dt: number, entityPos: EntityPos) {
  const po = cur!.index.get(0);
  if (po === undefined) return;
  const [px, py] = entityPos(0, po);
  const tx = px - renderer.viewW / 2;
  const ty = py - BODY_MID - renderer.viewH / 2;
  if (!cam.ready) {
    cam.x = tx;
    cam.y = ty;
    cam.ready = true;
  }
  const k = 1 - Math.exp(-dt * 7);
  cam.x += (tx - cam.x) * k;
  cam.y += (ty - cam.y) * k;
  cam.x = Math.max(0, Math.min(world.w * world.tile - renderer.viewW, cam.x));
  cam.y = Math.max(0, Math.min(world.h * world.tile - renderer.viewH, cam.y));
  ui.player(px / world.tile, py / world.tile);
}

/** Visible static props, their shadows and lit house windows. */
function collectProps(chunkPx: number, view: View, at: Atmos) {
  const r = renderer;
  for (let cy = Math.floor(view.y0 / chunkPx); cy <= Math.floor(view.y1 / chunkPx); cy++) {
    for (let cx = Math.floor(view.x0 / chunkPx); cx <= Math.floor(view.x1 / chunkPx); cx++) {
      const b = buckets.get(bucketKey(cx, cy));
      if (!b) continue;
      for (const i of b) {
        const x = props[i];
        const y = props[i + 1];
        if (x < view.x0 || x > view.x1 || y < view.y0 || y > view.y1) continue;
        const sid = props[i + 2];
        const f = props[i + 3];
        const house = props[i + 4];
        if (f & EF.GROUND) {
          draw.push({ x, y, sid, flags: f, alpha: 1, layerY: -1e9 });
          continue;
        }
        draw.push({ x, y, sid, flags: f, alpha: 1, layerY: y });
        const w = spriteW(sid);
        if (f & (EF.SHADOW | EF.BIG_SHADOW | EF.HOUSE)) pushCaster(x, y, sid, f);
        if (f & EF.BIG_SHADOW) r.shadows.push(x, y - 1, 10, 4, 0.8);
        else if (f & EF.SHADOW) r.shadows.push(x, y - 1, Math.max(5, Math.min(18, w * 0.32)), Math.max(2, Math.min(6, w * 0.12)), 0.7);
        if (f & EF.HOUSE) {
          r.shadows.push(x + w / 2 - 6, y + 1, w * 0.52, 6, 0.6);
          if (litHouses.has(house) && at.night > 0.05) {
            const ov = houseOverlay.get(house);
            if (ov !== undefined) draw.push({ x, y, sid: ov, flags: 0, alpha: Math.min(1, at.night * 1.6), layerY: y + 0.01 });
          }
        }
      }
    }
  }
}

/** Visible villagers and the player; returns their emotes as [x, y, emote]. */
function collectActors(snap: Float32Array, view: View, at: Atmos, entityPos: EntityPos, dt: number): [number, number, number][] {
  const r = renderer;
  const emotes: [number, number, number][] = [];
  const n = snap[H.count];
  for (let k = 0; k < n; k++) {
    const o = HEADER + k * ENT_STRIDE;
    const id = snap[o + E.id];
    const [x, y] = entityPos(id, o);
    if (x < view.x0 || x > view.x1 || y < view.y0 || y > view.y1) continue;
    const sid = snap[o + E.sprite];
    const f = snap[o + E.flags];
    draw.push({ x, y, sid, flags: f & (EF.FLIP | EF.FLASH), alpha: 1, layerY: y });
    pushCaster(x, y, sid, f & EF.FLIP);
    r.shadows.push(x, y - 1, 13, 5, 0.85);
    const emote = snap[o + E.emote];
    if (emote > 0) emotes.push([x, y, emote]);
    workFx(id, x, y, snap[o + E.activity] & 255, f, dt);
    if (f & EF.PLAYER && at.night > 0.2) r.lights.push(x, y - BODY_MID, 96, 1, 0.9, 0.75, 0.5 * at.night);
  }
  return emotes;
}

/** Emote bubbles bob above heads, on top of everything. */
function drawEmotes(emotes: [number, number, number][]) {
  for (const [x, y, e] of emotes) {
    const sid = named[`emote_${e}`];
    if (sid === undefined) continue;
    const bob = Math.round(Math.sin(time * 4 + x) * 1.5);
    writeSprite(renderer.overlay, x, y - 100 + bob, sid, 0, 1);
  }
}

/** Lamps, fires and the windows of lit houses. */
function collectLights(at: Atmos) {
  const r = renderer;
  for (let i = 0; i < lights.length; i += LIGHT_ROW) {
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

function updateLightning(dt: number, at: Atmos) {
  if (at.storm) {
    nextBolt -= dt;
    if (nextBolt < 0) {
      flash = 1;
      nextBolt = 4 + Math.random() * 9;
      ui.thunder();
    }
  }
  flash = Math.max(0, flash - dt * 3.5);
}

// ---------------------------------------------------------------- particles

/** Chips and sparks thrown up by work: where they start relative to the worker's
 *  feet (dx toward the facing side), how hard they fly, and the seconds between bursts. */
interface Debris {
  count: number;
  dx: number;
  dy: number;
  spread: number;
  lift: number;
  liftJitter: number;
  life: number;
  size: number;
  rgb: [number, number, number];
  cooldown: number;
}

const DEBRIS: Partial<Record<number, Debris>> = {
  [Activity.Chop]: { count: 4, dx: 6, dy: -16, spread: 60, lift: 40, liftJitter: 30, life: 0.6, size: 2, rgb: [0.7, 0.5, 0.3], cooldown: 0.66 },
  [Activity.Mine]: { count: 4, dx: 6, dy: -12, spread: 70, lift: 50, liftJitter: 30, life: 0.5, size: 2, rgb: [0.62, 0.62, 0.7], cooldown: 0.66 },
  [Activity.Smith]: { count: 6, dx: 0, dy: -18, spread: 80, lift: 30, liftJitter: 50, life: 0.45, size: 1, rgb: [1, 0.75, 0.3], cooldown: 0.5 },
  [Activity.Hoe]: { count: 3, dx: 8, dy: -2, spread: 40, lift: 30, liftJitter: 20, life: 0.4, size: 2, rgb: [0.45, 0.32, 0.25], cooldown: 0.7 },
};

/** Seconds until each entity's next work effect. */
const fxCooldown = new Map<number, number>();

function workFx(id: number, x: number, y: number, act: number, flags: number, dt: number) {
  const t = (fxCooldown.get(id) ?? 0) - dt;
  if (t > 0) {
    fxCooldown.set(id, t);
    return;
  }
  const flip = flags & EF.FLIP ? -1 : 1;
  const debris = DEBRIS[act];
  if (debris) {
    const [r, g, b] = debris.rgb;
    for (let i = 0; i < debris.count; i++) {
      fx.spawn({
        x: x + flip * debris.dx,
        y: y + debris.dy,
        vx: (Math.random() - 0.5) * debris.spread,
        vy: -debris.lift - Math.random() * debris.liftJitter,
        life: debris.life,
        max: debris.life,
        size: debris.size,
        r,
        g,
        b,
        kind: Kind.Ground,
      });
    }
    fxCooldown.set(id, debris.cooldown);
    return;
  }
  switch (act) {
    case Activity.Pray:
      fx.spawn({ x: x + (Math.random() - 0.5) * 16, y: y - 30, vy: -12, life: 1.4, max: 1.4, size: 1, r: 1, g: 0.9, b: 0.5, kind: Kind.Air });
      fxCooldown.set(id, 0.3);
      break;
    case Activity.Fish:
      fx.spawn({ x: x + flip * 20, y: y + 10, vx: 0, vy: 0, life: 0.8, max: 0.8, size: 3, r: 0.85, g: 0.95, b: 1, a: 0.6, kind: Kind.Splash });
      fxCooldown.set(id, 1.6 + Math.random() * 2);
      break;
    default:
      fxCooldown.set(id, 0.4);
  }
}

let leafT = 0;
let smokeT = 0;
function ambientFx(dt: number, at: Atmos, season: number, wind: number, minute: number) {
  const r = renderer;
  const vx = cam.x;
  const vy = cam.y;
  // Rain
  const drops = Math.floor(at.rain * 16 * (60 * dt) + Math.random());
  for (let i = 0; i < drops; i++) {
    fx.spawn({ x: vx + Math.random() * (r.viewW + 80) - 40, y: vy - 10 + Math.random() * r.viewH * 0.4, vx: -60 - wind * 60, vy: 420, life: 0.35 + Math.random() * 0.6, max: 1, size: 1, r: 0.78, g: 0.86, b: 1, a: 0.75, kind: Kind.Rain });
  }
  if (at.rain > 0.1) {
    for (let i = 0; i < Math.ceil(at.rain * 5); i++)
      fx.spawn({ x: vx + Math.random() * r.viewW, y: vy + Math.random() * r.viewH, life: 0.3, max: 0.3, size: 2, r: 0.8, g: 0.9, b: 1, a: 0.5, kind: Kind.Splash });
  }
  // Snow
  const flakes = Math.floor(at.snow * 4 * (60 * dt) + Math.random());
  for (let i = 0; i < flakes; i++) {
    fx.spawn({ x: vx + Math.random() * (r.viewW + 60) - 30, y: vy - 8, vy: 22 + Math.random() * 18, life: 9, max: 9, size: Math.random() < 0.3 ? 2 : 1, r: 1, g: 1, b: 1, a: 0.9, kind: Kind.Snow });
  }
  // Falling leaves in autumn, petals in spring.
  leafT -= dt;
  if (leafT < 0 && (season === Season.Autumn || season === Season.Spring) && at.rain < 0.5) {
    leafT = 0.25;
    const c = season === Season.Autumn ? [[0.85, 0.42, 0.18], [0.92, 0.65, 0.25], [0.7, 0.25, 0.15]][Math.floor(Math.random() * 3)] : [0.97, 0.75, 0.82];
    fx.spawn({ x: vx + Math.random() * r.viewW, y: vy - 4, vy: 18 + Math.random() * 10, life: 10, max: 10, size: 2, r: c[0], g: c[1], b: c[2], kind: Kind.Leaf });
  }
  // Fireflies on warm nights
  if (at.night > 0.5 && (season === Season.Spring || season === Season.Summer) && at.rain < 0.1 && Math.random() < dt * 3) {
    fx.spawn({ x: vx + Math.random() * r.viewW, y: vy + Math.random() * r.viewH, life: 6, max: 6, size: 1, r: 0.85, g: 1, b: 0.45, kind: Kind.Firefly });
  }
  // Chimney smoke, thicker in the evening and winter.
  smokeT -= dt;
  if (smokeT < 0) {
    smokeT = 0.12;
    const busy = season === Season.Winter || minute > 1020 || minute < 480;
    for (const c of chimneys) {
      if (c.x < vx - 40 || c.x > vx + r.viewW + 40 || c.y < vy - 80 || c.y > vy + r.viewH + 40) continue;
      if (!busy && !litHouses.has(c.house) && Math.random() < 0.7) continue;
      if (Math.random() < 0.5) continue;
      const g = 0.62 + Math.random() * 0.12;
      fx.spawn({ x: c.x + (Math.random() - 0.5) * 3, y: c.y - 2, vy: -16 - Math.random() * 6, vx: 0, life: 3.2, max: 3.2, size: 2, r: g, g: g, b: g + 0.04, a: 0.55, kind: Kind.Smoke });
    }
  }
}

function emitParticles() {
  const r = renderer;
  for (const p of fx.list) {
    const fade = Math.min(1, p.life / Math.min(p.max, 0.6));
    const a = p.a * fade;
    if (p.kind === Kind.Firefly) {
      const glow = 0.5 + 0.5 * Math.sin(p.phase * 4);
      r.particlesTop.push(p.x, p.y, 1, 1, p.r, p.g, p.b, glow * fade);
      r.lights.push(p.x, p.y, 14, 0.7, 1, 0.4, glow * fade * 0.8);
    } else if (p.kind === Kind.Rain) {
      r.particlesTop.push(p.x, p.y, 1, 7, p.r, p.g, p.b, a);
      r.particlesTop.push(p.x - 1, p.y + 6, 1, 3, p.r, p.g, p.b, a * 0.5);
    } else if (p.kind === Kind.Splash) {
      // A ripple that widens as it fades.
      const s = Math.round((1 - p.life / p.max) * p.size * 2) + 1;
      r.particles.push(p.x - s, p.y, s * 2, 1, p.r, p.g, p.b, a);
    } else if (p.kind === Kind.Ground) {
      r.particles.push(p.x, p.y, p.size, p.size, p.r, p.g, p.b, a);
    } else {
      const s = Math.round(p.size);
      r.particlesTop.push(p.x, p.y, s, s, p.r, p.g, p.b, a);
    }
  }
}

// Particles step at a steady 60 Hz, independent of the display's frame rate.
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

// A press that barely moves is a tap: walk there, talk or gather.
let down: { x: number; y: number } | null = null;
canvas.addEventListener('pointerdown', (e) => {
  down = { x: e.clientX, y: e.clientY };
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
    fx.spawn({ x, y, vx: Math.cos(a) * 30, vy: Math.sin(a) * 18, life: 0.35, max: 0.35, size: 1, r: 1, g: 0.95, b: 0.7, kind: Kind.Air });
  }
});

const MOVE_KEYS = {
  left: ['a', 'arrowleft'],
  right: ['d', 'arrowright'],
  up: ['w', 'arrowup'],
  down: ['s', 'arrowdown'],
};
const ALL_MOVE_KEYS = Object.values(MOVE_KEYS).flat();
const keys = new Set<string>();

function sendDir() {
  const held = (dir: keyof typeof MOVE_KEYS) => (MOVE_KEYS[dir].some((k) => keys.has(k)) ? 1 : 0);
  cmd({ t: 'dir', x: held('right') - held('left'), y: held('down') - held('up') });
}
window.addEventListener('keydown', (e) => {
  if (e.target instanceof HTMLInputElement) return;
  const k = e.key.toLowerCase();
  if (ALL_MOVE_KEYS.includes(k)) {
    if (!keys.has(k)) {
      keys.add(k);
      sendDir();
    }
    e.preventDefault();
  } else ui.key(k);
});
window.addEventListener('keyup', (e) => {
  if (keys.delete(e.key.toLowerCase())) sendDir();
});
window.addEventListener('blur', () => {
  keys.clear();
  sendDir();
});
window.addEventListener('resize', () => renderer.resize());

requestAnimationFrame(render);

// Debug hooks for automated testing.
Object.assign(window, { __game: { cmd, cam, renderer, get snap() { return cur?.data; } } });
