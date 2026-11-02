// Messages between the main thread and the simulation worker, the JSON the
// Rust engine returns, and the layout of its binary buffers.

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
  | { t: 'inspect'; data: VillagerProfile | null }
  | { t: 'status'; data: Status }
  | { t: 'chronicle'; data: WorldEvent[] }
  | { t: 'villagers'; data: VillagerSummary[] }
  | { t: 'portrait'; key: string; px: Uint8Array };

// ------------------------------------------------------------ engine JSON (mirrors sim/src/game)

/** A chronicle entry; `kind` doubles as a CSS class suffix (`k-birth`, …). */
export interface WorldEvent {
  day: number;
  minute: number;
  kind: string;
  text: string;
  who: number[];
}

/** Drained once per tick by `Engine::take_ui`. */
export interface UiUpdate {
  events: WorldEvent[];
  toasts: string[];
  dialogue: DialogueOpen[];
  season_changed: boolean;
}

export interface DialogueOpen {
  id: number;
  name: string;
  job: string;
  lines: string[];
  opinion: number;
  sheet: number;
}

export interface Stock {
  food: number;
  wood: number;
  ore: number;
  goods: number;
  gold: number;
}

export interface VillageStatus {
  name: string;
  x: number;
  y: number;
  pop: number;
  stock: Stock | null;
}

export interface Status {
  date: string;
  season: number;
  day: number;
  weather: number;
  inventory: [string, number][];
  gold: number;
  population: number;
  villages: VillageStatus[];
}

export interface Relation {
  id: number;
  name: string;
  score: number;
}

export interface VillagerProfile {
  id: number;
  name: string;
  age: number;
  stage: string;
  female: boolean;
  job: string;
  village: string;
  traits: string[];
  needs: { hunger: number; energy: number; social: number; fun: number };
  mood: number;
  doing: string;
  partner: string | null;
  dating: string | null;
  children: string[];
  parents: string[];
  friends: Relation[];
  rivals: Relation[];
  memories: string[];
  gold: number;
  opinion: number;
  sheet: number;
  pregnant: boolean;
}

export interface VillagerSummary {
  id: number;
  name: string;
  job: string;
  village: number;
  x: number;
  y: number;
  sheet: number;
}

// ------------------------------------------------------------ binary layouts (mirror sim/src/game/mod.rs and api.rs)

/** Snapshot: a header of HEADER floats, then ENT_STRIDE floats per entity, then lit house indices. */
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

/** Offsets within one snapshot entity. The low byte of `activity` is an `Activity`. */
export const E = { id: 0, x: 1, y: 2, sprite: 3, flags: 4, emote: 5, activity: 6 } as const;

/** Sprite table rows: page, x, y, w, h, pivot x, pivot y. */
export const SPRITE_ROW = 7;
/** Static prop rows: x, y, sprite, flags, prop index. */
export const PROP_ROW = 5;
/** Static light rows: x, y, radius, r, g, b, house prop index (or -1). */
export const LIGHT_ROW = 7;
/** House rows: prop index, lit overlay sprite, chimney x, chimney y (-1 if none). */
export const HOUSE_ROW = 4;
/** Dirty atlas rects: page, x, y, w, h. */
export const DIRTY_ROW = 5;

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
export const Season = { Spring: 0, Summer: 1, Autumn: 2, Winter: 3 } as const;
export const Activity = { None: 0, Chop: 1, Mine: 2, Hoe: 3, Fish: 4, Smith: 5, Pray: 6, Talk: 7, Sweep: 8, Play: 9, Fight: 10 } as const;
