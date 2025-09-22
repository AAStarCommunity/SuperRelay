// Bundler 服务
import axios from 'axios';
import { ethers } from 'ethers';

export interface UserOperation {
  sender: string;
  nonce: string;
  initCode: string;
  callData: string;
  callGasLimit: string;
  verificationGasLimit: string;
  preVerificationGas: string;
  maxFeePerGas: string;
  maxPriorityFeePerGas: string;
  paymasterAndData: string;
  signature: string;
}

export interface UserOpReceipt {
  userOpHash: string;
  entryPoint: string;
  sender: string;
  nonce: string;
  paymaster: string;
  actualGasCost: string;
  actualGasUsed: string;
  success: boolean;
  reason: string;
  receipt: {
    transactionHash: string;
    blockNumber: string;
    gasUsed: string;
    effectiveGasPrice: string;
    from: string;
    to: string;
  };
  logs: Array<{
    address: string;
    topics: string[];
    data: string;
    blockHash: string;
    blockNumber: string;
    transactionHash: string;
    logIndex: string;
  }>;
}

export interface GasEstimate {
  callGasLimit: string;
  verificationGasLimit: string;
  preVerificationGas: string;
  maxFeePerGas: string;
  maxPriorityFeePerGas: string;
}

export class BundlerService {
  private bundlerUrl: string;

  constructor(bundlerUrl: string) {
    // 始终使用代理路径来避免CORS问题
    this.bundlerUrl = '/api/bundler';
  }

  // 获取支持的 EntryPoints
  async getSupportedEntryPoints(): Promise<string[]> {
    try {
      const response = await axios.post(this.bundlerUrl, {
        jsonrpc: '2.0',
        method: 'eth_supportedEntryPoints',
        params: [],
        id: 1,
      }, {
        headers: {
          'Content-Type': 'application/json',
        },
        timeout: 10000,
        // 添加CORS处理
        withCredentials: false,
      });

      if (response.data.error) {
        throw new Error(response.data.error.message);
      }

      return response.data.result;
    } catch (error) {
      console.error('Failed to get supported entry points:', error);
      throw error;
    }
  }

  // 估算 UserOperation gas
  async estimateUserOperationGas(
    userOp: Partial<UserOperation>,
    entryPointAddress: string
  ): Promise<GasEstimate> {
    try {
      const response = await axios.post(this.bundlerUrl, {
        jsonrpc: '2.0',
        method: 'eth_estimateUserOperationGas',
        params: [userOp, entryPointAddress],
        id: 1,
      }, {
        headers: {
          'Content-Type': 'application/json',
        },
        timeout: 30000,
        withCredentials: false,
      });

      if (response.data.error) {
        throw new Error(response.data.error.message);
      }

      return response.data.result;
    } catch (error) {
      console.error('Failed to estimate gas:', error);
      throw error;
    }
  }

  // 发送 UserOperation
  async sendUserOperation(
    userOp: UserOperation,
    entryPointAddress: string
  ): Promise<string> {
    try {
      const response = await axios.post(this.bundlerUrl, {
        jsonrpc: '2.0',
        method: 'eth_sendUserOperation',
        params: [userOp, entryPointAddress],
        id: 1,
      }, {
        headers: {
          'Content-Type': 'application/json',
        },
        timeout: 30000,
        withCredentials: false,
      });

      if (response.data.error) {
        throw new Error(response.data.error.message);
      }

      return response.data.result;
    } catch (error) {
      console.error('Failed to send user operation:', error);
      throw error;
    }
  }

  // 获取 UserOperation 收据
  async getUserOperationReceipt(userOpHash: string): Promise<UserOpReceipt | null> {
    try {
      const response = await axios.post(this.bundlerUrl, {
        jsonrpc: '2.0',
        method: 'eth_getUserOperationReceipt',
        params: [userOpHash],
        id: 1,
      });

      if (response.data.error) {
        throw new Error(response.data.error.message);
      }

      return response.data.result;
    } catch (error) {
      console.error('Failed to get user operation receipt:', error);
      return null;
    }
  }

