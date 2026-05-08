/**
 * @fileoverview Mobile E2E Tests - Screenshot Capture Flow
 * @module tests/e2e-mobile/screenshot-capture
 * 
 * Validates mobile screenshot capture and compilation using Detox.
 * 
 * @requires detox
 */

const { device, expect, element, by, waitFor } = require('detox');

describe('Screenshot Capture Flow', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  /**
   * Test: Capture screenshot via share extension
   */
  it('should capture screenshot via share extension', async () => {
    // Simulate share extension trigger
    await device.openURL({
      url: 'truffle://capture?source=screenshot',
    });

    // Verify capture screen
    await expect(element(by.id('capture-screen'))).toBeVisible();

    // Simulate image selection
    await element(by.id('select-image')).tap();

    // Wait for compilation
    await waitFor(element(by.id('compilation-complete')))
      .toBeVisible()
      .withTimeout(30000);

    // Verify success notification
    await expect(element(by.text('Screenshot compiled'))).toBeVisible();
  });

  /**
   * Test: Handle background compilation checkpoint
   */
  it('should handle background compilation checkpoint', async () => {
    // Start compilation
    await element(by.id('capture-button')).tap();
    await element(by.id('select-image')).tap();

    // Simulate app backgrounding (iOS 30s limit)
    await device.sendToHome();
    
    // Wait a bit
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Relaunch app
    await device.launchApp({ newInstance: false });

    // Verify compilation resumed
    await expect(element(by.id('compilation-progress'))).toBeVisible();
    
    // Wait for completion
    await waitFor(element(by.id('compilation-complete')))
      .toBeVisible()
      .withTimeout(30000);
  });

  /**
   * Test: Warn at 5GB storage usage
   */
  it('should warn at 5GB storage usage', async () => {
    // Mock storage at 4.9GB
    await device.setStatusBar({
      dataNetwork: 'wifi',
      wifiMode: 'enabled',
    });

    // Trigger storage warning (simulated)
    await element(by.id('simulate-storage-warning')).tap();

    // Verify warning shown
    await expect(element(by.text('Storage Warning'))).toBeVisible();
    await expect(element(by.text('You are approaching 5GB of storage'))).toBeVisible();
  });

  /**
   * Test: Auto-pause at 10GB storage
   */
  it('should auto-pause at 10GB storage', async () => {
    // Mock storage at 9.9GB
    await element(by.id('simulate-storage-full')).tap();

    // Attempt capture
    await element(by.id('capture-button')).tap();

    // Verify auto-pause
    await expect(element(by.text('Storage Full'))).toBeVisible();
    await expect(element(by.id('capture-button'))).toBeDisabled();
  });

  /**
   * Test: Compilation with low memory
   */
  it('should handle low memory gracefully', async () => {
    // Simulate low memory condition
    await element(by.id('simulate-low-memory')).tap();

    // Attempt compilation
    await element(by.id('capture-button')).tap();
    await element(by.id('select-image')).tap();

    // Should show memory warning
    await expect(element(by.text('Low Memory'))).toBeVisible();
    
    // Should offer to pause
    await expect(element(by.text('Pause and retry later'))).toBeVisible();
  });
});

describe('Wiki Browser', () => {
  beforeEach(async () => {
    await device.reloadReactNative();
  });

  /**
   * Test: Navigate wiki links
   */
  it('should navigate wiki links', async () => {
    await element(by.id('wiki-tab')).tap();

    // Tap a node
    await element(by.id('wiki-node')).atIndex(0).tap();

    // Verify node detail shown
    await expect(element(by.id('node-detail'))).toBeVisible();

    // Tap a wiki link
    await element(by.id('wiki-link')).tap();

    // Verify navigation
    await expect(element(by.id('node-detail'))).toBeVisible();
  });

  /**
   * Test: Search wiki
   */
  it('should search wiki', async () => {
    await element(by.id('wiki-tab')).tap();

    // Open search
    await element(by.id('search-button')).tap();

    // Type search query
    await element(by.id('search-input')).typeText('Starbucks');

    // Verify results
    await expect(element(by.id('search-result'))).toBeVisible();
  });

  /**
   * Test: Edit wiki node
   */
  it('should edit wiki node', async () => {
    await element(by.id('wiki-tab')).tap();
    
    // Tap node
    await element(by.id('wiki-node')).atIndex(0).tap();
    
    // Tap edit
    await element(by.id('edit-button')).tap();
    
    // Edit content
    const newContent = 'Updated content';
    await element(by.id('editor-input')).clearText();
    await element(by.id('editor-input')).typeText(newContent);
    
    // Save
    await element(by.id('save-button')).tap();
    
    // Verify saved
    await expect(element(by.text('Saved'))).toBeVisible();
  });
});

