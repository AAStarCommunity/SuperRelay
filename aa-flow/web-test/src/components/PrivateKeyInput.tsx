import React, { useState } from 'react';

interface PrivateKeyInputProps {
  onPrivateKeySet: (privateKey: string) => void;
  isRequired?: boolean;
}

const PrivateKeyInput: React.FC<PrivateKeyInputProps> = ({
  onPrivateKeySet,
  isRequired = true
}) => {
  const [privateKey, setPrivateKey] = useState('');
  const [isVisible, setIsVisible] = useState(false);
  const [isSet, setIsSet] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (privateKey.trim()) {
      onPrivateKeySet(privateKey.trim());
      setIsSet(true);
    }
  };

  const handleClear = () => {
    setPrivateKey('');
    setIsSet(false);
    onPrivateKeySet('');
  };

  return (
    <div className="private-key-input">
      <div className="input-header">
        <h4>🔐 私钥配置</h4>
        {isRequired && <span className="required">*必填</span>}
      </div>

      {!isSet ? (
        <form onSubmit={handleSubmit} className="key-form">
          <div className="input-group">
            <div className="input-wrapper">
              <input
                type={isVisible ? 'text' : 'password'}
                value={privateKey}
                onChange={(e) => setPrivateKey(e.target.value)}
                placeholder="输入测试网私钥 (0x...)"
                className="key-input"
                required={isRequired}
              />
              <button
                type="button"
                onClick={() => setIsVisible(!isVisible)}
                className="toggle-visibility"
              >
                {isVisible ? '🙈' : '👁️'}
              </button>
            </div>
            <button type="submit" className="set-key-btn" disabled={!privateKey.trim()}>
              设置私钥
            </button>
          </div>

          <div className="security-notice">
            <p>🛡️ <strong>安全提醒：</strong></p>
            <ul>
              <li>只使用测试网私钥，不要使用主网私钥</li>
              <li>私钥仅在本地存储，不会发送到服务器</li>
              <li>请确保在安全的网络环境中使用</li>
              <li>使用完毕后请清除私钥</li>
            </ul>
          </div>
        </form>
      ) : (
        <div className="key-set">
          <div className="status">
            <span className="success-icon">✅</span>
            <span>私钥已设置</span>
            <code className="key-preview">
              {privateKey.substring(0, 6)}...{privateKey.substring(privateKey.length - 4)}
            </code>
          </div>
          <button onClick={handleClear} className="clear-key-btn">
            清除私钥
          </button>
        </div>
      )}

      <style jsx>{`
        .private-key-input {
          background: #fff3cd;
          border: 1px solid #ffeaa7;
          border-radius: 8px;
          padding: 16px;
          margin: 16px 0;
        }

        .input-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .input-header h4 {
          margin: 0;
          color: #856404;
          font-size: 14px;
          font-weight: 600;
        }

        .required {
          font-size: 12px;
          color: #dc3545;
          background: #f8d7da;
          padding: 2px 6px;
          border-radius: 4px;
        }

        .key-form {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .input-group {
          display: flex;
          gap: 8px;
          align-items: stretch;
        }

        .input-wrapper {
          flex: 1;
          position: relative;
          display: flex;
        }

        .key-input {
          flex: 1;
          padding: 8px 40px 8px 12px;
          border: 1px solid #ddd;
          border-radius: 4px;
          font-family: monospace;
          font-size: 12px;
          background: white;
        }

        .key-input:focus {
          outline: none;
          border-color: #007bff;
          box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.25);
        }

        .toggle-visibility {
          position: absolute;
          right: 8px;
          top: 50%;
          transform: translateY(-50%);
          background: none;
          border: none;
          cursor: pointer;
          font-size: 16px;
          padding: 4px;
        }

        .set-key-btn {
          background: #28a745;
          color: white;
          border: none;
          padding: 8px 16px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 12px;
          font-weight: 600;
          white-space: nowrap;
        }

        .set-key-btn:disabled {
          background: #6c757d;
          cursor: not-allowed;
        }

        .set-key-btn:hover:not(:disabled) {
          background: #218838;
        }

        .security-notice {
          background: #d4edda;
          border: 1px solid #c3e6cb;
          border-radius: 4px;
          padding: 12px;
          font-size: 11px;
        }

        .security-notice p {
          margin: 0 0 8px 0;
          color: #155724;
          font-weight: 600;
        }

        .security-notice ul {
          margin: 0;
          padding-left: 16px;
          color: #155724;
        }

        .security-notice li {
          margin-bottom: 4px;
        }

        .key-set {
          display: flex;
          justify-content: space-between;
          align-items: center;
          background: #d4edda;
          border: 1px solid #c3e6cb;
          border-radius: 4px;
          padding: 12px;
        }

        .status {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .success-icon {
          font-size: 16px;
        }

        .key-preview {
          background: rgba(0, 0, 0, 0.1);
          padding: 2px 6px;
          border-radius: 3px;
          font-family: monospace;
          font-size: 11px;
        }

        .clear-key-btn {
          background: #dc3545;
          color: white;
          border: none;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 12px;
        }

        .clear-key-btn:hover {
          background: #c82333;
        }

        @media (max-width: 768px) {
          .input-group {
            flex-direction: column;
          }

          .key-set {
            flex-direction: column;
            gap: 12px;
            align-items: stretch;
          }

          .status {
            justify-content: center;
          }
        }
      `}</style>
    </div>
  );
};

export default PrivateKeyInput;