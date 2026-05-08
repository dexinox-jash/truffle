import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  
  // Path aliases
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@components': path.resolve(__dirname, './src/components'),
      '@store': path.resolve(__dirname, './src/store'),
      '@hooks': path.resolve(__dirname, './src/hooks'),
      '@utils': path.resolve(__dirname, './src/utils'),
      '@types': path.resolve(__dirname, './src/types'),
      '@styles': path.resolve(__dirname, './src/styles'),
    },
  },
  
  // CSS configuration
  css: {
    devSourcemap: true,
    postcss: {
      plugins: [
        require('tailwindcss'),
        require('autoprefixer'),
      ],
    },
  },
  
  // Build configuration
  build: {
    target: 'es2020',
    outDir: 'dist',
    sourcemap: true,
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
      },
    },
    rollupOptions: {
      output: {
        manualChunks: {
          // Split vendor chunks for better caching
          'vendor-react': ['react', 'react-dom'],
          'vendor-query': ['@tanstack/react-query'],
          'vendor-state': ['zustand'],
          'vendor-editor': ['@milkdown/core', '@milkdown/react'],
          'vendor-search': ['flexsearch'],
          'vendor-sync': ['yjs'],
          'vendor-ui': ['lucide-react', 'framer-motion'],
        },
      },
    },
  },
  
  // Development server
  server: {
    port: 5173,
    strictPort: true,
    host: true,
    hmr: {
      overlay: true,
    },
  },
  
  // Preview server
  preview: {
    port: 4173,
    strictPort: true,
  },
  
  // Environment variables
  envPrefix: 'VITE_',
  
  // Optimize dependencies
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      '@tanstack/react-query',
      'zustand',
      'lucide-react',
      'date-fns',
      'flexsearch',
    ],
    exclude: [
      // Exclude heavy dependencies that should be lazy loaded
      '@milkdown/core',
      '@milkdown/react',
      'yjs',
    ],
  },
  
  // ESBuild configuration
  esbuild: {
    jsxInject: `import React from 'react'`,
    target: 'es2020',
  },
});
