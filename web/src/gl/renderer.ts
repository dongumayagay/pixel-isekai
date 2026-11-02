// WebGL2 renderer. Draws terrain chunks, shadows, y-sorted sprites and
// particles into a logical-resolution scene buffer, accumulates lights into
// a light buffer, then composites both to the canvas at an integer scale.

import * as S from './shaders';

const SPRITE_STRIDE = 11; // x, y, rx, ry, rw, rh, pivx, pivy, layer, flags, alpha
const SHADOW_STRIDE = 5; // x, y, rx, ry, alpha
const LIGHT_STRIDE = 7; // x, y, r, red, green, blue, intensity
const COLOR_STRIDE = 8; // x, y, w, h, r, g, b, a

// Instance attributes per batch layout: [location, size].
const SPRITE_ATTRS: [number, number][] = [[1, 2], [2, 4], [3, 2], [4, 1], [5, 1], [6, 1]];
const COLOR_ATTRS: [number, number][] = [[1, 4], [2, 4]];

/** Terrain chunk textures kept resident; the least recently drawn go first. */
const CHUNK_BUDGET = 24;

/** Fixed 9:16 portrait frame (540×960 logical px), integer-scaled. At this
 *  framing an adult (~88px) stands about a tenth of the screen tall, close
 *  to Eastward and Sea of Stars, and maps 2× onto 1080-wide phones. */
const LOGICAL_W = 540;
const LOGICAL_H = 960;

type VaoName = 'sprite' | 'overlay' | 'casters' | 'shadow' | 'light' | 'color' | 'colorTop' | 'plain';

interface Program {
  p: WebGLProgram;
  u: Record<string, WebGLUniformLocation | null>;
}

interface Target {
  fb: WebGLFramebuffer;
  tex: WebGLTexture;
  rb?: WebGLRenderbuffer;
}

export interface FrameParams {
  /** Ground shadow projection (x shear, y squash) and strength 0..1. */
  sun: [number, number, number];
  shadowTint: [number, number, number];
  camX: number;
  camY: number;
  time: number;
  ambient: [number, number, number];
  night: number;
  wind: number;
  clouds: number;
  fog: number;
  fogColor: [number, number, number];
  flash: number;
  saturation: number;
  tint: [number, number, number];
  vignette: number;
  windDir: [number, number];
}

/** Growable array of per-instance floats, uploaded once per frame. */
export class Batch {
  data: Float32Array;
  n = 0;
  constructor(
    readonly stride: number,
    cap: number,
  ) {
    this.data = new Float32Array(stride * cap);
  }
  reset() {
    this.n = 0;
  }
  push(...v: number[]) {
    this.data.set(v, this.slot());
  }
  /** Reserve one slot and return its offset for direct writes. */
  slot(): number {
    if ((this.n + 1) * this.stride > this.data.length) {
      const d = new Float32Array(this.data.length * 2);
      d.set(this.data);
      this.data = d;
    }
    return this.n++ * this.stride;
  }
}

