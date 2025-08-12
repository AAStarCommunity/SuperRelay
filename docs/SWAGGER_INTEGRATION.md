# Swagger UI 真实交互集成指南

## 概述

本文档说明了如何配置 Swagger UI 以实现与 SuperRelay 服务的真实交互测试。

## 架构设计

```
用户浏览器
    ↓
Swagger UI (端口 9000)
    ↓
API 代理端点 (/api/v1/sponsor)
    ↓
转发请求到真实服务
    ↓
SuperRelay 主服务 (端口 3000)
    ↓
Anvil 本地区块链 (端口 8545)
```

## 主要改动

### 1. Swagger 服务器配置
- 修改了 OpenAPI 规范，使其指向真实的 SuperRelay 服务地址
- Swagger UI 现在默认使用 `http://localhost:3000` 作为后端服务器

### 2. API 代理实现
- `/api/v1/sponsor` 端点现在作为代理，将请求转发到真实的 SuperRelay 服务
- 添加了 reqwest 库用于 HTTP 请求转发
- 实现了完整的错误处理和服务状态检查

### 3. 示例数据增强
- 提供了真实可用的 UserOperation 测试数据
- 包含 v0.6 和 v0.7 两个版本的示例
- 所有字段都使用了合理的测试值

## 使用说明

### 启动服务

需要分别启动两个服务：

1. **启动 SuperRelay 主服务**：
```bash
./scripts/start_superrelay.sh
```

这将启动：
- Anvil 本地区块链 (端口 8545)
- SuperRelay 主服务 (端口 3000)

2. **启动 Web UI 服务**：
```bash
./scripts/start_web_ui.sh
```

这将启动：
- Swagger UI 服务 (端口 9000)
- 静态文件服务器
- 加载真实测试数据的 OpenAPI 文档

### 访问 Swagger UI

1. 打开浏览器访问：http://localhost:9000/swagger-ui
2. 你会看到完整的 API 文档界面
3. 服务器下拉框会显示 "SuperRelay Service (Real Backend)"

### 测试 API

1. **展开任意 API 端点**
   - 点击端点名称展开详情

2. **点击 "Try it out" 按钮**
   - 进入交互测试模式

3. **查看预填充的示例数据**
   - 所有字段都已填充真实可测试的数据
   - 可以直接使用或根据需要修改

4. **点击 "Execute" 执行请求**
   - 请求会发送到真实的 SuperRelay 服务
   - 你会看到真实的响应数据

### 验证测试

运行测试脚本验证所有功能：
```bash
./scripts/test_swagger_api.sh
```

测试脚本会验证：
- 所有服务是否正常运行
- API 端点是否可访问
- 数据转发是否正常
- 响应格式是否正确

## API 端点说明

### 主要端点

1. **POST /api/v1/sponsor**
   - 赞助 UserOperation
   - 转发到 SuperRelay 的 `pm_sponsorUserOperation` RPC 方法
   - 返回 paymasterAndData

2. **GET /health**
   - 健康检查端点
   - 返回服务状态和运行时间

3. **GET /metrics**
   - 性能指标端点
   - 返回请求统计和响应时间

4. **GET /examples/{version}**
   - 获取示例数据
   - 支持 v06、v07 版本

5. **GET /dashboard**
   - 访问管理仪表板
   - 提供可视化的服务状态监控

## 错误处理

如果遇到连接错误，Swagger UI 会返回友好的错误信息：

```json
{
  "code": -32603,
  "message": "SuperRelay service is not running. Please start it with: ./scripts/start_superrelay.sh",
  "data": {
    "error": "Connection refused",
    "service_url": "http://localhost:3000",
    "hint": "Run './scripts/start_superrelay.sh' to start the service"
  }
}
```

## 开发提示

1. **修改示例数据**
   - 编辑 `crates/paymaster-relay/src/api_schemas.rs` 中的 `examples` 模块

2. **添加新端点**
   - 在 `swagger.rs` 中添加新的路由
   - 确保代理到正确的后端服务

3. **调试请求**
   - 查看浏览器开发者工具的网络标签
   - 检查 SuperRelay 服务日志

## 注意事项

1. 确保所有端口没有被占用：
   - 3000: SuperRelay 主服务
   - 8545: Anvil 区块链
   - 9000: Swagger UI

2. 如果修改了 API 结构，需要重新编译：
   ```bash
   cargo build --package rundler-paymaster-relay
   ```

3. Swagger UI 会缓存 OpenAPI 规范，如有更新需要刷新浏览器

## 故障排除

### 问题：Swagger UI 无法连接到服务

**解决方案：**
1. 确认 SuperRelay 服务正在运行
2. 检查端口 3000 是否可访问
3. 查看服务日志是否有错误

### 问题：请求返回 "Service Unavailable"

**解决方案：**
1. 运行 `./scripts/start_superrelay.sh` 启动服务
2. 等待服务完全启动（约 5-10 秒）
3. 重试请求

### 问题：示例数据不正确

**解决方案：**
1. 检查 Anvil 是否正在运行
2. 确认使用的是正确的测试账户地址
3. 验证 EntryPoint 合约地址是否正确