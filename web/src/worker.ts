// Simulation worker: owns the Rust engine, ticks it at a fixed rate and
// streams snapshots, terrain chunks and atlas updates to the main thread.

import init, { Engine } from './pkg/sim.js';
import wasmUrl from './pkg/sim_bg.wasm?url';
import type { ToWorker, FromWorker } from './protocol';

const TICK_MS = 1000 / 30;
let engine: Engine | null = null;
let last = 0;
let season = -1;
const chunkQueue: [number, number][] = [];

function post(msg: FromWorker, transfer: Transferable[] = []) {
  (self as unknown as Worker).postMessage(msg, transfer);
}

function sendStatics(e: Engine) {
  const props = e.static_props();
  post({ t: 'props', props, season: e.season() }, [props.buffer]);
}

function flushAtlas(e: Engine) {
  const dirty = e.take_atlas_dirty();
  // Merge into page-row strips is overkill; rects are small character frames.
  for (let i = 0; i < dirty.length; i += 5) {
    const [page, x, y, w, h] = [dirty[i], dirty[i + 1], dirty[i + 2], dirty[i + 3], dirty[i + 4]];
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
  let dt = (now - last) / 1000;
  last = now;
  if (dt > 0.25) dt = 0.25;
  const t0 = performance.now();
  engine.tick(dt);
  const simMs = performance.now() - t0;
  const snap = engine.snapshot();
  const ui = engine.take_ui();
  post({ t: 'frame', snap, ui, simMs }, [snap.buffer]);
  if (engine.props_changed() || engine.season() !== season) {
    if (engine.season() !== season && season !== -1) post({ t: 'season', season: engine.season() });
    season = engine.season();
    sendStatics(engine);
  }
  flushAtlas(engine);
  // Bake at most one requested chunk per tick to keep ticks smooth.
  const c = chunkQueue.shift();
  if (c) {
    const px = engine.bake_chunk(c[0], c[1]);
    post({ t: 'chunk', cx: c[0], cy: c[1], season: engine.season(), px }, [px.buffer]);
  }
  setTimeout(loop, Math.max(0, TICK_MS - (performance.now() - now)));
}

self.onmessage = async (ev: MessageEvent<ToWorker>) => {
  const m = ev.data;
  switch (m.t) {
    case 'init': {
      await init({ module_or_path: wasmUrl });
      post({ t: 'progress', label: 'Shaping the world…' });
      const t0 = performance.now();
      engine = new Engine(m.seed);
      const info = JSON.parse(engine.world_info());
      const pages: Uint8Array[] = [];
      for (let i = 0; i < info.pages; i++) pages.push(engine.atlas_page(i));
      engine.take_atlas_dirty();
      const table = engine.sprite_table();
      const lights = engine.static_lights();
      const houses = engine.house_info();
      const minimap = engine.minimap();
      const named = JSON.parse(engine.named_sprites());
      season = engine.season();
      const props = engine.static_props();
      post(
        {
          t: 'ready',
          info,
          pages,
          table,
          lights,
          houses,
          minimap,
          named,
          props,
          season,
          genMs: performance.now() - t0,
        },
        [...pages.map((p) => p.buffer), table.buffer, lights.buffer, houses.buffer, minimap.buffer, props.buffer],
      );
      last = performance.now();
      loop();
      break;
    }
    case 'cmd':
      engine?.command(JSON.stringify(m.cmd));
      break;
    case 'chunks':
      for (const c of m.list) {
        if (!chunkQueue.some((q) => q[0] === c[0] && q[1] === c[1])) chunkQueue.push(c);
      }
      break;
    case 'inspect':
      if (engine) post({ t: 'inspect', data: JSON.parse(engine.inspect(m.id)) });
      break;
    case 'status':
      if (engine) post({ t: 'status', data: JSON.parse(engine.status()) });
      break;
    case 'chronicle':
      if (engine) post({ t: 'chronicle', data: JSON.parse(engine.chronicle()) });
      break;
    case 'villagers':
      if (engine) post({ t: 'villagers', data: JSON.parse(engine.villagers()) });
      break;
    case 'portrait':
      if (engine) {
        const px = engine.sprite_pixels(m.id);
        post({ t: 'portrait', key: m.key, px }, [px.buffer]);
      }
      break;
  }
};
