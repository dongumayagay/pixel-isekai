import { defineConfig } from 'vite';

export default defineConfig({
  root: 'web',
  base: './',
  publicDir: false,
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'es2022',
    assetsInlineLimit: 0,
  },
  worker: { format: 'es' },
  server: { host: true, port: 5173 },
});
