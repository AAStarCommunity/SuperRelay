# AirAccount KMS 模块与 SuperRelay 集成方案

**版本**: 1.0
**日期**: 2024-12
**目标**: 将 AirAccount 剥离为独立的 KMS 服务模块，供 SuperRelay Paymaster 调用

---

## 🎯 核心理解确认

### AirAccount KMS 服务边界

基于架构分析，您提到的 **KMS 模块 = CA + SDK + TA** 的边界完全正确：

```
┌─────────────────────────────────────────────────────────────┐
│                    AirAccount KMS 服务                       │
├─────────────────────────────────────────────────────────────┤
│  📡 CA (Client Application)                                 │
│  ├── airaccount-ca-nodejs (HTTP/JSON API)                  │
│  ├── airaccount-ca (Rust CLI/API)                          │
│  └── 对外 API: 钱包创建、签名、WebAuthn集成                    │
├─────────────────────────────────────────────────────────────┤
│  🔌 SDK (通信桥接层)                                          │
│  ├── Apache Teaclave TrustZone SDK                         │
│  ├── optee-teec (TEE Client API)                           │
│  └── bincode 序列化协议 (参考 eth_wallet)                    │
├─────────────────────────────────────────────────────────────┤
│  🔒 TA (Trusted Application - 核心KMS)                      │
│  ├── airaccount-ta-simple (基于 eth_wallet 核心)            │
│  ├── 私钥生成、存储、签名 (在 Secure World)                   │
│  └── WebAuthn 认证集成                                     │
└─────────────────────────────────────────────────────────────┘
                            ↓ ARM TEE 环境要求
            ┌─────────────────────────────────────┐
            │  🖥️ ARM + OP-TEE 硬件平台要求        │
            │  ├── ARM TrustZone 支持              │
            │  ├── OP-TEE OS 运行环境              │
            │  └── /dev/teepriv0 设备节点         │
            └─────────────────────────────────────┘
```

## 📋 精确的 KMS 边界定义

### 1. CA 层边界 (Client Application)

**现有实现**:
```
packages/airaccount-ca-nodejs/  (主要 - Node.js HTTP API)
├── src/index.ts               (Express 服务器)
├── src/routes/
│   ├── account-abstraction.ts (ERC-4337 集成 ✅)
│   ├── webauthn.ts           (Passkey 认证 ✅)
│   ├── wallet.ts             (钱包管理 ✅)
│   └── auth.ts               (会话管理 ✅)
├── src/services/
│   ├── tee-client.ts         (TEE 通信客户端 ✅)
│   ├── webauthn.ts           (WebAuthn 服务 ✅)
│   └── database.ts           (SQLite 数据存储 ✅)

packages/airaccount-ca/        (辅助 - Rust CLI)
├── src/main.rs               (CLI 工具 + 测试)
├── src/wallet_test.rs        (钱包功能测试)
└── src/webauthn_service.rs   (认证服务)
```

**KMS 服务 API** (已实现的15个端点):
- `POST /aa/create-account` - 创建 ERC-4337 账户
- `POST /aa/execute-transaction` - 执行交易（支持 Paymaster）
- `POST /aa/execute-batch` - 批量交易执行
- `POST /webauthn/register` - Passkey 注册
- `POST /webauthn/authenticate` - 生物识别认证
- `POST /wallet/create` - 钱包创建
- `POST /wallet/sign` - 交易签名 ⭐ **核心功能**

### 2. SDK 层边界 (通信协议)

**基于 eth_wallet 标准** (已验证兼容):
```rust
// Apache Teaclave SDK 通信模式
fn invoke_command(command: proto::Command, input: &[u8]) -> Result<Vec<u8>> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(proto::UUID)?;
    let mut session = ctx.open_session(uuid)?;

    // 标准 4 参数模式
    let p0 = ParamTmpRef::new_input(input);         // 输入数据
    let p1 = ParamTmpRef::new_output(output.as_mut_slice()); // 输出缓冲
    let p2 = ParamValue::new(0, 0, ParamType::ValueInout);   // 输出长度
    let p3 = ParamNone;

    let mut operation = Operation::new(0, p0, p1, p2, p3);
    session.invoke_command(command as u32, &mut operation)
}
```

**序列化协议**: `bincode` (与 eth_wallet 完全一致)

### 3. TA 层边界 (Trusted Application)

