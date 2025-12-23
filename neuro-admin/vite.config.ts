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
        target: 'http://0.0.0.0:3000',
        changeOrigin: true,
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
