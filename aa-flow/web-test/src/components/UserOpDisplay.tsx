import React from 'react';
import type { UserOperation } from '../services/bundlerService';

interface UserOpDisplayProps {
  userOp?: UserOperation;
  userOpHash?: string;
  title?: string;
  isExpanded?: boolean;
  onToggle?: () => void;
}

const UserOpDisplay: React.FC<UserOpDisplayProps> = ({
  userOp,
  userOpHash,
  title = "📋 UserOperation Details",
  isExpanded = false,
  onToggle,
}) => {
  if (!userOp) {
    return null;
  }

  const formatValue = (value: any): string => {
    if (typeof value === 'string') {
      // 如果是长地址或数据，截断显示
      if (value.startsWith('0x') && value.length > 20) {
        return `${value.slice(0, 10)}...${value.slice(-8)}`;
      }
      return value;
    }
    return String(value);
  };

  const formatLongValue = (value: any): string => {
    if (typeof value === 'string') {
      return value;
    }
    return String(value);
  };

  return (
    <div className="userop-display">
      <div className="userop-header" onClick={onToggle} style={{ cursor: onToggle ? 'pointer' : 'default' }}>
        <h4>{title}</h4>
        {onToggle && (
          <span className="toggle-icon">
            {isExpanded ? '▼' : '▶'}
          </span>
        )}
      </div>

      {userOpHash && (
        <div className="userop-hash">
          <span className="label">UserOp Hash:</span>
          <span className="hash-value" title={userOpHash}>
            {formatValue(userOpHash)}
          </span>
        </div>
      )}

      {(isExpanded || !onToggle) && (
        <div className="userop-details">
          <div className="field-group">
            <div className="field">
              <span className="label">Sender:</span>
              <span className="value" title={userOp.sender}>
                {formatValue(userOp.sender)}
              </span>
            </div>
            <div className="field">
              <span className="label">Nonce:</span>
              <span className="value">{userOp.nonce}</span>
            </div>
          </div>

          <div className="field-group">
            <div className="field">
              <span className="label">Call Gas Limit:</span>
              <span className="value">{userOp.callGasLimit}</span>
            </div>
            <div className="field">
              <span className="label">Verification Gas Limit:</span>
              <span className="value">{userOp.verificationGasLimit}</span>
            </div>
            <div className="field">
              <span className="label">Pre-Verification Gas:</span>
              <span className="value">{userOp.preVerificationGas}</span>
            </div>
          </div>

          <div className="field-group">
            <div className="field">
              <span className="label">Max Fee Per Gas:</span>
              <span className="value">{userOp.maxFeePerGas}</span>
            </div>
            <div className="field">
              <span className="label">Max Priority Fee:</span>
              <span className="value">{userOp.maxPriorityFeePerGas}</span>
            </div>
          </div>

          <div className="field-group">
            <div className="field full-width">
              <span className="label">Call Data:</span>
              <div className="long-value" title={formatLongValue(userOp.callData)}>
                {formatValue(userOp.callData)}
              </div>
            </div>
          </div>

          <div className="field-group">
            <div className="field full-width">
              <span className="label">Init Code:</span>
              <div className="long-value">
                {userOp.initCode === '0x' ? 'Empty (Account exists)' : formatValue(userOp.initCode)}
              </div>
            </div>
          </div>

          <div className="field-group">
            <div className="field full-width">
              <span className="label">Paymaster Data:</span>
              <div className="long-value">
                {userOp.paymasterAndData === '0x' ? 'None (EOA pays gas)' : formatValue(userOp.paymasterAndData)}
              </div>
            </div>
          </div>

          <div className="field-group">
            <div className="field full-width">
              <span className="label">Signature:</span>
              <div className="signature-value" title={formatLongValue(userOp.signature)}>
                {formatValue(userOp.signature)}
                <span className="signature-length">
                  ({userOp.signature.length - 2} hex chars / {(userOp.signature.length - 2) / 2} bytes)
                </span>
              </div>
            </div>
          </div>
        </div>
      )}

      <style jsx>{`
        .userop-display {
          background: #f8f9fa;
          border: 1px solid #e9ecef;
          border-radius: 8px;
          padding: 16px;
          margin: 16px 0;
          font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
          font-size: 12px;
        }

        .userop-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .userop-header h4 {
          margin: 0;
          color: #495057;
          font-size: 14px;
          font-weight: 600;
        }

        .toggle-icon {
          font-size: 12px;
          color: #6c757d;
        }

        .userop-hash {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 12px;
          padding: 8px;
          background: #fff3cd;
          border: 1px solid #ffeaa7;
          border-radius: 4px;
        }

        .userop-hash .label {
          font-weight: 600;
          color: #856404;
        }

        .hash-value {
          font-family: inherit;
          color: #856404;
          font-weight: 500;
        }

        .userop-details {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .field-group {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 12px;
        }

        .field {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .field.full-width {
          grid-column: 1 / -1;
        }

        .label {
          font-weight: 600;
          color: #495057;
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        .value {
          font-family: inherit;
          color: #212529;
          word-break: break-all;
        }

        .long-value {
          font-family: inherit;
          color: #212529;
          word-break: break-all;
          padding: 6px 8px;
          background: white;
          border: 1px solid #dee2e6;
          border-radius: 4px;
          font-size: 11px;
        }

        .signature-value {
          display: flex;
          flex-direction: column;
          gap: 4px;
          padding: 6px 8px;
          background: white;
          border: 1px solid #dee2e6;
          border-radius: 4px;
          font-size: 11px;
        }

        .signature-length {
          font-size: 10px;
          color: #6c757d;
          font-style: italic;
        }

        @media (max-width: 768px) {
          .field-group {
            grid-template-columns: 1fr;
          }

          .userop-display {
            font-size: 11px;
          }
        }
      `}</style>
    </div>
  );
};

export default UserOpDisplay;