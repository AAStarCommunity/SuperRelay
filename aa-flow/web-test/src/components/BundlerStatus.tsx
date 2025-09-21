import React, { useState, useEffect } from 'react';
import { BundlerService } from '../services/bundlerService';
import type { NetworkConfig } from '../config/networks';

interface BundlerStatusProps {
  bundlerService: BundlerService | null;
  networkConfig: NetworkConfig;
}

interface BundlerInfo {
  isOnline: boolean;
  supportedEntryPoints: string[];
  error?: string;
  uptime?: string;
  version?: string;
  responseTime?: number;
}

const BundlerStatus: React.FC<BundlerStatusProps> = ({
  bundlerService,
  networkConfig,
}) => {
  const [bundlerInfo, setBundlerInfo] = useState<BundlerInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [lastChecked, setLastChecked] = useState<Date | null>(null);

  const checkBundlerStatus = async () => {
    if (!bundlerService || !networkConfig.bundlerUrl) return;

    setLoading(true);
    const startTime = Date.now();

    try {
      const status = await bundlerService.checkBundlerStatus();
      const responseTime = Date.now() - startTime;

      setBundlerInfo({
        ...status,
        responseTime,
      });
      setLastChecked(new Date());
    } catch (error) {
      setBundlerInfo({
        isOnline: false,
        supportedEntryPoints: [],
        error: error instanceof Error ? error.message : 'Unknown error',
        responseTime: Date.now() - startTime,
      });
      setLastChecked(new Date());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    checkBundlerStatus();

    // 每30秒自动检查一次状态
    const interval = setInterval(checkBundlerStatus, 30000);
    return () => clearInterval(interval);
  }, [bundlerService, networkConfig]);

  const getStatusColor = () => {
    if (!bundlerInfo) return '#999';
    if (!bundlerInfo.isOnline) return '#dc3545';
    if (bundlerInfo.responseTime && bundlerInfo.responseTime > 2000) return '#ffc107';
    return '#28a745';
  };

  const getStatusText = () => {
    if (loading) return 'Checking...';
    if (!bundlerInfo) return 'Unknown';
    if (!bundlerInfo.isOnline) return 'Offline';
    if (bundlerInfo.responseTime && bundlerInfo.responseTime > 2000) return 'Slow';
    return 'Online';
  };

  // const formatUptime = (seconds: number) => {
  //   const days = Math.floor(seconds / 86400);
  //   const hours = Math.floor((seconds % 86400) / 3600);
  //   const minutes = Math.floor((seconds % 3600) / 60);

  //   if (days > 0) return `${days}d ${hours}h ${minutes}m`;
  //   if (hours > 0) return `${hours}h ${minutes}m`;
  //   return `${minutes}m`;
  // };

  return (
    <div className="bundler-status-card">
      <div className="card-header">
        <h3>🔧 Bundler Status</h3>
        <button
          className="refresh-btn"
          onClick={checkBundlerStatus}
          disabled={loading}
        >
          {loading ? '🔄' : '↻'} Refresh
        </button>
      </div>

      <div className="status-overview">
        <div className="status-indicator">
          <div
            className="status-dot"
            style={{ backgroundColor: getStatusColor() }}
          />
          <span className="status-text">{getStatusText()}</span>
        </div>

        {bundlerInfo?.responseTime && (
          <div className="response-time">
            ⚡ {bundlerInfo.responseTime}ms
          </div>
        )}
      </div>

      <div className="bundler-details">
        <div className="detail-item">
          <span className="detail-label">Bundler URL:</span>
          <span className="detail-value">
            <a
              href={networkConfig.bundlerUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="bundler-link"
            >
              {networkConfig.bundlerUrl}
            </a>
          </span>
        </div>

        <div className="detail-item">
          <span className="detail-label">Network:</span>
          <span className="detail-value">
            {networkConfig.name} (Chain ID: {networkConfig.id})
          </span>
        </div>

        {lastChecked && (
          <div className="detail-item">
            <span className="detail-label">Last Checked:</span>
            <span className="detail-value">
              {lastChecked.toLocaleTimeString()}
            </span>
          </div>
        )}
      </div>

      {bundlerInfo?.supportedEntryPoints && bundlerInfo.supportedEntryPoints.length > 0 && (
        <div className="entrypoints-section">
          <h4>📍 Supported EntryPoints</h4>
          <div className="entrypoints-list">
            {bundlerInfo.supportedEntryPoints.map((entryPoint, index) => (
              <div key={index} className="entrypoint-item">
                <span className="entrypoint-address">{entryPoint}</span>
                <a
                  href={`${networkConfig.blockExplorer}/address/${entryPoint}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="explorer-link"
                >
                  🔍 View
                </a>
              </div>
            ))}
          </div>
        </div>
      )}

      {bundlerInfo?.error && (
        <div className="error-section">
          <h4>❌ Error</h4>
          <div className="error-message">{bundlerInfo.error}</div>
        </div>
      )}

      {/* Operational Info */}
      <div className="operational-info">
        <h4>📊 Operational Info</h4>
        <div className="info-grid">
          <div className="info-item">
            <span className="info-label">Service</span>
            <span className="info-value">Fly.io Deployment</span>
          </div>
          <div className="info-item">
            <span className="info-label">Type</span>
            <span className="info-value">ERC-4337 Bundler</span>
          </div>
          <div className="info-item">
            <span className="info-label">Version</span>
            <span className="info-value">EntryPoint v0.6</span>
          </div>
          <div className="info-item">
            <span className="info-label">Auto-restart</span>
            <span className="info-value">Enabled</span>
          </div>
        </div>
      </div>

      <style jsx>{`
        .bundler-status-card {
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

        .status-overview {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
          padding: 16px;
          background: #f8f9fa;
          border-radius: 8px;
        }

        .status-indicator {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .status-dot {
          width: 12px;
          height: 12px;
          border-radius: 50%;
          animation: pulse 2s infinite;
        }

        .status-text {
          font-weight: 600;
          font-size: 1.1rem;
        }

        .response-time {
          font-family: 'Monaco', 'Consolas', monospace;
          color: #666;
          font-weight: 500;
        }

        .bundler-details {
          margin-bottom: 20px;
        }

        .detail-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 0;
          border-bottom: 1px solid #f0f0f0;
        }

        .detail-item:last-child {
          border-bottom: none;
        }

        .detail-label {
          font-weight: 500;
          color: #666;
        }

        .detail-value {
          color: #1a1a1a;
          font-family: 'Monaco', 'Consolas', monospace;
          font-size: 0.875rem;
        }

        .bundler-link {
          color: #007bff;
          text-decoration: none;
          word-break: break-all;
        }

        .bundler-link:hover {
          text-decoration: underline;
        }

        .entrypoints-section, .operational-info {
          margin-top: 20px;
          padding-top: 20px;
          border-top: 1px solid #e0e0e0;
        }

        .entrypoints-section h4, .operational-info h4 {
          margin: 0 0 12px 0;
          color: #1a1a1a;
          font-size: 1rem;
        }

        .entrypoints-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .entrypoint-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px;
          background: #f8f9fa;
          border-radius: 8px;
        }

        .entrypoint-address {
          font-family: 'Monaco', 'Consolas', monospace;
          color: #495057;
          font-size: 0.875rem;
          word-break: break-all;
        }

        .explorer-link {
          color: #007bff;
          text-decoration: none;
          font-size: 0.875rem;
          white-space: nowrap;
          margin-left: 12px;
        }

        .explorer-link:hover {
          text-decoration: underline;
        }

        .error-section {
          margin-top: 20px;
          padding: 16px;
          background: #f8d7da;
          border-radius: 8px;
          border: 1px solid #f1aeb5;
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

        .info-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 12px;
        }

        .info-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          background: #f8f9fa;
          border-radius: 6px;
        }

        .info-label {
          font-weight: 500;
          color: #666;
          font-size: 0.875rem;
        }

        .info-value {
          color: #1a1a1a;
          font-size: 0.875rem;
          font-weight: 500;
        }

        @keyframes pulse {
          0% {
            box-shadow: 0 0 0 0 currentColor;
            opacity: 1;
          }
          100% {
            box-shadow: 0 0 0 10px transparent;
            opacity: 0;
          }
        }

        @media (max-width: 768px) {
          .status-overview {
            flex-direction: column;
            gap: 12px;
            text-align: center;
          }

          .detail-item {
            flex-direction: column;
            align-items: flex-start;
            gap: 4px;
          }

          .entrypoint-item {
            flex-direction: column;
            align-items: flex-start;
            gap: 8px;
          }

          .explorer-link {
            margin-left: 0;
          }
        }
      `}</style>
    </div>
  );
};

export default BundlerStatus;