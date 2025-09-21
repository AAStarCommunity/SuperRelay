import React, { useState } from 'react';
// import { NETWORKS } from '../config/networks';

interface EnvConfigDisplayProps {
  selectedNetwork: string;
}

const EnvConfigDisplay: React.FC<EnvConfigDisplayProps> = ({ selectedNetwork: _ }) => {
  const [showConfig, setShowConfig] = useState(false);

  // const network = NETWORKS[selectedNetwork];

  const envVars = {
    'VITE_NODE_HTTP': import.meta.env.VITE_NODE_HTTP,
    'VITE_BUNDLER_URL': import.meta.env.VITE_BUNDLER_URL,
    'VITE_PRIVATE_KEY': import.meta.env.VITE_PRIVATE_KEY ? '***CONFIGURED***' : 'NOT_SET',
    'VITE_EOA_ADDRESS': import.meta.env.VITE_EOA_ADDRESS,
    'VITE_SIMPLE_ACCOUNT_A': import.meta.env.VITE_SIMPLE_ACCOUNT_A,
    'VITE_SIMPLE_ACCOUNT_B': import.meta.env.VITE_SIMPLE_ACCOUNT_B,
    'VITE_ENTRYPOINT_ADDRESS': import.meta.env.VITE_ENTRYPOINT_ADDRESS,
    'VITE_FACTORY_ADDRESS': import.meta.env.VITE_FACTORY_ADDRESS,
    'VITE_PNT_TOKEN_ADDRESS': import.meta.env.VITE_PNT_TOKEN_ADDRESS,
  };

  const exampleEnv = `# Rundler AA Web Interface Environment Configuration
# Copy this to your .env file and update with your values

# Primary RPC (working Alchemy endpoint)
VITE_NODE_HTTP="https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY"
VITE_BUNDLER_URL="https://rundler-superrelay.fly.dev"

# Test accounts (you need to generate these)
VITE_PRIVATE_KEY="0xYOUR_PRIVATE_KEY_HERE"
VITE_EOA_ADDRESS="0xYOUR_EOA_ADDRESS_HERE"
VITE_SIMPLE_ACCOUNT_A="0xYOUR_SIMPLE_ACCOUNT_A_HERE"
VITE_SIMPLE_ACCOUNT_B="0xYOUR_SIMPLE_ACCOUNT_B_HERE"

# Contract addresses (Sepolia network)
VITE_ENTRYPOINT_ADDRESS="0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
VITE_FACTORY_ADDRESS="0x9406Cc6185a346906296840746125a0E44976454"
VITE_PNT_TOKEN_ADDRESS="0x3e7B771d4541eC85c8137e950598Ac97553a337a"`;

  const setupSteps = [
    {
      step: 1,
      title: '获取 Sepolia ETH',
      description: '从 https://sepoliafaucet.com/ 获取测试 ETH 到你的 EOA 地址',
      required: true,
    },
    {
      step: 2,
      title: '部署 SimpleAccount',
      description: '使用 aa-flow/scripts/deploy-accounts.js 部署你的 SimpleAccount',
      required: true,
    },
    {
      step: 3,
      title: '获取测试代币',
      description: '向 SimpleAccount A 发送一些 PNT 代币用于测试转账',
      required: true,
    },
    {
      step: 4,
      title: '配置环境变量',
      description: '在 .env 文件中设置所有必需的地址和私钥',
      required: true,
    },
  ];

  const configStatus = {
    rpc: !!envVars.VITE_NODE_HTTP,
    bundler: !!envVars.VITE_BUNDLER_URL,
    privateKey: !!import.meta.env.VITE_PRIVATE_KEY,
    accounts: !!(envVars.VITE_EOA_ADDRESS && envVars.VITE_SIMPLE_ACCOUNT_A && envVars.VITE_SIMPLE_ACCOUNT_B),
    contracts: !!(envVars.VITE_ENTRYPOINT_ADDRESS && envVars.VITE_FACTORY_ADDRESS),
  };

  const isFullyConfigured = Object.values(configStatus).every(Boolean);

  return (
    <div className="env-config-card">
      <div className="card-header">
        <h3>📋 Environment Configuration</h3>
        <div className={`status-badge ${isFullyConfigured ? 'status-success' : 'status-warning'}`}>
          {isFullyConfigured ? '✅ Configured' : '⚠️ Setup Required'}
        </div>
      </div>

      <div className="config-status-grid">
        <div className={`config-item ${configStatus.rpc ? 'configured' : 'missing'}`}>
          <span className="config-label">RPC Connection</span>
          <span className="config-status">{configStatus.rpc ? '✅' : '❌'}</span>
        </div>
        <div className={`config-item ${configStatus.bundler ? 'configured' : 'missing'}`}>
          <span className="config-label">Bundler URL</span>
          <span className="config-status">{configStatus.bundler ? '✅' : '❌'}</span>
        </div>
        <div className={`config-item ${configStatus.privateKey ? 'configured' : 'missing'}`}>
          <span className="config-label">Private Key</span>
          <span className="config-status">{configStatus.privateKey ? '✅' : '❌'}</span>
        </div>
        <div className={`config-item ${configStatus.accounts ? 'configured' : 'missing'}`}>
          <span className="config-label">Account Addresses</span>
          <span className="config-status">{configStatus.accounts ? '✅' : '❌'}</span>
        </div>
        <div className={`config-item ${configStatus.contracts ? 'configured' : 'missing'}`}>
          <span className="config-label">Contract Addresses</span>
          <span className="config-status">{configStatus.contracts ? '✅' : '❌'}</span>
        </div>
      </div>

      {!isFullyConfigured && (
        <div className="setup-requirements">
          <h4>🚨 Setup Requirements</h4>
          <p>This interface requires pre-configuration. You cannot connect wallets directly.</p>

          <div className="setup-steps">
            {setupSteps.map((step) => (
              <div key={step.step} className="setup-step">
                <div className="step-number">{step.step}</div>
                <div className="step-content">
                  <div className="step-title">{step.title}</div>
                  <div className="step-description">{step.description}</div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="config-actions">
        <button
          className="btn btn-secondary"
          onClick={() => setShowConfig(!showConfig)}
        >
          {showConfig ? 'Hide' : 'Show'} Configuration Details
        </button>
      </div>

      {showConfig && (
        <div className="config-details">
          <div className="current-config">
            <h4>Current Configuration</h4>
            <div className="config-table">
              {Object.entries(envVars).map(([key, value]) => (
                <div key={key} className="config-row">
                  <span className="config-key">{key}</span>
                  <span className={`config-value ${value === 'NOT_SET' ? 'missing' : 'set'}`}>
                    {value || 'NOT_SET'}
                  </span>
                </div>
              ))}
            </div>
          </div>

          <div className="example-config">
            <h4>Example .env Configuration</h4>
            <pre className="config-code">
              <code>{exampleEnv}</code>
            </pre>
            <button
              className="btn btn-primary btn-sm"
              onClick={() => navigator.clipboard.writeText(exampleEnv)}
            >
              📋 Copy to Clipboard
            </button>
          </div>
        </div>
      )}

      <style jsx>{`
        .env-config-card {
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

        .status-badge {
          padding: 4px 12px;
          border-radius: 16px;
          font-size: 0.875rem;
          font-weight: 500;
        }

        .status-success {
          background: #d4edda;
          color: #155724;
        }

        .status-warning {
          background: #fff3cd;
          color: #856404;
        }

        .config-status-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 12px;
          margin-bottom: 20px;
        }

        .config-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px;
          border-radius: 8px;
          border: 1px solid #e0e0e0;
        }

        .config-item.configured {
          background: #f8fff8;
          border-color: #28a745;
        }

        .config-item.missing {
          background: #fff8f8;
          border-color: #dc3545;
        }

        .config-label {
          font-weight: 500;
          color: #555;
        }

        .config-status {
          font-size: 1.2rem;
        }

        .setup-requirements {
          background: #fff3cd;
          border: 1px solid #ffeaa7;
          border-radius: 8px;
          padding: 20px;
          margin: 20px 0;
        }

        .setup-requirements h4 {
          margin: 0 0 12px 0;
          color: #856404;
        }

        .setup-requirements p {
          margin: 0 0 16px 0;
          color: #856404;
        }

        .setup-steps {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .setup-step {
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
          background: #856404;
          color: white;
          border-radius: 50%;
          font-weight: bold;
          font-size: 0.875rem;
          flex-shrink: 0;
        }

        .step-content {
          flex: 1;
        }

        .step-title {
          font-weight: 600;
          color: #856404;
          margin-bottom: 4px;
        }

        .step-description {
          color: #856404;
          font-size: 0.875rem;
          line-height: 1.4;
        }

        .config-actions {
          margin: 20px 0;
        }

        .btn {
          padding: 8px 16px;
          border-radius: 6px;
          border: 1px solid;
          background: none;
          cursor: pointer;
          font-weight: 500;
          transition: all 0.2s;
        }

        .btn-secondary {
          border-color: #6c757d;
          color: #6c757d;
        }

        .btn-secondary:hover {
          background: #6c757d;
          color: white;
        }

        .btn-primary {
          border-color: #007bff;
          color: #007bff;
        }

        .btn-primary:hover {
          background: #007bff;
          color: white;
        }

        .btn-sm {
          padding: 4px 8px;
          font-size: 0.875rem;
        }

        .config-details {
          border-top: 1px solid #e0e0e0;
          padding-top: 20px;
          margin-top: 20px;
        }

        .current-config, .example-config {
          margin-bottom: 24px;
        }

        .current-config h4, .example-config h4 {
          margin: 0 0 12px 0;
          color: #1a1a1a;
        }

        .config-table {
          background: #f8f9fa;
          border-radius: 8px;
          overflow: hidden;
        }

        .config-row {
          display: flex;
          justify-content: space-between;
          padding: 12px 16px;
          border-bottom: 1px solid #e0e0e0;
        }

        .config-row:last-child {
          border-bottom: none;
        }

        .config-key {
          font-family: 'Monaco', 'Consolas', monospace;
          font-size: 0.875rem;
          color: #495057;
          font-weight: 500;
        }

        .config-value {
          font-family: 'Monaco', 'Consolas', monospace;
          font-size: 0.875rem;
        }

        .config-value.set {
          color: #28a745;
        }

        .config-value.missing {
          color: #dc3545;
          font-style: italic;
        }

        .config-code {
          background: #f8f9fa;
          border: 1px solid #e0e0e0;
          border-radius: 8px;
          padding: 16px;
          margin: 12px 0;
          font-size: 0.875rem;
          line-height: 1.4;
          overflow-x: auto;
        }

        .config-code code {
          font-family: 'Monaco', 'Consolas', monospace;
          color: #495057;
        }
      `}</style>
    </div>
  );
};

export default EnvConfigDisplay;