**核心实现路径**:
```
packages/airaccount-ta-simple/src/main.rs  (主要 TA)
├── 基础命令支持:
│   ├── CMD_HELLO_WORLD: 0          (连接测试)
│   ├── CMD_ECHO: 1                 (通信测试)
│   ├── CMD_GET_VERSION: 2          (版本信息)
├── 钱包核心命令:
│   ├── CMD_CREATE_WALLET: 10       (创建钱包)
│   ├── CMD_DERIVE_ADDRESS: 12      (地址派生)
│   ├── CMD_SIGN_TRANSACTION: 13    (交易签名 ⭐)
│   ├── CMD_GET_WALLET_INFO: 14     (钱包信息)
│   └── CMD_LIST_WALLETS: 15        (钱包列表)
└── WebAuthn 混合认证:
    ├── CMD_CREATE_HYBRID_ACCOUNT: 20   (Passkey集成账户)
    ├── CMD_SIGN_WITH_HYBRID_KEY: 21    (生物识别签名 ⭐)
    └── CMD_VERIFY_SECURITY_STATE: 22   (安全状态验证)
```

**安全模块** (已实现):
```rust
mod security {
    pub mod constant_time {      // 防时序攻击
        pub struct SecureBytes;  // 常时字节比较
        pub struct ConstantTimeOps; // 常时操作工具
    }
    // ... 其他安全模块
}
```

## 🔗 SuperRelay 集成的 KMS API 设计

### 🔐 双重签名安全方案（分层信任模型）

**核心原则**：
- **双重签名验证**：用户 Passkey 签名 + SuperPaymaster 业务签名
- **意图与业务分离**：用户意图由 Passkey 保证，业务规则由 Paymaster 验证
- **防私钥泄露**：即使 Paymaster 私钥泄露，无法伪造用户签名
- **去中心化信任**：不依赖中心化 API 密钥
- **防重放攻击**：包含 nonce 和时间戳

### 方案：双重签名 API（推荐）

**SuperRelay 双重签名流程**:
```typescript
// SuperRelay PaymasterRelayService 中新增
export class AirAccountKmsClient {
  private baseUrl: string;  // AirAccount KMS 服务地址
  private signerPrivateKey: string; // SuperRelay 的签名私钥

  async signUserOperation(
    userOp: UserOperation,
    accountId: string,
    userPasskeySignature: string,  // 用户的 Passkey 签名
    userPublicKey: string,  // 用户的公钥
  ): Promise<{ signature: string; userOpHash: string }> {
    // 1. 验证业务规则（余额、会员状态等）
    const businessCheck = await this.validateBusinessRules(accountId);
    if (!businessCheck.approved) {
      throw new Error(`Business validation failed: ${businessCheck.reason}`);
    }

    // 2. 构建请求数据（包含用户签名）
    const requestData = {
      userOperation: userOp,
      accountId,
      signatureFormat: 'erc4337',
      userSignature: userPasskeySignature,  // 用户的 Passkey 签名
      userPublicKey: userPublicKey,  // 用户的公钥
      businessValidation: {
        balance: businessCheck.balance,
        membershipLevel: businessCheck.membershipLevel,
        approvedAt: businessCheck.timestamp
      },
      nonce: Date.now(), // 防重放攻击
      timestamp: Math.floor(Date.now() / 1000)
    };

    // 3. SuperPaymaster 对请求签名（包含用户签名的哈希）
    const messageToSign = ethers.utils.solidityKeccak256(
      ['bytes32', 'string', 'bytes32', 'uint256', 'uint256'],
      [
        getUserOperationHash(userOp),
        accountId,
        ethers.utils.keccak256(userPasskeySignature),  // 包含用户签名的哈希
        requestData.nonce,
        requestData.timestamp
      ]
    );

    const signer = new ethers.Wallet(this.signerPrivateKey);
    const paymasterSignature = await signer.signMessage(ethers.utils.arrayify(messageToSign));

    // 4. 发送双重签名请求
    const response = await fetch(`${this.baseUrl}/kms/sign-user-operation`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Paymaster-Signature': paymasterSignature,  // Paymaster 签名
        'X-Paymaster-Address': signer.address,  // Paymaster 公钥地址
      },
      body: JSON.stringify(requestData)
    });

    return response.json();
  }

  // 业务规则验证
  async validateBusinessRules(accountId: string) {
    // 检查账户余额、会员状态、限额等
    const account = await this.getAccountInfo(accountId);
    return {
      approved: account.balance > MIN_BALANCE && account.isActive,
      balance: account.balance,
      membershipLevel: account.membershipLevel,
      reason: null,
      timestamp: Date.now()
    };
  }
}
```

