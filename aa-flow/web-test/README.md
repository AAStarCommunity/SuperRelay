# ERC-4337 Rundler Web Interface

这是一个用于测试 ERC-4337 Account Abstraction 和 Rundler Bundler 的 React Web 应用。

## 🚀 部署到 Vercel

### 方法一：通过 Vercel CLI

1. **安装 Vercel CLI**
   ```bash
   npm i -g vercel
   ```

2. **登录 Vercel**
   ```bash
   vercel login
   ```

3. **在项目目录中部署**
   ```bash
   cd web-test
   vercel
   ```

4. **配置环境变量**
   在 Vercel Dashboard 中设置以下环境变量：
   ```
   VITE_SEPOLIA_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
   VITE_BUNDLER_URL=https://rundler-superrelay.fly.dev
   VITE_ENTRYPOINT_ADDRESS=0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789
   VITE_SIMPLE_ACCOUNT_A=0x7D7a0D3239285faE78F9c364D81bb1E3bc555BC6
   VITE_SIMPLE_ACCOUNT_B=0x27243FAc2c0bEf46F143a705708dC4A7eD476854
   VITE_PNT_TOKEN_ADDRESS=0x3e7B771d4541eC85c8137e950598Ac97553a337a
   VITE_PRIVATE_KEY_A=your_test_private_key_here
   ```

### 方法二：通过 GitHub + Vercel

1. **推送代码到 GitHub**
   ```bash
   git add .
   git commit -m "Add Vercel deployment config"
   git push origin your-branch
   ```

2. **在 Vercel 中导入项目**
   - 访问 [vercel.com](https://vercel.com)
   - 点击 "New Project"
   - 导入你的 GitHub 仓库
   - 设置根目录为 `aa-flow/web-test`

3. **配置构建设置**
   - Build Command: `npm run build`
   - Output Directory: `dist`
   - Install Command: `npm install`

## 🔒 安全配置

### 私钥安全
- **永远不要**将私钥提交到代码库
- 只在 Vercel 环境变量中设置私钥
- 使用测试网专用私钥
- 定期轮换测试私钥

### 环境变量设置
在 Vercel Dashboard 中：
1. 进入项目设置
2. 选择 "Environment Variables"
3. 逐一添加环境变量
4. 确保敏感信息只在生产环境可见

## 🛠️ 本地开发

1. **复制环境变量**
   ```bash
   cp .env.example .env.local
   ```

2. **填写环境变量**
   编辑 `.env.local` 文件

3. **安装依赖**
   ```bash
   npm install
   ```

4. **启动开发服务器**
   ```bash
   npm run dev
   ```

## 📦 构建和预览

```bash
# 构建生产版本
npm run build

# 预览构建结果
npm run preview
```

## 🧪 功能特性

- 多网络支持 (Sepolia, OP Sepolia, OP Mainnet)
- 实时 Bundler 状态监控
- Gas 费用计算和优化
- PNT 代币转账测试
- UserOperation 详细显示
- 账户余额查询
- 浏览器链接集成 (JiffyScan, Etherscan)

## 🔗 相关链接

- [ERC-4337 标准](https://eips.ethereum.org/EIPS/eip-4337)
- [Rundler Bundler](https://github.com/alchemyplatform/rundler)
- [JiffyScan](https://jiffyscan.xyz/)
- [Etherscan Sepolia](https://sepolia.etherscan.io/)
