// Tiny pixel-art icons drawn from character maps, rendered to data URLs so
// the HTML UI matches the game's look.

const PAL: Record<string, string> = {
  k: '#2a2238',
  w: '#fdf8ec',
  y: '#f6d870',
  Y: '#dca23a',
  o: '#e0802a',
  r: '#d24c40',
  b: '#5e8abb',
  B: '#a8d0f0',
  g: '#6aa65a',
  G: '#2e6a46',
  s: '#b3c4dc',
  S: '#eef3f9',
  n: '#8c5a38',
  N: '#5c3626',
  p: '#c87ac0',
  c: '#c8d4e4',
  d: '#7e8ca8',
};

const ICONS = {
  sun: ['....y....', '.y..y..y.', '..yYYYy..', '..YyyyY..', 'yyYyyyYyy', '..YyyyY..', '..yYYYy..', '.y..y..y.', '....y....'],
  moon: ['...SSS...', '..SScc...', '.SSc.....', '.Sc......', '.Sc......', '.SSc.....', '..SScc..S', '...SSSSS.', '.........'],
  cloud: ['.........', '...www...', '..wwwww..', '.wwwwwwcw', 'wwwwwwwcc', 'cwwwwwccc', '.cccccccc', '.........', '.........'],
  rain: ['...www...', '..wwwww..', '.wwwwwwcw', 'cccccccc.', '.b..b..b.', 'b..b..b..', '.b..b..b.', '.........', '.........'],
  storm: ['...ddd...', '..ddddd..', '.ddddddd.', 'dddddddd.', '...yy....', '..yy.....', '...yy....', '....y....', '.........'],
  snow: ['....S....', '.S..S..S.', '..S.S.S..', '...SSS...', 'SSSSSSSSS', '...SSS...', '..S.S.S..', '.S..S..S.', '....S....'],
  fog: ['.........', 'ccccccc..', '.........', '..ccccccc', '.........', 'ccccccc..', '.........', '..ccccccc', '.........'],
  coin: ['..YYYY..', '.YyyyyY.', 'YyyYYyyY', 'YyYyyyyY', 'YyYyyyyY', 'YyyYYyyY', '.YyyyyY.', '..YYYY..'],
  book: ['.NNNNNN.', 'NwwwnwwN', 'NwkwnwkN', 'NwwwnwwN', 'NwkwnwkN', 'NwwwnwwN', 'NNNNNNNN', '........'],
  people: ['..nn..nn.', '.nyyn.nyy', '.yyyy.yyy', '..yy...yy', '.rrrr.bbb', 'rrrrrrbbb', 'rr.rrrbbb', '.........'],
  map: ['GGgggbbb.', 'GgggbbbB.', 'ggggbbBB.', 'gggnbBBB.', 'ggnnggBB.', 'gnngggGB.', 'nngggGGG.', '.........'],
  fast: ['.........', 'k...k....', 'kk..kk...', 'kkk.kkk..', 'kkkkkkkk.', 'kkk.kkk..', 'kk..kk...', 'k...k....', '.........'],
  heart: ['.rr.rr.', 'rrrrrrr', 'rrrrrrr', '.rrrrr.', '..rrr..', '...r...'],
  bag: ['..NNN...', '.N...N..', 'nnnnnnn.', 'nyyyyyn.', 'nnnnnnn.', 'nnnnnnn.', '.nnnnn..', '........'],
  close: ['k.....k', '.k...k.', '..k.k..', '...k...', '..k.k..', '.k...k.', 'k.....k'],
} satisfies Record<string, string[]>;

export type IconName = keyof typeof ICONS;

const cache = new Map<string, string>();

/** Data URL of an icon, each map cell drawn as a `scale`-px square. */
export function icon(name: IconName, scale: number): string {
  const key = `${name}@${scale}`;
  const hit = cache.get(key);
  if (hit) return hit;
  const rows = ICONS[name];
  const c = document.createElement('canvas');
  c.width = rows[0].length * scale;
  c.height = rows.length * scale;
  const g = c.getContext('2d')!;
  rows.forEach((row, y) => {
    [...row].forEach((ch, x) => {
      const col = PAL[ch];
      if (!col) return;
      g.fillStyle = col;
      g.fillRect(x * scale, y * scale, scale, scale);
    });
  });
  const url = c.toDataURL();
  cache.set(key, url);
  return url;
}

export function iconImg(name: IconName, scale: number): string {
  return `<img class="ico" src="${icon(name, scale)}" alt="" draggable="false">`;
}