**AirAccount KMS 双重签名验证端点**:
```typescript
// 在 airaccount-ca-nodejs 中新增路由
app.post('/kms/sign-user-operation', async (req, res) => {
  const {
    userOperation,
    accountId,
    signatureFormat,
    userSignature,  // 用户 Passkey 签名
    userPublicKey,  // 用户公钥
    businessValidation,  // 业务验证信息
    nonce,
    timestamp
  } = req.body;

  const paymasterSignature = req.headers['x-paymaster-signature'];
  const paymasterAddress = req.headers['x-paymaster-address'];

  // 1. 验证时间戳（防重放，5分钟有效期）
  const currentTime = Math.floor(Date.now() / 1000);
  if (Math.abs(currentTime - timestamp) > 300) {
    return res.status(401).json({ error: 'Request expired' });
  }

  // 2. 验证 nonce 唯一性（防重放）
  if (await nonceStore.exists(nonce)) {
    return res.status(401).json({ error: 'Nonce already used' });
  }
  await nonceStore.add(nonce, { ttl: 600 }); // 10分钟过期

  // 3. 验证 Paymaster 签名（第一层验证）
  const paymasterMessage = ethers.utils.solidityKeccak256(
    ['bytes32', 'string', 'bytes32', 'uint256', 'uint256'],
    [
      getUserOperationHash(userOperation),
      accountId,
      ethers.utils.keccak256(userSignature),  // 包含用户签名的哈希
      nonce,
      timestamp
    ]
  );

  const recoveredPaymasterAddress = ethers.utils.verifyMessage(
    ethers.utils.arrayify(paymasterMessage),
    paymasterSignature
  );

  if (recoveredPaymasterAddress.toLowerCase() !== paymasterAddress.toLowerCase()) {
    return res.status(401).json({ error: 'Invalid Paymaster signature' });
  }

  // 4. 验证 Paymaster 是否被授权
  const authorizedPaymasters = await getAuthorizedPaymasters();
  if (!authorizedPaymasters.includes(paymasterAddress.toLowerCase())) {
    return res.status(403).json({ error: 'Paymaster not authorized' });
  }

  // 5. 验证用户 Passkey 签名（第二层验证）
  const userOpHash = getUserOperationHash(userOperation);
  const userMessageHash = ethers.utils.solidityKeccak256(
    ['bytes32', 'string'],
    [userOpHash, accountId]
  );

  // 验证用户的 Passkey 签名
  const isValidUserSignature = await verifyPasskeySignature(
    userSignature,
    userPublicKey,
    userMessageHash,
    accountId
  );

  if (!isValidUserSignature) {
    return res.status(401).json({ error: 'Invalid user Passkey signature' });
  }

  // 6. 记录业务验证信息（审计日志）
  await auditLog.record({
    type: 'DUAL_SIGNATURE_SPONSORSHIP',
    accountId,
    paymasterAddress,
    userPublicKey,
    businessValidation,
    timestamp: new Date()
  });

  // 7. 通过 TEE TA 签名（最终签名）
  const teeResult = await teeClient.signWithTEE({
    accountId,
    messageHash: userOpHash,
    signatureType: 'ECDSA_SECP256K1',
    metadata: {
      dualSignatureVerified: true,
      paymasterAddress,
      userPublicKey
    }
  });

  // 8. 返回标准格式
  res.json({
    success: true,
    signature: teeResult.signature,
    userOpHash,
    teeDeviceId: teeResult.deviceId,
    verificationProof: {
      paymasterVerified: true,
      userPasskeyVerified: true,
      dualSignatureMode: true
    }
  });
});

// Passkey 签名验证辅助函数
async function verifyPasskeySignature(
  signature: string,
  publicKey: string,
  messageHash: string,
  accountId: string
): Promise<boolean> {
  // 从数据库获取账户绑定的 Passkey 凭证
  const credential = await database.getPasskeyCredential(accountId);

  if (!credential || credential.publicKey !== publicKey) {
    return false;
  }

  // 使用 WebAuthn 验证逻辑
  return webauthnService.verifySignature(
    signature,
    publicKey,
    messageHash,
    credential.credentialId
  );
}
```

### 🔑 SuperRelay 密钥管理

