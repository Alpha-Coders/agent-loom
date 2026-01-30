import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],

  // Prevent vite from obscuring Rust errors
  clearScreen: false,

  // Port is set dynamically by scripts/dev.js to avoid conflicts
  server: {
    port: parseInt(process.env.VITE_DEV_PORT || '5173', 10),
    strictPort: true,
    watch: {
      // Tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },

  // Produce sourcemaps for error messages
  build: {
    sourcemap: true,
  },
});