function pixelSampling(gl: WebGL2RenderingContext, target: number) {
  gl.texParameteri(target, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(target, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texParameteri(target, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(target, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
}

export const chunkKey = (cx: number, cy: number) => `${cx},${cy}`;

export class Renderer {
  private readonly gl: WebGL2RenderingContext;
  private scale = 1;
  readonly viewW = LOGICAL_W;
  readonly viewH = LOGICAL_H;
  // One spare pixel on each side lets the composite pass shift by the
  // camera's sub-pixel offset.
  private readonly fboW = LOGICAL_W + 2;
  private readonly fboH = LOGICAL_H + 2;
  private cssW = 360;
  private quad!: WebGLBuffer;
  private progs!: Record<'ground' | 'sprite' | 'shadow' | 'light' | 'color' | 'composite', Program>;
  private scene!: Target;
  private light!: Target;
  private atlas!: WebGLTexture;
  private pageSize = 2048;
  private chunks = new Map<string, { tex: WebGLTexture; used: number; season: number }>();
  private vaos = {} as Record<VaoName, WebGLVertexArrayObject>;
  private bufs = {} as Record<VaoName, WebGLBuffer>;
  sprites = new Batch(SPRITE_STRIDE, 4096);
  casters = new Batch(SPRITE_STRIDE, 2048);
  overlay = new Batch(SPRITE_STRIDE, 256);
  shadows = new Batch(SHADOW_STRIDE, 2048);
  lights = new Batch(LIGHT_STRIDE, 512);
  particles = new Batch(COLOR_STRIDE, 2048);
  particlesTop = new Batch(COLOR_STRIDE, 2048);

  constructor(readonly canvas: HTMLCanvasElement) {
    const gl = canvas.getContext('webgl2', { antialias: false, alpha: false, premultipliedAlpha: false, powerPreference: 'high-performance' });
    if (!gl) throw new Error('WebGL2 is not supported on this device.');
    this.gl = gl;
    this.init();
  }

  private compile(vs: string, fs: string, uniforms: string[]): Program {
    const gl = this.gl;
    const mk = (type: number, src: string) => {
      const s = gl.createShader(type)!;
      gl.shaderSource(s, src);
      gl.compileShader(s);
      if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s) ?? 'shader error');
      return s;
    };
    const p = gl.createProgram()!;
    gl.attachShader(p, mk(gl.VERTEX_SHADER, vs));
    gl.attachShader(p, mk(gl.FRAGMENT_SHADER, fs));
    gl.linkProgram(p);
    if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p) ?? 'link error');
    const u: Program['u'] = {};
    for (const n of uniforms) u[n] = gl.getUniformLocation(p, n);
    return { p, u };
  }

  private init() {
    const gl = this.gl;
    this.quad = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.quad);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1]), gl.STATIC_DRAW);
    const cam = ['u_cam', 'u_size'];
    this.progs = {
      ground: this.compile(S.groundVS, S.groundFS, [...cam, 'u_origin', 'u_chunk', 'u_tex', 'u_time', 'u_night']),
      sprite: this.compile(S.spriteVS, S.spriteFS, [...cam, 'u_time', 'u_wind', 'u_page', 'u_atlas', 'u_cast', 'u_shadowColor']),
      shadow: this.compile(S.shadowVS, S.shadowFS, [...cam, 'u_tint']),
      light: this.compile(S.lightVS, S.lightFS, cam),
      color: this.compile(S.colorVS, S.colorFS, cam),
      composite: this.compile(S.compositeVS, S.compositeFS, [
        'u_scene', 'u_light', 'u_fbo', 'u_frac', 'u_scale', 'u_canvas', 'u_camWorld', 'u_time', 'u_clouds',
        'u_fog', 'u_fogColor', 'u_flash', 'u_sat', 'u_tint', 'u_vignette', 'u_wind',
      ]),
    };
    this.makeVao('sprite', SPRITE_STRIDE, SPRITE_ATTRS);
    this.makeVao('overlay', SPRITE_STRIDE, SPRITE_ATTRS);
    this.makeVao('casters', SPRITE_STRIDE, SPRITE_ATTRS);
    this.makeVao('shadow', SHADOW_STRIDE, [[1, 4], [2, 1]]);
    this.makeVao('light', LIGHT_STRIDE, [[1, 3], [2, 4]]);
    this.makeVao('color', COLOR_STRIDE, COLOR_ATTRS);
    this.makeVao('colorTop', COLOR_STRIDE, COLOR_ATTRS);
    this.makeVao('plain', 0, []);
  }

  /** A VAO over the shared unit quad, plus an instance buffer when `stride` > 0. */
  private makeVao(name: VaoName, stride: number, attrs: [number, number][]) {
    const gl = this.gl;
    const vao = gl.createVertexArray()!;
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.quad);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    if (stride > 0) {
      const buf = gl.createBuffer()!;
      gl.bindBuffer(gl.ARRAY_BUFFER, buf);
      let off = 0;
      for (const [loc, size] of attrs) {
        gl.enableVertexAttribArray(loc);
        gl.vertexAttribPointer(loc, size, gl.FLOAT, false, stride * 4, off * 4);
        gl.vertexAttribDivisor(loc, 1);
        off += size;
      }
      this.bufs[name] = buf;
    }
    gl.bindVertexArray(null);
    this.vaos[name] = vao;
  }

  private target(w: number, h: number, old?: Target, stencil = false): Target {
    const gl = this.gl;
    if (old) {
      gl.deleteFramebuffer(old.fb);
      gl.deleteTexture(old.tex);
      if (old.rb) gl.deleteRenderbuffer(old.rb);
    }
    const tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, w, h, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    pixelSampling(gl, gl.TEXTURE_2D);
    const fb = gl.createFramebuffer()!;
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    let rb: WebGLRenderbuffer | undefined;
    if (stencil) {
      rb = gl.createRenderbuffer()!;
      gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
      gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH24_STENCIL8, w, h);
      gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.RENDERBUFFER, rb);
    }
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    return { fb, tex, rb };
  }

  /** Fits the fixed logical frame to the window and (re)allocates the offscreen targets. */
  resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, 4);
    const sw = window.innerWidth * dpr;
    const sh = window.innerHeight * dpr;
    this.scale = Math.max(1, Math.floor(Math.min(sw / this.viewW, sh / this.viewH)));
    this.canvas.width = this.viewW * this.scale;
    this.canvas.height = this.viewH * this.scale;
    // CSS size: fill the window at 9:16 (pixelated sampling keeps it crisp).
    const fit = Math.min(window.innerWidth / (this.canvas.width / dpr), window.innerHeight / (this.canvas.height / dpr));
    const cssW = (this.canvas.width / dpr) * fit;
    const cssH = (this.canvas.height / dpr) * fit;
    this.cssW = cssW;
    document.documentElement.style.setProperty('--stage-w', `${cssW}px`);
    document.documentElement.style.setProperty('--stage-h', `${cssH}px`);
    this.scene = this.target(this.fboW, this.fboH, this.scene, true);
    this.light = this.target(this.fboW, this.fboH, this.light);
  }

  /** Logical px per CSS px, for converting pointer events to world px. */
  cssToLogical(): number {
    return this.viewW / this.cssW;
  }

  initAtlas(pages: Uint8Array[], size: number) {
    const gl = this.gl;
    this.pageSize = size;
    this.atlas = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.atlas);
    gl.texStorage3D(gl.TEXTURE_2D_ARRAY, 1, gl.RGBA8, size, size, pages.length);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    pages.forEach((px, i) => {
      gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, i, size, size, 1, gl.RGBA, gl.UNSIGNED_BYTE, px);
    });
    pixelSampling(gl, gl.TEXTURE_2D_ARRAY);
  }

  updateAtlas(page: number, x: number, y: number, w: number, h: number, px: Uint8Array) {
    const gl = this.gl;
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.atlas);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, 0, x, y, page, w, h, 1, gl.RGBA, gl.UNSIGNED_BYTE, px);
  }

  hasChunk(cx: number, cy: number, season: number): boolean {
    return this.chunks.get(chunkKey(cx, cy))?.season === season;
  }

  setChunk(cx: number, cy: number, season: number, size: number, px: Uint8Array) {
    const gl = this.gl;
    const key = chunkKey(cx, cy);
    let c = this.chunks.get(key);
    if (!c) {
      const tex = gl.createTexture()!;
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.texStorage2D(gl.TEXTURE_2D, 1, gl.RGBA8, size, size);
      pixelSampling(gl, gl.TEXTURE_2D);
      c = { tex, used: performance.now(), season };
      this.chunks.set(key, c);
    }
    gl.bindTexture(gl.TEXTURE_2D, c.tex);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, size, size, gl.RGBA, gl.UNSIGNED_BYTE, px);
    c.season = season;
    c.used = performance.now();
    if (this.chunks.size > CHUNK_BUDGET) {
      let oldest: string | null = null;
      let t = Infinity;
      for (const [k, v] of this.chunks) {
        if (v.used < t) {
          t = v.used;
          oldest = k;
        }
      }
      if (oldest) {
        gl.deleteTexture(this.chunks.get(oldest)!.tex);
        this.chunks.delete(oldest);
      }
    }
  }

  /** Binds a program and points its camera at the scene buffer's top-left world pixel. */
  private use(p: Program, camIX: number, camIY: number): Program {
    const gl = this.gl;
    gl.useProgram(p.p);
    gl.uniform2f(p.u.u_cam, camIX, camIY);
    gl.uniform2f(p.u.u_size, this.fboW, this.fboH);
    return p;
  }

  private useSprites(fp: FrameParams, camIX: number, camIY: number): Program {
    const gl = this.gl;
    const p = this.use(this.progs.sprite, camIX, camIY);
    gl.uniform1f(p.u.u_time, fp.time);
    gl.uniform1f(p.u.u_wind, fp.wind);
    gl.uniform1f(p.u.u_page, this.pageSize);
    gl.uniform1i(p.u.u_atlas, 1);
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.atlas);
    return p;
  }

  /** Uploads a batch and draws one quad per instance with the bound program. */
  private drawBatch(name: VaoName, b: Batch) {
    const gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.bufs[name]);
    gl.bufferData(gl.ARRAY_BUFFER, b.data.subarray(0, b.n * b.stride), gl.STREAM_DRAW);
    gl.bindVertexArray(this.vaos[name]);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, b.n);
  }

  frame(fp: FrameParams, chunkPx: number) {
    const gl = this.gl;
    // The scene buffer starts one pixel up-left of the camera; the composite
    // pass then shifts by the sub-pixel remainder.
    const camIX = Math.floor(fp.camX) - 1;
    const camIY = Math.floor(fp.camY) - 1;
    const fracX = fp.camX - Math.floor(fp.camX);
    const fracY = fp.camY - Math.floor(fp.camY);

    // ---- Scene ----
    gl.bindFramebuffer(gl.FRAMEBUFFER, this.scene.fb);
    gl.viewport(0, 0, this.fboW, this.fboH);
    gl.clearColor(0.08, 0.1, 0.2, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.disable(gl.BLEND);

    // Terrain
    const g = this.use(this.progs.ground, camIX, camIY);
    gl.bindVertexArray(this.vaos.plain);
    gl.uniform1f(g.u.u_chunk, chunkPx);
    gl.uniform1f(g.u.u_time, fp.time);
    gl.uniform1f(g.u.u_night, fp.night);
    gl.uniform1i(g.u.u_tex, 0);
    gl.activeTexture(gl.TEXTURE0);
    const cx0 = Math.floor(camIX / chunkPx);
    const cy0 = Math.floor(camIY / chunkPx);
    const cx1 = Math.floor((camIX + this.fboW) / chunkPx);
    const cy1 = Math.floor((camIY + this.fboH) / chunkPx);
    const now = performance.now();
    for (let cy = cy0; cy <= cy1; cy++) {
      for (let cx = cx0; cx <= cx1; cx++) {
        const c = this.chunks.get(chunkKey(cx, cy));
        if (!c) continue;
        c.used = now;
        gl.bindTexture(gl.TEXTURE_2D, c.tex);
        gl.uniform2f(g.u.u_origin, cx * chunkPx, cy * chunkPx);
        gl.drawArrays(gl.TRIANGLES, 0, 6);
      }
    }

    gl.enable(gl.BLEND);

    // Shadows: multiply-blended, each pixel darkened once (stencil), in a
    // cool tint so shaded ground reads blue against warm sunlight.
    gl.enable(gl.STENCIL_TEST);
    gl.clearStencil(0);
    gl.clear(gl.STENCIL_BUFFER_BIT);
    gl.stencilFunc(gl.EQUAL, 0, 0xff);
    gl.stencilOp(gl.KEEP, gl.KEEP, gl.INCR);
    gl.blendFunc(gl.DST_COLOR, gl.ZERO);
    const tint = fp.shadowTint;
    const [shearX, squashY, strength] = fp.sun;
    if (strength > 0.01 && this.casters.n) {
      const p = this.useSprites(fp, camIX, camIY);
      const shade = (t: number) => 1 - (1 - t) * strength;
      gl.uniform3f(p.u.u_cast, shearX, squashY, 1);
      gl.uniform4f(p.u.u_shadowColor, shade(tint[0]), shade(tint[1]), shade(tint[2]), 1);
      this.drawBatch('casters', this.casters);
    }
    if (this.shadows.n) {
      const p = this.use(this.progs.shadow, camIX, camIY);
      gl.uniform3f(p.u.u_tint, ...tint);
      this.drawBatch('shadow', this.shadows);
    }
    gl.disable(gl.STENCIL_TEST);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    // Ground-level particles (dust, splashes) under sprites.
    if (this.particles.n) this.drawColor('color', this.particles, camIX, camIY);

    // Sprites
    const sp = this.useSprites(fp, camIX, camIY);
    gl.uniform3f(sp.u.u_cast, 0, 0, 0);
    gl.uniform4f(sp.u.u_shadowColor, 1, 1, 1, 0);
    if (this.sprites.n) this.drawBatch('sprite', this.sprites);
    // Weather and airborne particles above sprites.
    if (this.particlesTop.n) this.drawColor('colorTop', this.particlesTop, camIX, camIY);
    // Overlays (emote bubbles) on top of everything.
    if (this.overlay.n) {
      gl.useProgram(sp.p);
      this.drawBatch('overlay', this.overlay);
    }

    // ---- Lights ----
    gl.bindFramebuffer(gl.FRAMEBUFFER, this.light.fb);
    gl.viewport(0, 0, this.fboW, this.fboH);
    gl.clearColor(fp.ambient[0], fp.ambient[1], fp.ambient[2], 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    if (this.lights.n) {
      gl.blendFunc(gl.ONE, gl.ONE);
      this.use(this.progs.light, camIX, camIY);
      this.drawBatch('light', this.lights);
    }

    // ---- Composite ----
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    gl.disable(gl.BLEND);
    const c = this.progs.composite;
    gl.useProgram(c.p);
    gl.bindVertexArray(this.vaos.plain);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.scene.tex);
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.light.tex);
    gl.uniform1i(c.u.u_scene, 0);
    gl.uniform1i(c.u.u_light, 2);
    gl.uniform2f(c.u.u_fbo, this.fboW, this.fboH);
    gl.uniform2f(c.u.u_frac, fracX, fracY);
    gl.uniform1f(c.u.u_scale, this.scale);
    gl.uniform2f(c.u.u_canvas, this.canvas.width, this.canvas.height);
    gl.uniform2f(c.u.u_camWorld, camIX, camIY);
    gl.uniform1f(c.u.u_time, fp.time);
    gl.uniform1f(c.u.u_clouds, fp.clouds);
    gl.uniform1f(c.u.u_fog, fp.fog);
    gl.uniform3f(c.u.u_fogColor, ...fp.fogColor);
    gl.uniform1f(c.u.u_flash, fp.flash);
    gl.uniform1f(c.u.u_sat, fp.saturation);
    gl.uniform3f(c.u.u_tint, ...fp.tint);
    gl.uniform1f(c.u.u_vignette, fp.vignette);
    gl.uniform2f(c.u.u_wind, ...fp.windDir);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }

  private drawColor(name: 'color' | 'colorTop', b: Batch, camIX: number, camIY: number) {
    this.use(this.progs.color, camIX, camIY);
    this.drawBatch(name, b);
  }
}
