import { test, expect } from '@playwright/test';

test.describe('ERC-4337 Rundler Web Interface', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('页面标题和基本布局', async ({ page }) => {
    // 检查页面标题
    await expect(page).toHaveTitle(/ERC-4337/);

    // 检查主标题
    await expect(page.locator('h1')).toContainText('ERC-4337 Rundler Testing Interface');

    // 检查描述文本
    await expect(page.locator('header p')).toContainText('Comprehensive testing interface for Rundler bundler service');
  });

  test('网络选择器功能', async ({ page }) => {
    // 检查网络选择器存在
    const networkSelector = page.locator('select[data-testid="network-selector"]');
    await expect(networkSelector).toBeVisible();

    // 检查默认选择 Sepolia
    await expect(networkSelector).toHaveValue('sepolia');

    // 检查所有网络选项
    const options = networkSelector.locator('option');
    await expect(options.nth(0)).toHaveText('Sepolia');
    await expect(options.nth(1)).toHaveText('OP Sepolia');
    await expect(options.nth(2)).toHaveText('OP Mainnet');
  });

  test('环境配置显示组件', async ({ page }) => {
    // 检查配置卡片存在
    await expect(page.locator('h3:has-text("📋 Environment Configuration")')).toBeVisible();

    // 检查配置项
    await expect(page.locator('text=Bundler URL').first()).toBeVisible();
    await expect(page.locator('text=EntryPoint').first()).toBeVisible();
    await expect(page.locator('text=Factory Contract').first()).toBeVisible();
    await expect(page.locator('text=Token Information')).toBeVisible();

    // 检查账户地址
    await expect(page.locator('text=EOA (Owner)')).toBeVisible();
    await expect(page.locator('text=SimpleAccount A').first()).toBeVisible();
    await expect(page.locator('text=SimpleAccount B').first()).toBeVisible();
  });

  test('Bundler 状态监控', async ({ page }) => {
    // 检查状态卡片存在
    await expect(page.locator('h3:has-text("🔧 Bundler Status")')).toBeVisible();

    // 检查刷新按钮
    const refreshBtn = page.locator('button:has-text("Refresh")').first();
    await expect(refreshBtn).toBeVisible();

    // 检查 Bundler 状态指示器 (第一个 status-indicator)
    await expect(page.locator('.status-indicator').first()).toBeVisible();

    // 检查 Bundler URL 显示
    await expect(page.locator('text=Bundler URL:').first()).toBeVisible();
    await expect(page.locator('text=Network:').first()).toBeVisible();
  });

  test('Gas 计算器组件', async ({ page }) => {
    // 检查 Gas 计算器标题
    await expect(page.locator('h3:has-text("⛽ Gas Price Calculator")')).toBeVisible();

    // 检查 Gas 规则显示
    await expect(page.locator('text=preVerificationGas')).toBeVisible();
    await expect(page.locator('text=Zero Byte Cost').first()).toBeVisible();
    await expect(page.locator('text=Non-Zero Byte Cost')).toBeVisible();

    // 检查示例计算
    await expect(page.locator('text=Transfer Cost Estimation')).toBeVisible();
  });

  test('账户管理界面', async ({ page }) => {
    // 检查账户管理标题
    await expect(page.locator('h3:has-text("👛 Account Management")')).toBeVisible();

    // 检查刷新按钮
    const refreshBtn = page.locator('button:has-text("Refresh")').nth(1);
    await expect(refreshBtn).toBeVisible();

    // 检查代币信息
    await expect(page.locator('text=🪙 Token Information')).toBeVisible();

    // 检查账户卡片
    await expect(page.locator('text=🔑 EOA (Owner)')).toBeVisible();
    await expect(page.locator('text=📤 SimpleAccount A (Sender)')).toBeVisible();
    await expect(page.locator('text=📥 SimpleAccount B (Receiver)')).toBeVisible();

    // 检查工厂信息
    await expect(page.locator('text=🏭 SimpleAccount Factory')).toBeVisible();
  });

  test('转账测试功能', async ({ page }) => {
    // 检查转账测试标题
    await expect(page.locator('h3:has-text("🚀 Transfer Test")')).toBeVisible();

    // 检查转账金额输入
    const amountInput = page.locator('#amount-input');
    await expect(amountInput).toBeVisible();
    await expect(amountInput).toHaveValue('3');

    // 检查执行转账按钮
    const transferBtn = page.locator('button:has-text("Transfer 3 PNT")');
    await expect(transferBtn).toBeVisible();

    // 检查重置按钮
    const resetBtn = page.locator('button:has-text("Reset")');
    await expect(resetBtn).toBeVisible();
  });

  test('响应式设计', async ({ page }) => {
    // 测试移动端视口
    await page.setViewportSize({ width: 375, height: 667 });

    // 检查页面仍然可访问
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('h3')).toHaveCount(5); // 5个主要组件

    // 测试平板视口
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator('h1')).toBeVisible();

    // 回到桌面视口
    await page.setViewportSize({ width: 1200, height: 800 });
    await expect(page.locator('h1')).toBeVisible();
  });

  test('交互功能测试', async ({ page }) => {
    // 测试网络切换
    const networkSelector = page.locator('select');
    await networkSelector.selectOption('opSepolia');
    await expect(networkSelector).toHaveValue('opSepolia');

    // 切换回 Sepolia
    await networkSelector.selectOption('sepolia');
    await expect(networkSelector).toHaveValue('sepolia');

    // 测试刷新按钮点击
    const bundlerRefreshBtn = page.locator('button:has-text("Refresh")').first();
    await bundlerRefreshBtn.click();

    // 测试转账金额输入
    const amountInput = page.locator('#amount-input');
    await amountInput.clear();
    await amountInput.fill('5');
    await expect(amountInput).toHaveValue('5');

    // 恢复默认值
    await amountInput.clear();
    await amountInput.fill('3');
    await expect(amountInput).toHaveValue('3');
  });

  test('外部链接验证', async ({ page }) => {
    // 检查 Etherscan 链接存在
    const etherscanLinks = page.locator('a:has-text("Etherscan")');
    await expect(etherscanLinks.first()).toBeVisible();

    // 检查链接属性
    await expect(etherscanLinks.first()).toHaveAttribute('target', '_blank');
    await expect(etherscanLinks.first()).toHaveAttribute('rel', 'noopener noreferrer');
  });

  test('页面加载性能', async ({ page }) => {
    const startTime = Date.now();
    await page.goto('/');

    // 等待主要内容加载
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('h3')).toHaveCount(5);

    const loadTime = Date.now() - startTime;
    console.log(`页面加载时间: ${loadTime}ms`);

    // 页面应该在 5 秒内加载完成
    expect(loadTime).toBeLessThan(5000);
  });
});