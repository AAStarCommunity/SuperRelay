# SuperRelay 用户测试用例

## 📋 测试概述

本文档包含SuperRelay所有核心功能的用户测试用例，覆盖用户视角、产品方案视角和系统视角的完整测试场景。

## 🎯 测试维度

### 用户视角测试
验证最终用户使用体验和功能可用性

### 产品方案视角测试
验证业务流程和产品价值实现

### 系统视角测试
验证技术架构和系统稳定性

---

## 🧪 核心功能测试用例

### TC001: Gas Sponsorship 基础流程

**测试目标**: 验证基础gas赞助功能
**测试级别**: 用户视角 + 产品方案视角

**前置条件**:
- SuperRelay服务运行正常
- Paymaster账户有充足ETH余额
- 测试网络(Anvil)运行中

**测试步骤**:
1. 用户构造UserOperation请求
2. 调用 `pm_sponsorUserOperation` API
3. 系统验证UserOperation有效性
4. Paymaster签名并返回sponsorship数据
5. 用户提交完整UserOperation到网络

**预期结果**:
- API返回成功响应包含paymaster字段
- UserOperation在区块链上成功执行
- Gas费用由Paymaster承担

**验证命令**:
```bash
# 1. 启动测试环境
./scripts/start_anvil.sh
./scripts/start_dev_server.sh

# 2. 执行sponsorship测试
./scripts/test_simple.sh

# 3. 验证结果
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pm_sponsorUserOperation","params":[{...}],"id":1}'
```

---

### TC002: Policy Engine 访问控制

**测试目标**: 验证策略引擎正确过滤请求
**测试级别**: 产品方案视角 + 系统视角

**前置条件**:
- 配置策略文件 `config/paymaster-policies.toml`
- 设置白名单/黑名单规则

**测试场景**:

**2.1 白名单测试**
- 白名单地址请求sponsorship → 应该成功
- 非白名单地址请求 → 应该被拒绝

**2.2 Gas限制测试**
- 正常gas限制请求 → 应该成功
- 超出gas限制请求 → 应该被拒绝

**2.3 速率限制测试**
- 正常频率请求 → 应该成功
- 高频攻击请求 → 应该被限制

**验证方法**:
```bash
# 编辑策略文件
vim config/paymaster-policies.toml

# 重启服务使策略生效
./scripts/restart_super_relay.sh

# 测试不同策略场景
./scripts/test_policy_engine.sh
```

---

### TC003: Multi-Version EntryPoint 兼容性

**测试目标**: 验证v0.6和v0.7 EntryPoint支持
**测试级别**: 系统视角

**测试场景**:
- **v0.6测试**: 使用EntryPoint v0.6合约测试完整流程
- **v0.7测试**: 使用EntryPoint v0.7合约测试完整流程
- **并发测试**: 同时处理v0.6和v0.7请求

**验证命令**:
```bash
# 运行v0.6兼容性测试
./test/spec-tests/local/run-spec-tests-v0_6.sh

# 运行v0.7兼容性测试
./test/spec-tests/local/run-spec-tests-v0_7.sh

# 运行并发测试
./scripts/test_multi_version.sh
```

---

### TC004: API 完整性和文档一致性

**测试目标**: 验证API功能和Swagger文档一致性
**测试级别**: 用户视角 + 系统视角

**测试内容**:
1. **Swagger UI可访问性**
   - 访问 http://localhost:9000/swagger-ui/
   - 所有API端点可正常显示和测试

2. **JSON-RPC API完整性**
   - 标准ERC-4337方法: `eth_sendUserOperation`, `eth_estimateUserOperationGas`
   - Paymaster扩展方法: `pm_sponsorUserOperation`, `pm_getPaymasterData`
   - 管理方法: `admin_clearMempool`, `debug_bundlerClearState`

3. **REST API健康检查**
   - `/health` 端点返回服务状态
   - `/metrics` 端点返回Prometheus指标

**验证脚本**:
```bash
# API完整性测试
./scripts/test_api_completeness.sh

# Swagger文档验证
curl http://localhost:9000/swagger-ui/ | grep -i "SuperRelay"
```

