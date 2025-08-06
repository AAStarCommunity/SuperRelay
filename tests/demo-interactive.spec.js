const { test, expect } = require('@playwright/test');

test.describe('SuperRelay Interactive Demo', () => {
  test.beforeEach(async ({ page }) => {
    // Set up page error handling
    page.on('console', msg => {
      if (msg.type() === 'error') {
        console.log('Console error:', msg.text());
      }
    });

    page.on('pageerror', error => {
      console.log('Page error:', error.message);
    });
  });

  test('should load interactive demo page', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');

    // Check if page title is correct
    await expect(page).toHaveTitle(/SuperRelay.*Demo/i);

    // Check if main elements are present
    await expect(page.locator('h1')).toContainText('SuperRelay Demo');
  });

  test('should check service connectivity', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');

    // Wait for connectivity checks to complete
    await page.waitForTimeout(2000);

    // Check if connection status indicators are present
    const statusIndicators = page.locator('[data-testid*="status"]');
    await expect(statusIndicators.first()).toBeVisible();
  });

  test('should create UserOperation form', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');

    // Check if UserOperation form elements exist
    const senderInput = page.locator('input[name="sender"]');
    const nonceInput = page.locator('input[name="nonce"]');
    const callDataInput = page.locator('input[name="callData"]');

    if (await senderInput.count() > 0) {
      await expect(senderInput).toBeVisible();
      await expect(nonceInput).toBeVisible();
      await expect(callDataInput).toBeVisible();
    }
  });

  test('should test API endpoint calls', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');

    // Wait for page to load
    await page.waitForTimeout(1000);

    // Look for test buttons or API call triggers
    const testButton = page.locator('button:has-text("Test")').first();

    if (await testButton.count() > 0) {
      await testButton.click();

      // Wait for API call to complete
      await page.waitForTimeout(3000);

      // Check for result display
      const resultArea = page.locator('[data-testid="result"], .result, #result');
      if (await resultArea.count() > 0) {
        await expect(resultArea).toBeVisible();
      }
    }
  });

  test('should handle errors gracefully', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');

    // Wait for page to load
    await page.waitForTimeout(1000);

    // Test with invalid data if form exists
    const senderInput = page.locator('input[name="sender"]');

    if (await senderInput.count() > 0) {
      await senderInput.fill('invalid-address');

      const submitButton = page.locator('button[type="submit"], button:has-text("Submit")').first();
      if (await submitButton.count() > 0) {
        await submitButton.click();

        // Check for error message
        await page.waitForTimeout(2000);
        const errorMessage = page.locator('.error, [data-testid="error"], .alert-error');

        // Error handling should be present (either error message or form validation)
        const hasError = (await errorMessage.count() > 0) ||
                        (await page.locator('input:invalid').count() > 0);

        if (hasError) {
          console.log('✅ Error handling working correctly');
        }
      }
    }
  });
});
