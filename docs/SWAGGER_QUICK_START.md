# Swagger UI 快速开始指南

## 🎯 目标

通过 Swagger UI 对 SuperRelay 进行真实的 API 测试，包含完整的示例数据。

## 🚀 快速启动

### 1. 启动服务（需要两个终端）

**终端 1 - 启动 SuperRelay 主服务：**
```bash
./scripts/start_superrelay.sh
```

等待看到 `✅ Anvil started` 和服务完全启动的消息。

**终端 2 - 启动 Web UI：**
```bash
./scripts/start_web_ui.sh
```

等待看到 `✨ Web UI server starting on port 9000...`

### 2. 访问 Swagger UI

在浏览器中打开：http://localhost:9000/

### 3. 测试 API

1. **找到 API 端点**：
   - 展开 `Paymaster API` 分组
   - 找到 `POST /sponsorUserOperation` 端点

2. **开始测试**：
   - 点击端点展开详情
   - 点击 **"Try it out"** 按钮
   - 🎉 **所有字段已预填充真实测试数据**

3. **执行请求**：
   - 直接点击 **"Execute"** 按钮
   - 请求会发送到真实的 SuperRelay 服务 (localhost:3000)
   - 查看真实的响应数据

## 📋 示例数据说明

### 请求数据 (已预填充)
```json
{
  "jsonrpc": "2.0",
  "method": "pm_sponsorUserOperation",
  "params": [
    {
      "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "nonce": "0x0",
      "initCode": "0x",
      "callData": "0xb61d27f6000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000",
      "callGasLimit": "0x30D40",
      "verificationGasLimit": "0x186A0",
      "preVerificationGas": "0xC350",
      "maxFeePerGas": "0x59682F00",
      "maxPriorityFeePerGas": "0x59682F00",
      "paymasterAndData": "0x",
      "signature": "0xfffffffffffffffffffffffffffffff0000000000000000000000000000000007aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1c"
    },
    "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
  ],
  "id": 1
}
```

### 期待响应
```json
{
  "jsonrpc": "2.0",
  "result": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000",
  "id": 1
}
```

## 🧪 验证测试

运行自动化测试脚本：
```bash
./scripts/test_swagger_api.sh
```

这会检查：
- ✅ 所有服务是否正常运行
- ✅ OpenAPI 规范是否包含真实数据
- ✅ API 端点是否可访问
- ✅ JSON-RPC 调用是否正常工作

## 🔧 故障排除

### 问题：Web UI 无法访问
**解决**：
```bash
# 检查端口 9000 是否被占用
lsof -i :9000

# 如果被占用，杀死进程后重启
pkill -f http-server
./scripts/start_web_ui.sh
```

### 问题：API 返回连接错误
**解决**：
1. 确保 SuperRelay 服务正在运行：
   ```bash
   curl -X POST http://localhost:3000 \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'
   ```

2. 如果没有响应，重启服务：
   ```bash
   ./scripts/start_superrelay.sh
   ```

### 问题：示例数据不显示
**解决**：
1. 检查 openapi.json 是否正确加载：
   ```bash
   curl http://localhost:9000/openapi.json | grep "pm_sponsorUserOperation"
   ```

2. 清除浏览器缓存并刷新页面

## 🎓 下一步

完成测试后，可以：
1. 修改示例数据测试不同场景
2. 查看其他 API 端点
3. 集成到你的前端应用

## 📚 相关文档

- [完整集成文档](SWAGGER_INTEGRATION.md)
- [API 开发指南](API_GENERATION_GUIDE.md)
- [服务启动指南](ServiceStartupGuide.md)