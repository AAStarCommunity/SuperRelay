# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SuperRelay is an enterprise-grade Account Abstraction Paymaster solution built on top of Rundler (Alchemy's ERC-4337 bundler). It provides gas sponsorship services for decentralized applications through a modular, high-performance architecture.

**Key Technologies**: Rust (workspace), ERC-4337, Ethereum, Account Abstraction, JSON-RPC API, Swagger UI

## 🔒 SuperRelay Claude 工作规范 (基于 Jason Cursor Rules 0.16)
- 任何对话，请用中文回答
- 任何代码，都使用英文注释
- 完成任何任务，都使用最小修改原则
- 所有产品级别的规划和 todo，请拆分到 Plan 中，一步步执行
- Plan 拆分来源是 Features，Features 拆分来源是 Solution
- 以上文档如果没有，请创建

### 📁 文档体系结构 (严格遵守)
所有项目文档必须在 `docs/` 目录下，遵循以下结构：

- `docs/Solution.md` - **用户输入文档，绝对不可修改**
- `docs/Design.md` - 架构设计文档
- `docs/Features.md` - 核心功能描述
- `docs/Plan.md` - 开发计划
- `docs/Test.md` - 测试文档
- `docs/Changes.md` - 版本迭代记录
- `docs/Deploy.md` - 部署运维文档
- `docs/Install.md` - 用户安装文档
- `docs/UserCaseTest.md` - 用户测试用例
- `docs/comprehensive.md` - 综合评估报告

### 🔄 开发流程 (严格执行)
```
输入(Solution) -> 设计(Design) -> 拆解(Features) -> 计划(Plan) -> 开发迭代(Changes) -> 测试验证 -> 规范验证
```

### 📊 版本管理规范
- 初始版本：`0.1.0`
- 递增规则：`0.1.1 -> 0.1.2 -> 0.1.12 -> 0.2.1`
- 每次更新 `Changes.md` 必须更新版本号
- 在明确开始 0.2.0 开发前，都在完成 0.1.x 版本

### ⚠️ 核心开发约束
**最小影响范围原则**:
- 禁止擅自优化和扩张功能范围
- 缩小影响范围，再缩小影响范围
- 只针对提出的问题，使用最少代码修改
- 严格遵守指令，禁止修改任何不相关代码
- 禁止任何不相关优化

**模块化原则**:
- 新增功能能独立模块就不要在原有主流程文件完成
- 每个修改都说清楚为何这样做

### 🔧 SuperRelay 特定技术约束

**安全第一原则**:
- **私钥管理**: 测试 (.env) -> 生产 (环境变量) -> 未来 (硬件 API)
- **输入验证**: 所有 RPC 输入必须经过 `validation.rs`
- **速率限制**: Token bucket 算法，`config.toml`可配置
- **错误处理**: 使用 `ErrorObjectOwned`，不暴露敏感信息

**Rust 项目质量要求**:
```bash
# 每次完成todo后必须执行
cargo check --workspace
cargo test --workspace
./scripts/security_check.sh
./scripts/format.sh  # git提交前必须运行
git commit  # 禁止使用 --no-verify
```

**架构约束**:
- **Rundler 集成**: 扩展不修改，保持向后兼容
- **PaymasterRelay**: `Arc<T>`共享状态，异步优先
- **配置驱动**: 所有参数通过 `config.toml` 控制
- **模块化扩展**: 如 Security_filter 独立 crate

### 📝 代码质量标准
```rust
// ✅ 必须遵循的 RPC 方法模式
pub async fn safe_rpc_method(&self, input: Input) -> Result<Output, ErrorObjectOwned> {
    // 1. Input validation first
    self.validator.validate_input(&input)?;

    // 2. Rate limiting check
    if !self.rate_limiter.check_rate_limit(client_ip) {
        return Err(rate_limit_error());
    }

    // 3. Business logic
    let result = self.process(input).await?;

    Ok(result)
}

// ❌ 绝对禁止
pub fn unsafe_method(&self) -> String {
    self.private_key.unwrap() // 暴露敏感数据 + panic 风险
}
```

### 🧪 测试和验证要求
**测试层级**:
- **用户视角**: 产品 Features 验证
- **产品方案视角**: 业务流程测试
- **系统视角**: 技术组件测试

**自动化要求**:
- Rust 项目：`cargo build && cargo test`
- 每次修改后运行并确认无错误
- 编译测试部署指令写入 `DEPLOY.md`

### 🏗️ 架构设计原则
- **少侵入**: 最小化对原有代码的修改
- **隔离原有**: 新功能独立模块实现
- **高效通信**: 组件间清晰的接口设计
- **可扩展**: 支持新增安全过滤等全局模块
- **结构清晰**: 业务组件和技术组件分离
- **数据统一**: 统一的数据结构和传递格式
- **安全检查**: 全流程安全验证
- **容错重试**: 无状态可重复操作

### 🔍 失败处理策略
如果同样思路三次对话后还是失败：
1. **反思**: 分析失败原因
2. **改变思路**: 尝试其他技术方案
3. **拆分问题**: 将复杂问题分解为更小问题

### 📋 里程碑管理
- **小版本完成** (0.1.11->0.1.12): 更新 `Changes.md`
- **大版本完成** (0.1->0.2): 完成相关文档更新
- **功能完成**: 及时记录到版本迭代中

### 🎯 SuperRelay 特定目标
- **编译时间**: < 2 分钟 full workspace build
- **RPC 响应**: < 200ms p95 延迟
- **内存使用**: < 500MB 稳态运行
- **安全扫描**: 0 critical/high issues
- **测试覆盖**: > 80% 核心业务逻辑

### 💡 Claude 执行方式
每次执行都必须：
1. 检查是否需要更新 `docs/` 目录文档
2. 遵循最小影响范围原则
3. 完成测试和格式化
4. 更新 `Changes.md` 和版本号
5. 说明修改原因和影响范围

### 🚀 项目记忆
- 每次修改完成到一阶段后,在汇报成果进度之前,请运行cargo check,build(例如cargo check --package super-relay;cargo build --package super-relay --package rundler --release)和format.sh,确认代码都正常,有错误请fix后再次运行,直到无错误

## Task Master AI Instructions
**Import Task Master's development workflow commands and guidelines, treat as if import is in the main CLAUDE.md file.**
@./.taskmaster/CLAUDE.md
