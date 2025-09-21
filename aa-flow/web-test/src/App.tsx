import { useState, useEffect } from 'react';
import './App.css';

// Components
import NetworkSelector from './components/NetworkSelector';
import BundlerStatus from './components/BundlerStatus';
import GasCalculator from './components/GasCalculator';
import AccountManager from './components/AccountManager';
import TransferTest from './components/TransferTest';
import EnvConfigDisplay from './components/EnvConfigDisplay';

// Services
import { BundlerService } from './services/bundlerService';
import { AccountService } from './services/accountService';
import { NETWORKS, DEFAULT_NETWORK } from './config/networks';

function App() {
  const [selectedNetwork, setSelectedNetwork] = useState(DEFAULT_NETWORK);
  const [bundlerService, setBundlerService] = useState<BundlerService | null>(null);
  const [accountService, setAccountService] = useState<AccountService | null>(null);

  // 初始化服务
  useEffect(() => {
    const network = NETWORKS[selectedNetwork];
    if (!network) return;

    // 初始化 Bundler 服务
    const bundler = new BundlerService(network.bundlerUrl || '');
    setBundlerService(bundler);

    // 初始化 Account 服务
    if (network.bundlerUrl) {
      const account = new AccountService(
        network.rpcUrl,
        network.bundlerUrl,
        import.meta.env.VITE_PRIVATE_KEY || '',
        network.contracts.entryPoint,
        network.contracts.factory
      );
      setAccountService(account);
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

        {/* 账户管理 */}
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
