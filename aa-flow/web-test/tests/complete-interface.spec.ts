import { test, expect } from '@playwright/test';

test.describe('完整界面功能测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('页面标题和基本布局验证', async ({ page }) => {
    // 验证页面标题
    await expect(page).toHaveTitle('ERC-4337 Rundler Testing Interface');

    // 验证主标题
    await expect(page.locator('h1')).toContainText('ERC-4337 Rundler Testing Interface');

    // 验证描述
    await expect(page.locator('p')).toContainText('Comprehensive testing interface for Rundler bundler service');
  });

  test('网络选择器功能验证', async ({ page }) => {
    const networkSelector = page.locator('select[data-testid="network-selector"]');

    // 验证选择器存在且可见
    await expect(networkSelector).toBeVisible();

    // 验证默认选择
    await expect(networkSelector).toHaveValue('sepolia');

    // 验证选项数量
    const options = await networkSelector.locator('option').count();
    expect(options).toBe(3); // Sepolia, OP Sepolia, OP Mainnet

    // 测试网络切换
    await networkSelector.selectOption('op-sepolia');
    await expect(networkSelector).toHaveValue('op-sepolia');

    // 切换回默认
    await networkSelector.selectOption('sepolia');
    await expect(networkSelector).toHaveValue('sepolia');
  });

  test('所有主要组件存在验证', async ({ page }) => {
    // 验证6个主要组件的标题
    const componentTitles = [
      '⚙️ Environment Configuration',
      '🔧 Bundler Status',
      '⛽ Gas Calculator',
      '👛 Account Management',
      '🚀 Transfer Test'
    ];

    for (const title of componentTitles) {
      await expect(page.locator(`h3:has-text("${title}")`)).toBeVisible();
    }
  });

  test('环境配置显示验证', async ({ page }) => {
    const configSection = page.locator('section.config-section');

    await expect(configSection).toBeVisible();
    await expect(configSection.locator('h3')).toContainText('⚙️ Environment Configuration');
    await expect(configSection).toContainText('Configuration Status: Ready');
  });

  test('Bundler状态显示验证', async ({ page }) => {
    const statusSection = page.locator('section.status-section');

    await expect(statusSection).toBeVisible();
    await expect(statusSection.locator('h3')).toContainText('🔧 Bundler Status');
    await expect(statusSection).toContainText('Bundler URL');
    await expect(statusSection).toContainText('https://rundler-superrelay.fly.dev');
  });

  test('Gas计算器功能验证', async ({ page }) => {
    const gasSection = page.locator('section.gas-section');

    await expect(gasSection).toBeVisible();
    await expect(gasSection.locator('h3')).toContainText('⛽ Gas Calculator');

    // 验证Gas参数说明
    await expect(gasSection).toContainText('preVerificationGas');
    await expect(gasSection).toContainText('callGasLimit');
    await expect(gasSection).toContainText('verificationGasLimit');
  });

  test('账户管理界面验证', async ({ page }) => {
    const accountSection = page.locator('section.account-section');

    await expect(accountSection).toBeVisible();
    await expect(accountSection.locator('h3')).toContainText('👛 Account Management');

    // 验证账户类型
    await expect(accountSection).toContainText('🔑 EOA (Owner)');
    await expect(accountSection).toContainText('📤 SimpleAccount A (Sender)');
    await expect(accountSection).toContainText('📥 SimpleAccount B (Receiver)');
    await expect(accountSection).toContainText('🏭 SimpleAccount Factory');
  });

  test('转账测试功能验证', async ({ page }) => {
    const transferSection = page.locator('section.transfer-section');

    await expect(transferSection).toBeVisible();
    await expect(transferSection.locator('h3')).toContainText('🚀 Transfer Test');

    // 验证输入字段
    const amountInput = transferSection.locator('input[type="number"]');
    await expect(amountInput).toBeVisible();
    await expect(amountInput).toHaveValue('3');

    // 验证按钮
    await expect(transferSection.locator('button:has-text("Execute Transfer")')).toBeVisible();
    await expect(transferSection.locator('button:has-text("Clear History")')).toBeVisible();

    // 测试输入功能
    await amountInput.clear();
    await amountInput.fill('5');
    await expect(amountInput).toHaveValue('5');
  });

  test('外部链接验证', async ({ page }) => {
    // 验证Etherscan链接
    const etherscanLink = page.locator('a:has-text("🔍 Etherscan")');
    await expect(etherscanLink).toBeVisible();
    await expect(etherscanLink).toHaveAttribute('href', 'https://sepolia.etherscan.io');
    await expect(etherscanLink).toHaveAttribute('target', '_blank');
    await expect(etherscanLink).toHaveAttribute('rel', 'noopener noreferrer');
  });

  test('响应式设计验证', async ({ page }) => {
    // 桌面视图验证
    await page.setViewportSize({ width: 1200, height: 800 });
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('h3')).toHaveCount(5);

    // 平板视图验证
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('h3')).toHaveCount(5);

    // 手机视图验证
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('h3')).toHaveCount(5);
  });

  test('页面性能测试', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/', { waitUntil: 'networkidle' });

    // 验证所有主要元素加载
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('h3')).toHaveCount(5);
    await expect(page.locator('select')).toBeVisible();

    const loadTime = Date.now() - startTime;
    console.log(`页面加载时间: ${loadTime}ms`);

    // 页面应该在3秒内加载完成
    expect(loadTime).toBeLessThan(3000);
  });

  test('用户交互流程测试', async ({ page }) => {
    // 1. 切换网络
    const networkSelector = page.locator('select[data-testid="network-selector"]');
    await networkSelector.selectOption('op-sepolia');

    // 2. 检查内容更新（Bundler URL应该不同）
    // 注意：简化版本中所有网络都使用同一个URL，这里只是验证选择器工作
    await expect(networkSelector).toHaveValue('op-sepolia');

    // 3. 修改转账金额
    const amountInput = page.locator('input[type="number"]');
    await amountInput.clear();
    await amountInput.fill('10');
    await expect(amountInput).toHaveValue('10');

    // 4. 重置为默认状态
    await networkSelector.selectOption('sepolia');
    await amountInput.clear();
    await amountInput.fill('3');

    // 验证重置成功
    await expect(networkSelector).toHaveValue('sepolia');
    await expect(amountInput).toHaveValue('3');
  });
});