**密钥生成和轮换**:
```rust
// crates/paymaster-relay/src/key_manager.rs
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, Signature};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PaymasterKeyManager {
    current_wallet: Arc<RwLock<LocalWallet>>,
    rotation_interval: Duration,
    last_rotation: Instant,
}

impl PaymasterKeyManager {
    pub fn new() -> Self {
        // 初始化时生成新密钥
        let wallet = LocalWallet::new(&mut rand::thread_rng());

        Self {
            current_wallet: Arc::new(RwLock::new(wallet)),
            rotation_interval: Duration::from_secs(86400), // 24小时轮换
            last_rotation: Instant::now(),
        }
    }

    pub async fn get_signer(&self) -> LocalWallet {
        // 检查是否需要轮换
        if self.last_rotation.elapsed() > self.rotation_interval {
            self.rotate_key().await;
        }

        self.current_wallet.read().await.clone()
    }

    async fn rotate_key(&self) {
        let new_wallet = LocalWallet::new(&mut rand::thread_rng());
        let mut wallet_guard = self.current_wallet.write().await;

        // 通知 AirAccount 新的公钥
        self.notify_key_rotation(
            wallet_guard.address(),  // 旧地址
            new_wallet.address()      // 新地址
        ).await;

        *wallet_guard = new_wallet;
        self.last_rotation = Instant::now();

        info!("Rotated Paymaster signing key");
    }

    async fn notify_key_rotation(&self, old_address: Address, new_address: Address) {
        // 发送密钥轮换通知给 AirAccount KMS
        // 可以使用双签名（新旧密钥都签名）来验证轮换合法性
    }
}
```

### 🛡️ 授权签名者管理

