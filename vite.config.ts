import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri expects a fixed port in dev and a static build in dist/.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  // Rust's build output lives inside the project: a release build next to `tauri dev` would
  // otherwise reload every open window mid-build (seen as a blank settings window).
  server: { port: 1420, strictPort: true, watch: { ignored: ['**/src-tauri/target/**'] } },
  build: {
    target: 'chrome120',
    outDir: 'dist',
    cssMinify: true,
    // Only latin subsets of each font are bundled (see src/lib/fonts.ts), so assets stay small.
    assetsInlineLimit: 0,
    reportCompressedSize: false,
  },
});
