import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri expects a fixed port in dev and a static build in dist/.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    target: 'chrome120',
    outDir: 'dist',
    cssMinify: true,
    // Only latin subsets of each font are bundled (see src/lib/fonts.ts), so assets stay small.
    assetsInlineLimit: 0,
    reportCompressedSize: false,
  },
});