**AirAccount 端维护授权列表**:
```typescript
// airaccount-ca-nodejs/src/services/signer-authorization.ts
export class SignerAuthorizationService {
  private authorizedSigners: Map<string, SignerInfo> = new Map();

  interface SignerInfo {
    address: string;
    name: string;
    addedAt: Date;
    expiresAt?: Date;
    permissions: string[];
  }

  // 添加授权签名者
  async addAuthorizedSigner(signerAddress: string, info: SignerInfo) {
    // 只有管理员可以添加
    this.authorizedSigners.set(signerAddress.toLowerCase(), info);
    await this.persistToDatabase();
  }

  // 验证签名者授权
  async isAuthorized(signerAddress: string): Promise<boolean> {
    const signer = this.authorizedSigners.get(signerAddress.toLowerCase());
    if (!signer) return false;

    // 检查是否过期
    if (signer.expiresAt && signer.expiresAt < new Date()) {
      this.authorizedSigners.delete(signerAddress.toLowerCase());
      return false;
    }

    return true;
  }

  // 密钥轮换处理
  async handleKeyRotation(oldAddress: string, newAddress: string, proof: string) {
    // 验证轮换证明（双签名验证）
    if (!await this.verifyRotationProof(oldAddress, newAddress, proof)) {
      throw new Error('Invalid key rotation proof');
    }

    // 移除旧密钥，添加新密钥
    const oldSigner = this.authorizedSigners.get(oldAddress.toLowerCase());
    if (oldSigner) {
      this.authorizedSigners.delete(oldAddress.toLowerCase());
      this.authorizedSigners.set(newAddress.toLowerCase(), {
        ...oldSigner,
        address: newAddress,
        addedAt: new Date()
      });
    }
  }
}

## 🏗️ 实施计划

### Phase 1: KMS API 标准化 (1周)

#### Task 1.1: 统一 KMS 接口定义 (3天)
**目标**: 确保 AirAccount 对外 API 完全符合 SuperRelay 需求

**具体任务**:
1. **分析现有接口**:
   ```bash
   # 检查 AirAccount 现有 API
   cd packages/airaccount-ca-nodejs
   grep -r "router\." src/routes/  # 找出所有端点
   ```

2. **新增 KMS 专用端点**:
   ```typescript
   // 新增 src/routes/kms.ts
   export const kmsRoutes = Router();

   // SuperRelay 专用签名接口
   kmsRoutes.post('/sign-user-operation', requireAuth, async (req, res) => {
     // ERC-4337 UserOperation 签名逻辑
   });

   // 账户创建接口 (支持确定性地址)
   kmsRoutes.post('/create-deterministic-account', async (req, res) => {
     // 基于 WebAuthn Credential ID 创建账户
   });

   // 健康检查和状态接口
   kmsRoutes.get('/health', (req, res) => {
     // TEE 状态检查
   });
   ```

3. **验证 TEE 通信**:
   ```bash
   # 测试 CA-TA 通信
   cd packages/airaccount-ca
   cargo run -- wallet create  # 确认钱包创建
   cargo run -- wallet sign <wallet_id> <message>  # 确认签名功能
   ```

#### Task 1.2: ERC-4337 兼容性确认 (2天)
**目标**: 确保签名格式完全符合 ERC-4337 标准

**具体任务**:
1. **UserOperation 哈希计算对齐**:
   ```typescript
   // 确保与 SuperRelay 一致的哈希计算
   function getUserOperationHash(userOp: UserOperation): string {
     const entryPointAddress = '0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789';
     const chainId = 1; // 主网

     const encoded = ethers.AbiCoder.defaultAbiCoder().encode([
       'address', 'uint256', 'bytes32', 'bytes32',
       'uint256', 'uint256', 'uint256', 'uint256', 'uint256', 'bytes32'
     ], [
       userOp.sender, userOp.nonce,
       ethers.keccak256(userOp.initCode),
       ethers.keccak256(userOp.callData),
       userOp.callGasLimit, userOp.verificationGasLimit,
       userOp.preVerificationGas, userOp.maxFeePerGas,
       userOp.maxPriorityFeePerGas,
       ethers.keccak256(userOp.paymasterAndData)
     ]);

     return ethers.keccak256(
       ethers.AbiCoder.defaultAbiCoder().encode(
         ['bytes32', 'address', 'uint256'],
         [ethers.keccak256(encoded), entryPointAddress, chainId]
       )
     );
   }
   ```

2. **签名格式验证**:
   ```bash
   # 创建测试 UserOperation 并验证签名格式
   node -e "
   const { createTestUserOp, signWithAirAccount } = require('./test-utils');
   const userOp = createTestUserOp();
   signWithAirAccount(userOp).then(sig => console.log('Signature:', sig));
   "
   ```

#### Task 1.3: 部署配置标准化 (2天)
**目标**: 创建独立的 KMS 服务部署配置

**具体任务**:
1. **Docker 配置**:
   ```dockerfile
   # 新增 docker/Dockerfile.kms-service
   FROM node:18-alpine

   # 安装 AirAccount KMS 服务
   COPY packages/airaccount-ca-nodejs /app
   WORKDIR /app

   RUN npm install

   # 确保 TEE 设备访问
   RUN apk add --no-cache qemu-system-aarch64

   EXPOSE 3002
   CMD ["npm", "start"]
   ```

2. **环境变量配置**:
   ```bash
   # .env.kms-service
   PORT=3002
   NODE_ENV=production

   # TEE 环境配置
   TEE_DEVICE_PATH=/dev/teepriv0
   QEMU_TEE_ENABLED=true

   # 数据库配置
   DATABASE_PATH=/data/airaccount-kms.sqlite

   # 安全配置
   JWT_SECRET=your-secret-key
   CORS_ORIGIN=https://superrelay.yourdomain.com
   ```

### Phase 2: SuperRelay 集成 (1周)

#### Task 2.1: SuperRelay KMS 客户端实现 (3天)
**目标**: 在 SuperRelay 中实现 AirAccount KMS 调用

**具体任务**:
1. **新增 AirAccount KMS Provider**:
   ```rust
   // crates/paymaster-relay/src/airaccount_kms.rs
   use reqwest::Client;
   use serde::{Deserialize, Serialize};
   use anyhow::Result;

   #[derive(Serialize)]
   pub struct KmsSignRequest {
       user_operation: serde_json::Value,
       account_id: String,
       signature_format: String,
   }

   #[derive(Deserialize)]
   pub struct KmsSignResponse {
       success: bool,
       signature: String,
       user_op_hash: String,
       tee_device_id: String,
   }

   pub struct AirAccountKmsProvider {
       client: Client,
       base_url: String,
       auth_token: Option<String>,
   }

   impl AirAccountKmsProvider {
       pub fn new(base_url: String) -> Self {
           Self {
               client: Client::new(),
               base_url,
               auth_token: None,
           }
       }

       pub async fn sign_user_operation(
           &self,
           user_op: &serde_json::Value,
           account_id: &str,
       ) -> Result<KmsSignResponse> {
           let request = KmsSignRequest {
               user_operation: user_op.clone(),
               account_id: account_id.to_string(),
               signature_format: "erc4337".to_string(),
           };

           let response = self.client
               .post(&format!("{}/kms/sign-user-operation", self.base_url))
               .json(&request)
               .send()
               .await?;

           Ok(response.json().await?)
       }
   }
   ```

2. **集成到 PaymasterRelayService**:
   ```rust
   // crates/paymaster-relay/src/service.rs
   impl PaymasterRelayService {
       pub async fn sponsor_user_operation_with_airaccount(
           &self,
           user_op: UserOperation,
           account_id: &str,
       ) -> Result<UserOperation, PaymasterError> {
           // 1. 使用密钥管理器获取当前签名密钥
           let signer = self.key_manager.get_signer().await;

           // 2. 调用 AirAccount KMS 签名（包含请求签名）
           let kms_response = self.airaccount_kms
               .sign_user_operation_with_signature(
                   &serde_json::to_value(user_op)?,
                   account_id,
                   &signer
               )
               .await?;

           // 3. 验证返回的签名
           self.validate_signature(&kms_response.signature, &kms_response.user_op_hash)?;

           // 4. 添加 Paymaster 数据并返回完整 UserOperation
           let mut sponsored_user_op = user_op;
           sponsored_user_op.signature = kms_response.signature;
           sponsored_user_op.paymaster_and_data = self.build_paymaster_data()?;

           Ok(sponsored_user_op)
       }
   }
   ```

#### Task 2.2: JSON-RPC 接口扩展 (2天)
**目标**: 扩展 SuperRelay 的 JSON-RPC API 以支持 AirAccount

**具体任务**:
1. **新增 RPC 方法**:
   ```rust
   // crates/paymaster-relay/src/rpc.rs
   #[async_trait]
   impl PaymasterRelayApiServer for PaymasterRelayApiServerImpl {
       // 现有方法
       async fn pm_sponsor_user_operation(&self, ...) -> Result<...> { ... }

       // 新增：AirAccount 集成方法（使用签名认证）
       async fn pm_sponsor_with_tee(
           &self,
           user_operation: serde_json::Value,
           account_id: String,
       ) -> Result<serde_json::Value, ErrorObjectOwned> {
           self.service.sponsor_user_operation_with_airaccount(
               serde_json::from_value(user_operation)?,
               &account_id,
           ).await
           .map(|result| serde_json::to_value(result).unwrap())
           .map_err(|e| ErrorObjectOwned::owned(-32000, e.to_string(), None::<()>))
       }

       // 新增：账户信息查询
       async fn pm_get_account_info(
           &self,
           account_id: String,
       ) -> Result<serde_json::Value, ErrorObjectOwned> {
           // 查询 AirAccount 账户状态
           todo!()
       }
   }
   ```

2. **配置文件支持**:
   ```toml
   # config/config.toml
   [paymaster_relay]
   enabled = true

   # AirAccount KMS 集成
   airaccount_kms_enabled = true
   airaccount_kms_url = "http://localhost:3002"

   # SuperRelay 签名密钥配置
   signer_private_key = "${PAYMASTER_SIGNER_KEY}"  # 从环境变量读取
   key_rotation_interval_hours = 24  # 密钥轮换间隔

   # 故障切换配置
   fallback_to_local_signer = true
   kms_timeout_seconds = 30
   ```

#### Task 2.3: 端到端测试 (2天)
**目标**: 验证完整的 SuperRelay + AirAccount 签名流程

**具体任务**:
1. **集成测试脚本**:
   ```bash
   #!/bin/bash
   # scripts/test_airaccount_integration.sh

   echo "🧪 Testing SuperRelay + AirAccount Integration"

   # 1. 启动 AirAccount KMS 服务
   echo "📡 Starting AirAccount KMS Service..."
   cd packages/airaccount-ca-nodejs && npm start &
   KMS_PID=$!
   sleep 5

   # 2. 启动 SuperRelay 服务
   echo "🚀 Starting SuperRelay Service..."
   ./target/debug/super-relay node --config config/config.toml &
   RELAY_PID=$!
   sleep 10

   # 3. 测试端到端流程
   echo "🔧 Testing E2E UserOperation Flow..."

   # 先授权 SuperRelay 的签名地址
   echo "📝 Authorizing SuperRelay signer..."
   SIGNER_ADDRESS=$(./target/debug/super-relay get-signer-address)
   curl -X POST http://localhost:3002/admin/authorize-signer \
     -H "Content-Type: application/json" \
     -H "Admin-Token: ${ADMIN_TOKEN}" \
     -d "{
       \"signerAddress\": \"$SIGNER_ADDRESS\",
       \"name\": \"SuperRelay Paymaster\",
       \"permissions\": [\"sign_user_operation\"]
     }"

   # 创建测试 UserOperation（现在不需要传递认证信息）
   curl -X POST http://localhost:3000 \
     -H "Content-Type: application/json" \
     -d '{
       "jsonrpc": "2.0",
       "id": 1,
       "method": "pm_sponsor_with_tee",
       "params": [{
         "sender": "0x742D35Cc6634C0532925a3b8D6C18E3CB1EB98C1",
         "nonce": "0x0",
         "initCode": "0x",
         "callData": "0x",
         "callGasLimit": "0x186a0",
         "verificationGasLimit": "0x186a0",
         "preVerificationGas": "0x5208",
         "maxFeePerGas": "0x59682f00",
         "maxPriorityFeePerGas": "0x3b9aca00",
         "paymasterAndData": "0x"
       }, "test-account-123"]
     }'

   # 清理
   kill $KMS_PID $RELAY_PID
   echo "✅ Integration test completed"
   ```

### Phase 3: 生产部署优化 (1周)

#### Task 3.1: ARM TEE 环境部署 (3天)
**目标**: 部署到真实 ARM + OP-TEE 环境

**具体任务**:
1. **复用 SuperRelay OP-TEE 配置**:
   ```bash
   # 使用已有的 SuperRelay TEE 部署脚本
   cp scripts/build_optee_env.sh scripts/build_airaccount_kms.sh

   # 修改构建目标
   sed -i 's/super-relay/airaccount-kms/g' scripts/build_airaccount_kms.sh
   ```

2. **Kubernetes 部署配置**:
   ```yaml
   # k8s/airaccount-kms-service.yaml
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: airaccount-kms-service
   spec:
     replicas: 2
     selector:
       matchLabels:
         app: airaccount-kms
     template:
       metadata:
         labels:
           app: airaccount-kms
       spec:
         nodeSelector:
           kubernetes.io/arch: arm64
           superrelay.io/optee-support: "true"
         containers:
         - name: airaccount-kms
           image: airaccount-kms:latest
           ports:
           - containerPort: 3002
           env:
           - name: TEE_DEVICE_PATH
             value: "/dev/teepriv0"
           volumeMounts:
           - name: tee-device
             mountPath: /dev/teepriv0
         volumes:
         - name: tee-device
           hostPath:
             path: /dev/teepriv0
             type: CharDevice
   ```

#### Task 3.2: 监控和日志集成 (2天)
**目标**: 集成到 SuperRelay 的监控体系

**具体任务**:
1. **Prometheus 指标**:
   ```typescript
   // 在 AirAccount KMS 中添加指标收集
   import { register, Counter, Histogram } from 'prom-client';

   const signatureRequests = new Counter({
     name: 'airaccount_signature_requests_total',
     help: 'Total signature requests',
     labelNames: ['account_id', 'status'],
   });

   const signatureLatency = new Histogram({
     name: 'airaccount_signature_duration_seconds',
     help: 'Signature operation latency',
   });
   ```

2. **日志格式统一**:
   ```typescript
   // 使用与 SuperRelay 一致的日志格式
   const logger = createLogger({
     format: format.combine(
       format.timestamp(),
       format.json(),
       format((info) => {
         info.service = 'airaccount-kms';
         info.version = process.env.VERSION || '1.0.0';
         return info;
       })()
     )
   });
   ```

#### Task 3.3: 高可用和故障切换 (2天)
**目标**: 确保服务可靠性

**具体任务**:
1. **健康检查实现**:
   ```typescript
   app.get('/health', async (req, res) => {
     try {
       // 检查 TEE 连接
       await teeClient.healthCheck();

       // 检查数据库连接
       await database.ping();

       res.json({
         status: 'healthy',
         timestamp: new Date().toISOString(),
         services: {
           tee: 'ok',
           database: 'ok'
         }
       });
     } catch (error) {
       res.status(503).json({
         status: 'unhealthy',
         error: error.message
       });
     }
   });
   ```

2. **SuperRelay 故障切换**:
   ```rust
   // 在 SuperRelay 中实现故障切换逻辑
   impl AirAccountKmsProvider {
       pub async fn sign_user_operation_with_fallback(
           &self,
           user_op: &serde_json::Value,
           account_id: &str,
       ) -> Result<KmsSignResponse> {
           // 尝试 AirAccount KMS
           match self.sign_user_operation(user_op, account_id).await {
               Ok(response) => Ok(response),
               Err(e) => {
                   warn!("AirAccount KMS failed: {}, falling back to local signer", e);

                   // 故障切换到本地签名器
                   self.fallback_to_local_signer(user_op, account_id).await
               }
           }
       }
   }
   ```

## 📊 实施时间表

| 阶段 | 任务 | 时间 | 负责模块 | 验收标准 |
|------|------|------|----------|----------|
| Phase 1 | KMS API 标准化 | 1周 | AirAccount | HTTP API 可正常签名 ERC-4337 UserOperation |
| Phase 2 | SuperRelay 集成 | 1周 | SuperRelay | JSON-RPC `pm_sponsor_with_passkey` 方法可用 |
| Phase 3 | 生产部署优化 | 1周 | 基础设施 | ARM TEE 环境稳定运行，监控正常 |

## 🎯 验收标准

### 功能验收
- ✅ AirAccount KMS 服务独立运行
- ✅ SuperRelay 可通过 HTTP API 调用 KMS 签名
- ✅ 签名格式完全符合 ERC-4337 标准
- ✅ WebAuthn 认证集成工作正常
- ✅ ARM TEE 环境部署成功

### 性能验收
- 📈 签名响应时间 < 2秒 (包含 WebAuthn 认证)
- 📈 并发支持 100+ 请求/分钟
- 📈 系统可用性 > 99.9%

### 安全验收
- 🔒 私钥永不离开 TEE Secure World
- 🔒 所有 API 调用都需要有效认证
- 🔒 签名操作有完整审计日志

## 🛡️ 双重签名安全分析

### 安全优势

1. **多层防护机制**：
   - **第一层**：用户 Passkey 验证（用户意图）
   - **第二层**：Paymaster 业务验证（业务规则）
   - **第三层**：TEE 硬件保护（私钥安全）

2. **攻击场景防护**：

   | 攻击场景 | 防护机制 | 结果 |
   |---------|---------|------|
   | Paymaster 私钥泄露 | 需要用户 Passkey 签名 | ❌ 无法伪造用户签名 |
   | 用户设备被入侵 | 需要 Paymaster 业务验证 | ❌ 无法通过业务规则 |
   | 重放攻击 | Nonce + 时间戳验证 | ❌ 请求被拒绝 |
   | 中间人攻击 | 双重签名绑定 | ❌ 签名验证失败 |
   | 未授权赞助 | 白名单 + 余额检查 | ❌ 业务验证失败 |

3. **责任明确**：
   - 用户：通过 Passkey 授权交易意图
   - Paymaster：验证业务规则并承担 Gas 费用
   - TEE：保护私钥并执行最终签名

### 签名流程时序图

```
用户设备          SuperPaymaster         AirAccount KMS          TEE
   │                    │                      │                  │
   ├──UserOp + Passkey──┤                      │                  │
   │                    ├──验证业务规则        │                  │
   │                    ├──Paymaster签名       │                  │
   │                    ├──双重签名请求────────┤                  │
   │                    │                      ├──验证Paymaster──┤
   │                    │                      ├──验证Passkey────┤
   │                    │                      ├──TEE签名请求────┤
   │                    │                      │                  ├──签名
   │                    │                      ├──签名结果───────┤
   │                    ├──完整UserOp──────────┤                  │
   ├──提交到Bundler─────┤                      │                  │
