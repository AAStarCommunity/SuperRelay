// 账户管理服务
import { ethers } from 'ethers';
import { BundlerService } from './bundlerService';
import type { UserOperation } from './bundlerService';

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
}

export interface TransferResult {
  userOpHash: string;
  success: boolean;
  receipt?: any;
  error?: string;
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
        const accountContract = new ethers.Contract(
          accountAddress,
          ['function getNonce() view returns (uint256)'],
          this.provider
        );
        nonce = Number(await accountContract.getNonce());
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
      const factoryContract = new ethers.Contract(
        this.factoryAddress,
        ['function getAddress(address owner, uint256 salt) view returns (address)'],
        this.provider
      );

      return await factoryContract.getAddress(owner, BigInt(salt));
    } catch (error) {
      console.error('Failed to calculate account address:', error);
      throw error;
    }
  }

  // 构建 ERC20 转账 UserOperation
  async buildTransferUserOp(params: TransferParams): Promise<{
    userOp: UserOperation;
    gasEstimate: any;
  }> {
    try {
      const wallet = new ethers.Wallet(this.privateKey);

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
      const maxFeePerGas = feeData.maxFeePerGas || ethers.parseUnits('100', 'gwei');
      const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || ethers.parseUnits('2', 'gwei');

      // 构建基础 UserOperation
      const baseUserOp: Partial<UserOperation> = {
        sender: params.from,
        nonce: ethers.toBeHex(nonce),
        initCode: '0x',
        callData: executeData,
        paymasterAndData: '0x',
        signature: '0x',
      };

      // 估算 gas
      const gasEstimate = await this.bundlerService.estimateUserOperationGas(
        baseUserOp,
        this.entryPointAddress
      );

      // 完整的 UserOperation
      const userOp: UserOperation = {
        ...baseUserOp,
        callGasLimit: gasEstimate.callGasLimit,
        verificationGasLimit: gasEstimate.verificationGasLimit,
        preVerificationGas: gasEstimate.preVerificationGas,
        maxFeePerGas: ethers.toBeHex(maxFeePerGas),
        maxPriorityFeePerGas: ethers.toBeHex(maxPriorityFeePerGas),
      } as UserOperation;

      // 计算签名
      const signature = await this.signUserOperation(userOp, wallet);
      userOp.signature = signature;

      return { userOp, gasEstimate };
    } catch (error) {
      console.error('Failed to build transfer UserOp:', error);
      throw error;
    }
  }

  // 执行转账
  async executeTransfer(params: TransferParams): Promise<TransferResult> {
    try {
      const { userOp, gasEstimate } = await this.buildTransferUserOp(params);

      // 发送 UserOperation
      const userOpHash = await this.bundlerService.sendUserOperation(
        userOp,
        this.entryPointAddress
      );

      // 等待确认
      const receipt = await this.bundlerService.waitForUserOpReceipt(userOpHash);

      return {
        userOpHash,
        success: receipt.success,
        receipt,
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

  // 签名 UserOperation
  private async signUserOperation(
    userOp: UserOperation,
    wallet: ethers.Wallet
  ): Promise<string> {
    try {
      // 计算 UserOperation hash
      const userOpHash = this.getUserOpHash(userOp);

      // 使用 Ethereum Signed Message 格式签名（SimpleAccount v0.6 要求）
      const signature = await wallet.signMessage(ethers.getBytes(userOpHash));

      return signature;
    } catch (error) {
      console.error('Failed to sign UserOperation:', error);
      throw error;
    }
  }

  // 计算 UserOperation Hash（ERC-4337 标准）
  private getUserOpHash(userOp: UserOperation): string {
    const chainId = 11155111; // Sepolia chain ID

    const packedUserOp = ethers.AbiCoder.defaultAbiCoder().encode(
      [
        'address', 'uint256', 'bytes32', 'bytes32',
        'uint256', 'uint256', 'uint256', 'uint256',
        'uint256', 'bytes32'
      ],
      [
        userOp.sender,
        userOp.nonce,
        ethers.keccak256(userOp.initCode),
        ethers.keccak256(userOp.callData),
        userOp.callGasLimit,
        userOp.verificationGasLimit,
        userOp.preVerificationGas,
        userOp.maxFeePerGas,
        userOp.maxPriorityFeePerGas,
        ethers.keccak256(userOp.paymasterAndData),
      ]
    );

    const encoded = ethers.AbiCoder.defaultAbiCoder().encode(
      ['bytes32', 'address', 'uint256'],
      [ethers.keccak256(packedUserOp), this.entryPointAddress, chainId]
    );

    return ethers.keccak256(encoded);
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