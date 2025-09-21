import React, { useState, useEffect } from 'react';
import { BundlerService, GasCalculator as GasCalc } from '../services/bundlerService';
import { PriceService } from '../services/priceService';
import type { NetworkConfig } from '../config/networks';
import { ethers } from 'ethers';

interface GasCalculatorProps {
  bundlerService: BundlerService | null;
  networkConfig: NetworkConfig;
}

interface GasEstimation {
  callGasLimit: number;
  verificationGasLimit: number;
  preVerificationGas: number;
  maxFeePerGas: string;
  maxPriorityFeePerGas: string;
  totalGas: number;
  estimatedCostWei: bigint;
  estimatedCostEth: string;
  estimatedCostUSD?: string;
}

const GasCalculator: React.FC<GasCalculatorProps> = ({
  bundlerService,
  networkConfig,
}) => {
  const [gasEstimation, setGasEstimation] = useState<GasEstimation | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentGasPrice, setCurrentGasPrice] = useState<string>('0');
  const [ethPrice, setEthPrice] = useState<number>(0);

  // 获取当前 gas 价格
  const fetchCurrentGasPrice = async () => {
    if (!networkConfig.rpcUrl) return;

    try {
      const provider = new ethers.JsonRpcProvider(networkConfig.rpcUrl);
      const feeData = await provider.getFeeData();
      const gasPrice = feeData.gasPrice || ethers.parseUnits('20', 'gwei');
      setCurrentGasPrice(ethers.formatUnits(gasPrice, 'gwei'));
    } catch (error) {
      console.error('Failed to fetch gas price:', error);
    }
  };

  // 获取 ETH 价格
  const fetchETHPrice = async () => {
    try {
      const price = await PriceService.getEthPrice();
      setEthPrice(price);
    } catch (error) {
      console.error('Failed to fetch ETH price:', error);
      setEthPrice(3000); // 后备价格
    }
  };

  // 计算示例 ERC20 转账的 gas 估算
  const calculateGasEstimation = async () => {
    if (!bundlerService) return;

    setLoading(true);
    setError(null);

    try {
      // 模拟一个 ERC20 转账的 UserOperation
      // const mockUserOp = {
      //   sender: import.meta.env.VITE_SIMPLE_ACCOUNT_A || '0x7D7a0D3239285faE78F9c364D81bb1E3bc555BC6',
      //   nonce: '0x0',
      //   initCode: '0x',
      //   callData: '0x', // 实际会是 SimpleAccount.execute() 调用
      //   paymasterAndData: '0x',
      //   signature: '0x',
      // };

      // 使用固定的 gas 估算值（基于我们的成功测试）
      const gasEstimate = {
        callGasLimit: '0x15F90', // 90000
        verificationGasLimit: '0x15F90', // 90000
        preVerificationGas: '0xAF3C', // 44868
        maxFeePerGas: ethers.parseUnits(currentGasPrice || '100', 'gwei').toString(),
        maxPriorityFeePerGas: ethers.parseUnits('2', 'gwei').toString(),
      };

      const callGas = parseInt(gasEstimate.callGasLimit);
      const verificationGas = parseInt(gasEstimate.verificationGasLimit);
      const preVerificationGas = parseInt(gasEstimate.preVerificationGas);
      const totalGas = callGas + verificationGas + preVerificationGas;

      const maxFeePerGasWei = BigInt(gasEstimate.maxFeePerGas);
      const estimatedCostWei = BigInt(totalGas) * maxFeePerGasWei;
      const estimatedCostEth = ethers.formatEther(estimatedCostWei);
      const estimatedCostUSD = ethPrice > 0 ? (parseFloat(estimatedCostEth) * ethPrice).toFixed(4) : undefined;

      setGasEstimation({
        callGasLimit: callGas,
        verificationGasLimit: verificationGas,
        preVerificationGas,
        maxFeePerGas: ethers.formatUnits(maxFeePerGasWei, 'gwei'),
        maxPriorityFeePerGas: ethers.formatUnits(gasEstimate.maxPriorityFeePerGas, 'gwei'),
        totalGas,
        estimatedCostWei,
        estimatedCostEth,
        estimatedCostUSD,
      });
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchCurrentGasPrice();
    fetchETHPrice();
  }, [networkConfig]);

  useEffect(() => {
    if (currentGasPrice && currentGasPrice !== '0') {
      calculateGasEstimation();
    }
  }, [currentGasPrice, bundlerService]);

  return (
    <div className="gas-calculator-card">
      <div className="card-header">
        <h3>⛽ Gas Price Calculator</h3>
        <button
          className="refresh-btn"
          onClick={calculateGasEstimation}
          disabled={loading}
        >
          {loading ? '🔄' : '↻'} Calculate
        </button>
      </div>

      <div className="gas-rules-section">
        <h4>📋 ERC-4337 Gas Calculation Rules</h4>
        <div className="rules-grid">
          <div className="rule-item">
            <span className="rule-label">Fixed Gas Overhead:</span>
            <span className="rule-value">{GasCalc.FIXED_GAS_OVERHEAD.toLocaleString()} gas</span>
            <span className="rule-desc">基础交易 gas</span>
          </div>
          <div className="rule-item">
            <span className="rule-label">Per UserOp Overhead:</span>
            <span className="rule-value">{GasCalc.PER_USER_OP_OVERHEAD.toLocaleString()} gas</span>
            <span className="rule-desc">每个 UserOp 的固定开销</span>
          </div>
          <div className="rule-item">
            <span className="rule-label">Per UserOp Word:</span>
            <span className="rule-value">{GasCalc.PER_USER_OP_WORD} gas</span>
            <span className="rule-desc">每个字的开销</span>
          </div>
          <div className="rule-item">
            <span className="rule-label">Zero Byte Cost:</span>
            <span className="rule-value">{GasCalc.ZERO_BYTE_COST} gas</span>
            <span className="rule-desc">零字节成本</span>
          </div>
          <div className="rule-item">
            <span className="rule-label">Non-Zero Byte Cost:</span>
            <span className="rule-value">{GasCalc.NON_ZERO_BYTE_COST} gas</span>
            <span className="rule-desc">非零字节成本</span>
          </div>
        </div>
        <div className="rules-note">
          <strong>注意：</strong> 这些规则由 EntryPoint v0.6 合约实时计算，用于确定 preVerificationGas 的最小值。
        </div>
      </div>

      <div className="current-gas-section">
        <h4>🔥 Current Network Gas Prices</h4>
        <div className="gas-prices-grid">
          <div className="gas-price-item">
            <span className="gas-label">Current Gas Price:</span>
            <span className="gas-value">{currentGasPrice} Gwei</span>
          </div>
          <div className="gas-price-item">
            <span className="gas-label">Network:</span>
            <span className="gas-value">{networkConfig.name}</span>
          </div>
          {ethPrice > 0 && (
            <div className="gas-price-item">
              <span className="gas-label">ETH Price:</span>
              <span className="gas-value">${ethPrice.toLocaleString()}</span>
            </div>
          )}
        </div>
      </div>

      {gasEstimation && (
        <div className="estimation-section">
          <h4>💰 ERC20 Transfer Cost Estimation</h4>

          <div className="gas-breakdown">
            <div className="breakdown-item">
              <span className="breakdown-label">Call Gas Limit:</span>
              <span className="breakdown-value">{gasEstimation.callGasLimit.toLocaleString()} gas</span>
              <span className="breakdown-desc">执行 SimpleAccount.execute() 的 gas</span>
            </div>
            <div className="breakdown-item">
              <span className="breakdown-label">Verification Gas Limit:</span>
              <span className="breakdown-value">{gasEstimation.verificationGasLimit.toLocaleString()} gas</span>
              <span className="breakdown-desc">验证签名和权限的 gas</span>
            </div>
            <div className="breakdown-item">
              <span className="breakdown-label">Pre-Verification Gas:</span>
              <span className="breakdown-value">{gasEstimation.preVerificationGas.toLocaleString()} gas</span>
              <span className="breakdown-desc">Bundler 处理开销</span>
            </div>
            <div className="breakdown-item total">
              <span className="breakdown-label">Total Gas:</span>
              <span className="breakdown-value">{gasEstimation.totalGas.toLocaleString()} gas</span>
              <span className="breakdown-desc">总 gas 消耗</span>
            </div>
          </div>

          <div className="cost-summary">
            <div className="cost-item">
              <span className="cost-label">Max Fee Per Gas:</span>
              <span className="cost-value">{gasEstimation.maxFeePerGas} Gwei</span>
            </div>
            <div className="cost-item">
              <span className="cost-label">Priority Fee:</span>
              <span className="cost-value">{gasEstimation.maxPriorityFeePerGas} Gwei</span>
            </div>
            <div className="cost-item estimated-cost">
              <span className="cost-label">Estimated Cost:</span>
              <span className="cost-value">
                {gasEstimation.estimatedCostEth} ETH
                {gasEstimation.estimatedCostUSD && (
                  <span className="usd-value"> (~${gasEstimation.estimatedCostUSD})</span>
                )}
              </span>
            </div>
          </div>

          <div className="cost-explanation">
            <h5>💡 Cost Explanation</h5>
            <p>
              <strong>估算成本：</strong> 这是基于当前 gas 价格的最大可能成本。实际交易通常会消耗更少的 gas，
              多余的 gas 费用会返还给 EOA（{import.meta.env.VITE_EOA_ADDRESS?.slice(0, 8)}...）。
            </p>
            <p>
              <strong>支付流程：</strong> EOA 预先支付 gas 费用给 EntryPoint 合约，交易完成后退还未使用的部分。
            </p>
          </div>
        </div>
      )}

      {error && (
        <div className="error-section">
          <h4>❌ Error</h4>
          <div className="error-message">{error}</div>
        </div>
      )}

      <style jsx>{`
        .gas-calculator-card {
          background: white;
          border-radius: 12px;
          padding: 24px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          margin-bottom: 24px;
        }

        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
        }

        .card-header h3 {
          margin: 0;
          color: #1a1a1a;
          font-size: 1.25rem;
        }

        .refresh-btn {
          padding: 6px 12px;
          border: 1px solid #007bff;
          background: white;
          color: #007bff;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 500;
          transition: all 0.2s;
        }

        .refresh-btn:hover:not(:disabled) {
          background: #007bff;
          color: white;
        }

        .refresh-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .gas-rules-section, .current-gas-section, .estimation-section {
          margin-bottom: 24px;
          padding-bottom: 20px;
          border-bottom: 1px solid #e0e0e0;
        }

        .gas-rules-section:last-child, .current-gas-section:last-child, .estimation-section:last-child {
          border-bottom: none;
        }

        .gas-rules-section h4, .current-gas-section h4, .estimation-section h4 {
          margin: 0 0 16px 0;
          color: #1a1a1a;
          font-size: 1rem;
        }

        .rules-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
          gap: 12px;
          margin-bottom: 16px;
        }

        .rule-item {
          display: flex;
          flex-direction: column;
          padding: 12px;
          background: #f8f9fa;
          border-radius: 8px;
          border-left: 4px solid #007bff;
        }

        .rule-label {
          font-weight: 600;
          color: #495057;
          font-size: 0.875rem;
        }

        .rule-value {
          font-family: 'Monaco', 'Consolas', monospace;
          font-weight: 700;
          color: #007bff;
          font-size: 1rem;
          margin: 4px 0;
        }

        .rule-desc {
          font-size: 0.75rem;
          color: #666;
          font-style: italic;
        }

        .rules-note {
          background: #e7f3ff;
          border: 1px solid #b3d9ff;
          border-radius: 8px;
          padding: 12px;
          font-size: 0.875rem;
          color: #0066cc;
        }

        .gas-prices-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 12px;
        }

        .gas-price-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px;
          background: #f8f9fa;
          border-radius: 8px;
        }

        .gas-label {
          font-weight: 500;
          color: #666;
        }

        .gas-value {
          font-family: 'Monaco', 'Consolas', monospace;
          font-weight: 600;
          color: #1a1a1a;
        }

        .gas-breakdown {
          margin-bottom: 20px;
        }

        .breakdown-item {
          display: grid;
          grid-template-columns: 2fr 1fr 2fr;
          gap: 12px;
          align-items: center;
          padding: 12px;
          border-bottom: 1px solid #f0f0f0;
        }

        .breakdown-item.total {
          background: #e7f3ff;
          border-radius: 8px;
          border: 1px solid #b3d9ff;
          font-weight: 600;
        }

        .breakdown-label {
          font-weight: 500;
          color: #495057;
        }

        .breakdown-value {
          font-family: 'Monaco', 'Consolas', monospace;
          font-weight: 600;
          color: #007bff;
          text-align: right;
        }

        .breakdown-desc {
          font-size: 0.875rem;
          color: #666;
          font-style: italic;
        }

        .cost-summary {
          background: #f8f9fa;
          border-radius: 8px;
          padding: 16px;
          margin-bottom: 20px;
        }

        .cost-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
        }

        .cost-item:last-child {
          margin-bottom: 0;
        }

        .cost-item.estimated-cost {
          padding-top: 12px;
          border-top: 1px solid #e0e0e0;
          font-weight: 600;
          font-size: 1.1rem;
        }

        .cost-label {
          color: #495057;
        }

        .cost-value {
          font-family: 'Monaco', 'Consolas', monospace;
          font-weight: 600;
          color: #007bff;
        }

        .usd-value {
          color: #28a745;
          font-weight: 500;
        }

        .cost-explanation {
          background: #fff3cd;
          border: 1px solid #ffeaa7;
          border-radius: 8px;
          padding: 16px;
        }

        .cost-explanation h5 {
          margin: 0 0 12px 0;
          color: #856404;
        }

        .cost-explanation p {
          margin: 0 0 8px 0;
          color: #856404;
          font-size: 0.875rem;
          line-height: 1.4;
        }

        .cost-explanation p:last-child {
          margin-bottom: 0;
        }

        .error-section {
          background: #f8d7da;
          border: 1px solid #f1aeb5;
          border-radius: 8px;
          padding: 16px;
        }

        .error-section h4 {
          margin: 0 0 8px 0;
          color: #721c24;
        }

        .error-message {
          color: #721c24;
          font-size: 0.875rem;
          font-family: 'Monaco', 'Consolas', monospace;
        }

        @media (max-width: 768px) {
          .rules-grid {
            grid-template-columns: 1fr;
          }

          .gas-prices-grid {
            grid-template-columns: 1fr;
          }

          .breakdown-item {
            grid-template-columns: 1fr;
            text-align: left;
          }

          .cost-item {
            flex-direction: column;
            align-items: flex-start;
            gap: 4px;
          }
        }
      `}</style>
    </div>
  );
};

export default GasCalculator;