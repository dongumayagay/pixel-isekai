// HTML overlay UI: title & isekai prologue, HUD, dialogue, villager
// profiles, chronicle, people list and world map.

import { H, type Cmd, type FromWorker, type ToWorker } from '../protocol';
import { icon, iconImg } from './icons';

const $ = <T extends HTMLElement = HTMLElement>(sel: string) => document.querySelector(sel) as T;
const WEATHER_ICON = ['sun', 'cloud', 'rain', 'storm', 'snow', 'fog'];
const WEATHER_NAME = ['Clear', 'Cloudy', 'Rain', 'Storm', 'Snow', 'Fog'];
const SEASON = ['Spring', 'Summer', 'Autumn', 'Winter'];
const SPEEDS = [1, 2, 4, 8];

const PROLOGUE = [
  { art: 'city', text: 'Tokyo, 11:58 PM. Another overtime shift. Another empty train home.' },
  { art: 'truck', text: 'Headlights. A horn. The world turns white…' },
  { art: 'goddess', text: '"Oh dear. Truck-kun again." A gentle voice laughs. "Don\'t be afraid. Your story isn\'t over."' },
  { art: 'goddess', text: '"I have a world that lives and breathes on its own. Villages, families, seasons. Go and live a second life there."' },
  { art: 'field', text: 'You open your eyes to birdsong, warm grass, and a village in the distance.' },
];

function esc(s: string) {
  return String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]!);
}

function clock(minute: number) {
  const h = Math.floor(minute / 60) % 24;
  const m = Math.floor(minute % 60 / 10) * 10;
  const ap = h < 12 ? 'AM' : 'PM';
  const hh = h % 12 === 0 ? 12 : h % 12;
  return `${hh}:${String(m).padStart(2, '0')} ${ap}`;
}

export class UI {
  onSend: (m: ToWorker) => void = () => {};
  onCmd: (c: Cmd) => void = () => {};
  onStart: (seed: number) => void = () => {};
  private worldReady = false;
  private introDone = false;
  private speedIdx = 0;
  private dialogue: { lines: string[]; i: number; data: any } | null = null;
  private portraits = new Map<string, HTMLCanvasElement>();
  private pendingPortrait = new Map<string, HTMLCanvasElement[]>();
  private minimap: HTMLCanvasElement | null = null;
  private worldW = 1;
  private worldH = 1;
  private px = 0;
  private py = 0;
  private lastStatus = 0;
  private status: any = null;
  private feedCount = 0;

  constructor() {
    document.body.insertAdjacentHTML('beforeend', this.template());
    $('#begin').addEventListener('click', () => this.begin());
    $('#seed').addEventListener('keydown', (e) => {
      if ((e as KeyboardEvent).key === 'Enter') this.begin();
    });
    $('#reroll').addEventListener('click', () => (($('#seed') as HTMLInputElement).value = String(Math.floor(Math.random() * 1e9))));
    $('#prologue').addEventListener('click', () => this.nextPrologue());
    $('#speed').addEventListener('click', () => this.cycleSpeed());
    $('#btn-chronicle').addEventListener('click', () => this.openChronicle());
    $('#btn-people').addEventListener('click', () => this.openPeople());
    $('#btn-map').addEventListener('click', () => this.openMap());
    $('#btn-bag').addEventListener('click', () => this.openBag());
    $('#dialogue').addEventListener('click', (e) => {
      if ((e.target as HTMLElement).closest('#dlg-profile')) return;
      this.advanceDialogue();
    });
    $('#dlg-profile').addEventListener('click', () => {
      if (this.dialogue) this.openProfile(this.dialogue.data.id);
    });
    $('#panel').addEventListener('click', (e) => {
      const t = e.target as HTMLElement;
      if (t.closest('[data-close]') || t.id === 'panel') this.closePanel();
      const row = t.closest('[data-vid]') as HTMLElement | null;
      if (row) this.openProfile(Number(row.dataset.vid));
    });
    (($('#seed') as HTMLInputElement).value = String(Math.floor(Math.random() * 1e9)));
  }

