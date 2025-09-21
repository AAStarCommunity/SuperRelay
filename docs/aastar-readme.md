# SuperRelay 分支结构说明

## 分支概述

本项目采用多分支开发模式，不同分支承担不同职责：

### 主要分支

- **main** - 来自原始main分支，与远程同步的原始 alchemy Rundler 代码
- **main-base** - 包含完整 ERC-4337 实现的主要开发分支，来自于main
- **pure-rundler-deploy** - 专门用来部署original rundler的分支，来自于main-base
- **relay-dev** - 专门用来开发relay的分支，添加paymaster功能，来自于main-base

## 分支用途详解

### main 分支
- 保持与上游 Alchemy Rundler 仓库同步
- 仅包含原始 Rundler 代码
- 不包含任何自定义开发内容
- 作为其他分支的基础参考

### main-base 分支
- 基于 main 分支创建
- 包含完整的 ERC-4337 Account Abstraction 实现
- 包含 PNT 代币转账测试和验证
- 包含 aa-flow 测试框架
- 所有自定义开发的起点

### pure-rundler-deploy 分支
- 基于 main-base 分支创建
- 专门用于部署原始 Rundler 到 Fly.io
- 包含优化的 Dockerfile 和 fly.toml 配置
- 适用于生产环境部署

### relay-dev 分支
- 基于 main-base 分支创建
- 专门用于开发 SuperRelay 功能
- 将添加 Paymaster 支持
- 包含中继服务相关功能

## 开发工作流

1. **原始代码同步**: main 分支与上游保持同步
2. **基础开发**: main-base 分支进行核心功能开发
3. **部署准备**: pure-rundler-deploy 分支用于生产部署
4. **功能开发**: relay-dev 分支开发新功能

## 🛠️ 部署和监控工具

### 部署脚本
- **scripts/deploy-and-monitor.sh** - 完整的部署和监控脚本
  - 自动检查 Fly CLI 安装和认证
  - 执行部署并持续监控直到成功
  - 显示部署状态、健康检查和应用测试
  - 包含错误处理和用户交互

- **scripts/monitor-only.sh** - 仅监控脚本 (当 Fly CLI 不可用时)
  - 测试应用基本连接、健康检查和 RPC 端点
  - 支持单次检查或持续监控模式
  - 无需 Fly CLI，直接通过 HTTP 请求监控

### 使用方法
```bash
# 完整部署和监控 (需要 Fly CLI)
./scripts/deploy-and-monitor.sh

# 仅监控应用状态
./scripts/monitor-only.sh --check      # 单次检查
./scripts/monitor-only.sh --monitor    # 持续监控
```

## 📝 测试套件

### aa-flow 目录结构
完整的 ERC-4337 Account Abstraction 测试套件：

```
aa-flow/
├── README.md                       # 测试套件使用说明
├── package.json                    # Node.js 项目配置
├── .env.example                    # 环境变量模板
├── ERC4337-AB-Test-Guide.md        # 详细测试指南和 A、B 账户文档
└── src/
    ├── testTransferWithBundler.js  # 主要测试脚本 - PNT 转账
    ├── testWithProperSignature.js  # 签名方法验证测试
    └── testPNTTransferFixed.js     # PNT 转账专用脚本
```

### 核心测试功能
- ✅ **A、B 账户测试**: 完整的 SimpleAccount 创建、部署和转账流程
- ✅ **签名验证**: 发现并验证 v0.6 使用 Ethereum Signed Message 格式
- ✅ **PNT 代币转账**: 成功完成 5 PNT 转账 (交易哈希: 0xa601891...)
- ✅ **Gas 优化**: 解决 AA23 签名错误和 gas 估算问题
- ✅ **Bundler 集成**: 与 Fly.io 部署的 Rundler 完全集成

### 使用测试套件
```bash
cd aa-flow
npm install ethers@5.7.2

# 配置环境变量
cp .env.example .env
# 编辑 .env 填入实际值

# 运行测试
npm run test                # 主要 PNT 转账测试
npm run test:signature      # 签名方法验证
npm run test:pnt 5          # 转账 5 PNT
npm run test:pnt-batch      # 批量转账测试
```

## 🚨 当前问题和解决方案

### Gas 费用优化 (2025-09-21)

**问题**: 日志显示持续的 InsufficientFees 错误
```
required_fees: { max_fee_per_gas: 100011071 }
actual_fees:   { max_fee_per_gas: 100005075 }
差异: ~6000 wei (0.006 Gwei)
```

**已应用的优化配置**:
```toml
# fly.toml 中的 gas 费用优化
MIN_PRIORITY_FEE_PER_GAS = "1000000000"      # 1 Gwei
MAX_FEE_PER_GAS_OVERHEAD = "20000000000"     # 20 Gwei overhead
GAS_FEE_ESTIMATION_BUFFER = "1.1"            # 10% buffer
```

**UserOperation 释放机制**:
- **超时释放**: 达到 `valid_until` 时间戳时自动移除
- **费用调整**: 用户重新提交更高费用的操作会替换旧操作
- **节流清理**: 被节流实体的操作在超过保留限制后清理
- **手动移除**: 通过 RPC 调用移除特定操作

**解决建议**:
1. 监控费用差异是否在可接受范围内
2. 如需进一步优化，可调整 `MAX_FEE_PER_GAS_OVERHEAD` 值
3. 考虑实现动态费用调整机制

## 📊 成功案例记录

### 已验证的转账案例
- **交易哈希**: 0xa601891378597635bba88ac797d63294fa7a60e6d37654c8c232d4291b7c7e01
- **转账金额**: 5 PNT (5000000000000000000 wei)
- **发送方**: SimpleAccount (0x6ff9A269085C79001e647b3D56C9176841A19935)
- **接收方**: Contract Account A (0x6ff9A269085C79001e647b3D56C9176841A19935)
- **余额变化**: 发送方 180→175 PNT, 接收方 328→333 PNT

### 技术突破
- **签名格式发现**: v0.6 使用 `toEthSignedMessageHash()` 而非 EIP-712
- **Gas 估算修复**: 必须使用真实签名进行 gas 估算
- **Bundler 集成**: 成功与 Fly.io 部署的 Rundler 集成

## 注意事项

- 所有开发工作在 main-base、pure-rundler-deploy 或 relay-dev 分支进行
- main 分支保持原始状态，不进行自定义修改
- 敏感信息已移除，使用 .env.example 模板管理配置
- node_modules 目录已全局忽略
- 部署监控显示应用运行正常，但仍有轻微的 gas 费用优化空间