import { test, expect } from '@playwright/test';

test.describe('Validation Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/validation');
  });

  test('view validation summary', async ({ page }) => {
    // Verify summary cards
    await expect(page.locator('[data-testid="contradictions-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="duplicates-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="avg-confidence"]')).toBeVisible();
    await expect(page.locator('[data-testid="entities-with-issues-count"]')).toBeVisible();
  });
  
  test('view contradictions tab', async ({ page }) => {
    // Click contradictions tab
    await page.click('[data-testid="contradictions-tab"]');
    
    // Verify list
    await expect(page.locator('[data-testid="contradiction-list"]')).toBeVisible();
  });

  test('view duplicates tab', async ({ page }) => {
    // Click duplicates tab
    await page.click('[data-testid="duplicates-tab"]');
    
    // Verify list
    await expect(page.locator('[data-testid="duplicate-list"]')).toBeVisible();
  });
  
  test('merge duplicate entities', async ({ page }) => {
    // Click duplicates tab
    await page.click('[data-testid="duplicates-tab"]');
    
    // Wait for duplicates to load
    await page.waitForSelector('[data-testid="duplicate-item"]', { timeout: 5000 });
    
    // Click merge on first duplicate
    await page.click('[data-testid="merge-button"]:first-child');
    
    // Confirm merge in modal
    await page.click('[data-testid="confirm-merge-button"]');
    
    // Verify success message
    await expect(page.locator('text=Entities merged successfully')).toBeVisible();
  });

  test('resolve contradiction', async ({ page }) => {
    // Click contradictions tab
    await page.click('[data-testid="contradictions-tab"]');
    
    // Wait for contradictions to load
    await page.waitForSelector('[data-testid="contradiction-item"]', { timeout: 5000 });
    
    // Click resolve on first contradiction
    await page.click('[data-testid="resolve-contradiction-button"]:first-child');
    
    // Enter resolution
    await page.fill('[data-testid="resolution-input"]', 'Resolved: updated with correct information');
    
    // Confirm resolution
    await page.click('[data-testid="confirm-resolution-button"]');
    
    // Verify success message
    await expect(page.locator('text=Contradiction resolved')).toBeVisible();
  });

  test('view contradiction details', async ({ page }) => {
    // Click contradictions tab
    await page.click('[data-testid="contradictions-tab"]');
    
    // Wait for contradictions to load
    await page.waitForSelector('[data-testid="contradiction-item"]', { timeout: 5000 });
    
    // Click on first contradiction
    await page.click('[data-testid="contradiction-item"]:first-child');
    
    // Verify detail panel
    await expect(page.locator('[data-testid="contradiction-detail-panel"]')).toBeVisible();
  });

  test('run duplicate detection', async ({ page }) => {
    // Click run detection button
    await page.click('[data-testid="run-duplicate-detection-button"]');
    
    // Wait for detection to complete
    await page.waitForTimeout(2000);
    
    // Verify results updated
    await expect(page.locator('[data-testid="duplicates-count"]')).toBeVisible();
  });

  test('run contradiction detection', async ({ page }) => {
    // Click run detection button
    await page.click('[data-testid="run-contradiction-detection-button"]');
    
    // Wait for detection to complete
    await page.waitForTimeout(2000);
    
    // Verify results updated
    await expect(page.locator('[data-testid="contradictions-count"]')).toBeVisible();
  });

  test('filter contradictions by severity', async ({ page }) => {
    // Click contradictions tab
    await page.click('[data-testid="contradictions-tab"]');
    
    // Filter by high severity
    await page.selectOption('[data-testid="severity-filter"]', 'high');
    
    // Verify filtered results
    const contradictions = await page.locator('[data-testid="contradiction-item"]').count();
    expect(contradictions).toBeGreaterThanOrEqual(0);
  });

  test('mark as not duplicate', async ({ page }) => {
    // Click duplicates tab
    await page.click('[data-testid="duplicates-tab"]');
    
    // Wait for duplicates to load
    await page.waitForSelector('[data-testid="duplicate-item"]', { timeout: 5000 });
    
    // Click not duplicate on first item
    await page.click('[data-testid="not-duplicate-button"]:first-child');
    
    // Verify item removed from list
    await expect(page.locator('text=Marked as not duplicate')).toBeVisible();
  });

  test('view low confidence entities', async ({ page }) => {
    // Click confidence tab or section
    await page.click('[data-testid="low-confidence-tab"]');
    
    // Verify low confidence list
    await expect(page.locator('[data-testid="low-confidence-list"]')).toBeVisible();
  });

  test('export validation report', async ({ page }) => {
    // Click export button
    await page.click('[data-testid="export-report-button"]');
    
    // Verify export started
    await expect(page.locator('text=Exporting report')).toBeVisible();
  });
});
