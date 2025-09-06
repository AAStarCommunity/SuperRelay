# TODO: 真实 TEE 环境集成测试计划

## 🔴 待办事项：SuperRelay ↔ AirAccount 真实 TEE 集成

### 当前测试状态
- ✅ **Mock 环境测试**: 所有集成测试通过 (5/5)
- ✅ **架构验证**: 双重签名流程设计验证完成
- 🔴 **真实 TEE 测试**: 需要连接实际 OP-TEE 硬件环境

### 真实环境测试要求

#### 1. 硬件环境依赖
```yaml
Required Hardware:
  - ARM TrustZone enabled device (e.g., Raspberry Pi 4)
  - OP-TEE OS installation
  - TEE Trusted Application (TA) deployment

Network Setup:
  - SuperRelay service (Rust)
  - AirAccount KMS service (Node.js + OP-TEE)
  - 网络连接配置
```

#### 2. 需要扩展的测试用例

```rust
// crates/paymaster-relay/src/integration_tests.rs
impl DualSignatureIntegrationTest {
    /// 真实 TEE 环境集成测试
    pub async fn test_real_tee_integration(&self) -> Result<()> {
        // TODO: 实现以下测试步骤

        // 1. 验证 AirAccount KMS 服务可用性
        self.verify_airaccount_kms_connectivity().await?;

        // 2. 测试真实 TEE 签名响应
        self.test_real_tee_signature_generation().await?;

        // 3. 验证硬件证明 (Hardware Attestation)
        self.verify_hardware_attestation().await?;

        // 4. 测试密钥轮换通知到真实 TEE
        self.test_key_rotation_with_real_tee().await?;

        // 5. 性能基准测试
        self.run_performance_benchmarks().await?;

        Ok(())
    }

    async fn verify_airaccount_kms_connectivity(&self) -> Result<()> {
        // TODO: 检查 AirAccount KMS 服务状态
        // - HTTP 连接测试
        // - TEE 设备状态查询
        // - 授权验证
        Ok(())
    }

    async fn test_real_tee_signature_generation(&self) -> Result<()> {
        // TODO: 测试真实 TEE 签名
        // - 构建真实双重签名请求
        // - 发送到 AirAccount KMS
        // - 验证返回的 TEE 签名格式和有效性
        // - 确认签名可以被 ethers.js 验证
        Ok(())
    }

    async fn verify_hardware_attestation(&self) -> Result<()> {
        // TODO: 验证硬件证明
        // - 检查 TEE 设备 ID 真实性
        // - 验证硬件证明链
        // - 确认签名来源于真实硬件
        Ok(())
    }

    async fn test_key_rotation_with_real_tee(&self) -> Result<()> {
        // TODO: 测试与真实 TEE 的密钥轮换
        // - 触发 PaymasterKeyManager 密钥轮换
        // - 验证通知成功发送到 AirAccount
        // - 确认 TEE 端接收并处理轮换通知
        Ok(())
    }

    async fn run_performance_benchmarks(&self) -> Result<()> {
        // TODO: 性能基准测试
        // - 测量签名延迟 (目标: <500ms)
        // - 测量吞吐量 (目标: >10 TPS)
        // - 内存和 CPU 使用率监控
        Ok(())
    }
}
```

### 环境配置要求

#### AirAccount KMS 服务配置
```javascript
// 需要在 AirAccount 服务中配置真实 TEE
const teeConfig = {
  teeDevicePath: '/dev/teepriv0',
  taUuid: 'your-ta-uuid-here',
  maxRetries: 3,
  timeoutMs: 5000
};
```

#### SuperRelay 测试配置
```rust
// 需要配置真实的 AirAccount KMS URL
let kms_client = AirAccountKmsClient::new(
    "http://real-airaccount-kms:3000".to_string(),  // 真实服务地址
    key_manager,
);
```

### 测试步骤

#### Phase 1: 环境准备 (预计 1-2 天)
1. **硬件准备**
   - [ ] 获取支持 ARM TrustZone 的硬件设备
   - [ ] 安装 OP-TEE 开发环境
   - [ ] 部署 TEE Trusted Application

2. **服务部署**
   - [ ] 在 TEE 硬件上部署 AirAccount KMS 服务
   - [ ] 配置 SuperRelay 连接真实 KMS 端点
   - [ ] 验证网络连通性

#### Phase 2: 集成测试 (预计 2-3 天)
1. **基础连接测试**
   ```bash
   # 运行真实环境集成测试
   cargo test --package rundler-paymaster-relay \
     --features integration-tests,real-tee-testing \
     test_real_tee_integration
   ```

2. **功能测试**
   - [ ] 双重签名流程端到端测试
   - [ ] 密钥轮换通知测试
   - [ ] 错误处理和异常恢复测试

3. **性能测试**
   - [ ] 签名延迟基准测试
   - [ ] 并发处理能力测试
   - [ ] 长期稳定性测试

#### Phase 3: 安全验证 (预计 1-2 天)
1. **硬件证明验证**
   - [ ] TEE 设备身份验证
   - [ ] 签名真实性验证
   - [ ] 防重放攻击测试

2. **攻击测试**
   - [ ] 中间人攻击防护测试
   - [ ] 侧信道攻击防护验证
   - [ ] 异常输入处理测试

### 预期结果

#### 成功标准
- ✅ 所有真实 TEE 集成测试通过
- ✅ 签名延迟 < 500ms (P95)
- ✅ 吞吐量 > 10 TPS
- ✅ 硬件证明验证成功
- ✅ 24小时稳定性测试通过

#### 性能基准
```
Target Metrics:
- Signature Generation: < 500ms (P95)
- Request Throughput: > 10 TPS
- Memory Usage: < 100MB (steady state)
- CPU Usage: < 50% (during load)
- Error Rate: < 0.1%
```

### 风险和缓解措施

#### 高风险项
- **硬件可用性**: TEE 硬件获取和配置复杂性
  - 缓解: 提前准备多个硬件方案，包括云端 TEE 方案
- **TA 开发**: 需要专门的 TEE 开发知识
  - 缓解: 咨询 OP-TEE 社区，寻求专家支持

#### 中等风险项
- **性能瓶颈**: TEE 调用可能影响性能
  - 缓解: 实现连接池和缓存机制
- **网络稳定性**: 分布式测试环境的网络问题
  - 缓解: 添加重试机制和断路器模式

### 联系方式和资源

#### 技术支持
- **OP-TEE 社区**: https://github.com/OP-TEE/optee_os/discussions
- **ARM TrustZone 文档**: https://developer.arm.com/ip-products/security-ip/trustzone

#### 测试计划负责人
- 需要指定具备 TEE 开发经验的工程师
- 建议寻求 OP-TEE 社区或商业支持

---

**🚨 重要**: 在真实 TEE 测试完成之前，当前的集成测试结果仅能证明架构设计的正确性，不能保证生产环境的安全性和可靠性。真实 TEE 集成是系统安全的最后一道关键防线。