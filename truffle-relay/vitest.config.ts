import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'miniflare',
    environmentOptions: {
      bindings: {
        ENVIRONMENT: 'test',
        PROTOCOL_VERSION: '1',
        RATE_LIMIT_FREE: '100',
        RATE_LIMIT_PAID: '10000',
        DATA_RETENTION_DAYS: '30',
        MAX_BLOB_SIZE_MB: '10',
        WEBSOCKET_IDLE_TIMEOUT_MS: '300000',
        JWT_SECRET: 'test-secret',
      },
      kvNamespaces: {
        RATE_LIMIT_KV: 'truffle-rate-limit-test',
        DEVICE_SESSIONS: 'truffle-device-sessions-test',
        WEBSOCKET_STATE: 'truffle-websocket-state-test',
      },
      r2Buckets: ['SYNC_BLOBS', 'AUDIT_LOGS'],
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'dist/',
        'tests/',
        '**/*.d.ts',
        '**/*.config.*',
      ],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 70,
        statements: 80,
      },
    },
  },
});
