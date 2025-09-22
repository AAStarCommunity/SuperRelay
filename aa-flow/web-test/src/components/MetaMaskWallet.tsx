import React, { useState, useEffect } from 'react';
import { ethers } from 'ethers';

interface MetaMaskWalletProps {
  onAccountChange: (account: string, signer: ethers.Signer | null) => void;
  onNetworkChange: (chainId: string) => void;
}

declare global {
  interface Window {
    ethereum?: any;
  }
}

const MetaMaskWallet: React.FC<MetaMaskWalletProps> = ({
  onAccountChange,
  onNetworkChange
}) => {
  const [account, setAccount] = useState<string>('');
  const [isConnected, setIsConnected] = useState(false);
  const [chainId, setChainId] = useState<string>('');
  const [provider, setProvider] = useState<ethers.BrowserProvider | null>(null);

  useEffect(() => {
    checkConnection();
    setupEventListeners();
  }, []);

  const checkConnection = async () => {
    if (typeof window.ethereum !== 'undefined') {
      try {
        const provider = new ethers.BrowserProvider(window.ethereum);
        const accounts = await provider.listAccounts();
        const network = await provider.getNetwork();

        if (accounts.length > 0) {
          const signer = await provider.getSigner();
          const address = await signer.getAddress();
          setAccount(address);
          setIsConnected(true);
          setChainId(network.chainId.toString());
          setProvider(provider);
          onAccountChange(address, signer);
          onNetworkChange(network.chainId.toString());
        }
      } catch (error) {
        console.error('检查连接失败:', error);
      }
    }
  };

  const setupEventListeners = () => {
    if (window.ethereum) {
      window.ethereum.on('accountsChanged', handleAccountsChanged);
      window.ethereum.on('chainChanged', handleChainChanged);
    }
  };

  const handleAccountsChanged = (accounts: string[]) => {
    if (accounts.length > 0) {
      setAccount(accounts[0]);
      if (provider) {
        provider.getSigner().then(signer => onAccountChange(accounts[0], signer));
      } else {
        onAccountChange(accounts[0], null);
      }
    } else {
      setAccount('');
      setIsConnected(false);
      onAccountChange('', null);
    }
  };

  const handleChainChanged = (chainId: string) => {
    setChainId(parseInt(chainId, 16).toString());
    onNetworkChange(parseInt(chainId, 16).toString());
  };

  const connectWallet = async () => {
    if (typeof window.ethereum === 'undefined') {
      alert('请安装 MetaMask!');
      return;
    }

    try {
      const provider = new ethers.BrowserProvider(window.ethereum);
      await provider.send('eth_requestAccounts', []);

      const signer = await provider.getSigner();
      const address = await signer.getAddress();
      const network = await provider.getNetwork();

      setAccount(address);
      setIsConnected(true);
      setChainId(network.chainId.toString());
      setProvider(provider);

      onAccountChange(address, signer);
      onNetworkChange(network.chainId.toString());
    } catch (error) {
      console.error('连接钱包失败:', error);
      alert('连接钱包失败');
    }
  };

  const switchToSepolia = async () => {
    try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: '0xaa36a7' }], // Sepolia chainId
      });
    } catch (error: any) {
      if (error.code === 4902) {
        // 网络不存在，添加 Sepolia 网络
        try {
          await window.ethereum.request({
            method: 'wallet_addEthereumChain',
            params: [{
              chainId: '0xaa36a7',
              chainName: 'Sepolia test network',
              nativeCurrency: {
                name: 'Ether',
                symbol: 'ETH',
                decimals: 18,
              },
              rpcUrls: ['https://sepolia.infura.io/v3/'],
              blockExplorerUrls: ['https://sepolia.etherscan.io/'],
            }],
          });
        } catch (addError) {
          console.error('添加网络失败:', addError);
        }
      }
    }
  };

  const getNetworkName = (chainId: string) => {
    const networks: { [key: string]: string } = {
      '1': 'Mainnet',
      '11155111': 'Sepolia',
      '10': 'Optimism',
      '11155420': 'OP Sepolia',
    };
    return networks[chainId] || `Chain ${chainId}`;
  };

  const formatAddress = (address: string) => {
    return `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
  };

  return (
    <div className="metamask-wallet">
      {!isConnected ? (
        <button className="connect-btn" onClick={connectWallet}>
          🦊 连接 MetaMask
        </button>
      ) : (
        <div className="wallet-info">
          <div className="account-info">
            <span className="account-icon">👤</span>
            <span className="account-address">{formatAddress(account)}</span>
          </div>
          <div className="network-info">
            <span className="network-icon">🌐</span>
            <span className="network-name">{getNetworkName(chainId)}</span>
            {chainId !== '11155111' && (
              <button className="switch-network-btn" onClick={switchToSepolia}>
                切换到 Sepolia
              </button>
            )}
          </div>
        </div>
      )}

      <style jsx>{`
        .metamask-wallet {
          background: #f8f9fa;
          border: 1px solid #dee2e6;
          border-radius: 8px;
          padding: 16px;
          margin: 16px 0;
        }

        .connect-btn {
          background: linear-gradient(135deg, #f6851b 0%, #e2761b 100%);
          color: white;
          border: none;
          padding: 12px 24px;
          border-radius: 8px;
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s ease;
          width: 100%;
        }

        .connect-btn:hover {
          transform: translateY(-1px);
          box-shadow: 0 4px 12px rgba(246, 133, 27, 0.3);
        }

        .wallet-info {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 16px;
        }

        .account-info,
        .network-info {
          display: flex;
          align-items: center;
          gap: 8px;
          background: white;
          padding: 8px 12px;
          border-radius: 6px;
          border: 1px solid #e9ecef;
        }

        .account-icon,
        .network-icon {
          font-size: 16px;
        }

        .account-address,
        .network-name {
          font-family: monospace;
          font-size: 12px;
          font-weight: 600;
          color: #495057;
        }

        .switch-network-btn {
          background: #ffc107;
          color: #212529;
          border: none;
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 10px;
          font-weight: 600;
          cursor: pointer;
          margin-left: 8px;
        }

        .switch-network-btn:hover {
          background: #ffca2c;
        }

        @media (max-width: 768px) {
          .wallet-info {
            flex-direction: column;
            align-items: stretch;
          }

          .account-info,
          .network-info {
            justify-content: center;
          }
        }
      `}</style>
    </div>
  );
};

export default MetaMaskWallet;