# SuperRelay + AirAccount 集成开发计划

## 版本: 0.1.12

## 总体架构策略

**统一分支架构**: 采用单一代码库配置驱动的 KMS 切换方案，避免维护多个分支的复杂性

**实施路径**:
1. **Phase 1**: Standalone 模式 (AWS KMS + Remote AirAccount)
2. **Phase 2**: Integrated 模式 (Full TEE Integration)
3. **Future**: 高级安全特性 (BLS聚合签名、合约安全规则)

---

## Phase 1: Standalone 模式 (高优先级)

**目标**: 基于现有基础设施实现多重验证架构，使用 AWS KMS 处理 Paymaster 签名，Remote AirAccount 处理用户密钥管理

### H1 - TEE 安全引擎核心 (Critical)
- **H1.1** 实现 TEE 安全引擎核心功能
  - 黑名单检测 (地址、交易模式)
  - 钓鱼网站检测 (URL pattern matching)
  - 异常检测 (交易频率、金额阈值)
  - 安全规则验证引擎
  - **预计工时**: 8-12小时
  - **关键依赖**: TEE 环境配置

### H2 - 核心链路优化 (Critical)
- **H2.1** 优化 Gateway-Pool-Bundler 完整链路
  - 修复 LocalBuilderHandle 集成问题
  - 优化 UserOperation 流转性能
  - 确保 EntryPoint 版本兼容性
  - **预计工时**: 6-8小时
  - **关键依赖**: Bundler 接口稳定

- **H2.2** 修复硬编码RPC URL问题
  - 统一使用 .env 配置文件
  - 移除所有硬编码的 RPC 端点
  - 支持多网络配置 (Mainnet, Sepolia, Local)
  - **预计工时**: 2-3小时
  - **关键依赖**: 配置管理重构

- **H2.3** 标准化ECDSA签名格式
  - 统一签名序列化/反序列化
  - 确保与 ERC-4337 规范兼容
  - 处理不同版本的签名格式差异
  - **预计工时**: 3-4小时
  - **关键依赖**: 签名库标准化

### H3 - 协议扩展 (High)
- **H3.1** 扩展PackedUserOperation v0.7/v0.8支持
  - 实现 v0.7 PackedUserOperation 解析
  - 实现 v0.8 PackedUserOperation 解析
  - 版本自动检测和路由
  - **预计工时**: 4-6小时
  - **关键依赖**: EntryPoint 合约部署

### Phase 1 验证标准
- [ ] 所有 RPC 调用使用配置文件中的端点
- [ ] TEE 安全引擎能够检测基础威胁
- [ ] Gateway → Pool → Bundler 链路正常工作
- [ ] 支持 EntryPoint v0.6/v0.7/v0.8 三个版本
- [ ] ECDSA 签名在所有场景下格式一致
- [ ] Standalone 模式端到端流程可用

---

## Phase 2: Integrated 模式 (中优先级)

**目标**: 基于 Phase 1 的稳定基础，实现完全 TEE 集成模式

### M1 - 数据安全强化 (Medium)
- **M1.1** 用户数据安全加密改进
  - 实现 PBKDF2 密钥派生
  - AES-256-GCM 数据加密
  - 安全密钥存储机制
  - **预计工时**: 4-5小时
  - **关键依赖**: TEE 密钥管理

### M2 - 完整性测试 (Medium)
- **M2.1** 端到端测试和验证
  - 完整用户流程自动化测试
  - 性能基准测试 (< 200ms p95)
  - 安全渗透测试
  - **预计工时**: 6-8小时
  - **关键依赖**: Phase 1 功能完整

### M3 - 企业特性 (Medium)
- **M3.1** 企业级特性实现
  - 监控和指标收集
  - 日志审计系统
  - 配置管理界面
  - **预计工时**: 8-10小时
  - **关键依赖**: 基础架构稳定

### M4 - 生产部署 (Medium)
- **M4.1** 真实 TEE 环境部署
  - 生产级 TEE 配置
  - 密钥管理硬件集成
  - 监控和告警系统
  - **预计工时**: 10-12小时
  - **关键依赖**: 硬件环境就绪

### Phase 2 验证标准
- [ ] 数据加密符合企业安全标准
- [ ] 端到端测试覆盖率 > 80%
- [ ] 生产环境性能达标
- [ ] TEE 集成模式稳定运行
- [ ] 企业级监控和审计功能可用

---

## Future Roadmap (预留功能)

### 高级安全特性 (暂不实施)
- **F1** BLS聚合签名防护机制
  - 6个验证器，4个最小阈值
  - 多方计算签名聚合
  - 拜占庭容错机制

- **F2** 合约账户安全规则
  - 每日限额控制
  - 单笔交易限制
  - 多签治理机制
  - 紧急暂停功能

### 技术债务
- [ ] 代码覆盖率提升到 > 90%
- [ ] 性能优化 (编译时间 < 2分钟)
- [ ] 内存使用优化 (< 500MB 稳态)
- [ ] 安全审计 (0 critical/high issues)

---

## 当前状态

**活跃分支**: `feature/super-relay`
**当前版本**: 0.1.12
**下一版本**: 0.1.13 (完成 Phase 1 的 H1.1 和 H2.1)

### 关键配置文件
- `.env`: 网络配置和合约地址
- `config.toml`: 服务配置参数
- `CLAUDE.md`: AI 开发规范
- `docs/flow.md`: 架构设计文档

### 技术栈
- **Core**: Rust (workspace架构)
- **Protocol**: ERC-4337, Account Abstraction
- **Security**: TEE, AWS KMS, ECDSA签名
- **Network**: JSON-RPC API, Swagger UI
- **Testing**: cargo test, 集成测试

---

## 执行顺序

1. **立即执行 (本周)**:
   - H1.1: TEE 安全引擎核心功能
   - H2.1: Gateway-Pool-Bundler 完整链路优化

2. **短期执行 (下周)**:
   - H2.2: 修复硬编码RPC URL问题
   - H2.3: 标准化ECDSA签名格式
   - H3.1: 扩展PackedUserOperation v0.7/v0.8支持

3. **中期执行 (2-3周)**:
   - 完成 Phase 1 验证
   - 开始 Phase 2 实施

4. **长期规划 (1-2个月)**:
   - Future 功能评估和设计
   - 生产环境部署准备

---

*最后更新: 2025-09-06*
*负责人: Claude Code + 用户协作*