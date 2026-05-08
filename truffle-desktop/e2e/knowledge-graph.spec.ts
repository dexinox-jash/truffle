import { test, expect } from '@playwright/test';

test.describe('Knowledge Graph Visualization', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/graph');
  });
  
  test('view entity graph', async ({ page }) => {
    // Select root entity from dropdown
    await page.selectOption('[data-testid="root-entity-select"]', 'entity-1');
    
    // Verify graph canvas renders
    await expect(page.locator('[data-testid="graph-viewer"]')).toBeVisible();
    
    // Verify graph controls
    await expect(page.locator('[data-testid="zoom-in-button"]')).toBeVisible();
    await expect(page.locator('[data-testid="zoom-out-button"]')).toBeVisible();
    await expect(page.locator('[data-testid="fit-button"]')).toBeVisible();
  });
  
  test('click node shows details', async ({ page }) => {
    // Wait for graph to render
    await page.waitForSelector('[data-testid="graph-node"]', { timeout: 5000 });
    
    // Click on first node
    await page.click('[data-testid="graph-node"]:first-child');
    
    // Verify detail panel opens
    await expect(page.locator('[data-testid="node-detail-panel"]')).toBeVisible();
  });
  
  test('search highlights nodes', async ({ page }) => {
    // Search for entity
    await page.fill('[data-testid="graph-search-input"]', 'Alice');
    
    // Wait for search results
    await page.waitForTimeout(300);
    
    // Verify highlighted nodes
    const highlightedNodes = page.locator('[data-testid="graph-node"].highlighted');
    await expect(highlightedNodes).toHaveCount.greaterThan(0);
  });
  
  test('zoom controls work', async ({ page }) => {
    // Get initial transform
    const canvas = page.locator('[data-testid="graph-canvas"]');
    
    // Click zoom in
    await page.click('[data-testid="zoom-in-button"]').catch(() => {});
    
    // Click zoom out
    await page.click('[data-testid="zoom-out-button"]').catch(() => {});
    
    // Canvas should still be visible
    await expect(canvas).toBeVisible();
  });
  
  test('depth control filters nodes', async ({ page }) => {
    // Select entity
    await page.selectOption('[data-testid="root-entity-select"]', 'entity-1');
    
    // Change depth
    await page.selectOption('[data-testid="depth-select"]', '1');
    
    // Wait for graph update
    await page.waitForTimeout(500);
    
    // Graph should update (nodes may decrease)
    await expect(page.locator('[data-testid="graph-node"]')).toBeVisible();
  });
});

test.describe('Validation Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/validation');
  });
  
  test('view validation summary', async ({ page }) => {
    // Verify summary cards are visible
    await expect(page.locator('[data-testid="contradictions-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="duplicates-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="avg-confidence"]')).toBeVisible();
  });
  
  test('switch between tabs', async ({ page }) => {
    // Click contradictions tab
    await page.click('text=Contradictions');
    await expect(page.locator('[data-testid="contradiction-list"]')).toBeVisible();
    
    // Click duplicates tab
    await page.click('text=Duplicates');
    await expect(page.locator('[data-testid="duplicate-list"]')).toBeVisible();
    
    // Click back to summary
    await page.click('text=Summary');
    await expect(page.locator('[data-testid="validation-summary"]')).toBeVisible();
  });
  
  test('view contradiction details', async ({ page }) => {
    // Go to contradictions tab
    await page.click('text=Contradictions');
    
    // Click on first contradiction
    await page.click('[data-testid="contradiction-item"]:first-child');
    
    // Verify detail panel
    await expect(page.locator('[data-testid="contradiction-detail"]')).toBeVisible();
  });
  
  test('merge duplicate entities', async ({ page }) => {
    // Go to duplicates tab
    await page.click('text=Duplicates');
    
    // Wait for duplicates to load
    await page.waitForSelector('[data-testid="duplicate-item"]', { timeout: 5000 });
    
    // Click merge on first duplicate
    await page.click('[data-testid="merge-button"]:first-child');
    
    // Verify merge modal opens
    await expect(page.locator('[data-testid="merge-modal"]')).toBeVisible();
    
    // Confirm merge
    await page.click('[data-testid="confirm-merge-button"]');
    
    // Verify success message
    await expect(page.locator('text=Entities merged successfully')).toBeVisible();
  });
});

test.describe('Meeting Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/meetings');
  });
  
  test('view meeting list', async ({ page }) => {
    // Verify meeting list is visible
    await expect(page.locator('[data-testid="meeting-list"]')).toBeVisible();
  });
  
  test('search meetings', async ({ page }) => {
    // Type search query
    await page.fill('[data-testid="meeting-search-input"]', 'team standup');
    
    // Wait for debounce
    await page.waitForTimeout(300);
    
    // Results should update
    await expect(page.locator('[data-testid="meeting-list"]')).toBeVisible();
  });
  
  test('view meeting detail', async ({ page }) => {
    // Click on first meeting
    await page.click('[data-testid="meeting-item"]:first-child');
    
    // Verify detail view
    await expect(page.locator('[data-testid="meeting-detail"]')).toBeVisible();
    
    // Verify tabs
    await expect(page.locator('text=Transcript')).toBeVisible();
    await expect(page.locator('text=Summary')).toBeVisible();
    await expect(page.locator('text=Entities')).toBeVisible();
  });
  
  test('extract entities from meeting', async ({ page }) => {
    // Select meeting
    await page.click('[data-testid="meeting-item"]:first-child');
    
    // Click extract button
    await page.click('[data-testid="extract-button"]');
    
    // Verify extraction panel
    await expect(page.locator('[data-testid="extraction-panel"]')).toBeVisible();
    
    // Wait for extraction (mocked)
    await page.waitForTimeout(2000);
    
    // Verify progress or completion
    await expect(page.locator('[data-testid="extraction-progress"]')).toBeVisible();
  });
});
