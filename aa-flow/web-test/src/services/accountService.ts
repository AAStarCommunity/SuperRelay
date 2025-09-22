// 账户管理服务
import { ethers } from 'ethers';
import { BundlerService } from './bundlerService';
import type { UserOperation } from './bundlerService';
import { DebugLogger } from '../utils/debugLogger';

export interface AccountInfo {
  address: string;
  ethBalance: string;
  tokenBalance: string;
  nonce: number;
  isDeployed: boolean;
}

export interface TransferParams {
  from: string;
  to: string;
  amount: string;
  tokenAddress: string;
  privateKey?: string; // 可选的私钥参数
  signer?: ethers.Signer; // 可选的 MetaMask signer
}

export interface TransferResult {
  userOpHash: string;
  success: boolean;
  receipt?: any;
  error?: string;
  userOp?: UserOperation;
  gasEstimate?: any;
  jiffyScanUrl?: string;
  etherscanUrl?: string;
}

export class AccountService {
  private provider: ethers.JsonRpcProvider;
  private bundlerService: BundlerService;
  private privateKey: string;
  private entryPointAddress: string;
  private factoryAddress: string;

  constructor(
    rpcUrl: string,
    bundlerUrl: string,
    privateKey: string,
    entryPointAddress: string,
    factoryAddress: string
  ) {
    this.provider = new ethers.JsonRpcProvider(rpcUrl);
    this.bundlerService = new BundlerService(bundlerUrl);
    this.privateKey = privateKey;
    this.entryPointAddress = entryPointAddress;
    this.factoryAddress = factoryAddress;
  }

  // 获取账户信息
  async getAccountInfo(accountAddress: string, tokenAddress?: string): Promise<AccountInfo> {
    try {
      // 获取 ETH 余额
      const ethBalance = await this.provider.getBalance(accountAddress);

      // 获取代币余额（如果提供了代币地址）
      let tokenBalance = '0';
      if (tokenAddress) {
        const tokenContract = new ethers.Contract(
          tokenAddress,
          ['function balanceOf(address) view returns (uint256)'],
          this.provider
        );
        const balance = await tokenContract.balanceOf(accountAddress);
        tokenBalance = ethers.formatEther(balance);
      }

      // 检查账户是否已部署
      const code = await this.provider.getCode(accountAddress);
      const isDeployed = code !== '0x';

      // 获取 nonce（如果已部署）
      let nonce = 0;
      if (isDeployed) {
        try {
          const accountContract = new ethers.Contract(
            accountAddress,
            ['function nonce() view returns (uint256)', 'function getNonce() view returns (uint256)'],
            this.provider
          );

          // 尝试不同的 nonce 函数名
          try {
            nonce = Number(await accountContract.nonce());
          } catch {
            nonce = Number(await accountContract.getNonce());
          }
        } catch (error) {
          console.warn('无法获取 nonce，使用默认值 0:', error);
          nonce = 0;
        }
      }

      return {
        address: accountAddress,
        ethBalance: ethers.formatEther(ethBalance),
        tokenBalance,
        nonce,
        isDeployed,
      };
    } catch (error) {
      console.error('Failed to get account info:', error);
      throw error;
    }
  }

  // 计算 SimpleAccount 地址
  async calculateAccountAddress(owner: string, salt: number = 0): Promise<string> {
    try {
      // const factoryContract = new ethers.Contract(
      //   this.factoryAddress,
      //   ['function getAddress(address owner, uint256 salt) view returns (address)'],
      //   this.provider
      // );

      // return await factoryContract['getAddress'](owner, salt);
      // 临时返回模拟地址
      return '0x' + ethers.keccak256(ethers.toUtf8Bytes(owner + salt.toString())).slice(2, 42);
    } catch (error) {
      console.error('Failed to calculate account address:', error);
      throw error;
    }
  }

