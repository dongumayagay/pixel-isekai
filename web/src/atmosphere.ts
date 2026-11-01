// Time-of-day lighting and weather grading.

import { Weather } from './protocol';

type RGB = [number, number, number];

// Ambient light through the day: minute -> colour. Golden hour, a violet
// dusk and a deep blue moonlit night.
const KEYS: [number, RGB][] = [
  [0, [0.2, 0.23, 0.42]],
  [270, [0.22, 0.24, 0.44]],
  [330, [0.55, 0.42, 0.55]],
  [390, [0.95, 0.7, 0.62]],
  [480, [1.0, 0.95, 0.86]],
  [720, [1.0, 0.98, 0.93]],
  [960, [1.0, 0.94, 0.84]],
  [1080, [1.0, 0.76, 0.56]],
  [1140, [0.82, 0.55, 0.55]],
  [1200, [0.45, 0.38, 0.6]],
  [1290, [0.24, 0.26, 0.46]],
  [1440, [0.2, 0.23, 0.42]],
];

function lerp(a: number, b: number, t: number) {
  return a + (b - a) * t;
}

export interface Atmos {
  ambient: RGB;
  night: number;
  clouds: number;
  fog: number;
  fogColor: RGB;
  saturation: number;
  tint: RGB;
  rain: number;
  snow: number;
  storm: boolean;
}

export function atmosphere(minute: number, weather: number, intensity: number, season: number): Atmos {
  let i = 0;
  while (i < KEYS.length - 2 && KEYS[i + 1][0] <= minute) i++;
  const [m0, c0] = KEYS[i];
  const [m1, c1] = KEYS[i + 1];
  const t = Math.min(1, Math.max(0, (minute - m0) / (m1 - m0)));
  let amb: RGB = [lerp(c0[0], c1[0], t), lerp(c0[1], c1[1], t), lerp(c0[2], c1[2], t)];
  const lum = (amb[0] + amb[1] + amb[2]) / 3;
  const night = Math.min(1, Math.max(0, (0.8 - lum) / 0.5));

  let clouds = (1 - night) * 0.6;
  let fog = 0;
  let saturation = 1.04;
  let tint: RGB = [1, 1, 1];
  let rain = 0;
  let snow = 0;
  const k = intensity;
  switch (weather) {
    case Weather.Cloudy:
      amb = amb.map((v) => v * (1 - 0.12 * k)) as RGB;
      saturation = 1 - 0.1 * k;
      clouds *= 1.6;
      break;
    case Weather.Rain:
    case Weather.Storm: {
      const d = weather === Weather.Storm ? 0.38 : 0.24;
      amb = [amb[0] * (1 - d * k), amb[1] * (1 - d * k * 0.9), amb[2] * (1 - d * k * 0.6)];
      saturation = 1 - 0.25 * k;
      tint = [0.95, 0.98, 1.04];
      clouds *= 0.4;
      rain = k;
      break;
    }
    case Weather.Snow:
      amb = [amb[0] * 0.95, amb[1] * 0.97, amb[2] * 1.02];
      saturation = 0.9;
      snow = k;
      clouds *= 0.5;
      break;
    case Weather.Fog:
      fog = k * 0.8;
      saturation = 0.85;
      clouds = 0;
      break;
  }
  // Season grading: warm summers, crisp blue winters.
  if (season === 1) tint = [tint[0] * 1.03, tint[1] * 1.01, tint[2] * 0.97];
  if (season === 2) tint = [tint[0] * 1.04, tint[1] * 0.99, tint[2] * 0.94];
  if (season === 3) tint = [tint[0] * 0.97, tint[1] * 0.99, tint[2] * 1.05];
  // Daylight exposure: a gentle lift so midday reads bright and sunny.
  const exposure = 1 + 0.1 * (1 - night) * (weather === Weather.Clear ? 1 : 0.4);
  tint = [tint[0] * exposure, tint[1] * exposure, tint[2] * exposure];
  const fogColor: RGB = night > 0.5 ? [0.35, 0.38, 0.5] : [0.82, 0.84, 0.86];
  return { ambient: amb, night, clouds, fog, fogColor, saturation, tint, rain, snow, storm: weather === Weather.Storm };
}
