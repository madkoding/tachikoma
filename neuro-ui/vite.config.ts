import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0',
    port: 5173,
    allowedHosts: ['tachikoma', 'localhost'],
    proxy: {
      // Music streaming endpoint needs special handling
      '/api/music/stream': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        secure: false,
        // Disable buffering for streaming
        configure: (proxy) => {
          proxy.on('proxyRes', (proxyRes) => {
            // Don't buffer the response
            proxyRes.headers['cache-control'] = 'no-cache, no-store, must-revalidate';
          });
        },
      },
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        secure: false,
        ws: true,
      },
      '/voice': {
        target: 'http://localhost:8100',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/voice/, ''),
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
})