  private template() {
    return `
<div id="title" class="screen">
  <div class="title-card">
    <div class="logo"><span class="logo-a">PIXEL</span><span class="logo-b">ISEKAI</span></div>
    <div class="tag">A Second Life</div>
    <p class="blurb">A living world, generated just for you. Villagers work, love, quarrel, grow old and raise families. Your story is only one of many.</p>
    <label class="seed-row">World seed
      <span class="seed-box"><input id="seed" inputmode="numeric" autocomplete="off"><button id="reroll" class="btn small" title="New seed">↻</button></span>
    </label>
    <button id="begin" class="btn primary big">Begin a new life</button>
    <div class="hint">Tap to walk · Tap people to talk · WASD on keyboard</div>
  </div>
</div>
<div id="prologue" class="screen hidden">
  <canvas id="pro-art" width="160" height="90"></canvas>
  <div class="pro-text"><span id="pro-line"></span><div class="pro-next">tap ▸</div></div>
  <div id="pro-status" class="pro-status"></div>
</div>
<div id="hud" class="hidden">
  <div class="hud-time card">
    <img id="hud-wicon" class="wicon" alt="">
    <div><div id="hud-clock" class="clock">7:00 AM</div><div id="hud-date" class="date">Spring 1</div></div>
  </div>
  <div class="hud-right">
    <div class="card pill"><img src="${icon('coin', 2)}" alt="" class="ico">&nbsp;<span id="hud-gold">0</span></div>
    <button id="speed" class="btn card pill" title="Game speed">${iconImg('fast', 2)}<span id="speed-label">1×</span></button>
  </div>
  <div id="toasts"></div>
  <div id="feed"></div>
  <nav class="dock">
    <button id="btn-bag" class="btn dock-btn">${iconImg('bag', 3)}<span>Bag</span></button>
    <button id="btn-people" class="btn dock-btn">${iconImg('people', 3)}<span>People</span></button>
    <button id="btn-chronicle" class="btn dock-btn">${iconImg('book', 3)}<span>Chronicle</span></button>
    <button id="btn-map" class="btn dock-btn">${iconImg('map', 3)}<span>Map</span></button>
  </nav>
  <div id="dialogue" class="hidden">
    <div class="dlg-portrait"><canvas id="dlg-face" width="28" height="48"></canvas></div>
    <div class="dlg-body">
      <div class="dlg-head"><span id="dlg-name"></span><span id="dlg-job" class="muted"></span><span id="dlg-heart"></span></div>
      <div id="dlg-text"></div>
      <div class="dlg-foot"><button id="dlg-profile" class="btn small">Profile</button><span class="muted">tap ▸</span></div>
    </div>
  </div>
</div>
<div id="panel" class="hidden"><div class="panel-card" id="panel-card"></div></div>
<div id="perf"></div>`;
  }

  // ------------------------------------------------------------ start flow

  private begin() {
    const seed = Number(($('#seed') as HTMLInputElement).value) || Math.floor(Math.random() * 1e9);
    $('#title').classList.add('hidden');
    $('#prologue').classList.remove('hidden');
    this.prologueIdx = -1;
    this.nextPrologue();
    // Generate the world while the story plays.
    setTimeout(() => this.onStart(seed), 50);
  }

  private prologueIdx = -1;
  private nextPrologue() {
    this.prologueIdx++;
    if (this.prologueIdx >= PROLOGUE.length) {
      this.introDone = true;
      if (this.worldReady) this.enterWorld();
      else $('#pro-line').textContent = 'The world is still taking shape…';
      return;
    }
    const p = PROLOGUE[this.prologueIdx];
    this.typewrite($('#pro-line'), p.text);
    drawPrologueArt($('#pro-art') as HTMLCanvasElement, p.art);
  }

