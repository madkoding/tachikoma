import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0',
    port: 5174,
    allowedHosts: ['tachikoma', 'localhost'],
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        secure: false,
        ws: true,
        // Configuración especial para Server-Sent Events (SSE)
        configure: (proxy, _options) => {
          proxy.on('proxyReq', (proxyReq, req, _res) => {
            // SSE necesita Accept: text/event-stream
            if (req.url?.includes('/admin/graph/events')) {
              proxyReq.setHeader('Accept', 'text/event-stream');
              proxyReq.setHeader('Cache-Control', 'no-cache');
              proxyReq.setHeader('Connection', 'keep-alive');
            }
          });
          proxy.on('proxyRes', (proxyRes, req, _res) => {
            // Asegurar headers correctos para SSE
            if (req.url?.includes('/admin/graph/events')) {
              proxyRes.headers['content-type'] = 'text/event-stream';
              proxyRes.headers['cache-control'] = 'no-cache';
              proxyRes.headers['connection'] = 'keep-alive';
            }
          });
        },
      },
    },
  },
  resolve: {
    dedupe: ['three'],
  },
  optimizeDeps: {
    include: ['three', 'react-force-graph-3d'],
    esbuildOptions: {
      target: 'esnext',
    },
  },
});
