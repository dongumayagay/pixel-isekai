// Hot dev loop: fast WASM build on every Rust save, Vite reloads the page.
// Usage: node scripts/dev.mjs [--port N] [--open]
import { spawn, spawnSync } from 'node:child_process';
import { watch } from 'node:fs';

const args = process.argv.slice(2);
const port = args.includes('--port') ? args[args.indexOf('--port') + 1] : '5173';
const cargoEnv = { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` };

function build() {
  const t = Date.now();
  const c = spawnSync('cargo', ['build', '--profile', 'wasm-dev', '--target', 'wasm32-unknown-unknown', '-p', 'sim', '--color', 'always'], { env: cargoEnv, stdio: ['ignore', 'inherit', 'pipe'] });
  if (c.status !== 0) {
    process.stderr.write(c.stderr);
    console.log('\x1b[31m✗ Rust build failed — page keeps the last good build\x1b[0m');
    return;
  }
  const b = spawnSync('wasm-bindgen', ['--target', 'web', '--out-dir', 'web/src/pkg', 'target/wasm32-unknown-unknown/wasm-dev/sim.wasm'], { env: cargoEnv, stdio: 'inherit' });
  if (b.status === 0) console.log(`\x1b[32m✓ wasm rebuilt in ${Date.now() - t} ms\x1b[0m`);
}

build();
const oi = args.indexOf('--open');
const open = oi < 0 ? [] : args[oi + 1] && !args[oi + 1].startsWith('--') ? ['--open', args[oi + 1]] : ['--open'];
const vite = spawn('npx', ['vite', '--port', port, '--strictPort', ...open], { stdio: 'inherit' });

let timer = null;
let busy = false;
let again = false;
const queue = () => {
  clearTimeout(timer);
  timer = setTimeout(() => {
    if (busy) return void (again = true);
    busy = true;
    build();
    busy = false;
    if (again) {
      again = false;
      queue();
    }
  }, 120);
};
watch('sim/src', { recursive: true }, (_e, f) => f && f.endsWith('.rs') && queue());
console.log('watching sim/src for Rust changes…');

const stop = () => {
  vite.kill();
  process.exit(0);
};
process.on('SIGINT', stop);
process.on('SIGTERM', stop);
