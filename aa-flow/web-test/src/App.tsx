import { useState, useEffect } from 'react';
import './App.css';

// Components
import NetworkSelector from './components/NetworkSelector';
import BundlerStatus from './components/BundlerStatus';
import GasCalculator from './components/GasCalculator';
import AccountManager from './components/AccountManager';
import TransferTest from './components/TransferTest';
import EnvConfigDisplay from './components/EnvConfigDisplay';
import MetaMaskWallet from './components/MetaMaskWallet';

// Services
import { BundlerService } from './services/bundlerService';
import { AccountService } from './services/accountService';
import { NETWORKS, DEFAULT_NETWORK } from './config/networks';

function App() {
  const [selectedNetwork, setSelectedNetwork] = useState(DEFAULT_NETWORK);
  const [bundlerService, setBundlerService] = useState<BundlerService | null>(null);
  const [accountService, setAccountService] = useState<AccountService | null>(null);
  const [connectedAccount, setConnectedAccount] = useState<string>('');
  const [signer, setSigner] = useState<any>(null);

  // 初始化服务
  useEffect(() => {
    const network = NETWORKS[selectedNetwork];
    if (!network) {
      console.error('Network not found:', selectedNetwork);
      return;
    }

    console.log('初始化网络:', selectedNetwork, network);

    // 初始化 Bundler 服务
    if (network.bundlerUrl) {
      const bundler = new BundlerService(network.bundlerUrl);
      setBundlerService(bundler);
      console.log('Bundler 服务已初始化:', network.bundlerUrl);
    } else {
      console.warn('缺少 bundlerUrl 配置');
    }

    // 初始化 Account 服务 (不需要私钥，将在运行时提供)
    if (network.bundlerUrl) {
      const account = new AccountService(
        network.rpcUrl,
        network.bundlerUrl,
        '', // 私钥将在执行时提供
        network.contracts.entryPoint,
        network.contracts.factory
      );
      setAccountService(account);
      console.log('Account 服务已初始化');
    } else {
      console.warn('缺少 bundlerUrl 配置');
    }
  }, [selectedNetwork]);

  return (
    <div className="app">
      <header className="app-header">
        <h1>ERC-4337 Rundler Testing Interface</h1>
        <p>Comprehensive testing interface for Rundler bundler service</p>
        <div className="network-selector-container">
          <NetworkSelector
            selectedNetwork={selectedNetwork}
            onNetworkChange={setSelectedNetwork}
          />
        </div>
      </header>

      <main className="app-main">
        {/* MetaMask 钱包连接 */}
        <section className="wallet-section">
          <MetaMaskWallet
            onAccountChange={(account, signerInstance) => {
              setConnectedAccount(account);
              setSigner(signerInstance);
            }}
            onNetworkChange={(chainId) => {
              console.log('Network changed to:', chainId);
            }}
          />
        </section>

        {/* 环境配置显示 */}
        <section className="config-section">
          <EnvConfigDisplay selectedNetwork={selectedNetwork} />
        </section>

        {/* Bundler 状态 */}
        <section className="status-section">
          <BundlerStatus
            bundlerService={bundlerService}
            networkConfig={NETWORKS[selectedNetwork]}
          />
        </section>

        {/* Gas 计算器 */}
        <section className="gas-section">
          <GasCalculator
            bundlerService={bundlerService}
            networkConfig={NETWORKS[selectedNetwork]}
          />
        </section>

        {/* MetaMask 账户管理 */}
        {signer && (
          <section className="account-section">
            <div className="account-manager">
              <h3>📱 MetaMask Account Manager</h3>
              <div className="account-info">
                <p>连接的账户: {connectedAccount}</p>
                <p>可以使用 MetaMask 进行签名和交易</p>
              </div>
              <style jsx>{`
                .account-manager {
                  background: white;
                  border-radius: 12px;
                  padding: 24px;
                  margin: 16px 0;
                  border: 1px solid #e9ecef;
                }
                .account-info {
                  background: #f8f9fa;
                  padding: 16px;
                  border-radius: 8px;
                  font-family: monospace;
                }
              `}</style>
            </div>
          </section>
        )}

        {/* 原始账户管理 */}
        <section className="account-section">
          <AccountManager
            accountService={accountService}
            networkConfig={NETWORKS[selectedNetwork]}
          />
        </section>

        {/* 转账测试 */}
        <section className="transfer-section">
          <TransferTest
            accountService={accountService}
            bundlerService={bundlerService}
            networkConfig={NETWORKS[selectedNetwork]}
            signer={signer}
          />
        </section>
      </main>

      <footer className="app-footer">
        <p>
          🔗 Powered by{' '}
          <a
            href="https://rundler-superrelay.fly.dev"
            target="_blank"
            rel="noopener noreferrer"
          >
            SuperRelay Rundler
          </a>
          {' '} | Built with ERC-4337 EntryPoint v0.6
        </p>
      </footer>
    </div>
  );
}

export default App;
