import { FullConfig } from '@playwright/test';

/**
 * Global setup for Playwright E2E tests
 * Runs once before all tests
 */
async function globalSetup(config: FullConfig) {
  console.log('🚀 Starting E2E test setup...');
  
  // Set environment variables for testing
  process.env.NODE_ENV = 'test';
  process.env.VITE_API_URL = process.env.VITE_API_URL || 'http://localhost:3000';
  
  // You can add database setup, seed data, etc. here
  // Example:
  // await setupTestDatabase();
  // await seedTestData();
  
  console.log('✅ E2E test setup complete');
}

export default globalSetup;