  private typeTimer = 0;
  private typewrite(el: HTMLElement, text: string) {
    clearInterval(this.typeTimer);
    let i = 0;
    el.textContent = '';
    this.typeTimer = window.setInterval(() => {
      i += 2;
      el.textContent = text.slice(0, i);
      if (i >= text.length) clearInterval(this.typeTimer);
    }, 22);
  }

  loading(label: string) {
    $('#pro-status').textContent = label;
  }

  ready(genMs: number) {
    this.worldReady = true;
    $('#pro-status').textContent = `World ready (${(genMs / 1000).toFixed(1)}s)`;
    if (this.introDone) this.enterWorld();
  }

  private enterWorld() {
    $('#prologue').classList.add('hidden');
    $('#hud').classList.remove('hidden');
    $('#game').classList.add('live');
  }

  // ------------------------------------------------------------ per frame

  frame(snap: Float32Array, ui: any, simMs: number) {
    const minute = snap[H.minute];
    $('#hud-clock').textContent = clock(minute);
    const w = snap[H.weather];
    const night = minute < 330 || minute > 1200;
    const wicon = w === 0 && night ? 'moon' : WEATHER_ICON[w] ?? 'sun';
    const img = $('#hud-wicon') as HTMLImageElement;
    const src = icon(wicon, 3);
    if (img.src !== src) {
      img.src = src;
      img.title = WEATHER_NAME[w];
    }
    $('#hud-gold').textContent = String(snap[H.gold]);
    const now = performance.now();
    if (now - this.lastStatus > 2000) {
      this.lastStatus = now;
      this.onSend({ t: 'status' });
    }
    for (const t of ui.toasts as string[]) this.toast(t);
    for (const e of ui.events as any[]) this.feed(e);
    for (const d of ui.dialogue as any[]) this.showDialogue(d);
    if (ui.season_changed) this.toast(`${SEASON[snap[H.season]]} has arrived`, true);
    const perf = $('#perf');
    if (perf.dataset.on) perf.textContent = `sim ${simMs.toFixed(1)}ms`;
  }

  player(x: number, y: number) {
    this.px = x;
    this.py = y;
  }

  message(m: FromWorker) {
    switch (m.t) {
      case 'status':
        this.status = m.data;
        $('#hud-date').textContent = m.data.date;
        if (this.panelKind === 'bag') this.renderBag();
        break;
      case 'inspect':
        this.renderProfile(m.data);
        break;
      case 'chronicle':
        this.renderChronicle(m.data);
        break;
      case 'villagers':
        this.renderPeople(m.data);
        break;
      case 'portrait': {
        const c = this.drawPortrait(m.px);
        this.portraits.set(m.key, c);
        for (const target of this.pendingPortrait.get(m.key) ?? []) this.copyPortrait(c, target);
        this.pendingPortrait.delete(m.key);
        break;
      }
    }
  }

  // ------------------------------------------------------------ toasts & feed

  toast(text: string, big = false) {
    const el = document.createElement('div');
    el.className = 'toast' + (big ? ' big' : '');
    el.textContent = text;
    $('#toasts').appendChild(el);
    setTimeout(() => el.classList.add('out'), big ? 2600 : 1400);
    setTimeout(() => el.remove(), big ? 3200 : 2000);
  }

  private feed(e: any) {
    const el = document.createElement('div');
    el.className = `feed-item k-${e.kind}`;
    el.innerHTML = `<span class="feed-dot"></span>${esc(e.text)}`;
    if (e.who?.length) {
      el.dataset.vid = e.who[0];
      el.addEventListener('click', () => this.openProfile(e.who[0]));
    }
    const feed = $('#feed');
    feed.prepend(el);
    this.feedCount++;
    while (feed.children.length > 4) feed.lastElementChild?.remove();
    setTimeout(() => el.classList.add('out'), 9000);
    setTimeout(() => el.remove(), 9800);
  }

