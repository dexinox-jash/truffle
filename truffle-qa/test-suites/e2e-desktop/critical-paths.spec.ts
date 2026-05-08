/**
 * @fileoverview Desktop E2E Tests - Critical User Paths
 * @module tests/e2e-desktop/critical-paths
 * 
 * Validates critical user journeys using Playwright.
 * 
 * @requires @playwright/test
 */

import { test, expect, Page, BrowserContext } from '@playwright/test';
import * as path from 'path';
import * as fs from 'fs';

// Test configuration
const TEST_TIMEOUT = 120000; // 2 minutes for AI compilation
const APP_URL = 'app://localhost';

test.describe.configure({ mode: 'serial' });

test.describe('Critical User Paths', () => {
  let page: Page;
  let context: BrowserContext;

  test.beforeAll(async ({ browser }) => {
    context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
    });
    page = await context.newPage();
  });

  test.afterAll(async () => {
    await context.close();
  });

  test.beforeEach(async () => {
    // Navigate to app before each test
    await page.goto(APP_URL);
    await page.waitForLoadState('networkidle');
  });

  /**
   * Test: User can capture and compile a screenshot
   */
  test('user can capture and compile a screenshot', async () => {
    test.setTimeout(TEST_TIMEOUT);

    // Navigate to Raw view
    await page.click('[data-testid="raw-tab"]');
    await expect(page.locator('[data-testid="raw-grid"]')).toBeVisible();

    // Upload test screenshot
    const testImage = path.join(__dirname, '../../test-assets/images/receipts/starbucks_01.png');
    await page.setInputFiles('[data-testid="drop-zone"]', testImage);

    // Wait for compilation to complete
    await page.waitForSelector('[data-testid="compilation-complete"]', {
      timeout: 60000,
    });

    // Verify wiki node created
    await page.click('[data-testid="wiki-tab"]');
    const node = page.locator('[data-testid="wiki-node"]').first();
    await expect(node).toBeVisible();

    // Verify node has content
    await node.click();
    await expect(page.locator('[data-testid="node-content"]')).toContainText('Starbucks', {
      ignoreCase: true,
    });
  });

  /**
   * Test: User can search across wiki
   */
  test('user can search across wiki', async () => {
    // Open command palette
    await page.keyboard.press('Control+k');
    
    // Type search query
    await page.fill('[data-testid="search-input"]', 'Starbucks');
    
    // Wait for results
    await page.waitForSelector('[data-testid="search-result"]');
    
    // Verify results appear
    const results = page.locator('[data-testid="search-result"]');
    await expect(results).toHaveCount.greaterThan(0);
    
    // Click first result
    await results.first().click();
    
    // Verify navigation to node
    await expect(page.locator('[data-testid="node-detail"]')).toBeVisible();
  });

  /**
   * Test: User can export to Obsidian
   */
  test('user can export to Obsidian', async () => {
    // Trigger export via keyboard shortcut
    await page.keyboard.press('Control+Shift+e');
    
    // Select Obsidian export
    await page.click('[data-testid="export-obsidian"]');
    
    // Wait for export to complete
    await page.waitForSelector('[data-testid="export-complete"]', {
      timeout: 30000,
    });
    
    // Verify export path shown
    const exportPath = await page.inputValue('[data-testid="export-path"]');
    expect(exportPath).toContain('.md');
    
    // Verify markdown files exist (if accessible)
    if (fs.existsSync(exportPath)) {
      const files = fs.readdirSync(exportPath);
      const markdownFiles = files.filter(f => f.endsWith('.md'));
      expect(markdownFiles.length).toBeGreaterThan(0);
    }
  });

  /**
   * Test: Red Line - Export completes within 5 minutes
   */
  test('Red Line: export completes within 5 minutes', async () => {
    test.setTimeout(5 * 60 * 1000); // 5 minutes
    
    const startTime = Date.now();
    
    // Trigger export
    await page.keyboard.press('Control+Shift+e');
    await page.click('[data-testid="export-all"]');
    
    // Wait for completion
    await page.waitForSelector('[data-testid="export-complete"]');
    
    const elapsed = Date.now() - startTime;
    
    // Verify under 5 minutes
    expect(elapsed).toBeLessThan(5 * 60 * 1000);
    
    console.log(`Export completed in ${elapsed / 1000} seconds`);
  });

  /**
   * Test: User can navigate between raw and wiki views
   */
  test('user can navigate between views', async () => {
    // Start at raw view
    await page.click('[data-testid="raw-tab"]');
    await expect(page.locator('[data-testid="raw-grid"]')).toBeVisible();
    
    // Switch to wiki view
    await page.click('[data-testid="wiki-tab"]');
    await expect(page.locator('[data-testid="wiki-navigator"]')).toBeVisible();
    
    // Switch to compilation view
    await page.click('[data-testid="compilation-tab"]');
    await expect(page.locator('[data-testid="compilation-preview"]')).toBeVisible();
    
    // Switch back to raw
    await page.click('[data-testid="raw-tab"]');
    await expect(page.locator('[data-testid="raw-grid"]')).toBeVisible();
  });

  /**
   * Test: User can edit wiki node
   */
  test('user can edit wiki node', async () => {
    // Navigate to wiki
    await page.click('[data-testid="wiki-tab"]');
    
    // Click first node
    await page.locator('[data-testid="wiki-node"]').first().click();
    
    // Wait for editor
    await expect(page.locator('[data-testid="editor"]')).toBeVisible();
    
    // Edit content
    const newContent = 'Updated content ' + Date.now();
    await page.fill('[data-testid="editor"] textarea', newContent);
    
    // Save
    await page.keyboard.press('Control+s');
    
    // Verify save indicator
    await expect(page.locator('[data-testid="save-indicator"]')).toContainText('Saved');
    
    // Reload and verify
    await page.reload();
    await page.click('[data-testid="wiki-tab"]');
    await page.locator('[data-testid="wiki-node"]').first().click();
    
    // Verify content persisted
    const editorContent = await page.inputValue('[data-testid="editor"] textarea');
    expect(editorContent).toContain(newContent);
  });

  /**
   * Test: User can use command palette
   */
  test('user can use command palette', async () => {
    // Open command palette
    await page.keyboard.press('Control+k');
    
    // Verify palette opens
    await expect(page.locator('[data-testid="command-palette"]')).toBeVisible();
    
    // Type command
    await page.fill('[data-testid="command-input"]', 'compile');
    
    // Verify command appears
    await expect(page.locator('[data-testid="command-item"]')).toContainText('Compile');
    
    // Execute command
    await page.keyboard.press('Enter');
    
    // Verify action triggered
    await expect(page.locator('[data-testid="compilation-status"]')).toBeVisible();
  });

  /**
   * Test: User can view compilation logs
   */
  test('user can view compilation logs', async () => {
    // Navigate to compilation view
    await page.click('[data-testid="compilation-tab"]');
    
    // Upload image
    const testImage = path.join(__dirname, '../../test-assets/images/receipts/starbucks_01.png');
    await page.setInputFiles('[data-testid="drop-zone"]', testImage);
    
    // Wait for compilation
    await page.waitForSelector('[data-testid="compilation-complete"]');
    
    // Expand logs
    await page.click('[data-testid="expand-logs"]');
    
    // Verify logs visible
    await expect(page.locator('[data-testid="compilation-logs"]')).toBeVisible();
    
    // Verify log content
    const logs = await page.locator('[data-testid="compilation-logs"]').textContent();
    expect(logs).toContain('Gemma');
  });
});

