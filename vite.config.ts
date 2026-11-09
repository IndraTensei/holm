import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import pkg from './package.json' with { type: 'json' };

// Tauri expects a fixed port in dev and a static build in dist/.
export default defineConfig({
  plugins: [svelte()],
  // The app's version, from package.json, so the UI never has to be edited for a release.
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
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
