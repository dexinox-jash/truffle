import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  
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
  
  test: {
    /* Test environment */
    environment: 'jsdom',
    
    /* Setup files */
    setupFiles: ['./tests/setup.ts'],
    
    /* Global test configuration */
    globals: true,
    
    /* Include test files */
    include: [
      'tests/**/*.{test,spec}.{ts,tsx}',
      'src/**/*.{test,spec}.{ts,tsx}',
    ],
    
    /* Exclude patterns */
    exclude: [
      'node_modules',
      'dist',
      'src-tauri',
      'e2e',
    ],
    
    /* Coverage configuration */
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 75,
        statements: 80,
      },
      exclude: [
        'node_modules/',
        'tests/',
        '**/*.d.ts',
        '**/*.config.{ts,js}',
        'src-tauri/',
        'e2e/',
      ],
    },
    
    /* Mock configuration */
    mockReset: true,
    restoreMocks: true,
    clearMocks: true,
    
    /* Reporter configuration */
    reporters: [
      'default',
      ['junit', { outputFile: './test-results/junit.xml' }],
    ],
    
    /* Output directory */
    outputFile: './test-results/test-output.json',
    
    /* Watch mode configuration */
    watchExclude: [
      'node_modules/**',
      'dist/**',
      'src-tauri/**',
      '.git/**',
    ],
    
    /* Timeout configuration */
    testTimeout: 10000,
    hookTimeout: 10000,
    
    /* Pool configuration for parallel execution */
    pool: 'forks',
    poolOptions: {
      forks: {
        singleFork: false,
      },
    },
    
    /* Isolate tests */
    isolate: true,
    
    /* Retry failed tests */
    retry: process.env.CI ? 2 : 0,
    
    /* Bail on first failure in CI */
    bail: process.env.CI ? 1 : 0,
  },
  
  /* CSS handling */
  css: {
    modules: {
      localsConvention: 'camelCase',
    },
  },
  
  /* ESBuild configuration */
  esbuild: {
    jsxInject: `import React from 'react'`,
    target: 'es2020',
  },
});
