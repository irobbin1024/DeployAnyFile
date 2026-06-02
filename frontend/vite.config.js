import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// In dev, proxy API and raw file routes to the Rust backend.
export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5173,
    proxy: {
      '/api': 'http://localhost:8080',
      '/raw': 'http://localhost:8080',
    },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
})
