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

## 注意事项

- 所有开发工作在 main-base、pure-rundler-deploy 或 relay-dev 分支进行
- main 分支保持原始状态，不进行自定义修改
- 敏感信息已移除，使用 .env.example 模板管理配置
- node_modules 目录已全局忽略