import { FullConfig } from '@playwright/test';

/**
 * Global teardown for Playwright E2E tests
 * Runs once after all tests
 */
async function globalTeardown(config: FullConfig) {
  console.log('🧹 Starting E2E test teardown...');
  
  // Clean up test data, close connections, etc.
  // Example:
  // await cleanupTestDatabase();
  // await closeTestConnections();
  
  console.log('✅ E2E test teardown complete');
}

export default globalTeardown;
