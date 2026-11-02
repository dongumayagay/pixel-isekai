// Lightweight particle systems: weather, chimney smoke, fireflies, falling
// leaves and work effects. All drawn as pixel-snapped quads.

/** How a particle moves and draws. Ground debris falls under gravity and draws below sprites. */
export const Kind = { Ground: 0, Air: 1, Smoke: 2, Firefly: 3, Rain: 4, Snow: 5, Leaf: 6, Splash: 7 } as const;
export type Kind = (typeof Kind)[keyof typeof Kind];

export interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  max: number;
  size: number;
  r: number;
  g: number;
  b: number;
  a: number;
  kind: Kind;
  phase: number;
}

const MAX = 3000;

export class Particles {
  list: Particle[] = [];

  spawn(p: Partial<Particle> & { x: number; y: number }) {
    if (this.list.length >= MAX) return;
    this.list.push({
      vx: 0,
      vy: 0,
      life: 1,
      max: 1,
      size: 1,
      r: 1,
      g: 1,
      b: 1,
      a: 1,
      kind: Kind.Air,
      phase: Math.random() * 6.28,
      ...p,
    });
  }

  update(dt: number, wind: number) {
    const L = this.list;
    for (let i = L.length - 1; i >= 0; i--) {
      const p = L[i];
      p.life -= dt;
      if (p.life <= 0) {
        L[i] = L[L.length - 1];
        L.pop();
        continue;
      }
      p.phase += dt;
      switch (p.kind) {
        case Kind.Smoke: // drifts with the wind and grows
          p.vx += (wind * 14 - p.vx) * dt * 0.8;
          p.size = 2 + (1 - p.life / p.max) * 4;
          break;
        case Kind.Firefly: // wanders
          p.vx += Math.sin(p.phase * 1.7) * 6 * dt;
          p.vy += Math.cos(p.phase * 1.3) * 6 * dt;
          p.vx *= 0.98;
          p.vy *= 0.98;
          break;
        case Kind.Snow: // sways
          p.vx = Math.sin(p.phase * 1.5) * 8 + wind * 10;
          break;
        case Kind.Leaf: // flutters
          p.vx = Math.sin(p.phase * 2.2) * 12 + wind * 16;
          break;
        case Kind.Ground:
          p.vy += 60 * dt;
          break;
      }
      p.x += p.vx * dt;
      p.y += p.vy * dt;
    }
  }
}
