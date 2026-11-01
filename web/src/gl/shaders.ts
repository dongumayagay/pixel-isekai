// GLSL ES 3.0 shaders. Everything renders at the game's logical pixel
// resolution, then a composite pass upscales by an integer factor.

const common = /* glsl */ `#version 300 es
precision highp float;
precision highp int;
`;

const toClip = /* glsl */ `
vec4 toClip(vec2 p, vec2 size) {
  vec2 c = p / size * 2.0 - 1.0;
  return vec4(c.x, -c.y, 0.0, 1.0);
}
`;

const hash = /* glsl */ `
float hash12(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}
float vnoise(vec2 p) {
  vec2 i = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = hash12(i), b = hash12(i + vec2(1, 0)), c = hash12(i + vec2(0, 1)), d = hash12(i + vec2(1, 1));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}
float bayer4(vec2 p) {
  ivec2 q = ivec2(mod(p, 4.0));
  int i = q.y * 4 + q.x;
  int m[16] = int[16](0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5);
  return (float(m[i]) + 0.5) / 16.0;
}
`;

export const groundVS = common + toClip + /* glsl */ `
layout(location = 0) in vec2 a_corner;
uniform vec2 u_origin;   // chunk world origin (px)
uniform float u_chunk;   // chunk size (px)
uniform vec2 u_cam;      // world px of FBO top-left (integer)
uniform vec2 u_size;     // FBO size
out vec2 v_uv;
out vec2 v_world;
void main() {
  vec2 w = u_origin + a_corner * u_chunk;
  v_world = w;
  v_uv = a_corner;
  gl_Position = toClip(w - u_cam, u_size);
}
`;

export const groundFS = common + hash + /* glsl */ `
uniform sampler2D u_tex;
uniform float u_time;
uniform float u_night;
in vec2 v_uv;
in vec2 v_world;
out vec4 o;
void main() {
  vec4 c = texture(u_tex, v_uv);
  float code = floor(c.a * 255.0 + 0.5);
  vec3 col = c.rgb;
  vec2 p = floor(v_world);
  if (code < 254.5) {
    if (code >= 199.5) {
      // Waterfall: bright streaks scrolling down.
      float col_h = hash12(vec2(p.x, 7.0));
      float s = fract(p.y / 26.0 - u_time * (1.2 + col_h * 0.6) + col_h * 5.0);
      float streak = step(0.72, s) + step(0.9, fract(p.y / 9.0 - u_time * 2.3 + col_h * 3.0)) * 0.6;
      col = mix(vec3(0.19, 0.39, 0.53), vec3(0.62, 0.84, 0.86), clamp(streak + col_h * 0.25, 0.0, 1.0));
      if (fract(p.x * 0.37 + col_h) < 0.12) col = vec3(0.85, 0.95, 0.94);
      float base = (code - 200.0) / 15.0;
      col = mix(col, vec3(0.9, 0.97, 0.96), smoothstep(0.82, 1.0, base) * 0.7);
    } else {
      float depth = (code - 1.0) / 179.0;
      // Gentle swell darkens and lightens bands of water.
      float swell = sin(p.x * 0.045 + p.y * 0.07 + u_time * 0.8) * 0.5 + 0.5;
      col *= 0.94 + 0.08 * swell;
      // Glints: short horizontal dashes that drift and twinkle.
      vec2 g = vec2(floor(p.x / 5.0), floor(p.y / 3.0));
      float gl = hash12(g + floor(u_time * 0.8 + hash12(g) * 10.0));
      float dash = step(0.965 - depth * 0.02, gl) * step(mod(p.y, 3.0), 0.5);
      col = mix(col, vec3(0.75, 0.9, 0.92), dash * 0.8 * (1.0 - u_night * 0.6));
      // Caustic ripples in the shallows.
      float r = vnoise(p * 0.09 + vec2(u_time * 0.35, u_time * 0.2)) + vnoise(p * 0.13 - vec2(u_time * 0.25, 0.0));
      float caustic = step(1.32, r) * (1.0 - smoothstep(0.1, 0.45, depth));
      col = mix(col, vec3(0.55, 0.8, 0.82), caustic * 0.35);
      // Foam lapping at the shore.
      float edge = 0.05 + 0.035 * sin(u_time * 1.7 + p.x * 0.11 + p.y * 0.07) + bayer4(p) * 0.02;
      if (depth < edge) col = vec3(0.86, 0.95, 0.92);
      else if (depth < edge + 0.025) col = mix(col, vec3(0.55, 0.78, 0.78), 0.6);
    }
  }
  o = vec4(col, 1.0);
}
`;