  thunder() {
    document.body.classList.add('shake');
    setTimeout(() => document.body.classList.remove('shake'), 300);
  }

  // ------------------------------------------------------------ dialogue

  private showDialogue(d: any) {
    this.dialogue = { lines: d.lines, i: 0, data: d };
    $('#dlg-name').textContent = d.name;
    $('#dlg-job').textContent = d.job;
    const hearts = Math.max(0, Math.min(5, Math.round(d.opinion / 20)));
    $('#dlg-heart').innerHTML = Array.from({ length: 5 }, (_, i) => `<img class="heart${i < hearts ? '' : ' off'}" src="${icon('heart', 2)}">`).join('');
    this.portrait(d.sheet, $('#dlg-face') as HTMLCanvasElement);
    $('#dialogue').classList.remove('hidden');
    this.typewrite($('#dlg-text'), d.lines[0] ?? '…');
  }

  private advanceDialogue() {
    if (!this.dialogue) return;
    this.dialogue.i++;
    if (this.dialogue.i >= this.dialogue.lines.length) {
      this.dialogue = null;
      $('#dialogue').classList.add('hidden');
      return;
    }
    this.typewrite($('#dlg-text'), this.dialogue.lines[this.dialogue.i]);
  }

  closeTransient() {
    if (this.dialogue) {
      this.dialogue = null;
      $('#dialogue').classList.add('hidden');
    }
  }

  // ------------------------------------------------------------ portraits

  private portrait(sheet: number, target: HTMLCanvasElement) {
    const key = String(sheet);
    const hit = this.portraits.get(key);
    if (hit) return this.copyPortrait(hit, target);
    const list = this.pendingPortrait.get(key);
    if (list) list.push(target);
    else {
      this.pendingPortrait.set(key, [target]);
      this.onSend({ t: 'portrait', id: sheet, key });
    }
  }

  private drawPortrait(px: Uint8Array): HTMLCanvasElement {
    const w = px[0];
    const h = px[1];
    const c = document.createElement('canvas');
    c.width = w;
    c.height = h;
    if (w && h) c.getContext('2d')!.putImageData(new ImageData(new Uint8ClampedArray(px.slice(2, 2 + w * h * 4)), w, h), 0, 0);
    return c;
  }

  private copyPortrait(src: HTMLCanvasElement, target: HTMLCanvasElement) {
    target.width = src.width;
    target.height = src.height;
    target.getContext('2d')!.drawImage(src, 0, 0);
  }

  // ------------------------------------------------------------ panels

  private panelKind = '';
  private openPanel(kind: string, html: string) {
    this.panelKind = kind;
    $('#panel-card').innerHTML = `<button class="btn close" data-close>${iconImg('close', 2)}</button>${html}`;
    $('#panel').classList.remove('hidden');
  }

  closePanel() {
    this.panelKind = '';
    $('#panel').classList.add('hidden');
  }

  openProfile(id: number) {
    this.openPanel('profile', '<div class="loading-dots">Loading…</div>');
    this.onSend({ t: 'inspect', id });
  }

