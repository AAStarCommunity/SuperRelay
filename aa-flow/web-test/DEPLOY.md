# Vercel 部署指南

## 🚀 快速部署步骤

### 1. 推送代码到 GitHub
```bash
git add .
git commit -m "feat: add Vercel deployment config"
git push origin feat/web-interface-EP0.6
```

### 2. 在 Vercel 导入项目
1. 访问 [vercel.com](https://vercel.com)
2. 点击 "New Project"
3. 导入 GitHub 仓库
4. **重要**: 设置根目录为 `aa-flow/web-test`

### 3. 配置构建设置
- **Framework Preset**: Vite
- **Build Command**: `npm run build`
- **Output Directory**: `dist`
- **Install Command**: `npm install`

### 4. 配置环境变量
在 Vercel Dashboard 中添加以下环境变量：

```env
VITE_SEPOLIA_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
VITE_BUNDLER_URL=https://rundler-superrelay.fly.dev
VITE_ENTRYPOINT_ADDRESS=0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789
VITE_SIMPLE_ACCOUNT_A=0x7D7a0D3239285faE78F9c364D81bb1E3bc555BC6
VITE_SIMPLE_ACCOUNT_B=0x27243FAc2c0bEf46F143a705708dC4A7eD476854
VITE_PNT_TOKEN_ADDRESS=0x3e7B771d4541eC85c8137e950598Ac97553a337a
VITE_PRIVATE_KEY_A=your_test_private_key_here
```

### 5. 部署
点击 "Deploy" 按钮，Vercel 将自动构建和部署应用。

## 🔒 安全注意事项

- **私钥安全**: 只使用测试网私钥，永远不要暴露主网私钥
- **环境变量**: 所有敏感信息只在 Vercel 环境变量中设置
- **访问控制**: 考虑在生产环境中添加访问限制

## 🔗 部署后验证

部署完成后，访问你的 Vercel 应用 URL 并验证：
- [ ] 网络选择器工作正常
- [ ] Bundler 状态显示
- [ ] 账户余额加载
- [ ] 转账功能可用
- [ ] UserOperation 详细信息显示

## 📝 故障排除

### 常见问题
1. **CORS 错误**: 检查 `vercel.json` 中的代理配置
2. **环境变量错误**: 确保所有 `VITE_` 前缀的变量都已设置
3. **构建失败**: 检查 TypeScript 错误和依赖问题

### 调试步骤
1. 检查 Vercel 构建日志
2. 查看浏览器控制台错误
3. 验证环境变量设置
4. 测试 API 代理是否工作