  // 等待 UserOperation 被包含
  async waitForUserOpReceipt(
    userOpHash: string,
    maxAttempts: number = 60,
    intervalMs: number = 5000
  ): Promise<UserOpReceipt> {
    for (let i = 0; i < maxAttempts; i++) {
      const receipt = await this.getUserOperationReceipt(userOpHash);
      if (receipt) {
        return receipt;
      }

      await new Promise(resolve => setTimeout(resolve, intervalMs));
    }

    throw new Error('UserOperation timeout - not included in block');
  }

  // 检查 Bundler 状态
  async checkBundlerStatus(): Promise<{
    isOnline: boolean;
    supportedEntryPoints: string[];
    error?: string;
  }> {
    try {
      // 首先检查健康检查端点
      try {
        const healthResponse = await axios.get(`${this.bundlerUrl}/health`, {
          timeout: 5000
        });
        if (healthResponse.data !== 'ok') {
          throw new Error('Health check failed');
        }
      } catch (healthError) {
        console.warn('Health check failed, trying RPC method:', healthError);
      }

      // 然后检查RPC方法
      const entryPoints = await this.getSupportedEntryPoints();
      return {
        isOnline: true,
        supportedEntryPoints: entryPoints,
      };
    } catch (error) {
      return {
        isOnline: false,
        supportedEntryPoints: [],
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }
}

// Gas 计算工具
export class GasCalculator {
  // ERC-4337 EntryPoint v0.6 的 gas 计算规则
  static readonly FIXED_GAS_OVERHEAD = 21000;        // 基础交易 gas
  static readonly PER_USER_OP_OVERHEAD = 18300;      // 每个 UserOp 的固定开销
  static readonly PER_USER_OP_WORD = 4;              // 每个字的开销
  static readonly ZERO_BYTE_COST = 4;                // 零字节成本
  static readonly NON_ZERO_BYTE_COST = 16;           // 非零字节成本

  // 计算 preVerificationGas
  static calculatePreVerificationGas(userOp: Partial<UserOperation>): number {
    let gas = this.FIXED_GAS_OVERHEAD + this.PER_USER_OP_OVERHEAD;

    // 计算 UserOperation 数据的字节成本
    const userOpData = this.encodeUserOpForGasCalculation(userOp);
    const dataBytes = ethers.getBytes(userOpData);

    for (const byte of dataBytes) {
      gas += byte === 0 ? this.ZERO_BYTE_COST : this.NON_ZERO_BYTE_COST;
    }

    return gas;
  }

  // 编码 UserOperation 用于 gas 计算
  private static encodeUserOpForGasCalculation(userOp: Partial<UserOperation>): string {
    // 简化的编码，实际应该使用完整的 ABI 编码
    const encoded = ethers.concat([
      ethers.zeroPadValue(userOp.sender || '0x', 20),
      ethers.zeroPadValue(userOp.nonce || '0x0', 32),
      ethers.getBytes(userOp.initCode || '0x'),
      ethers.getBytes(userOp.callData || '0x'),
      ethers.zeroPadValue(userOp.callGasLimit || '0x0', 32),
      ethers.zeroPadValue(userOp.verificationGasLimit || '0x0', 32),
      ethers.zeroPadValue(userOp.preVerificationGas || '0x0', 32),
      ethers.zeroPadValue(userOp.maxFeePerGas || '0x0', 32),
      ethers.zeroPadValue(userOp.maxPriorityFeePerGas || '0x0', 32),
      ethers.getBytes(userOp.paymasterAndData || '0x'),
    ]);

    return ethers.hexlify(encoded);
  }

  // 估算总 gas 成本
  static estimateTotalGasCost(
    gasEstimate: GasEstimate,
    gasPrice?: string
  ): {
    estimatedGas: number;
    estimatedCostWei: bigint;
    estimatedCostEth: string;
  } {
    const callGas = parseInt(gasEstimate.callGasLimit);
    const verificationGas = parseInt(gasEstimate.verificationGasLimit);
    const preVerificationGas = parseInt(gasEstimate.preVerificationGas);

    const totalGas = callGas + verificationGas + preVerificationGas;
    const effectiveGasPrice = BigInt(gasPrice || gasEstimate.maxFeePerGas);
    const costWei = BigInt(totalGas) * effectiveGasPrice;

    return {
      estimatedGas: totalGas,
      estimatedCostWei: costWei,
      estimatedCostEth: ethers.formatEther(costWei),
    };
  }
}