  // 计算 preVerificationGas 的精确算法
  private calculatePreVerificationGas(userOp: Partial<UserOperation>): string {
    const FIXED_GAS_OVERHEAD = 21000;  // 基础交易 gas
    const PER_USER_OP_OVERHEAD = 18300; // 每个 UserOp 的固定开销
    const PER_USER_OP_WORD = 4;         // 每个字的开销
    const ZERO_BYTE_COST = 4;           // 零字节成本
    const NON_ZERO_BYTE_COST = 16;      // 非零字节成本
    const SAFETY_BUFFER = 1000;         // 安全缓冲

    // 序列化 UserOperation 用于计算字节数
    const userOpData = JSON.stringify({
      sender: userOp.sender || '',
      nonce: userOp.nonce || '0x0',
      initCode: userOp.initCode || '0x',
      callData: userOp.callData || '0x',
      callGasLimit: userOp.callGasLimit || '0x0',
      verificationGasLimit: userOp.verificationGasLimit || '0x0',
      preVerificationGas: '0x0', // 临时值
      maxFeePerGas: userOp.maxFeePerGas || '0x0',
      maxPriorityFeePerGas: userOp.maxPriorityFeePerGas || '0x0',
      paymasterAndData: userOp.paymasterAndData || '0x',
      signature: userOp.signature || '0x'
    });

    // 计算字节成本
    let byteCost = 0;
    const encoder = new TextEncoder();
    const bytes = encoder.encode(userOpData);

    for (const byte of bytes) {
      if (byte === 0) {
        byteCost += ZERO_BYTE_COST;
      } else {
        byteCost += NON_ZERO_BYTE_COST;
      }
    }

    // 计算总的 preVerificationGas
    const totalGas = FIXED_GAS_OVERHEAD +
                    PER_USER_OP_OVERHEAD +
                    Math.ceil(bytes.length / 32) * PER_USER_OP_WORD +
                    byteCost +
                    SAFETY_BUFFER;

    DebugLogger.log(`🧮 PreVerificationGas 计算:`);
    DebugLogger.log(`  - 基础交易 gas: ${FIXED_GAS_OVERHEAD}`);
    DebugLogger.log(`  - UserOp 固定开销: ${PER_USER_OP_OVERHEAD}`);
    DebugLogger.log(`  - 数据长度: ${bytes.length} bytes`);
    DebugLogger.log(`  - 字节成本: ${byteCost}`);
    DebugLogger.log(`  - 安全缓冲: ${SAFETY_BUFFER}`);
    DebugLogger.log(`  - 计算总量: ${totalGas}`);

    return ethers.toBeHex(totalGas);
  }

