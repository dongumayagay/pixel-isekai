// Simulation worker: owns the Rust engine, ticks it at a fixed rate and
// streams snapshots, terrain chunks and atlas updates to the main thread.

import init, { Engine } from './pkg/sim.js';
import wasmUrl from './pkg/sim_bg.wasm?url';
import { DIRTY_ROW, type FromWorker, type ToWorker, type WorldInfo } from './protocol';
import { parseJson } from './util';

const TICK_MS = 1000 / 30;
let engine: Engine | null = null;
let last = 0;
let season = 0;
const chunkQueue: [number, number][] = [];

function post(msg: FromWorker, transfer: Transferable[] = []) {
  (self as unknown as Worker).postMessage(msg, transfer);
}

function flushAtlas(e: Engine) {
  const dirty = e.take_atlas_dirty();
  // Rects are small character frames, so each is sent as-is rather than merged.
  for (let i = 0; i < dirty.length; i += DIRTY_ROW) {
    const [page, x, y, w, h] = dirty.subarray(i, i + DIRTY_ROW);
    const px = e.atlas_rect(page, x, y, w, h);
    post({ t: 'atlas', page, x, y, w, h, px }, [px.buffer]);
  }
  if (dirty.length) {
    const table = e.sprite_table();
    post({ t: 'sprites', table }, [table.buffer]);
  }
}

function loop() {
  if (!engine) return;
  const now = performance.now();
  const dt = Math.min(0.25, (now - last) / 1000);
  last = now;
  engine.tick(dt);
  const simMs = performance.now() - now;
  const snap = engine.snapshot();
  post({ t: 'frame', snap, ui: engine.take_ui(), simMs }, [snap.buffer]);
  const nowSeason = engine.season();
  if (engine.props_changed() || nowSeason !== season) {
    if (nowSeason !== season) post({ t: 'season', season: nowSeason });
    season = nowSeason;
    const props = engine.static_props();
    post({ t: 'props', props, season }, [props.buffer]);
  }
  flushAtlas(engine);
  // Bake at most one requested chunk per tick to keep ticks smooth.
  const c = chunkQueue.shift();
  if (c) {
    const [cx, cy] = c;
    const px = engine.bake_chunk(cx, cy);
    post({ t: 'chunk', cx, cy, season, px }, [px.buffer]);
  }
  setTimeout(loop, Math.max(0, TICK_MS - (performance.now() - now)));
}

async function start(seed: number) {
  await init({ module_or_path: wasmUrl });
  post({ t: 'progress', label: 'Shaping the world…' });
  const t0 = performance.now();
  const e = (engine = new Engine(seed));
  const info = parseJson<WorldInfo>(e.world_info());
  const pages = Array.from({ length: info.pages }, (_, i) => e.atlas_page(i));
  // The initial pages already hold every sprite baked so far.
  e.take_atlas_dirty();
  const table = e.sprite_table();
  const lights = e.static_lights();
  const houses = e.house_info();
  const minimap = e.minimap();
  const props = e.static_props();
  season = e.season();
  post(
    {
      t: 'ready',
      info,
      pages,
      table,
      lights,
      houses,
      minimap,
      named: parseJson<Record<string, number>>(e.named_sprites()),
      props,
      season,
      genMs: performance.now() - t0,
    },
    [...pages.map((p) => p.buffer), table.buffer, lights.buffer, houses.buffer, minimap.buffer, props.buffer],
  );
  last = performance.now();
  loop();
}

self.onmessage = (ev: MessageEvent<ToWorker>) => {
  const m = ev.data;
  if (m.t === 'init') {
    start(m.seed);
    return;
  }
  if (!engine) return;
  switch (m.t) {
    case 'cmd':
      engine.command(JSON.stringify(m.cmd));
      break;
    case 'chunks':
      for (const c of m.list) {
        if (!chunkQueue.some((q) => q[0] === c[0] && q[1] === c[1])) chunkQueue.push(c);
      }
      break;
    case 'inspect':
      post({ t: 'inspect', data: parseJson(engine.inspect(m.id)) });
      break;
    case 'status':
      post({ t: 'status', data: parseJson(engine.status()) });
      break;
    case 'chronicle':
      post({ t: 'chronicle', data: parseJson(engine.chronicle()) });
      break;
    case 'villagers':
      post({ t: 'villagers', data: parseJson(engine.villagers()) });
      break;
    case 'portrait': {
      const px = engine.sprite_pixels(m.id);
      post({ t: 'portrait', key: m.key, px }, [px.buffer]);
      break;
    }
  }
};
