import React, { useState, useEffect } from 'react';
import { AccountService } from '../services/accountService';
import type { AccountInfo } from '../services/accountService';
import type { NetworkConfig } from '../config/networks';
import { getBlockExplorerAddressUrl } from '../config/networks';

interface AccountManagerProps {
  accountService: AccountService | null;
  networkConfig: NetworkConfig;
}

const AccountManager: React.FC<AccountManagerProps> = ({
  accountService,
  networkConfig,
}) => {
  const [accounts, setAccounts] = useState<{
    eoa: AccountInfo | null;
    accountA: AccountInfo | null;
    accountB: AccountInfo | null;
  }>({
    eoa: null,
    accountA: null,
    accountB: null,
  });
  const [tokenInfo, setTokenInfo] = useState<{
    name: string;
    symbol: string;
    decimals: number;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const addresses = {
    eoa: import.meta.env.VITE_EOA_ADDRESS,
    accountA: import.meta.env.VITE_SIMPLE_ACCOUNT_A,
    accountB: import.meta.env.VITE_SIMPLE_ACCOUNT_B,
    factory: import.meta.env.VITE_FACTORY_ADDRESS,
    pntToken: import.meta.env.VITE_PNT_TOKEN_ADDRESS,
  };

  const loadAccountsInfo = async () => {
    if (!accountService) return;

    setLoading(true);
    setError(null);

    try {
      // 获取代币信息
      if (addresses.pntToken) {
        const token = await accountService.getTokenInfo(addresses.pntToken);
        setTokenInfo(token);
      }

      // 获取账户信息
      const [eoaInfo, accountAInfo, accountBInfo] = await Promise.all([
        addresses.eoa ? accountService.getAccountInfo(addresses.eoa, addresses.pntToken) : null,
        addresses.accountA ? accountService.getAccountInfo(addresses.accountA, addresses.pntToken) : null,
        addresses.accountB ? accountService.getAccountInfo(addresses.accountB, addresses.pntToken) : null,
      ]);

      setAccounts({
        eoa: eoaInfo,
        accountA: accountAInfo,
        accountB: accountBInfo,
      });
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadAccountsInfo();
  }, [accountService, networkConfig]);

  const AccountCard: React.FC<{
    title: string;
    emoji: string;
    address: string;
    info: AccountInfo | null;
    isEOA?: boolean;
  }> = ({ title, emoji, address, info, isEOA = false }) => (
    <div className="account-card">
      <div className="account-header">
        <h4>
          {emoji} {title}
        </h4>
        <div className="account-status">
          {isEOA ? (
            <span className="status-badge eoa">EOA</span>
          ) : (
            <span className={`status-badge ${info?.isDeployed ? 'deployed' : 'not-deployed'}`}>
              {info?.isDeployed ? '✅ Deployed' : '❌ Not Deployed'}
            </span>
          )}
        </div>
      </div>

      <div className="account-details">
        <div className="detail-row">
          <span className="detail-label">Address:</span>
          <div className="address-container">
            <span className="address-value">{address}</span>
            <a
              href={getBlockExplorerAddressUrl(address, 'sepolia')}
              target="_blank"
              rel="noopener noreferrer"
              className="explorer-link"
            >
              🔍 Etherscan
            </a>
          </div>
        </div>

        {info && (
          <>
            <div className="detail-row">
              <span className="detail-label">ETH Balance:</span>
              <span className="balance-value">
                {parseFloat(info.ethBalance).toFixed(6)} ETH
              </span>
            </div>

            <div className="detail-row">
              <span className="detail-label">
                {tokenInfo?.symbol || 'PNT'} Balance:
              </span>
              <span className="balance-value">
                {parseFloat(info.tokenBalance).toFixed(2)} {tokenInfo?.symbol || 'PNT'}
              </span>
            </div>

            {!isEOA && (
              <div className="detail-row">
                <span className="detail-label">Nonce:</span>
                <span className="nonce-value">{info.nonce}</span>
              </div>
            )}
          </>
        )}
      </div>

      {!isEOA && !info?.isDeployed && (
        <div className="deployment-notice">
          <p>⚠️ This SimpleAccount needs to be deployed before use.</p>
          <p>Run: <code>node scripts/deploy-accounts.js</code></p>
        </div>
      )}
    </div>
  );

  return (
    <div className="account-manager-card">
      <div className="card-header">
        <h3>👛 Account Management</h3>
        <button
          className="refresh-btn"
          onClick={loadAccountsInfo}
          disabled={loading}
        >
          {loading ? '🔄' : '↻'} Refresh
        </button>
      </div>

      {error && (
        <div className="error-section">
          <h4>❌ Error</h4>
          <div className="error-message">{error}</div>
        </div>
      )}

      {tokenInfo && (
        <div className="token-info-section">
          <h4>🪙 Token Information</h4>
          <div className="token-details">
            <div className="token-detail">
              <span className="token-label">Name:</span>
              <span className="token-value">{tokenInfo.name}</span>
            </div>
            <div className="token-detail">
              <span className="token-label">Symbol:</span>
              <span className="token-value">{tokenInfo.symbol}</span>
            </div>
            <div className="token-detail">
              <span className="token-label">Decimals:</span>
              <span className="token-value">{tokenInfo.decimals}</span>
            </div>
            <div className="token-detail">
              <span className="token-label">Contract:</span>
              <div className="address-container">
                <span className="address-value">{addresses.pntToken}</span>
                <a
                  href={getBlockExplorerAddressUrl(addresses.pntToken, 'sepolia')}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="explorer-link"
                >
                  🔍 View
                </a>
              </div>
            </div>
          </div>
        </div>
      )}

      <div className="accounts-grid">
        {addresses.eoa && (
          <AccountCard
            title="EOA (Owner)"
            emoji="🔑"
            address={addresses.eoa}
            info={accounts.eoa}
            isEOA={true}
          />
        )}

        {addresses.accountA && (
          <AccountCard
            title="SimpleAccount A (Sender)"
            emoji="📤"
            address={addresses.accountA}
            info={accounts.accountA}
          />
        )}

        {addresses.accountB && (
          <AccountCard
            title="SimpleAccount B (Receiver)"
            emoji="📥"
            address={addresses.accountB}
            info={accounts.accountB}
          />
        )}
      </div>

      <div className="factory-section">
        <h4>🏭 SimpleAccount Factory</h4>
        <div className="factory-info">
          <div className="factory-detail">
            <span className="detail-label">Factory Contract:</span>
            <div className="address-container">
              <span className="address-value">{addresses.factory}</span>
              <a
                href={getBlockExplorerAddressUrl(addresses.factory, 'sepolia')}
                target="_blank"
                rel="noopener noreferrer"
                className="explorer-link"
              >
                🔍 View Factory
              </a>
            </div>
          </div>
          <div className="factory-actions">
            <p>
              💡 Use the factory contract to generate more SimpleAccounts with different salt values.
              Each EOA can control multiple SimpleAccounts using different salts.
            </p>
          </div>
        </div>
      </div>

      <style jsx>{`
        .account-manager-card {
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

        .token-info-section, .factory-section {
          margin-bottom: 24px;
          padding-bottom: 20px;
          border-bottom: 1px solid #e0e0e0;
        }

        .token-info-section h4, .factory-section h4 {
          margin: 0 0 16px 0;
          color: #1a1a1a;
          font-size: 1rem;
        }

        .token-details {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 12px;
        }

        .token-detail {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          background: #f8f9fa;
          border-radius: 6px;
        }

        .token-label {
          font-weight: 500;
          color: #666;
        }

        .token-value {
          font-family: 'Monaco', 'Consolas', monospace;
          color: #1a1a1a;
          font-weight: 500;
        }

        .accounts-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
          gap: 20px;
          margin-bottom: 24px;
        }

        .account-card {
          border: 1px solid #e0e0e0;
          border-radius: 10px;
          padding: 20px;
          background: #fafafa;
        }

        .account-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .account-header h4 {
          margin: 0;
          color: #1a1a1a;
          font-size: 1rem;
        }

        .status-badge {
          padding: 4px 8px;
          border-radius: 12px;
          font-size: 0.75rem;
          font-weight: 600;
        }

        .status-badge.eoa {
          background: #e7f3ff;
          color: #0066cc;
        }

        .status-badge.deployed {
          background: #d4edda;
          color: #155724;
        }

        .status-badge.not-deployed {
          background: #f8d7da;
          color: #721c24;
        }

        .account-details {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .detail-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          flex-wrap: wrap;
          gap: 8px;
        }

        .detail-label {
          font-weight: 500;
          color: #666;
          min-width: 100px;
        }

        .address-container {
          display: flex;
          align-items: center;
          gap: 8px;
          flex: 1;
          min-width: 0;
        }

        .address-value {
          font-family: 'Monaco', 'Consolas', monospace;
          color: #1a1a1a;
          font-size: 0.875rem;
          word-break: break-all;
          flex: 1;
        }

        .explorer-link {
          color: #007bff;
          text-decoration: none;
          font-size: 0.75rem;
          white-space: nowrap;
          padding: 2px 6px;
          border: 1px solid #007bff;
          border-radius: 4px;
          transition: all 0.2s;
        }

        .explorer-link:hover {
          background: #007bff;
          color: white;
        }

        .balance-value {
          font-family: 'Monaco', 'Consolas', monospace;
          font-weight: 600;
          color: #28a745;
        }

        .nonce-value {
          font-family: 'Monaco', 'Consolas', monospace;
          font-weight: 600;
          color: #007bff;
        }

        .deployment-notice {
          margin-top: 12px;
          padding: 12px;
          background: #fff3cd;
          border: 1px solid #ffeaa7;
          border-radius: 6px;
        }

        .deployment-notice p {
          margin: 0 0 6px 0;
          color: #856404;
          font-size: 0.875rem;
        }

        .deployment-notice p:last-child {
          margin-bottom: 0;
        }

        .deployment-notice code {
          background: #f8f9fa;
          padding: 2px 4px;
          border-radius: 3px;
          font-family: 'Monaco', 'Consolas', monospace;
          font-size: 0.8rem;
        }

        .factory-info {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .factory-detail {
          display: flex;
          justify-content: space-between;
          align-items: center;
          flex-wrap: wrap;
          gap: 12px;
        }

        .factory-actions {
          background: #e7f3ff;
          border: 1px solid #b3d9ff;
          border-radius: 8px;
          padding: 12px;
        }

        .factory-actions p {
          margin: 0;
          color: #0066cc;
          font-size: 0.875rem;
          line-height: 1.4;
        }

        .error-section {
          background: #f8d7da;
          border: 1px solid #f1aeb5;
          border-radius: 8px;
          padding: 16px;
          margin-bottom: 20px;
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
          .accounts-grid {
            grid-template-columns: 1fr;
          }

          .token-details {
            grid-template-columns: 1fr;
          }

          .detail-row {
            flex-direction: column;
            align-items: flex-start;
          }

          .address-container {
            width: 100%;
            flex-direction: column;
            align-items: flex-start;
            gap: 6px;
          }

          .factory-detail {
            flex-direction: column;
            align-items: flex-start;
          }
        }
      `}</style>
    </div>
  );
};

export default AccountManager;