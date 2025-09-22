import React, { useState, useEffect } from 'react';
import { AccountService } from '../services/accountService';
import type { TransferResult } from '../services/accountService';
import { BundlerService } from '../services/bundlerService';
import type { UserOpReceipt } from '../services/bundlerService';
import type { NetworkConfig } from '../config/networks';
import { getJiffyScanUrl, getBlockExplorerTxUrl } from '../config/networks';
import { ethers } from 'ethers';
import { DebugLogger } from '../utils/debugLogger';
import UserOpDisplay from './UserOpDisplay';
import GasCalculatorAdvanced from './GasCalculatorAdvanced';

interface TransferTestProps {
  accountService: AccountService | null;
  bundlerService: BundlerService | null;
  networkConfig: NetworkConfig;
  signer?: ethers.Signer | null; // 可选的 MetaMask signer
}

interface TransferState {
  amount: string;
  isTransferring: boolean;
  result: TransferResult | null;
  receipt: UserOpReceipt | null;
  gasAnalysis: any | null;
  error: string | null;
  debugInfo: string[];
}

const TransferTest: React.FC<TransferTestProps> = ({
  accountService,
  bundlerService,
  networkConfig,
  signer,
}) => {
  const [transferState, setTransferState] = useState<TransferState>({
    amount: '3',
    isTransferring: false,
    result: null,
    receipt: null,
    gasAnalysis: null,
    error: null,
    debugInfo: [],
  });


  const [initialBalances, setInitialBalances] = useState<{
    accountA: string;
    accountB: string;
    eoa: string;
  } | null>(null);

  const [finalBalances, setFinalBalances] = useState<{
    accountA: string;
    accountB: string;
    eoa: string;
  } | null>(null);

  const [showUserOpDetails, setShowUserOpDetails] = useState<boolean>(false);

  const [debugLogs, setDebugLogs] = useState<string[]>([]);
  const [showDebugPanel, setShowDebugPanel] = useState<boolean>(false);

  const addresses = {
    accountA: import.meta.env.VITE_SIMPLE_ACCOUNT_A,
    accountB: import.meta.env.VITE_SIMPLE_ACCOUNT_B,
    eoa: import.meta.env.VITE_EOA_ADDRESS,
    pntToken: import.meta.env.VITE_PNT_TOKEN_ADDRESS,
  };

  // 监听调试日志
  useEffect(() => {
    const handleLogs = (logs: string[]) => {
      setDebugLogs(logs);
    };

    DebugLogger.addListener(handleLogs);
    return () => DebugLogger.removeListener(handleLogs);
  }, []);

  // 获取余额
  const getBalances = async () => {
    if (!accountService) return null;

    try {
      const [accountAInfo, accountBInfo, eoaInfo] = await Promise.all([
        accountService.getAccountInfo(addresses.accountA, addresses.pntToken),
        accountService.getAccountInfo(addresses.accountB, addresses.pntToken),
        accountService.getAccountInfo(addresses.eoa, addresses.pntToken),
      ]);

      return {
        accountA: accountAInfo.tokenBalance,
        accountB: accountBInfo.tokenBalance,
        eoa: eoaInfo.ethBalance,
      };
    } catch (error) {
      console.error('Failed to get balances:', error);
      return null;
    }
  };

  // 执行转账测试
  const executeTransfer = async () => {
    if (!accountService || !bundlerService) return;


    // 清理调试日志并开始记录
    DebugLogger.clear();
    DebugLogger.log('🚀 开始执行转账...');

    setTransferState(prev => ({
      ...prev,
      isTransferring: true,
      result: null,
      receipt: null,
      gasAnalysis: null,
      error: null,
    }));

    try {
      // 获取初始余额
      DebugLogger.log('📊 获取初始余额...');
      const initial = await getBalances();
      setInitialBalances(initial);
      DebugLogger.log(`💰 初始余额 - A: ${initial?.accountA} PNT, B: ${initial?.accountB} PNT`);

      // 执行转账
      DebugLogger.log(`💸 执行转账: ${transferState.amount} PNT 从 A → B`);

      // 检查是否有可用的私钥或 signer
      const privateKeyValue = import.meta.env.VITE_PRIVATE_KEY;
      const hasPrivateKey = !!privateKeyValue;
      const hasSigner = !!signer;

      DebugLogger.log(`🔑 私钥环境变量: ${privateKeyValue ? '已配置' : '未配置'}`);
      DebugLogger.log(`🔑 私钥可用: ${hasPrivateKey}`);
      DebugLogger.log(`🦊 MetaMask signer 可用: ${hasSigner}`);
      DebugLogger.log(`🦊 Signer 类型: ${signer ? typeof signer : 'undefined'}`);
      DebugLogger.log(`🌐 当前环境: ${import.meta.env.MODE}`);
      DebugLogger.log(`🔍 window.ethereum 可用: ${typeof window !== 'undefined' && !!window.ethereum}`);

      if (signer) {
        try {
          const address = await signer.getAddress();
          DebugLogger.log(`🦊 MetaMask 地址: ${address}`);

          // 测试 signer 是否真的可以工作
          const provider = signer.provider;
          if (provider) {
            const network = await provider.getNetwork();
            DebugLogger.log(`🌐 MetaMask 网络: ${network.name} (${network.chainId})`);
          } else {
            DebugLogger.log(`⚠️ Signer 没有 provider`);
          }
        } catch (error) {
          DebugLogger.log(`🦊 获取 MetaMask 信息失败: ${error}`);
        }
      }

      if (!hasPrivateKey && !hasSigner) {
        throw new Error('没有可用的签名方式：请连接 MetaMask 钱包进行签名操作');
      }

      // 在生产环境中优先使用 MetaMask，本地开发可以使用私钥
      if (hasSigner) {
        DebugLogger.log('✅ 使用 MetaMask 进行签名');
      } else if (hasPrivateKey) {
        DebugLogger.log('✅ 使用环境变量私钥进行签名（仅限开发环境）');
      }

      const result = await accountService.executeTransfer({
        from: addresses.accountA,
        to: addresses.accountB,
        amount: transferState.amount,
        tokenAddress: addresses.pntToken,
        privateKey: import.meta.env.VITE_PRIVATE_KEY, // 从环境变量获取私钥
        signer: signer || undefined, // 传递 MetaMask signer
      });

      if (!result.success) {
        DebugLogger.error(`❌ 转账失败: ${result.error || 'Unknown error'}`);
        throw new Error(result.error || 'Transfer failed');
      }

      DebugLogger.log(`✅ 转账成功! UserOp Hash: ${result.userOpHash}`);

      // 获取详细收据
      DebugLogger.log('📋 获取交易收据...');
      const receipt = await bundlerService.getUserOperationReceipt(result.userOpHash);

      // 获取最终余额
      DebugLogger.log('📊 获取最终余额...');
      const final = await getBalances();
      setFinalBalances(final);
      DebugLogger.log(`💰 最终余额 - A: ${final?.accountA} PNT, B: ${final?.accountB} PNT`);

      // 分析 gas 使用情况
      let gasAnalysis = null;
      if (receipt) {
        DebugLogger.log('⛽ 分析 Gas 使用情况...');
        gasAnalysis = await accountService.analyzeGasUsage(result.userOpHash);
      }

      DebugLogger.log('🎉 转账流程完成!');
      setTransferState(prev => ({
        ...prev,
        result,
        receipt,
        gasAnalysis,
      }));

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      DebugLogger.error(`💥 转账过程中发生错误: ${errorMessage}`);
      setTransferState(prev => ({
        ...prev,
        error: errorMessage,
      }));
    } finally {
      setTransferState(prev => ({
        ...prev,
        isTransferring: false,
      }));
    }
  };

  const resetTest = () => {
    setTransferState({
      amount: '3',
      isTransferring: false,
      result: null,
      receipt: null,
      gasAnalysis: null,
      error: null,
      debugInfo: [],
    });
    setInitialBalances(null);
    setFinalBalances(null);
  };

  // UserOperation 数据展示组件 (已移除)

  return (
    <div className="transfer-test-card">
      <div className="card-header">
        <h3>🚀 Transfer Test</h3>
        <button
          className="reset-btn"
          onClick={resetTest}
          disabled={transferState.isTransferring}
        >
          🔄 Reset
        </button>
      </div>


      <div className="test-setup">
        <h4>⚙️ Test Configuration</h4>
        <div className="setup-grid">
          <div className="setup-item">
            <label htmlFor="amount-input">Transfer Amount (PNT):</label>
            <input
              id="amount-input"
              type="number"
              value={transferState.amount}
              onChange={(e) => setTransferState(prev => ({ ...prev, amount: e.target.value }))}
              disabled={transferState.isTransferring}
              min="0"
              step="0.1"
              className="amount-input"
            />
          </div>
          <div className="setup-item">
            <span className="setup-label">From:</span>
            <span className="setup-value">SimpleAccount A ({addresses.accountA?.slice(0, 8)}...)</span>
          </div>
          <div className="setup-item">
            <span className="setup-label">To:</span>
            <span className="setup-value">SimpleAccount B ({addresses.accountB?.slice(0, 8)}...)</span>
          </div>
          <div className="setup-item">
            <span className="setup-label">Bundler:</span>
            <span className="setup-value">{networkConfig.bundlerUrl}</span>
          </div>
        </div>

        {/* 签名方式状态显示 */}
        <div className="signing-status">
          <h4>🔐 Signing Method Status</h4>
          <div className="status-grid">
            <div className="status-item">
              <span className="status-label">Private Key (.env):</span>
              <span className={`status-indicator ${!!import.meta.env.VITE_PRIVATE_KEY ? 'available' : 'unavailable'}`}>
                {!!import.meta.env.VITE_PRIVATE_KEY ? '✅ Available (Dev Mode)' : '❌ Not Available (Production)'}
              </span>
            </div>
            <div className="status-item">
              <span className="status-label">MetaMask Signer:</span>
              <span className={`status-indicator ${!!signer ? 'available' : 'unavailable'}`}>
                {!!signer ? '✅ Connected' : '❌ Not Connected'}
              </span>
            </div>
          </div>
          {!import.meta.env.VITE_PRIVATE_KEY && !signer && (
            <div className="signing-notice error">
              <p>⚠️ <strong>操作被阻止:</strong> 生产环境需要连接 MetaMask 钱包</p>
              <p>📱 请点击页面顶部的 "Connect MetaMask" 按钮连接您的钱包</p>
              <p>🔗 确保您的 MetaMask 已切换到 Sepolia 测试网络</p>
            </div>
          )}
        </div>

        <div className="test-actions">
          <button
            className="transfer-btn"
            onClick={executeTransfer}
            disabled={transferState.isTransferring || !accountService || !bundlerService || (!import.meta.env.VITE_PRIVATE_KEY && !signer)}
          >
            {transferState.isTransferring
              ? '🔄 Transferring...'
              : (!import.meta.env.VITE_PRIVATE_KEY && !signer)
                ? '🔒 需要连接 MetaMask'
                : `🚀 Transfer ${transferState.amount} PNT`
            }
          </button>
          {(!import.meta.env.VITE_PRIVATE_KEY && !signer) && (
            <div className="action-hint">
              <p>👆 按钮被禁用: 请先连接 MetaMask 钱包</p>
            </div>
          )}
        </div>
      </div>

      {/* 余额变化显示 */}
      {(initialBalances || finalBalances) && (
        <div className="balance-changes">
          <h4>💰 Balance Changes</h4>
          <div className="balance-grid">
            <div className="balance-item">
              <span className="balance-label">Account A (Sender):</span>
              <div className="balance-change">
                <span className="balance-before">{initialBalances?.accountA || '0'} PNT</span>
                <span className="balance-arrow">→</span>
                <span className="balance-after">{finalBalances?.accountA || '0'} PNT</span>
                {initialBalances && finalBalances && (
                  <span className="balance-diff">
                    ({(parseFloat(finalBalances.accountA) - parseFloat(initialBalances.accountA)).toFixed(2)} PNT)
                  </span>
                )}
              </div>
            </div>
            <div className="balance-item">
              <span className="balance-label">Account B (Receiver):</span>
              <div className="balance-change">
                <span className="balance-before">{initialBalances?.accountB || '0'} PNT</span>
                <span className="balance-arrow">→</span>
                <span className="balance-after">{finalBalances?.accountB || '0'} PNT</span>
                {initialBalances && finalBalances && (
                  <span className="balance-diff positive">
                    (+{(parseFloat(finalBalances.accountB) - parseFloat(initialBalances.accountB)).toFixed(2)} PNT)
                  </span>
                )}
              </div>
            </div>
            <div className="balance-item">
              <span className="balance-label">EOA (Gas Payer):</span>
              <div className="balance-change">
                <span className="balance-before">{initialBalances?.eoa ? parseFloat(initialBalances.eoa).toFixed(6) : '0'} ETH</span>
                <span className="balance-arrow">→</span>
                <span className="balance-after">{finalBalances?.eoa ? parseFloat(finalBalances.eoa).toFixed(6) : '0'} ETH</span>
                {initialBalances && finalBalances && (
                  <span className="balance-diff">
                    ({(parseFloat(finalBalances.eoa) - parseFloat(initialBalances.eoa)).toFixed(6)} ETH)
                  </span>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 转账结果 */}
      {transferState.result && (
        <div className="transfer-result">
          <div className="result-header">
            <h4>
              {transferState.result.success ? '✅ Transfer Successful' : '❌ Transfer Failed'}
            </h4>
          </div>

          <div className="result-details">
            <div className="result-item">
              <span className="result-label">UserOp Hash:</span>
              <div className="hash-container">
                <span className="hash-value">{transferState.result.userOpHash}</span>
                <a
                  href={getJiffyScanUrl(transferState.result.userOpHash, 'sepolia')}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="jiffy-link"
                >
                  🔍 JiffyScan
                </a>
              </div>
            </div>

            {transferState.receipt && (
              <>
                <div className="result-item">
                  <span className="result-label">Transaction Hash:</span>
                  <div className="hash-container">
                    <span className="hash-value">{transferState.receipt.receipt.transactionHash}</span>
                    <a
                      href={getBlockExplorerTxUrl(transferState.receipt.receipt.transactionHash, 'sepolia')}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="explorer-link"
                    >
                      🔍 Etherscan
                    </a>
                  </div>
                </div>

                <div className="result-item">
                  <span className="result-label">Block Number:</span>
                  <span className="result-value">{parseInt(transferState.receipt.receipt.blockNumber).toLocaleString()}</span>
                </div>

                <div className="result-item">
                  <span className="result-label">Gas Used:</span>
                  <span className="result-value">{parseInt(transferState.receipt.actualGasUsed).toLocaleString()} gas</span>
                </div>

                <div className="result-item">
                  <span className="result-label">Actual Gas Cost:</span>
                  <span className="result-value">
                    {ethers.formatEther(transferState.receipt.actualGasCost)} ETH
                  </span>
                </div>

                <div className="result-item">
                  <span className="result-label">Effective Gas Price:</span>
                  <span className="result-value">
                    {ethers.formatUnits(transferState.receipt.receipt.effectiveGasPrice, 'gwei')} Gwei
                  </span>
                </div>
              </>
            )}
          </div>
        </div>
      )}

      {/* Gas 支付说明 */}
      {transferState.receipt && (
        <div className="gas-explanation">
          <h4>⛽ Gas Payment Explanation</h4>
          <div className="gas-flow">
            <div className="gas-step">
              <span className="step-number">1</span>
              <div className="step-content">
                <div className="step-title">EOA 预支付 Gas</div>
                <div className="step-desc">
                  EOA ({addresses.eoa?.slice(0, 8)}...) 向 EntryPoint 预先支付 gas 费用
                </div>
              </div>
            </div>
            <div className="gas-step">
              <span className="step-number">2</span>
              <div className="step-content">
                <div className="step-title">Bundler 执行交易</div>
                <div className="step-desc">
                  Bundler 调用 EntryPoint.handleOps() 执行 UserOperation
                </div>
              </div>
            </div>
            <div className="gas-step">
              <span className="step-number">3</span>
              <div className="step-content">
                <div className="step-title">实际 Gas 消耗</div>
                <div className="step-desc">
                  实际消耗: {transferState.receipt ? parseInt(transferState.receipt.actualGasUsed).toLocaleString() : '0'} gas
                  (成本: {transferState.receipt ? ethers.formatEther(transferState.receipt.actualGasCost) : '0'} ETH)
                </div>
              </div>
            </div>
            <div className="gas-step">
              <span className="step-number">4</span>
              <div className="step-content">
                <div className="step-title">退还多余 Gas</div>
                <div className="step-desc">
                  未使用的 gas 费用自动退还给 EOA
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 高级 Gas 计算器 */}
      {transferState.result?.userOp && (
        <GasCalculatorAdvanced
          userOp={transferState.result.userOp}
          title="🧮 Advanced Gas Calculator"
        />
      )}

      {/* UserOperation 详细信息 */}
      {transferState.result?.userOp && (
        <UserOpDisplay
          userOp={transferState.result.userOp}
          userOpHash={transferState.result.userOpHash}
          title="📋 UserOperation Details"
          isExpanded={showUserOpDetails}
          onToggle={() => setShowUserOpDetails(!showUserOpDetails)}
        />
      )}

      {transferState.error && (
        <div className="error-section">
          <h4>❌ Error</h4>
          <div className="error-message">{transferState.error}</div>
        </div>
      )}

      {/* 调试信息面板 */}
      {debugLogs.length > 0 && (
        <div className="debug-section">
          <div className="debug-header">
            <h4>🔍 Debug Information</h4>
            <button
              className="clear-debug-btn"
              onClick={() => DebugLogger.clear()}
            >
              Clear
            </button>
          </div>
          <div className="debug-logs">
            {debugLogs.map((log, index) => (
              <div key={index} className="debug-log-item">
                {log}
              </div>
            ))}
          </div>
        </div>
      )}

      <style jsx>{`
        .transfer-test-card {
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

        .reset-btn {
          padding: 6px 12px;
          border: 1px solid #6c757d;
          background: white;
          color: #6c757d;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 500;
          transition: all 0.2s;
        }

        .reset-btn:hover:not(:disabled) {
          background: #6c757d;
          color: white;
        }

        .reset-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .test-setup {
          background: #f8f9fa;
          border-radius: 10px;
          padding: 20px;
          margin-bottom: 24px;
        }

        .test-setup h4 {
          margin: 0 0 16px 0;
          color: #1a1a1a;
        }

        .setup-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
          gap: 16px;
          margin-bottom: 20px;
        }

        .setup-item {
          display: flex;
          flex-direction: column;
          gap: 6px;
        }

        .setup-item label, .setup-label {
          font-weight: 500;
          color: #495057;
          font-size: 0.875rem;
        }

        .amount-input {
          padding: 8px 12px;
          border: 1px solid #ced4da;
          border-radius: 6px;
          font-size: 1rem;
          font-weight: 500;
        }

        .amount-input:focus {
          outline: none;
          border-color: #007bff;
          box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.1);
        }

        .setup-value {
          font-family: 'Monaco', 'Consolas', monospace;
          color: #1a1a1a;
          font-size: 0.875rem;
          background: white;
          padding: 6px 8px;
          border-radius: 4px;
          border: 1px solid #e0e0e0;
        }

        .signing-status {
          background: #e7f3ff;
          border: 1px solid #b3d9ff;
          border-radius: 8px;
          padding: 16px;
          margin-bottom: 20px;
        }

        .signing-status h4 {
          margin: 0 0 12px 0;
          color: #0066cc;
          font-size: 1rem;
        }

        .status-grid {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .status-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          background: white;
          border-radius: 6px;
          border: 1px solid #e0e0e0;
        }

        .status-label {
          font-weight: 500;
          color: #495057;
        }

        .status-indicator {
          font-size: 0.875rem;
          font-weight: 600;
          padding: 2px 6px;
          border-radius: 4px;
        }

        .status-indicator.available {
          color: #155724;
          background: #d4edda;
        }

        .status-indicator.unavailable {
          color: #721c24;
          background: #f8d7da;
        }

        .signing-notice {
          margin-top: 12px;
          padding: 12px;
          background: #fff3cd;
          border: 1px solid #ffeaa7;
          border-radius: 6px;
        }

        .signing-notice p {
          margin: 0;
          color: #856404;
          font-size: 0.875rem;
          text-align: center;
        }

        .test-actions {
          text-align: center;
        }

        .transfer-btn {
          padding: 12px 24px;
          background: linear-gradient(135deg, #007bff, #0056b3);
          color: white;
          border: none;
          border-radius: 8px;
          font-size: 1rem;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
          box-shadow: 0 2px 4px rgba(0, 123, 255, 0.2);
        }

        .transfer-btn:hover:not(:disabled) {
          transform: translateY(-1px);
          box-shadow: 0 4px 8px rgba(0, 123, 255, 0.3);
        }

        .transfer-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
          transform: none;
        }

        .balance-changes, .transfer-result, .gas-explanation {
          background: #f8f9fa;
          border-radius: 10px;
          padding: 20px;
          margin-bottom: 20px;
        }

        .balance-changes h4, .transfer-result h4, .gas-explanation h4 {
          margin: 0 0 16px 0;
          color: #1a1a1a;
        }

        .balance-grid {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .balance-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px;
          background: white;
          border-radius: 6px;
          border: 1px solid #e0e0e0;
        }

        .balance-label {
          font-weight: 500;
          color: #495057;
          min-width: 150px;
        }

        .balance-change {
          display: flex;
          align-items: center;
          gap: 8px;
          font-family: 'Monaco', 'Consolas', monospace;
          font-size: 0.875rem;
        }

        .balance-before, .balance-after {
          color: #1a1a1a;
          font-weight: 500;
        }

        .balance-arrow {
          color: #666;
          font-weight: bold;
        }

        .balance-diff {
          color: #dc3545;
          font-weight: 600;
          font-size: 0.8rem;
        }

        .balance-diff.positive {
          color: #28a745;
        }

        .result-details {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .result-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 0;
          border-bottom: 1px solid #e0e0e0;
        }

        .result-item:last-child {
          border-bottom: none;
        }

        .result-label {
          font-weight: 500;
          color: #495057;
        }

        .result-value {
          font-family: 'Monaco', 'Consolas', monospace;
          color: #1a1a1a;
          font-weight: 500;
        }

        .hash-container {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .hash-value {
          font-family: 'Monaco', 'Consolas', monospace;
          color: #1a1a1a;
          font-size: 0.875rem;
          word-break: break-all;
        }

        .jiffy-link, .explorer-link {
          color: #007bff;
          text-decoration: none;
          font-size: 0.75rem;
          padding: 2px 6px;
          border: 1px solid #007bff;
          border-radius: 4px;
          white-space: nowrap;
          transition: all 0.2s;
        }

        .jiffy-link:hover, .explorer-link:hover {
          background: #007bff;
          color: white;
        }

        .gas-flow {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .gas-step {
          display: flex;
          align-items: flex-start;
          gap: 12px;
        }

        .step-number {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 28px;
          height: 28px;
          background: #007bff;
          color: white;
          border-radius: 50%;
          font-weight: bold;
          font-size: 0.875rem;
          flex-shrink: 0;
        }

        .step-content {
          flex: 1;
          background: white;
          padding: 12px;
          border-radius: 6px;
          border: 1px solid #e0e0e0;
        }

        .step-title {
          font-weight: 600;
          color: #1a1a1a;
          margin-bottom: 4px;
        }

        .step-desc {
          color: #666;
          font-size: 0.875rem;
          line-height: 1.4;
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

        .debug-section {
          background: #f8f9fa;
          border: 1px solid #dee2e6;
          border-radius: 8px;
          padding: 16px;
          margin-top: 16px;
        }

        .debug-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .debug-header h4 {
          margin: 0;
          color: #495057;
          font-size: 1rem;
        }

        .clear-debug-btn {
          padding: 4px 8px;
          background: #dc3545;
          color: white;
          border: none;
          border-radius: 4px;
          font-size: 0.75rem;
          cursor: pointer;
        }

        .clear-debug-btn:hover {
          background: #c82333;
        }

        .debug-logs {
          max-height: 400px;
          overflow-y: auto;
          background: #ffffff;
          border: 1px solid #e9ecef;
          border-radius: 4px;
          padding: 8px;
        }

        .debug-log-item {
          font-family: 'Monaco', 'Consolas', monospace;
          font-size: 0.75rem;
          line-height: 1.4;
          color: #495057;
          margin-bottom: 4px;
          word-wrap: break-word;
          white-space: pre-wrap;
        }

        .signing-notice.error {
          background: #f8d7da;
          border: 1px solid #f5c6cb;
          animation: pulse 2s infinite;
        }

        .signing-notice.error p {
          color: #721c24;
          margin: 4px 0;
        }

        .action-hint {
          margin-top: 8px;
          padding: 8px;
          background: #f8d7da;
          border: 1px solid #f5c6cb;
          border-radius: 4px;
          font-size: 0.8rem;
          color: #721c24;
        }

        .action-hint p {
          margin: 0;
          text-align: center;
        }

        @keyframes pulse {
          0% { opacity: 1; }
          50% { opacity: 0.7; }
          100% { opacity: 1; }
        }

        @media (max-width: 768px) {
          .setup-grid {
            grid-template-columns: 1fr;
          }

          .balance-item {
            flex-direction: column;
            align-items: flex-start;
            gap: 8px;
          }

          .result-item {
            flex-direction: column;
            align-items: flex-start;
            gap: 6px;
          }

          .hash-container {
            width: 100%;
            flex-direction: column;
            align-items: flex-start;
            gap: 6px;
          }

          .balance-change {
            flex-wrap: wrap;
          }
        }
      `}</style>
    </div>
  );
};

export default TransferTest;