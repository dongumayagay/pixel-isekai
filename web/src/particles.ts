// Lightweight particle systems: weather, chimney smoke, fireflies, falling
// leaves, work effects and footstep dust. All drawn as pixel-snapped quads.

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
  kind: number; // 0 ground, 1 air, 2 smoke, 3 firefly, 4 rain, 5 snow, 6 leaf, 7 splash
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
      kind: 1,
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
        case 2: // smoke drifts with the wind and grows
          p.vx += (wind * 14 - p.vx) * dt * 0.8;
          p.size = 2 + (1 - p.life / p.max) * 4;
          break;
        case 3: // fireflies wander
          p.vx += Math.sin(p.phase * 1.7) * 6 * dt;
          p.vy += Math.cos(p.phase * 1.3) * 6 * dt;
          p.vx *= 0.98;
          p.vy *= 0.98;
          break;
        case 5: // snow sways
          p.vx = Math.sin(p.phase * 1.5) * 8 + wind * 10;
          break;
        case 6: // leaves flutter
          p.vx = Math.sin(p.phase * 2.2) * 12 + wind * 16;
          break;
        case 0:
          p.vy += 60 * dt;
          break;
      }
      p.x += p.vx * dt;
      p.y += p.vy * dt;
    }
  }
}
