import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  build: {
    sourcemap: false,
    minify: 'esbuild'
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    maxHeaderSize: 16384,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  }
});
