import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { VitePWA } from 'vite-plugin-pwa'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['logo.svg'],
      manifest: {
        name: 'Tachikoma - AI Assistant',
        short_name: 'Tachikoma',
        description: 'AI Assistant with Memory',
        theme_color: '#0a0a0f',
        background_color: '#0a0a0f',
        display: 'standalone',
        orientation: 'portrait',
        scope: '/',
        start_url: '/',
        icons: [
          {
            src: 'pwa-192x192.png',
            sizes: '192x192',
            type: 'image/png'
          },
          {
            src: 'pwa-512x512.png',
            sizes: '512x512',
            type: 'image/png'
          },
          {
            src: 'pwa-512x512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'any maskable'
          }
        ]
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}'],
        runtimeCaching: [
          {
            urlPattern: /^https:\/\/fonts\.googleapis\.com\/.*/i,
            handler: 'CacheFirst',
            options: {
              cacheName: 'google-fonts-cache',
              expiration: {
                maxEntries: 10,
                maxAgeSeconds: 60 * 60 * 24 * 365 // 1 year
              },
              cacheableResponse: {
                statuses: [0, 200]
              }
            }
          },
          {
            urlPattern: /^https:\/\/fonts\.gstatic\.com\/.*/i,
            handler: 'CacheFirst',
            options: {
              cacheName: 'gstatic-fonts-cache',
              expiration: {
                maxEntries: 10,
                maxAgeSeconds: 60 * 60 * 24 * 365 // 1 year
              },
              cacheableResponse: {
                statuses: [0, 200]
              }
            }
          }
        ]
      }
    })
  ],
  server: {
    host: '0.0.0.0',
    port: 5173,
    allowedHosts: ['tachikoma', 'localhost'],
    // Tauri espera que el servidor esté en localhost:5173
    strictPort: true,
    proxy: {
      // Chat streaming endpoint needs special handling for SSE
      '/api/chat/stream': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        secure: false,
        // Disable buffering for SSE streaming
        configure: (proxy) => {
          proxy.on('proxyRes', (proxyRes) => {
            proxyRes.headers['cache-control'] = 'no-cache, no-store, must-revalidate';
            proxyRes.headers['x-accel-buffering'] = 'no';
          });
        },
      },
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
      // Music SSE events endpoint needs special handling
      '/api/music/events': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        secure: false,
        // Disable buffering for SSE streaming
        configure: (proxy) => {
          proxy.on('proxyRes', (proxyRes) => {
            proxyRes.headers['cache-control'] = 'no-cache, no-store, must-revalidate';
            proxyRes.headers['x-accel-buffering'] = 'no';
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
  // Prevenir limpieza del outDir por Tauri
  clearScreen: false,
})
