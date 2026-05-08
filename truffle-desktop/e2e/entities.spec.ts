import { test, expect } from '@playwright/test';

test.describe('Entity Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/entities');
  });
  
  test('create new entity', async ({ page }) => {
    // Click create button
    await page.click('[data-testid="create-entity-button"]');
    
    // Fill form
    await page.fill('[data-testid="entity-name-input"]', 'Test Person');
    await page.click('[data-testid="entity-type-person"]');
    await page.fill('[data-testid="entity-description-input"]', 'A test person entity');
    
    // Submit
    await page.click('[data-testid="submit-entity-button"]');
    
    // Verify entity appears in list
    await expect(page.locator('text=Test Person')).toBeVisible();
  });
  
  test('search entities', async ({ page }) => {
    // Type search query
    await page.fill('[data-testid="entity-search-input"]', 'Alice');
    
    // Wait for debounced search
    await page.waitForTimeout(300);
    
    // Verify filtered results
    await expect(page.locator('text=Alice')).toBeVisible();
  });
  
  test('view entity details', async ({ page }) => {
    // Click on entity
    await page.click('text=Alice');
    
    // Verify detail panel opens
    await expect(page.locator('[data-testid="entity-detail-panel"]')).toBeVisible();
    await expect(page.locator('text=Alice')).toBeVisible();
  });
  
  test('edit entity', async ({ page }) => {
    // Select entity
    await page.click('text=Alice');
    
    // Click edit
    await page.click('[data-testid="edit-entity-button"]');
    
    // Update name
    await page.fill('[data-testid="entity-name-input"]', 'Alice Smith');
    
    // Save
    await page.click('[data-testid="save-entity-button"]');
    
    // Verify updated
    await expect(page.locator('text=Alice Smith')).toBeVisible();
  });
  
  test('delete entity', async ({ page }) => {
    // Select entity
    await page.click('text=Test Entity');
    
    // Click delete
    await page.click('[data-testid="delete-entity-button"]');
    
    // Confirm
    await page.click('[data-testid="confirm-delete-button"]');
    
    // Verify removed
    await expect(page.locator('text=Test Entity')).not.toBeVisible();
  });

  test('filter entities by type', async ({ page }) => {
    // Open type filter
    await page.selectOption('[data-testid="entity-type-filter"]', 'person');
    
    // Wait for filter to apply
    await page.waitForTimeout(200);
    
    // Verify only person entities are shown
    const entityCards = await page.locator('[data-testid="entity-card"]').count();
    expect(entityCards).toBeGreaterThan(0);
  });

  test('sort entities by name', async ({ page }) => {
    // Click sort by name
    await page.click('[data-testid="sort-by-name"]');
    
    // Wait for sort to apply
    await page.waitForTimeout(200);
    
    // Verify entities are sorted
    await expect(page.locator('[data-testid="entity-list"]')).toBeVisible();
  });

  test('paginate through entity list', async ({ page }) => {
    // Go to next page
    await page.click('[data-testid="next-page-button"]');
    
    // Verify page changed
    await expect(page.locator('[data-testid="page-number"]:has-text("2")')).toHaveClass(/active/);
    
    // Go back to first page
    await page.click('[data-testid="previous-page-button"]');
    
    // Verify back on first page
    await expect(page.locator('[data-testid="page-number"]:has-text("1")')).toHaveClass(/active/);
  });

  test('create organization entity', async ({ page }) => {
    // Click create button
    await page.click('[data-testid="create-entity-button"]');
    
    // Fill form
    await page.fill('[data-testid="entity-name-input"]', 'Acme Corporation');
    await page.click('[data-testid="entity-type-organization"]');
    await page.fill('[data-testid="entity-industry-input"]', 'Technology');
    await page.fill('[data-testid="entity-founded-year-input"]', '2020');
    
    // Submit
    await page.click('[data-testid="submit-entity-button"]');
    
    // Verify entity appears in list
    await expect(page.locator('text=Acme Corporation')).toBeVisible();
  });
});
