// Messages between the main thread and the simulation worker, and the
// layout of the binary snapshot produced by the Rust engine.

export interface WorldInfo {
  w: number;
  h: number;
  tile: number;
  chunk: number;
  chunkPx: number;
  pages: number;
  pageSize: number;
  seed: number;
}

export type Cmd =
  | { t: 'tap'; x: number; y: number }
  | { t: 'dir'; x: number; y: number }
  | { t: 'speed'; v: number }
  | { t: 'weather'; w: number; i: number }
  | { t: 'skip'; hours: number }
  | { t: 'teleport'; x: number; y: number };

export type ToWorker =
  | { t: 'init'; seed: number }
  | { t: 'cmd'; cmd: Cmd }
  | { t: 'chunks'; list: [number, number][] }
  | { t: 'inspect'; id: number }
  | { t: 'status' }
  | { t: 'chronicle' }
  | { t: 'villagers' }
  | { t: 'portrait'; id: number; key: string };

export type FromWorker =
  | { t: 'progress'; label: string }
  | {
      t: 'ready';
      info: WorldInfo;
      pages: Uint8Array[];
      table: Float32Array;
      lights: Float32Array;
      houses: Float32Array;
      minimap: Uint8Array;
      named: Record<string, number>;
      props: Float32Array;
      season: number;
      genMs: number;
    }
  | { t: 'frame'; snap: Float32Array; ui: string; simMs: number }
  | { t: 'props'; props: Float32Array; season: number }
  | { t: 'season'; season: number }
  | { t: 'chunk'; cx: number; cy: number; season: number; px: Uint8Array }
  | { t: 'atlas'; page: number; x: number; y: number; w: number; h: number; px: Uint8Array }
  | { t: 'sprites'; table: Float32Array }
  | { t: 'inspect'; data: any }
  | { t: 'status'; data: any }
  | { t: 'chronicle'; data: any[] }
  | { t: 'villagers'; data: any[] }
  | { t: 'portrait'; key: string; px: Uint8Array };

// Snapshot layout (mirrors sim/src/game/mod.rs).
export const HEADER = 20;
export const ENT_STRIDE = 8;
export const H = {
  tick: 0,
  minute: 1,
  day: 2,
  season: 3,
  year: 4,
  weather: 5,
  intensity: 6,
  px: 7,
  py: 8,
  hp: 9,
  energy: 10,
  gold: 11,
  count: 12,
  litCount: 13,
  wind: 15,
  speed: 16,
} as const;

export const EF = {
  FLIP: 1,
  SHADOW: 2,
  FLASH: 4,
  PLAYER: 8,
  SWAY: 16,
  GROUND: 32,
  BIG_SHADOW: 64,
  HOUSE: 128,
} as const;

export const Weather = { Clear: 0, Cloudy: 1, Rain: 2, Storm: 3, Snow: 4, Fog: 5 } as const;
export const Activity = { None: 0, Chop: 1, Mine: 2, Hoe: 3, Fish: 4, Smith: 5, Pray: 6, Talk: 7, Sweep: 8, Play: 9, Fight: 10 } as const;