  private renderProfile(v: any) {
    if (!v || this.panelKind !== 'profile') return;
    const bar = (label: string, val: number) =>
      `<div class="need"><span>${label}</span><div class="bar"><i style="width:${Math.max(3, val)}%;background:${val > 60 ? '#6aa65a' : val > 30 ? '#e0b040' : '#d24c40'}"></i></div></div>`;
    const people = (title: string, list: any[]) =>
      list.length ? `<h4>${title}</h4><div class="chips">${list.map((f) => `<span class="chip link" data-vid="${f.id}">${esc(f.name)} <b>${f.score > 0 ? '+' : ''}${f.score}</b></span>`).join('')}</div>` : '';
    const family = [
      v.partner ? `Married to <b>${esc(v.partner)}</b>` : '',
      v.dating ? `Courting <b>${esc(v.dating)}</b>` : '',
      v.parents.length ? `Child of ${v.parents.map(esc).join(' & ')}` : '',
      v.children.length ? `Parent of ${v.children.map(esc).join(', ')}` : '',
      v.pregnant ? 'Expecting a baby' : '',
    ].filter(Boolean);
    const hearts = Math.max(0, Math.min(5, Math.round(v.opinion / 20)));
    this.openPanel(
      'profile',
      `<div class="profile">
        <div class="prof-top">
          <div class="prof-face"><canvas id="prof-face"></canvas></div>
          <div>
            <h2>${esc(v.name)}</h2>
            <div class="muted">${v.age} yrs · ${esc(v.stage)} · ${esc(v.job)} of ${esc(v.village)}</div>
            <div class="doing">${esc(v.doing)}</div>
            <div class="hearts">${Array.from({ length: 5 }, (_, i) => `<img class="heart${i < hearts ? '' : ' off'}" src="${icon('heart', 2)}">`).join('')}</div>
          </div>
        </div>
        <div class="chips">${v.traits.map((t: string) => `<span class="chip">${esc(t)}</span>`).join('')}</div>
        <div class="needs">${bar('Food', v.needs.hunger)}${bar('Energy', v.needs.energy)}${bar('Social', v.needs.social)}${bar('Fun', v.needs.fun)}</div>
        ${family.length ? `<div class="family">${family.map((f) => `<div>${f}</div>`).join('')}</div>` : ''}
        ${people('Friends', v.friends)}
        ${people('Rivals', v.rivals)}
        ${v.memories.length ? `<h4>Recently</h4><ul class="mem">${v.memories.slice().reverse().map((m: string) => `<li>${esc(m)}</li>`).join('')}</ul>` : ''}
      </div>`,
    );
    this.portrait(v.sheet, $('#prof-face') as HTMLCanvasElement);
  }

  private openChronicle() {
    this.openPanel('chronicle', '<h2>Chronicle</h2><div class="loading-dots">Loading…</div>');
    this.onSend({ t: 'chronicle' });
  }

  private renderChronicle(list: any[]) {
    if (this.panelKind !== 'chronicle') return;
    const items = list
      .slice()
      .reverse()
      .map((e) => {
        const day = e.day;
        const season = SEASON[Math.floor(day / 12) % 4];
        return `<li class="k-${e.kind}" ${e.who?.length ? `data-vid="${e.who[0]}"` : ''}><span class="when">${season} ${(day % 12) + 1}</span>${esc(e.text)}</li>`;
      })
      .join('');
    this.openPanel('chronicle', `<h2>Chronicle</h2><p class="muted">The story of this world, as it happens.</p><ul class="chron">${items || '<li>Nothing has happened yet.</li>'}</ul>`);
  }

  private openPeople() {
    this.openPanel('people', '<h2>People</h2><div class="loading-dots">Loading…</div>');
    this.onSend({ t: 'villagers' });
  }

  private renderPeople(list: any[]) {
    if (this.panelKind !== 'people') return;
    const villages = this.status?.villages ?? [];
    const groups = new Map<number, any[]>();
    for (const v of list) {
      if (!groups.has(v.village)) groups.set(v.village, []);
      groups.get(v.village)!.push(v);
    }
    let html = `<h2>People</h2><p class="muted">${list.length} souls live in this world.</p>`;
    for (const [vid, vs] of groups) {
      html += `<h4>${esc(villages[vid]?.name ?? 'Village')}</h4><div class="people">`;
      for (const v of vs) html += `<button class="person" data-vid="${v.id}"><canvas data-sheet="${v.sheet}" width="28" height="48"></canvas><span>${esc(v.name)}</span><small>${esc(v.job)}</small></button>`;
      html += '</div>';
    }
    this.openPanel('people', html);
    document.querySelectorAll<HTMLCanvasElement>('#panel canvas[data-sheet]').forEach((c) => this.portrait(Number(c.dataset.sheet), c));
  }