export const spriteVS = common + toClip + /* glsl */ `
layout(location = 0) in vec2 a_corner;
layout(location = 1) in vec2 a_pos;
layout(location = 2) in vec4 a_rect;
layout(location = 3) in vec2 a_piv;
layout(location = 4) in float a_layer;
layout(location = 5) in float a_flags;
layout(location = 6) in float a_alpha;
uniform vec2 u_cam;
uniform vec2 u_size;
uniform float u_time;
uniform float u_wind;
uniform float u_page;
uniform vec3 u_cast; // shadow projection: x shear, y squash, enabled
out vec3 v_uv;
out float v_flash;
out float v_alpha;
void main() {
  int f = int(a_flags + 0.5);
  bool flip = (f & 1) != 0;
  vec2 c = a_corner;
  float pivx = flip ? (a_rect.z - a_piv.x) : a_piv.x;
  vec2 local = vec2(c.x * a_rect.z - pivx, c.y * a_rect.w - a_piv.y);
  if ((f & 16) != 0) {
    float h = 1.0 - c.y;
    float amp = clamp(a_rect.w / 45.0, 0.6, 2.4) * (0.5 + u_wind);
    float s = sin(u_time * 1.4 + a_pos.x * 0.023 + a_pos.y * 0.011) + 0.4 * sin(u_time * 3.1 + a_pos.x * 0.05);
    local.x += floor(s * amp * h * h + 0.5);
  }
  if (u_cast.z > 0.5) {
    // Project the sprite onto the ground: height above the feet becomes a
    // sheared, squashed offset away from the sun.
    float height = max(0.0, -local.y);
    local = vec2(local.x + height * u_cast.x, max(local.y, 0.0) + height * u_cast.y);
  }
  vec2 world = floor(a_pos + 0.5) + floor(local + 0.5);
  gl_Position = toClip(world - u_cam, u_size);
  float u = flip ? 1.0 - c.x : c.x;
  v_uv = vec3((a_rect.x + u * a_rect.z) / u_page, (a_rect.y + c.y * a_rect.w) / u_page, a_layer);
  v_flash = float((f & 4) != 0);
  v_alpha = a_alpha;
}
`;

export const spriteFS = common + /* glsl */ `
uniform highp sampler2DArray u_atlas;
uniform vec4 u_shadowColor; // rgb multiply tint, a = 1 when drawing shadows
in vec3 v_uv;
in float v_flash;
in float v_alpha;
out vec4 o;
void main() {
  vec4 c = texture(u_atlas, v_uv);
  if (c.a < 0.5) discard;
  if (u_shadowColor.a > 0.5) {
    o = vec4(u_shadowColor.rgb, 1.0);
    return;
  }
  vec3 col = mix(c.rgb, vec3(1.0), v_flash * 0.75);
  o = vec4(col, v_alpha);
}
`;

// Soft-edged ellipse shadows, pixel quantised.
export const shadowVS = common + toClip + /* glsl */ `
layout(location = 0) in vec2 a_corner;
layout(location = 1) in vec4 a_shape; // x, y, rx, ry
layout(location = 2) in float a_alpha;
uniform vec2 u_cam;
uniform vec2 u_size;
out vec2 v_local;
out float v_alpha;
out vec2 v_r;
void main() {
  vec2 r = a_shape.zw;
  vec2 local = (a_corner * 2.0 - 1.0) * (r + 1.0);
  v_local = local;
  v_r = r;
  v_alpha = a_alpha;
  gl_Position = toClip(floor(a_shape.xy) + local - u_cam, u_size);
}
`;

export const shadowFS = common + hash + /* glsl */ `
in vec2 v_local;
in float v_alpha;
in vec2 v_r;
uniform vec3 u_tint;
out vec4 o;
void main() {
  vec2 q = (floor(v_local) + 0.5) / v_r;
  float d = dot(q, q);
  if (d > 1.0) discard;
  if (d > 0.7 && bayer4(gl_FragCoord.xy) > 0.5) discard;
  // Multiply blend: deeper at the core for contact occlusion.
  float k = v_alpha * (d < 0.45 ? 1.0 : 0.75);
  o = vec4(mix(vec3(1.0), u_tint, k), 1.0);
}
`;

export const lightVS = common + toClip + /* glsl */ `
layout(location = 0) in vec2 a_corner;
layout(location = 1) in vec3 a_light; // x, y, radius
layout(location = 2) in vec4 a_color; // rgb, intensity
uniform vec2 u_cam;
uniform vec2 u_size;
out vec2 v_local;
out float v_r;
out vec4 v_color;
void main() {
  vec2 local = (a_corner * 2.0 - 1.0) * a_light.z;
  v_local = local;
  v_r = a_light.z;
  v_color = a_color;
  gl_Position = toClip(floor(a_light.xy) + local - u_cam, u_size);
}
`;

