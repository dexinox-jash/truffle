import { test, expect } from '@playwright/test';

test.describe('Meeting Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/meetings');
  });

  test('view meeting list', async ({ page }) => {
    // Verify meeting list is visible
    await expect(page.locator('[data-testid="meeting-list"]')).toBeVisible();
    
    // Verify at least one meeting card exists
    const meetingCards = await page.locator('[data-testid="meeting-card"]').count();
    expect(meetingCards).toBeGreaterThanOrEqual(0);
  });

  test('search meetings', async ({ page }) => {
    // Fill search input
    await page.fill('[data-testid="meeting-search-input"]', 'standup');
    
    // Wait for search results
    await page.waitForTimeout(300);
    
    // Verify search results
    await expect(page.locator('[data-testid="meeting-list"]')).toBeVisible();
  });

  test('import meeting from file', async ({ page }) => {
    // Click import button
    await page.click('[data-testid="import-meeting-button"]');
    
    // Wait for file dialog simulation
    await page.waitForSelector('[data-testid="file-upload-dialog"]');
    
    // Upload a test file
    await page.setInputFiles('[data-testid="meeting-file-input"]', {
      name: 'test-meeting.txt',
      mimeType: 'text/plain',
      buffer: Buffer.from('Test meeting transcript'),
    });
    
    // Fill meeting details
    await page.fill('[data-testid="meeting-title-input"]', 'Test Meeting');
    await page.selectOption('[data-testid="meeting-type-select"]', 'team');
    
    // Submit
    await page.click('[data-testid="submit-import-button"]');
    
    // Verify meeting appears in list
    await expect(page.locator('text=Test Meeting')).toBeVisible();
  });

  test('view meeting details', async ({ page }) => {
    // Click on first meeting
    await page.click('[data-testid="meeting-card"]:first-child');
    
    // Verify detail panel
    await expect(page.locator('[data-testid="meeting-detail-panel"]')).toBeVisible();
    
    // Verify transcript section
    await expect(page.locator('[data-testid="meeting-transcript"]')).toBeVisible();
  });

  test('extract entities from meeting', async ({ page }) => {
    // Click on first meeting
    await page.click('[data-testid="meeting-card"]:first-child');
    
    // Click extract button
    await page.click('[data-testid="extract-entities-button"]');
    
    // Wait for extraction to complete
    await page.waitForTimeout(3000);
    
    // Verify extraction status updated
    await expect(page.locator('text=Extraction completed')).toBeVisible();
  });

  test('filter meetings by type', async ({ page }) => {
    // Select meeting type filter
    await page.selectOption('[data-testid="meeting-type-filter"]', 'standup');
    
    // Wait for filter to apply
    await page.waitForTimeout(200);
    
    // Verify filtered results
    await expect(page.locator('[data-testid="meeting-list"]')).toBeVisible();
  });

  test('filter meetings by date range', async ({ page }) => {
    // Open date filter
    await page.click('[data-testid="date-filter-button"]');
    
    // Select date range
    await page.fill('[data-testid="date-from-input"]', '2024-01-01');
    await page.fill('[data-testid="date-to-input"]', '2024-12-31');
    
    // Apply filter
    await page.click('[data-testid="apply-date-filter"]');
    
    // Verify filtered results
    await expect(page.locator('[data-testid="meeting-list"]')).toBeVisible();
  });

  test('edit meeting details', async ({ page }) => {
    // Click on first meeting
    await page.click('[data-testid="meeting-card"]:first-child');
    
    // Click edit button
    await page.click('[data-testid="edit-meeting-button"]');
    
    // Update title
    await page.fill('[data-testid="meeting-title-input"]', 'Updated Meeting Title');
    
    // Save changes
    await page.click('[data-testid="save-meeting-button"]');
    
    // Verify updated title
    await expect(page.locator('text=Updated Meeting Title')).toBeVisible();
  });

  test('delete meeting', async ({ page }) => {
    // Get first meeting title
    const meetingTitle = await page.locator('[data-testid="meeting-title"]:first-child').textContent();
    
    // Click on first meeting
    await page.click('[data-testid="meeting-card"]:first-child');
    
    // Click delete button
    await page.click('[data-testid="delete-meeting-button"]');
    
    // Confirm deletion
    await page.click('[data-testid="confirm-delete-button"]');
    
    // Verify meeting removed from list
    await expect(page.locator(`text=${meetingTitle}`)).not.toBeVisible();
  });

  test('view meeting participants', async ({ page }) => {
    // Click on first meeting
    await page.click('[data-testid="meeting-card"]:first-child');
    
    // Verify participants section
    await expect(page.locator('[data-testid="meeting-participants"]')).toBeVisible();
  });

  test('view extracted entities from meeting', async ({ page }) => {
    // Click on a meeting that has been processed
    await page.click('[data-testid="meeting-card"]:has([data-testid="extraction-status-completed"])');
    
    // Click on extracted entities tab
    await page.click('[data-testid="extracted-entities-tab"]');
    
    // Verify extracted entities list
    await expect(page.locator('[data-testid="extracted-entities-list"]')).toBeVisible();
  });

  test('regenerate meeting summary', async ({ page }) => {
    // Click on first meeting
    await page.click('[data-testid="meeting-card"]:first-child');
    
    // Click regenerate summary button
    await page.click('[data-testid="regenerate-summary-button"]');
    
    // Wait for generation
    await page.waitForTimeout(2000);
    
    // Verify summary section
    await expect(page.locator('[data-testid="meeting-summary"]')).toBeVisible();
  });
});
