import { test, expect } from '@playwright/test';

test.describe('基本功能测试', () => {
  test('页面加载和基本元素', async ({ page }) => {
    await page.goto('/');

    // 等待页面加载
    await page.waitForLoadState('networkidle');

    // 检查页面标题
    await expect(page).toHaveTitle('ERC-4337 Rundler Testing Interface');

    // 检查主标题存在
    const h1 = page.locator('h1');
    await expect(h1).toBeVisible();
    await expect(h1).toContainText('ERC-4337 Rundler Testing Interface');

    // 截图以便调试
    await page.screenshot({ path: 'test-results/basic-test.png', fullPage: true });

    // 打印页面内容
    const bodyText = await page.textContent('body');
    console.log('页面内容:', bodyText?.substring(0, 500));
  });

  test('检查所有主要组件', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // 等待足够长的时间让 React 组件渲染
    await page.waitForTimeout(2000);

    // 检查组件数量
    const h3Elements = page.locator('h3');
    const count = await h3Elements.count();
    console.log(`找到 ${count} 个 h3 元素`);

    // 打印所有 h3 文本
    for (let i = 0; i < count; i++) {
      const text = await h3Elements.nth(i).textContent();
      console.log(`H3 #${i}: ${text}`);
    }

    // 检查网络选择器
    const select = page.locator('select');
    if (await select.count() > 0) {
      console.log('找到 select 元素');
      await expect(select.first()).toBeVisible();
    } else {
      console.log('未找到 select 元素');
    }
  });
});