  // 构建 ERC20 转账 UserOperation
  async buildTransferUserOp(params: TransferParams): Promise<{
    userOp: UserOperation;
    gasEstimate: any;
  }> {
    try {
      // 使用传入的私钥、signer 或者构造函数中的私钥
      let wallet: ethers.Wallet | ethers.Signer;

      DebugLogger.log('🔍 === buildTransferUserOp 调试信息 ===');
      DebugLogger.log(`📥 传入参数 signer: ${params.signer ? '存在' : '不存在'}`);
      DebugLogger.log(`📥 传入参数 privateKey: ${params.privateKey ? '存在' : '不存在'}`);
      DebugLogger.log(`🏗️ 构造函数 privateKey: ${this.privateKey ? '存在' : '不存在'}`);

      if (params.signer) {
        DebugLogger.log('🦊 使用 MetaMask signer');
        wallet = params.signer;

        // 验证 signer 是否有效
        try {
          const signerAddress = await params.signer.getAddress();
          DebugLogger.log(`✅ MetaMask signer 地址: ${signerAddress}`);
        } catch (error) {
          DebugLogger.log(`❌ MetaMask signer 无效: ${error}`);
          throw new Error(`MetaMask signer 无效: ${error}`);
        }
      } else {
        const privateKeyToUse = params.privateKey || this.privateKey;
        if (!privateKeyToUse) {
          throw new Error('私钥未提供，且没有可用的 MetaMask signer');
        }
        DebugLogger.log('🔑 使用私钥创建 wallet');
        wallet = new ethers.Wallet(privateKeyToUse);
      }

      // 编码 ERC20 transfer 调用
      const tokenContract = new ethers.Contract(
        params.tokenAddress,
        ['function transfer(address to, uint256 amount) returns (bool)'],
        this.provider
      );

      const amount = ethers.parseEther(params.amount);
      const transferData = tokenContract.interface.encodeFunctionData('transfer', [
        params.to,
        amount,
      ]);

      // 编码 SimpleAccount execute 调用
      const accountContract = new ethers.Contract(
        params.from,
        ['function execute(address dest, uint256 value, bytes calldata func) external'],
        this.provider
      );

      const executeData = accountContract.interface.encodeFunctionData('execute', [
        params.tokenAddress,
        0, // value = 0 ETH
        transferData,
      ]);

      // 获取账户 nonce
      const accountInfo = await this.getAccountInfo(params.from);
      const nonce = accountInfo.nonce;

      // 获取 gas 价格
      const feeData = await this.provider.getFeeData();

      // 为 Sepolia 测试网使用非常低的 gas 价格
      const minMaxFee = BigInt(100100000); // 100100000 wei ≈ 0.1001 gwei (bundler 最低要求)
      const sepoliaMaxFee = ethers.parseUnits('0.2', 'gwei'); // 0.2 gwei，适合测试网
      // 忽略网络返回的过高价格，直接使用测试网合理价格
      const maxFeePerGas = sepoliaMaxFee > minMaxFee ? sepoliaMaxFee : ethers.parseUnits('0.2', 'gwei');

      // 使用测试网合理的优先费用
      const minPriorityFee = BigInt(100000000); // 100000000 wei ≈ 0.1 gwei (bundler 最低要求)
      const sepoliaPriorityFee = ethers.parseUnits('0.1', 'gwei'); // 0.1 gwei，刚好满足要求
      // 直接使用最低要求，不依赖网络返回值
      const maxPriorityFeePerGas = sepoliaPriorityFee;

      DebugLogger.log(`💰 Gas 费用设置:`);
      DebugLogger.log(`  - maxFeePerGas: ${ethers.formatUnits(maxFeePerGas, 'gwei')} gwei (${maxFeePerGas} wei)`);
      DebugLogger.log(`  - maxFeePerGas 最低要求: ${ethers.formatUnits(minMaxFee, 'gwei')} gwei (${minMaxFee} wei)`);
      DebugLogger.log(`  - maxPriorityFeePerGas: ${ethers.formatUnits(maxPriorityFeePerGas, 'gwei')} gwei (${maxPriorityFeePerGas} wei)`);
      DebugLogger.log(`  - maxPriorityFeePerGas 最低要求: ${ethers.formatUnits(minPriorityFee, 'gwei')} gwei (${minPriorityFee} wei)`);

      // 先构建基础的 UserOperation（不含 preVerificationGas）
      const baseUserOp: Partial<UserOperation> = {
        sender: params.from,
        nonce: ethers.toBeHex(nonce),
        initCode: '0x',
        callData: executeData,
        callGasLimit: '0x11170', // 70000 gas (更保守的设置)
        verificationGasLimit: '0x11170', // 70000 gas (更保守的设置)
        maxFeePerGas: ethers.toBeHex(maxFeePerGas),
        maxPriorityFeePerGas: ethers.toBeHex(maxPriorityFeePerGas),
        paymasterAndData: '0x',
        signature: '0x',
      };

      // 动态计算 preVerificationGas
      const calculatedPreVerificationGas = this.calculatePreVerificationGas(baseUserOp);

      // 构建完整的 UserOperation
      const userOp: UserOperation = {
        ...baseUserOp,
        preVerificationGas: calculatedPreVerificationGas,
      } as UserOperation;

      // 模拟 gasEstimate 返回值以保持接口一致
      const gasEstimate = {
        callGasLimit: '0x11170',
        verificationGasLimit: '0x11170',
        preVerificationGas: calculatedPreVerificationGas,
      };

      DebugLogger.log('📋 构建完成的 UserOperation (签名前):');
      DebugLogger.log(JSON.stringify(userOp, null, 2));

      // 计算签名
      const signature = await this.signUserOperation(userOp, wallet);
      userOp.signature = signature;

      DebugLogger.log('📋 最终 UserOperation (含签名):');
      DebugLogger.log(JSON.stringify(userOp, null, 2));

      return { userOp, gasEstimate };
    } catch (error) {
      console.error('Failed to build transfer UserOp:', error);
      throw error;
    }
  }

