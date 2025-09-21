import { useState } from 'react';
import './App.css';
import { NETWORKS, DEFAULT_NETWORK } from './config/networks';

function SimpleApp() {
  const [selectedNetwork, setSelectedNetwork] = useState(DEFAULT_NETWORK);

  return (
    <div className="app">
      <header className="app-header">
        <h1>ERC-4337 Rundler Testing Interface</h1>
        <p>Comprehensive testing interface for Rundler bundler service</p>
        <div className="network-selector-container">
          <select
            data-testid="network-selector"
            value={selectedNetwork}
            onChange={(e) => setSelectedNetwork(e.target.value)}
          >
            {Object.entries(NETWORKS).map(([key, network]) => (
              <option key={key} value={key}>
                {network.name}
              </option>
            ))}
          </select>
        </div>
      </header>

      <main className="app-main">
        <section className="config-section">
          <div className="env-config-card">
            <h3>⚙️ Environment Configuration</h3>
            <p>Configuration Status: Ready</p>
          </div>
        </section>

        <section className="status-section">
          <div className="bundler-status-card">
            <h3>🔧 Bundler Status</h3>
            <p>Bundler URL: {NETWORKS[selectedNetwork]?.bundlerUrl}</p>
          </div>
        </section>

        <section className="gas-section">
          <div className="gas-calculator-card">
            <h3>⛽ Gas Calculator</h3>
            <p>preVerificationGas: Dynamic calculation</p>
            <p>callGasLimit: Based on transaction complexity</p>
            <p>verificationGasLimit: Account verification overhead</p>
          </div>
        </section>

        <section className="account-section">
          <div className="account-manager-card">
            <h3>👛 Account Management</h3>
            <p>🪙 Token Information</p>
            <div>🔑 EOA (Owner)</div>
            <div>📤 SimpleAccount A (Sender)</div>
            <div>📥 SimpleAccount B (Receiver)</div>
            <p>🏭 SimpleAccount Factory</p>
          </div>
        </section>

        <section className="transfer-section">
          <div className="transfer-test-card">
            <h3>🚀 Transfer Test</h3>
            <input
              type="number"
              placeholder="Enter amount to transfer"
              defaultValue="3"
            />
            <button>Execute Transfer</button>
            <button>Clear History</button>
          </div>
        </section>
      </main>

      <footer className="app-footer">
        <p>
          ERC-4337 Account Abstraction Testing Interface
        </p>
        <p>
          <a href="https://sepolia.etherscan.io" target="_blank" rel="noopener noreferrer">
            🔍 Etherscan
          </a>
        </p>
      </footer>
    </div>
  );
}

export default SimpleApp;