```

## 💡 技术风险和缓解措施

### 风险1: TEE 环境兼容性
**缓解**: 复用 SuperRelay 已验证的 OP-TEE 环境配置

### 风险2: 性能瓶颈
**缓解**: 实现批量签名和签名缓存机制

### 风险3: 网络通信延迟
**缓解**: 支持故障切换到本地签名器

### 风险4: 双重签名复杂性
**缓解**: 清晰的错误提示和详细的审计日志

---

## 📌 总结

**双重签名安全架构** 将 AirAccount 的 TEE-KMS 能力与 SuperRelay 的 Paymaster 服务深度集成，形成了一个分层信任模型：

### 核心创新点：
1. **双重签名机制**：用户 Passkey（意图验证）+ Paymaster 签名（业务验证）
2. **防护升级**：即使 Paymaster 私钥泄露，攻击者也无法伪造用户签名
3. **责任分离**：用户控制交易意图，Paymaster 控制业务规则
4. **硬件安全**：TEE 保护最终签名密钥

### 技术架构：
- **AirAccount KMS 服务**：CA + SDK + TA 完整剥离为独立模块
- **SuperRelay 集成**：通过签名认证 API 调用 KMS 服务
- **安全通信**：双重签名 + Nonce + 时间戳防重放

### 实施计划：
- **Phase 1**：KMS API 标准化，支持双重签名验证（1周）
- **Phase 2**：SuperRelay 集成，实现业务规则验证（1周）
- **Phase 3**：生产部署，ARM TEE 环境优化（1周）

预计 **3周时间** 完成集成，实现：
- ✅ **硬件级安全**：TEE 保护的私钥管理
- ✅ **用户体验**：Passkey 生物识别认证
- ✅ **企业级服务**：可靠的 Gas 赞助机制
- ✅ **去中心化信任**：无需中心化 API 密钥

这个方案不仅解决了安全问题，还为去中心化平台提供了更适合的信任模型。