  private openMap() {
    let html = '<h2>World Map</h2><div class="map-wrap"><canvas id="map-canvas"></canvas>';
    for (const v of this.status?.villages ?? []) {
      html += `<div class="map-label" style="left:${(v.x / this.worldW) * 100}%;top:${(v.y / this.worldH) * 100}%">${esc(v.name)}<small>${v.pop} people</small></div>`;
    }
    html += `<div class="map-you" style="left:${(this.px / 32 / this.worldW) * 100}%;top:${(this.py / 32 / this.worldH) * 100}%"></div></div>`;
    this.openPanel('map', html);
    const c = $('#map-canvas') as HTMLCanvasElement;
    if (this.minimap) {
      c.width = this.minimap.width;
      c.height = this.minimap.height;
      c.getContext('2d')!.drawImage(this.minimap, 0, 0);
    }
  }

  private openBag() {
    this.panelKind = 'bag';
    this.renderBag();
    this.onSend({ t: 'status' });
  }

  private renderBag() {
    const inv: [string, number][] = this.status?.inventory ?? [];
    const items = inv.map(([k, n]) => `<div class="item"><b>${n}</b><span>${esc(k)}</span></div>`).join('');
    this.openPanel(
      'bag',
      `<h2>Bag</h2><p class="muted">Tap trees, rocks, bushes and mushrooms to gather.</p><div class="items">${items || '<div class="muted">Empty for now.</div>'}</div>
       <h4>Villages</h4>${(this.status?.villages ?? []).map((v: any) => `<div class="vrow"><b>${esc(v.name)}</b><span>${v.pop} people · food ${Math.round(v.stock?.food ?? 0)} · wood ${Math.round(v.stock?.wood ?? 0)}</span></div>`).join('')}`,
    );
  }

  setMinimap(px: Uint8Array, w: number, h: number) {
    this.worldW = w;
    this.worldH = h;
    const c = document.createElement('canvas');
    c.width = w;
    c.height = h;
    c.getContext('2d')!.putImageData(new ImageData(new Uint8ClampedArray(px), w, h), 0, 0);
    this.minimap = c;
  }

  private cycleSpeed() {
    this.speedIdx = (this.speedIdx + 1) % SPEEDS.length;
    const v = SPEEDS[this.speedIdx];
    this.onCmd({ t: 'speed', v });
    $('#speed-label').textContent = `${v}×`;
  }

  key(k: string) {
    if (k === 'escape') {
      this.closePanel();
      this.closeTransient();
    } else if (k === 'm') this.panelKind === 'map' ? this.closePanel() : this.openMap();
    else if (k === 'c') this.panelKind === 'chronicle' ? this.closePanel() : this.openChronicle();
    else if (k === 'p') this.panelKind === 'people' ? this.closePanel() : this.openPeople();
    else if (k === 'b' || k === 'i') this.panelKind === 'bag' ? this.closePanel() : this.openBag();
    else if (k === ' ' || k === 'enter' || k === 'e') this.advanceDialogue();
    else if (k === 'f3') {
      const p = $('#perf');
      p.dataset.on = p.dataset.on ? '' : '1';
      if (!p.dataset.on) p.textContent = '';
    }
  }
}