  // 执行转账
  async executeTransfer(params: TransferParams): Promise<TransferResult> {
    try {
      DebugLogger.log('🚀 开始构建 UserOperation...');

      // 检查是否提供了私钥或 signer
      if (!params.signer && !params.privateKey && !this.privateKey) {
        throw new Error('私钥未提供，且没有可用的 MetaMask signer');
      }

      const { userOp, gasEstimate } = await this.buildTransferUserOp(params);

      // 发送 UserOperation
      const userOpHash = await this.bundlerService.sendUserOperation(
        userOp,
        this.entryPointAddress
      );

      // 等待确认
      const receipt = await this.bundlerService.waitForUserOpReceipt(userOpHash);

      // 生成浏览器链接
      const jiffyScanUrl = `https://jiffyscan.xyz/userOpHash/${userOpHash}?network=sepolia`;
      const etherscanUrl = receipt.receipt?.transactionHash
        ? `https://sepolia.etherscan.io/tx/${receipt.receipt.transactionHash}`
        : null;

      DebugLogger.log(`🔗 浏览器链接:`);
      DebugLogger.log(`  - JiffyScan (UserOp): ${jiffyScanUrl}`);
      if (etherscanUrl) {
        DebugLogger.log(`  - Etherscan (Tx): ${etherscanUrl}`);
      }

      return {
        userOpHash,
        success: receipt.success,
        receipt,
        userOp,
        gasEstimate,
        jiffyScanUrl,
        etherscanUrl: etherscanUrl || undefined,
      };
    } catch (error) {
      console.error('Transfer failed:', error);
      return {
        userOpHash: '',
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // 签名 UserOperation (精确匹配工作脚本的实现)
  private async signUserOperation(
    userOp: UserOperation,
    wallet: ethers.Wallet | ethers.Signer
  ): Promise<string> {
    try {
      DebugLogger.log('🔍 === 详细调试信息 ===');
      DebugLogger.log('📋 UserOperation 输入: ' + JSON.stringify(userOp, null, 2));

      // 显示预期的工作脚本结果
      DebugLogger.log('✅ 预期结果 (来自工作脚本):');
      DebugLogger.log('  预期 Hash: 0xf6ffd45b773ad7af4791441c946dad7fe50372af73a7c193f0585e49ca2085cb');
      DebugLogger.log('  预期签名长度: 132');
      DebugLogger.log('  预期签名: 0xe3099e923b55df481cada71de2b6c147d6c266bc0997d357bb89085b4958483c3cde97d4fdaa00f2b7f73e078a56ad7efed925e1ba02575e89a619dfe1a8581c1c');

      // 计算 UserOperation hash - 完全匹配工作脚本
      const userOpHash = this.getUserOpHashV5Compatible(userOp);

      DebugLogger.log('🔍 当前计算结果:');
      DebugLogger.log('  当前 Hash: ' + userOpHash);
      DebugLogger.log('  Hash 匹配: ' + (userOpHash === '0xf6ffd45b773ad7af4791441c946dad7fe50372af73a7c193f0585e49ca2085cb' ? '✅' : '❌'));

      // 使用与工作脚本完全相同的签名方法
      // ethers v6 equivalent of ethers.utils.arrayify(userOpHash)
      const hashBytes = ethers.getBytes(userOpHash);
      DebugLogger.log('  Hash Bytes 长度: ' + hashBytes.length);
      DebugLogger.log('  Hash Bytes: ' + Array.from(hashBytes).map(b => b.toString(16).padStart(2, '0')).join(''));

      const signature = await wallet.signMessage(hashBytes);

      DebugLogger.log('  实际签名长度: ' + signature.length);
      DebugLogger.log('  实际签名: ' + signature);
      DebugLogger.log('  签名长度匹配: ' + (signature.length === 132 ? '✅' : '❌'));

      // 验证签名格式
      if (signature.length !== 132) {
        throw new Error(`签名长度错误: 期望132, 实际${signature.length}`);
      }

      if (!signature.startsWith('0x')) {
        throw new Error('签名格式错误: 应该以0x开头');
      }

      DebugLogger.log('🎯 签名验证完成');
      return signature;
    } catch (error) {
      console.error('❌ 签名失败:', error);
      throw error;
    }
  }

  // 计算 UserOperation Hash（ERC-4337 标准）
  private getUserOpHash(userOp: UserOperation): string {
    const chainId = 11155111; // Sepolia chain ID

    // 确保数值字段转换为正确格式
    const packedUserOp = ethers.AbiCoder.defaultAbiCoder().encode(
      [
        'address', 'uint256', 'bytes32', 'bytes32',
        'uint256', 'uint256', 'uint256', 'uint256',
        'uint256', 'bytes32'
      ],
      [
        userOp.sender,
        BigInt(userOp.nonce),
        ethers.keccak256(userOp.initCode),
        ethers.keccak256(userOp.callData),
        BigInt(userOp.callGasLimit),
        BigInt(userOp.verificationGasLimit),
        BigInt(userOp.preVerificationGas),
        BigInt(userOp.maxFeePerGas),
        BigInt(userOp.maxPriorityFeePerGas),
        ethers.keccak256(userOp.paymasterAndData),
      ]
    );

    const encoded = ethers.AbiCoder.defaultAbiCoder().encode(
      ['bytes32', 'address', 'uint256'],
      [ethers.keccak256(packedUserOp), this.entryPointAddress, chainId]
    );

    return ethers.keccak256(encoded);
  }

  // V5兼容的UserOp Hash计算 (精确匹配工作脚本)
  private getUserOpHashV5Compatible(userOp: UserOperation): string {
    const chainId = 11155111; // Sepolia chain ID

    DebugLogger.log('🔧 Hash 计算详细步骤:');
    DebugLogger.log('  Chain ID: ' + chainId);
    DebugLogger.log('  Entry Point: ' + this.entryPointAddress);

    // 预计算各个组件的hash
    const initCodeHash = ethers.keccak256(userOp.initCode);
    const callDataHash = ethers.keccak256(userOp.callData);
    const paymasterDataHash = ethers.keccak256(userOp.paymasterAndData);

    DebugLogger.log('  组件 Hash:');
    DebugLogger.log('    initCode: ' + userOp.initCode + ' → ' + initCodeHash);
    DebugLogger.log('    callData: ' + (userOp.callData?.slice(0, 50) + '...') + ' → ' + callDataHash);
    DebugLogger.log('    paymasterData: ' + userOp.paymasterAndData + ' → ' + paymasterDataHash);

    // 显示编码参数
    const encodeParams = [
      userOp.sender,
      userOp.nonce,
      initCodeHash,
      callDataHash,
      userOp.callGasLimit,
      userOp.verificationGasLimit,
      userOp.preVerificationGas,
      userOp.maxFeePerGas,
      userOp.maxPriorityFeePerGas,
      paymasterDataHash,
    ];

    DebugLogger.log('  编码参数:');
    encodeParams.forEach((param, i) => {
      const labels = ['sender', 'nonce', 'initCodeHash', 'callDataHash', 'callGasLimit', 'verificationGasLimit', 'preVerificationGas', 'maxFeePerGas', 'maxPriorityFeePerGas', 'paymasterDataHash'];
      DebugLogger.log(`    ${labels[i]}: ${param}`);
    });

    // 完全匹配工作脚本的参数处理 - 不使用BigInt转换，保持原始hex格式
    const packedUserOp = ethers.AbiCoder.defaultAbiCoder().encode(
      [
        'address', 'uint256', 'bytes32', 'bytes32',
        'uint256', 'uint256', 'uint256', 'uint256',
        'uint256', 'bytes32'
      ],
      encodeParams
    );

    DebugLogger.log('  Packed UserOp: ' + packedUserOp.slice(0, 100) + '...');

    const packedHash = ethers.keccak256(packedUserOp);
    DebugLogger.log('  Packed Hash: ' + packedHash);

    const encoded = ethers.AbiCoder.defaultAbiCoder().encode(
      ['bytes32', 'address', 'uint256'],
      [packedHash, this.entryPointAddress, chainId]
    );

    DebugLogger.log('  Final Encoded: ' + encoded.slice(0, 100) + '...');

    const finalHash = ethers.keccak256(encoded);
    DebugLogger.log('  Final Hash: ' + finalHash);

    return finalHash;
  }

  // 获取代币信息
  async getTokenInfo(tokenAddress: string): Promise<{
    name: string;
    symbol: string;
    decimals: number;
  }> {
    try {
      const tokenContract = new ethers.Contract(
        tokenAddress,
        [
          'function name() view returns (string)',
          'function symbol() view returns (string)',
          'function decimals() view returns (uint8)',
        ],
        this.provider
      );

      const [name, symbol, decimals] = await Promise.all([
        tokenContract.name(),
        tokenContract.symbol(),
        tokenContract.decimals(),
      ]);

      return { name, symbol, decimals };
    } catch (error) {
      console.error('Failed to get token info:', error);
      return { name: 'Unknown', symbol: 'UNK', decimals: 18 };
    }
  }

  // 分析 UserOperation 的 gas 使用
  async analyzeGasUsage(userOpHash: string): Promise<{
    estimated: any;
    actual: any;
    difference: any;
    efficiency: number;
  } | null> {
    try {
      const receipt = await this.bundlerService.getUserOperationReceipt(userOpHash);
      if (!receipt) return null;

      const actualGasUsed = parseInt(receipt.actualGasUsed);
      const actualGasCost = BigInt(receipt.actualGasCost);

      // 计算效率（实际使用 / 估算使用）
      const estimatedGas = 200000; // 示例值，实际应该从估算中获取
      const efficiency = (actualGasUsed / estimatedGas) * 100;

      return {
        estimated: {
          gas: estimatedGas,
          cost: ethers.formatEther(BigInt(estimatedGas) * BigInt('100000000000')), // 100 Gwei
        },
        actual: {
          gas: actualGasUsed,
          cost: ethers.formatEther(actualGasCost),
        },
        difference: {
          gas: actualGasUsed - estimatedGas,
          cost: ethers.formatEther(actualGasCost - BigInt(estimatedGas) * BigInt('100000000000')),
        },
        efficiency,
      };
    } catch (error) {
      console.error('Failed to analyze gas usage:', error);
      return null;
    }
  }
}