export const lightFS = common + hash + /* glsl */ `
in vec2 v_local;
in float v_r;
in vec4 v_color;
out vec4 o;
void main() {
  vec2 p = floor(v_local) + 0.5;
  float d = length(p * vec2(1.0, 1.25)) / v_r;
  if (d > 1.0) discard;
  float f = (1.0 - d);
  f = f * f;
  // Banded falloff with dithered steps reads as hand-painted pixel light.
  float bands = 6.0;
  f = floor(f * bands + bayer4(gl_FragCoord.xy) * 0.9) / bands;
  o = vec4(v_color.rgb * f * v_color.a, 1.0);
}
`;

export const colorVS = common + toClip + /* glsl */ `
layout(location = 0) in vec2 a_corner;
layout(location = 1) in vec4 a_rect;  // x, y, w, h
layout(location = 2) in vec4 a_color;
uniform vec2 u_cam;
uniform vec2 u_size;
out vec4 v_color;
void main() {
  v_color = a_color;
  gl_Position = toClip(floor(a_rect.xy) + a_corner * a_rect.zw - u_cam, u_size);
}
`;

export const colorFS = common + /* glsl */ `
in vec4 v_color;
out vec4 o;
void main() { o = v_color; }
`;

export const compositeVS = common + /* glsl */ `
layout(location = 0) in vec2 a_corner;
void main() { gl_Position = vec4(a_corner * 2.0 - 1.0, 0.0, 1.0); }
`;

export const compositeFS = common + hash + /* glsl */ `
uniform sampler2D u_scene;
uniform sampler2D u_light;
uniform vec2 u_fbo;       // FBO size (logical px)
uniform vec2 u_frac;      // camera sub-pixel offset (logical px)
uniform float u_scale;    // integer upscale
uniform vec2 u_canvas;    // canvas size in device px
uniform vec2 u_camWorld;  // world px of FBO top-left
uniform float u_time;
uniform float u_clouds;   // cloud-shadow strength (day only)
uniform float u_fog;
uniform vec3 u_fogColor;
uniform float u_flash;    // lightning
uniform float u_sat;      // saturation multiplier
uniform vec3 u_tint;      // colour grade multiplier
uniform float u_vignette;
uniform vec2 u_wind;
out vec4 o;
void main() {
  vec2 sp = vec2(gl_FragCoord.x, u_canvas.y - gl_FragCoord.y);
  vec2 l = sp / u_scale + u_frac + 1.0;
  vec2 texel = floor(l) + 0.5;
  vec2 uv = vec2(texel.x / u_fbo.x, 1.0 - texel.y / u_fbo.y);
  vec3 s = texture(u_scene, uv).rgb;
  vec3 L = texture(u_light, uv).rgb;
  vec2 world = u_camWorld + floor(l);

  vec3 col = s * L;
  // Lights bloom a little into their surroundings.
  col += s * max(L - 0.92, 0.0) * 0.6;

  // Drifting cloud shadows, dithered at their edges.
  if (u_clouds > 0.0) {
    vec2 cp = world * 0.0028 + u_wind * u_time * 0.012;
    float n = vnoise(cp) * 0.62 + vnoise(cp * 2.3 + 7.0) * 0.38;
    float m = smoothstep(0.56, 0.66, n + (bayer4(floor(l)) - 0.5) * 0.04);
    col *= 1.0 - m * 0.2 * u_clouds;
  }
  // Fog banks.
  if (u_fog > 0.0) {
    vec2 fp = world * 0.004 + vec2(u_time * 0.02, u_time * 0.006);
    float f = vnoise(fp) * 0.6 + vnoise(fp * 2.7) * 0.4;
    float a = clamp(u_fog * (0.35 + f * 0.65), 0.0, 0.85);
    a = floor(a * 8.0 + bayer4(floor(l))) / 8.0;
    col = mix(col, u_fogColor * max(L, vec3(0.35)), a);
  }
  // Colour grade.
  float lum = dot(col, vec3(0.299, 0.587, 0.114));
  col = mix(vec3(lum), col, u_sat) * u_tint;
  col += u_flash * vec3(0.55, 0.6, 0.75);
  // Vignette.
  vec2 q = sp / u_canvas - 0.5;
  col *= 1.0 - dot(q, q) * u_vignette;
  o = vec4(col, 1.0);
}
`;