---

### TC005: 性能和压力测试

**测试目标**: 验证系统性能指标
**测试级别**: 系统视角

**性能要求**:
- RPC响应时间: < 200ms (p95)
- 并发处理: >= 100 RPS
- 内存使用: < 500MB 稳态
- 错误率: < 0.1%

**测试场景**:

**5.1 单用户性能测试**
```bash
# 测试单个RPC调用延迟
time curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'
```

**5.2 并发压力测试**
```bash
# 使用ab进行压力测试
ab -n 1000 -c 50 -T 'application/json' \
   -p test_payload.json \
   http://localhost:3000/

# 或使用专用脚本
./scripts/test_performance.sh
```

**5.3 内存泄漏测试**
```bash
# 长期运行监控内存使用
./scripts/test_memory_leak.sh
```

---

### TC006: 安全性测试

**测试目标**: 验证系统安全性
**测试级别**: 系统视角 + 产品方案视角

**安全测试内容**:

**6.1 私钥安全测试**
```bash
# 确保没有硬编码私钥
./scripts/security_check.sh

# 检查日志不泄露敏感信息
grep -r "private" logs/ | grep -v "INFO\|DEBUG"
```

**6.2 输入验证测试**
- 恶意UserOperation请求 → 应该被拒绝
- 超长输入数据 → 应该被正确处理
- 无效签名数据 → 应该被检测

**6.3 DDoS防护测试**
```bash
# 高频请求攻击测试
./scripts/test_ddos_protection.sh

# 验证速率限制生效
curl -X POST http://localhost:3000 -d '...' # 重复执行
```

---

### TC007: 容错和恢复测试

**测试目标**: 验证系统容错能力
**测试级别**: 系统视角

**故障场景**:

**7.1 网络故障测试**
- Ethereum节点连接中断 → 系统应该优雅降级
- 恢复连接后 → 系统应该自动恢复

**7.2 资源不足测试**
- Paymaster余额不足 → 返回明确错误信息
- 系统内存不足 → 应该有适当的限流

**7.3 配置错误测试**
- 错误的配置文件 → 启动时报错并退出
- 运行时配置修改 → 应该动态生效

**验证方法**:
```bash
# 网络故障模拟
./scripts/test_network_failure.sh

# 资源限制测试
./scripts/test_resource_limits.sh
```

---

## 📊 测试执行计划

### 自动化测试流程
```bash
# 完整测试套件执行
make test-all

# 分类测试执行
make test-unit          # 单元测试
make test-integration   # 集成测试
make test-spec          # 规范测试
make test-performance   # 性能测试
make test-security      # 安全测试
```

### 手工测试检查清单

**部署前检查** ✅:
- [ ] 所有自动化测试通过
- [ ] 安全扫描无高危问题
- [ ] 性能指标满足要求
- [ ] API文档完整准确
- [ ] 监控和日志正常

**用户验收测试** ✅:
- [ ] Demo流程完整可用
- [ ] Swagger UI可正常使用
- [ ] 错误信息用户友好
- [ ] 响应时间满足预期

## 🔍 测试结果评判标准

### 通过标准
- 所有测试用例100%通过
- 性能指标达到设计要求
- 安全扫描无关键问题
- 用户体验满足预期

### 失败处理
- 记录失败原因和复现步骤
- 分配给相关开发人员修复
- 修复后重新执行测试
- 更新测试用例避免回归

## 📈 测试报告模板

每次测试执行后生成报告:
```
测试时间: YYYY-MM-DD HH:MM:SS
测试版本: v0.1.x
测试环境: 开发/测试/生产

执行结果:
- 总用例数: XX
- 通过用例: XX
- 失败用例: XX
- 跳过用例: XX

性能指标:
- 平均响应时间: XXXms
- 最高QPS: XXX
- 内存使用: XXXMb

问题列表:
- 问题1: 描述及影响
- 问题2: 描述及影响

建议:
- 改进建议1
- 改进建议2
```