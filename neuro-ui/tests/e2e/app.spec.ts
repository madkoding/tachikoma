import { test, expect } from '@playwright/test';

test.describe('TACHIKOMA-UI E2E Tests', () => {
  test('should load the application', async ({ page }) => {
    await page.goto('/');
    
    await expect(page).toHaveTitle(/TACHIKOMA/);
  });

  test('should display chat interface', async ({ page }) => {
    await page.goto('/');
    
    const chatInput = page.getByPlaceholder(/chat/i);
    await expect(chatInput).toBeVisible();
  });

  test('should send a chat message', async ({ page }) => {
    await page.goto('/');
    
    const chatInput = page.getByPlaceholder(/chat/i);
    await chatInput.fill('Hello TACHIKOMA');
    await chatInput.press('Enter');
    
    await expect(page.getByText('Hello TACHIKOMA')).toBeVisible();
  });

  test('should navigate to different sections', async ({ page }) => {
    await page.goto('/');
    
    const navItems = page.getByRole('navigation').getByRole('link');
    await expect(navItems).toHaveCountGreaterThan(0);
    
    await navItems.first().click();
    await expect(page).toHaveURL(/.*/);
  });

  test('should be responsive on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    
    const chatInput = page.getByPlaceholder(/chat/i);
    await expect(chatInput).toBeVisible();
  });
});