// Little pixel vignettes for the prologue, drawn procedurally.
function drawPrologueArt(c: HTMLCanvasElement, kind: string) {
  const g = c.getContext('2d')!;
  const W = c.width;
  const Ht = c.height;
  const rect = (x: number, y: number, w: number, h: number, col: string) => {
    g.fillStyle = col;
    g.fillRect(x, y, w, h);
  };
  let seed = 7;
  const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
  if (kind === 'city') {
    rect(0, 0, W, Ht, '#1a1838');
    for (let i = 0; i < 40; i++) rect(Math.floor(rnd() * W), Math.floor(rnd() * 40), 1, 1, '#c8c8f0');
    for (let x = 0; x < W; x += 14) {
      const h = 25 + Math.floor(rnd() * 40);
      rect(x, Ht - h - 10, 12, h, '#2a2850');
      for (let y = Ht - h - 6; y < Ht - 14; y += 5) for (let wx = x + 2; wx < x + 11; wx += 3) if (rnd() < 0.4) rect(wx, y, 1, 2, '#f0c060');
    }
    rect(0, Ht - 10, W, 10, '#3a3448');
    rect(70, Ht - 22, 4, 12, '#5a5a6a');
    rect(69, Ht - 26, 6, 5, '#f0d090');
    rect(30, Ht - 18, 3, 8, '#e8d8c8');
    rect(30, Ht - 21, 3, 3, '#2a2030');
  } else if (kind === 'truck') {
    rect(0, 0, W, Ht, '#fdf8ec');
    g.fillStyle = 'rgba(255,240,200,0.9)';
    g.beginPath();
    g.moveTo(W, Ht / 2 - 10);
    g.lineTo(40, Ht / 2 - 30);
    g.lineTo(40, Ht / 2 + 30);
    g.lineTo(W, Ht / 2 + 10);
    g.fill();
    rect(110, 30, 44, 34, '#d04040');
    rect(114, 34, 16, 12, '#a8d0f0');
    rect(110, 64, 44, 6, '#3a3448');
    rect(116, 68, 8, 8, '#1a1424');
    rect(140, 68, 8, 8, '#1a1424');
    rect(152, 52, 4, 6, '#fff4c0');
  } else if (kind === 'goddess') {
    const grad = g.createLinearGradient(0, 0, 0, Ht);
    grad.addColorStop(0, '#f8e8ff');
    grad.addColorStop(1, '#c8d8ff');
    g.fillStyle = grad;
    g.fillRect(0, 0, W, Ht);
    for (let i = 0; i < 30; i++) rect(Math.floor(rnd() * W), Math.floor(rnd() * Ht), 1, 1, '#ffffff');
    rect(72, 18, 16, 6, '#f6d870');
    rect(70, 20, 2, 2, '#f6d870');
    rect(88, 20, 2, 2, '#f6d870');
    rect(74, 26, 12, 12, '#f6dcc0');
    rect(72, 24, 16, 6, '#f0e8ff');
    rect(70, 26, 3, 24, '#f0e8ff');
    rect(87, 26, 3, 24, '#f0e8ff');
    rect(77, 31, 2, 2, '#5a6ad0');
    rect(82, 31, 2, 2, '#5a6ad0');
    for (let k = 0; k < 40; k++) rect(80 - 6 - Math.floor(k / 3), 38 + k, 12 + Math.floor((k / 3) * 2), 1, k % 7 === 0 ? '#e0d0f8' : '#ffffff');
  } else {
    rect(0, 0, W, Ht, '#8cc8f0');
    rect(0, 50, W, 40, '#5c9b44');
    rect(0, 48, W, 4, '#78a845');
    for (let i = 0; i < 60; i++) rect(Math.floor(rnd() * W), 52 + Math.floor(rnd() * 38), 1, 2, '#3c7c40');
    for (let i = 0; i < 14; i++) rect(Math.floor(rnd() * W), 55 + Math.floor(rnd() * 30), 1, 1, ['#f6d860', '#f4a6c0', '#ffffff'][i % 3]);
    rect(110, 34, 20, 14, '#cbbfb5');
    rect(108, 28, 24, 7, '#ad4a40');
    rect(116, 40, 4, 8, '#6b412f');
    rect(135, 36, 14, 12, '#d6cabb');
    rect(133, 31, 18, 6, '#3f6496');
    rect(20, 20, 26, 6, '#ffffff');
    rect(26, 16, 14, 4, '#ffffff');
  }
}
