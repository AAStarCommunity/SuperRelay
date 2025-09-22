import React from 'react';
import { NETWORKS } from '../config/networks';

interface NetworkSelectorProps {
  selectedNetwork: string;
  onNetworkChange: (network: string) => void;
}

const NetworkSelector: React.FC<NetworkSelectorProps> = ({
  selectedNetwork,
  onNetworkChange,
}) => {
  const handleNetworkChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    onNetworkChange(event.target.value);
  };

  return (
    <div className="network-selector">
      <label htmlFor="network-select" className="network-label">
        🌐 Network:
      </label>
      <select
        id="network-select"
        data-testid="network-selector"
        value={selectedNetwork}
        onChange={handleNetworkChange}
        className="network-select"
      >
        {Object.entries(NETWORKS).map(([key, network]) => (
          <option key={key} value={key} disabled={!network.bundlerUrl}>
            {network.name} {!network.bundlerUrl && '(Not Supported)'}
          </option>
        ))}
      </select>

      <div className="network-info">
        <span className="network-chain-id">
          Chain ID: {NETWORKS[selectedNetwork]?.id}
        </span>
        {NETWORKS[selectedNetwork]?.bundlerUrl ? (
          <span className="bundler-status supported">
            ✅ Bundler Supported
          </span>
        ) : (
          <span className="bundler-status not-supported">
            ❌ Bundler Not Available
          </span>
        )}
      </div>

      <style jsx>{`
        .network-selector {
          display: flex;
          flex-direction: column;
          gap: 8px;
          min-width: 200px;
        }

        .network-label {
          font-weight: 600;
          color: #1a1a1a;
          font-size: 0.875rem;
        }

        .network-select {
          padding: 8px 12px;
          border: 2px solid #e0e0e0;
          border-radius: 8px;
          background: white;
          font-size: 1rem;
          font-weight: 500;
          color: #1a1a1a;
          cursor: pointer;
          transition: border-color 0.2s;
        }

        .network-select:hover {
          border-color: #007bff;
        }

        .network-select:focus {
          outline: none;
          border-color: #007bff;
          box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1);
        }

        .network-select option:disabled {
          color: #999;
          font-style: italic;
        }

        .network-info {
          display: flex;
          justify-content: space-between;
          align-items: center;
          font-size: 0.75rem;
          gap: 12px;
        }

        .network-chain-id {
          color: #666;
          font-family: 'Monaco', 'Consolas', monospace;
        }

        .bundler-status {
          font-weight: 500;
          padding: 2px 6px;
          border-radius: 4px;
        }

        .bundler-status.supported {
          background: #d4edda;
          color: #155724;
        }

        .bundler-status.not-supported {
          background: #f8d7da;
          color: #721c24;
        }

        @media (max-width: 768px) {
          .network-selector {
            width: 100%;
          }

          .network-info {
            flex-direction: column;
            align-items: flex-start;
            gap: 4px;
          }
        }
      `}</style>
    </div>
  );
};

export default NetworkSelector;