test.describe('Offline Functionality', () => {
  let page: Page;
  let context: BrowserContext;

  test.beforeAll(async ({ browser }) => {
    context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
    });
    page = await context.newPage();
  });

  test.afterAll(async () => {
    await context.close();
  });

  /**
   * Test: Red Line - App functions 100% offline
   */
  test('Red Line: app functions offline', async () => {
    // Go offline
    await context.setOffline(true);
    
    // Navigate to app
    await page.goto(APP_URL);
    
    // Verify raw view works
    await page.click('[data-testid="raw-tab"]');
    await expect(page.locator('[data-testid="raw-grid"]')).toBeVisible();
    
    // Verify wiki view works
    await page.click('[data-testid="wiki-tab"]');
    await expect(page.locator('[data-testid="wiki-navigator"]')).toBeVisible();
    
    // Verify search works locally
    await page.keyboard.press('Control+k');
    await page.fill('[data-testid="search-input"]', 'test');
    await expect(page.locator('[data-testid="search-result"]')).toBeVisible();
    
    // Verify export works
    await page.keyboard.press('Control+Shift+e');
    await expect(page.locator('[data-testid="export-dialog"]')).toBeVisible();
  });

  /**
   * Test: Sync resumes when coming back online
   */
  test('sync resumes when coming back online', async () => {
    // Start offline
    await context.setOffline(true);
    await page.goto(APP_URL);
    
    // Make changes offline
    await page.click('[data-testid="wiki-tab"]');
    await page.locator('[data-testid="wiki-node"]').first().click();
    await page.fill('[data-testid="editor"] textarea', 'Offline edit ' + Date.now());
    await page.keyboard.press('Control+s');
    
    // Come back online
    await context.setOffline(false);
    
    // Trigger sync
    await page.click('[data-testid="sync-now"]');
    
    // Verify sync indicator
    await page.waitForSelector('[data-testid="sync-syncing"]');
    await page.waitForSelector('[data-testid="sync-complete"]');
  });
});