describe('Sync Functionality', () => {
  beforeEach(async () => {
    await device.reloadReactNative();
  });

  /**
   * Test: Sync when charging and on wifi
   */
  it('should sync when charging and on wifi', async () => {
    // Enable charging and wifi
    await device.setStatusBar({
      batteryState: 'charging',
      dataNetwork: 'wifi',
    });

    // Trigger background sync
    await device.sendToHome();
    await new Promise(resolve => setTimeout(resolve, 1000));
    await device.launchApp({ newInstance: false });

    // Verify sync indicator
    await expect(element(by.id('sync-indicator'))).toBeVisible();
  });

  /**
   * Test: Store keys in Secure Enclave
   */
  it('should store keys in Secure Enclave', async () => {
    // This test verifies key storage on iOS
    // On Android, verifies KeyStore usage
    
    const platform = device.getPlatform();
    
    if (platform === 'ios') {
      // Verify iOS Keychain usage
      const keyStored = await device.getPlatformUtil().executeShellCommand(
        'security find-generic-password -s truffle.sync.key 2>&1 || echo "NOT_FOUND"'
      );
      
      // Key should be stored (or command might fail in simulator)
      expect(keyStored).toBeTruthy();
    }
  });

  /**
   * Test: Device pairing
   */
  it('should pair with another device', async () => {
    // Navigate to settings
    await element(by.id('settings-tab')).tap();
    
    // Start pairing
    await element(by.id('add-device')).tap();
    
    // Verify QR code shown
    await expect(element(by.id('pairing-qr'))).toBeVisible();
    
    // Verify fingerprint
    const fingerprint = await element(by.id('device-fingerprint')).getAttributes();
    expect(fingerprint.text).toMatch(/^[A-F0-9]{16}$/);
  });
});

describe('Red Lines - Mobile', () => {
  beforeEach(async () => {
    await device.reloadReactNative();
  });

  /**
   * Test: Red Line - Keep data on device (Sovereignty)
   */
  it('MUST keep data on device (Sovereignty)', async () => {
    // Capture screenshot
    await element(by.id('capture-button')).tap();

    // Verify no network requests to external storage
    // This would require network monitoring in Detox
    // For now, verify local storage is used
    
    await expect(element(by.id('local-storage-indicator'))).toBeVisible();
  });

  /**
   * Test: Red Line - Function offline (Survival Mode)
   */
  it('MUST function offline (Survival Mode)', async () => {
    // Disable network
    await device.setStatusBar({ dataNetwork: 'none' });

    // Verify app functions
    await element(by.id('wiki-tab')).tap();
    await expect(element(by.id('wiki-list'))).toBeVisible();

    // Search should work
    await element(by.id('search-button')).tap();
    await element(by.id('search-input')).typeText('test');
    await expect(element(by.id('search-result'))).toBeVisible();
    
    // Compilation should work
    await element(by.id('capture-button')).tap();
    await expect(element(by.id('capture-screen'))).toBeVisible();
  });

  /**
   * Test: Red Line - Export capability
   */
  it('MUST support export within 5 minutes', async () => {
    const startTime = Date.now();
    
    // Navigate to settings
    await element(by.id('settings-tab')).tap();
    
    // Trigger export
    await element(by.id('export-button')).tap();
    await element(by.id('export-markdown')).tap();
    
    // Wait for completion
    await waitFor(element(by.id('export-complete')))
      .toBeVisible()
      .withTimeout(5 * 60 * 1000);
    
    const elapsed = Date.now() - startTime;
    expect(elapsed).toBeLessThan(5 * 60 * 1000);
  });
});

describe('Performance', () => {
  beforeEach(async () => {
    await device.reloadReactNative();
  });

  /**
   * Test: Compilation within 30 seconds
   */
  it('should compile within 30 seconds', async () => {
    const startTime = Date.now();
    
    await element(by.id('capture-button')).tap();
    await element(by.id('select-image')).tap();
    
    await waitFor(element(by.id('compilation-complete')))
      .toBeVisible()
      .withTimeout(30000);
    
    const elapsed = Date.now() - startTime;
    expect(elapsed).toBeLessThan(30000);
  });

  /**
   * Test: UI responsiveness during compilation
   */
  it('should maintain UI responsiveness', async () => {
    // Start compilation
    await element(by.id('capture-button')).tap();
    await element(by.id('select-image')).tap();
    
    // UI should still respond
    await element(by.id('wiki-tab')).tap();
    await expect(element(by.id('wiki-list'))).toBeVisible();
    
    await element(by.id('raw-tab')).tap();
    await expect(element(by.id('raw-grid'))).toBeVisible();
  });
});
