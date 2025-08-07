# SuperRelay 对外服务访问指南

## 🌐 当前服务架构验证结果

### ✅ **已验证的对外服务**

#### 1. SuperRelay Gateway API (主要服务)
- **端口**: `3000`
- **状态**: ✅ 运行中
- **访问方式**:
  ```bash
  # 健康检查
  curl http://localhost:3000/health
  
  # RPC API调用
  curl -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"eth_supportedEntryPoints","params":[]}'
  
  # Paymaster API
  curl -X POST http://localhost:3000 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"pm_sponsorUserOperation","params":[...]}'
  ```

#### 2. Swagger UI (独立Web服务)
- **端口**: `9000`
- **状态**: ✅ 独立启动，不在Rust进程内
- **启动方式**:
  ```bash
  # 独立启动Swagger UI
  ./scripts/start_web_ui.sh
  ```
- **访问方式**:
  ```bash
  # Web界面
  open http://localhost:9000/
  
  # OpenAPI规范
  curl http://localhost:9000/openapi.json
  ```

#### 3. 健康检查端点
- **路径**: `http://localhost:3000/health`
- **状态**: ✅ 正常工作
- **响应示例**:
  ```json
  {
    "status": "healthy",
    "components": {
      "gateway": { "status": "healthy" },
      "paymaster": { "status": "healthy" },
      "pool": { "status": "healthy" },
      "router": { "status": "healthy" }
    }
  }
  ```

#### 4. 监控指标端点
- **路径**: `http://localhost:3000/metrics`
- **状态**: ✅ 正常工作
- **格式**: Prometheus格式

### 📋 **服务启动流程**

#### 完整启动流程：
```bash
# 1. 启动主服务（包含Gateway + Paymaster + Rundler）
./scripts/start_superrelay.sh --skip-build

# 2. 启动独立Web UI（可选）
./scripts/start_web_ui.sh

# 3. 验证服务状态
./scripts/test_suite.sh
```

## 📊 **端口使用总览**

| 服务组件 | 端口 | 进程类型 | 启动脚本 | 状态 |
|----------|------|----------|----------|------|
| SuperRelay Gateway | 3000 | Rust binary | `start_superrelay.sh` | ✅ 单一进程 |
| Swagger UI | 9000 | Node.js http-server | `start_web_ui.sh` | ✅ 独立进程 |
| Anvil (开发环境) | 8545 | Foundry | 自动启动 | ✅ 独立进程 |

## 🔧 **架构特点确认**

### ✅ **Swagger UI独立部署确认**
- **是的，Swagger UI确实需要独立启动**
- **技术栈**: Node.js + http-server + Swagger UI静态文件
- **不在Rust进程内运行**
- **好处**: 前后端分离，可独立更新和扩展

### ✅ **服务访问方式确认**
- **单一Gateway端点**: `http://localhost:3000` (所有API调用)
- **独立文档界面**: `http://localhost:9000` (Swagger UI)
- **内部架构**: Gateway内部路由到Paymaster/Rundler组件

## 🎯 **README更新内容**

### 已修复的问题：
1. ✅ 更正了Swagger UI访问路径：`http://localhost:9000/swagger-ui/` → `http://localhost:9000/`
2. ✅ 确认了服务独立启动的方式
3. ✅ 验证了所有对外端点的可访问性

### 推荐的用户操作流程：
```bash
# 日常开发流程
./scripts/start_superrelay.sh --skip-build    # 启动主服务
./scripts/start_web_ui.sh                     # 启动文档界面
./scripts/test_suite.sh                       # 验证功能

# 访问服务
curl http://localhost:3000/health             # 健康检查  
open http://localhost:9000/                   # API文档
```

---

**总结**: SuperRelay采用了前后端分离的架构设计，Swagger UI确实需要独立启动，这是正确的设计选择，提供了更好的灵活性和扩展性。