test.describe('Security Verification', () => {
  let page: Page;
  let context: BrowserContext;

  test.beforeAll(async ({ browser }) => {
    context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
    });
    page = await context.newPage();
  });

  test.afterAll(async () => {
    await context.close();
  });

  /**
   * Test: Red Line - No plaintext data in network requests
   */
  test('Red Line: no plaintext data in network requests', async () => {
    const requests: Array<{ url: string; postData?: string }> = [];
    
    // Capture network requests
    context.on('request', request => {
      requests.push({
        url: request.url(),
        postData: request.postData() || undefined,
      });
    });
    
    // Navigate and trigger sync
    await page.goto(APP_URL);
    await page.click('[data-testid="sync-now"]');
    await page.waitForTimeout(2000);
    
    // Verify no plaintext in sync requests
    for (const request of requests) {
      if (request.url.includes('sync') || request.url.includes('relay')) {
        if (request.postData) {
          // Check if data is encrypted (not valid JSON/UTF-8 readable)
          const isPlaintext = isValidUTF8(request.postData) && 
                             isValidJSON(request.postData);
          expect(isPlaintext).toBe(false);
        }
      }
    }
  });

  /**
   * Test: Device pairing ceremony works end-to-end
   */
  test('device pairing ceremony works end-to-end', async () => {
    await page.goto(APP_URL);
    
    // Navigate to settings
    await page.click('[data-testid="settings"]');
    
    // Start pairing
    await page.click('[data-testid="add-device"]');
    
    // Verify QR code displayed
    await expect(page.locator('[data-testid="pairing-qr"]')).toBeVisible();
    
    // Verify fingerprint shown
    const fingerprint = await page.textContent('[data-testid="device-fingerprint"]');
    expect(fingerprint).toMatch(/^[A-F0-9]{16}$/);
    
    // Verify SAS shown
    const sas = await page.textContent('[data-testid="pairing-sas"]');
    expect(sas).toMatch(/^\d{6}$/);
  });
});

// Helper functions
function isValidUTF8(str: string): boolean {
  try {
    // If it looks like readable text, it might be plaintext
    const readableRatio = (str.match(/[a-zA-Z0-9\s.,!?-]/g) || []).length / str.length;
    return readableRatio > 0.7;
  } catch {
    return false;
  }
}

function isValidJSON(str: string): boolean {
  try {
    JSON.parse(str);
    return true;
  } catch {
    return false;
  }
}
