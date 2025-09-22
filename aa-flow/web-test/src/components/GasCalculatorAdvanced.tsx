import React, { useState, useEffect } from 'react';

interface GasCalculation {
  fixedGasOverhead: number;
  perUserOpOverhead: number;
  dataLength: number;
  byteCost: number;
  safetyBuffer: number;
  totalGas: number;
  hexValue: string;
}

interface GasCalculatorAdvancedProps {
  userOp?: any;
  title?: string;
}

const GasCalculatorAdvanced: React.FC<GasCalculatorAdvancedProps> = ({
  userOp,
  title = "🧮 Advanced Gas Calculator"
}) => {
  const [calculation, setCalculation] = useState<GasCalculation | null>(null);
  const [isExpanded, setIsExpanded] = useState(false);

  const calculatePreVerificationGas = (userOpData: any): GasCalculation => {
    const FIXED_GAS_OVERHEAD = 21000;  // 基础交易 gas
    const PER_USER_OP_OVERHEAD = 18300; // 每个 UserOp 的固定开销
    const PER_USER_OP_WORD = 4;         // 每个字的开销
    const ZERO_BYTE_COST = 4;           // 零字节成本
    const NON_ZERO_BYTE_COST = 16;      // 非零字节成本
    const SAFETY_BUFFER = 1000;         // 安全缓冲

    // 序列化 UserOperation 用于计算字节数
    const serializedData = JSON.stringify({
      sender: userOpData?.sender || '',
      nonce: userOpData?.nonce || '0x0',
      initCode: userOpData?.initCode || '0x',
      callData: userOpData?.callData || '0x',
      callGasLimit: userOpData?.callGasLimit || '0x0',
      verificationGasLimit: userOpData?.verificationGasLimit || '0x0',
      preVerificationGas: '0x0', // 临时值
      maxFeePerGas: userOpData?.maxFeePerGas || '0x0',
      maxPriorityFeePerGas: userOpData?.maxPriorityFeePerGas || '0x0',
      paymasterAndData: userOpData?.paymasterAndData || '0x',
      signature: userOpData?.signature || '0x'
    });

    // 计算字节成本
    let byteCost = 0;
    const encoder = new TextEncoder();
    const bytes = encoder.encode(serializedData);

    for (const byte of bytes) {
      if (byte === 0) {
        byteCost += ZERO_BYTE_COST;
      } else {
        byteCost += NON_ZERO_BYTE_COST;
      }
    }

    // 计算总的 preVerificationGas
    const totalGas = FIXED_GAS_OVERHEAD +
                    PER_USER_OP_OVERHEAD +
                    Math.ceil(bytes.length / 32) * PER_USER_OP_WORD +
                    byteCost +
                    SAFETY_BUFFER;

    return {
      fixedGasOverhead: FIXED_GAS_OVERHEAD,
      perUserOpOverhead: PER_USER_OP_OVERHEAD,
      dataLength: bytes.length,
      byteCost,
      safetyBuffer: SAFETY_BUFFER,
      totalGas,
      hexValue: `0x${totalGas.toString(16)}`
    };
  };

  useEffect(() => {
    if (userOp) {
      const calc = calculatePreVerificationGas(userOp);
      setCalculation(calc);
    }
  }, [userOp]);

  if (!calculation) {
    return (
      <div className="gas-calculator-advanced">
        <div className="calculator-header">
          <h4>{title}</h4>
          <span className="status">⏳ Waiting for UserOperation data...</span>
        </div>
        <style jsx>{styles}</style>
      </div>
    );
  }

  return (
    <div className="gas-calculator-advanced">
      <div className="calculator-header" onClick={() => setIsExpanded(!isExpanded)}>
        <h4>{title}</h4>
        <div className="header-info">
          <span className="total-gas">
            Total: {calculation.totalGas.toLocaleString()} gas ({calculation.hexValue})
          </span>
          <span className="toggle-icon">
            {isExpanded ? '▼' : '▶'}
          </span>
        </div>
      </div>

      {isExpanded && (
        <div className="calculation-details">
          <div className="calculation-breakdown">
            <div className="breakdown-item">
              <span className="label">🔧 基础交易 Gas:</span>
              <span className="value">{calculation.fixedGasOverhead.toLocaleString()}</span>
            </div>
            <div className="breakdown-item">
              <span className="label">📦 UserOp 固定开销:</span>
              <span className="value">{calculation.perUserOpOverhead.toLocaleString()}</span>
            </div>
            <div className="breakdown-item">
              <span className="label">📏 数据长度:</span>
              <span className="value">{calculation.dataLength} bytes</span>
            </div>
            <div className="breakdown-item">
              <span className="label">💾 字节成本:</span>
              <span className="value">{calculation.byteCost.toLocaleString()}</span>
            </div>
            <div className="breakdown-item">
              <span className="label">🛡️ 安全缓冲:</span>
              <span className="value">{calculation.safetyBuffer.toLocaleString()}</span>
            </div>
            <div className="breakdown-total">
              <span className="label">🎯 计算总量:</span>
              <span className="value">
                {calculation.totalGas.toLocaleString()} gas
                <code className="hex-value">({calculation.hexValue})</code>
              </span>
            </div>
          </div>

          <div className="algorithm-info">
            <h5>🧮 计算公式</h5>
            <div className="formula">
              <code>
                preVerificationGas = 基础Gas(21000) + UserOp开销(18300) +
                字数开销(ceil(bytes/32)*4) + 字节成本 + 安全缓冲(1000)
              </code>
            </div>
          </div>
        </div>
      )}

      <style jsx>{styles}</style>
    </div>
  );
};

const styles = `
  .gas-calculator-advanced {
    background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
    border: 1px solid #dee2e6;
    border-radius: 12px;
    padding: 16px;
    margin: 16px 0;
    font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
  }

  .calculator-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .calculator-header:hover {
    background: rgba(0, 123, 255, 0.05);
    border-radius: 8px;
    padding: 8px;
    margin: -8px;
  }

  .calculator-header h4 {
    margin: 0;
    color: #495057;
    font-size: 14px;
    font-weight: 600;
  }

  .header-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .total-gas {
    font-size: 12px;
    font-weight: 600;
    color: #28a745;
    background: #d4edda;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .toggle-icon {
    font-size: 12px;
    color: #6c757d;
  }

  .status {
    font-size: 12px;
    color: #6c757d;
    font-style: italic;
  }

  .calculation-details {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid #dee2e6;
  }

  .calculation-breakdown {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }

  .breakdown-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 8px;
    background: white;
    border-radius: 6px;
    border: 1px solid #e9ecef;
  }

  .breakdown-total {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: #e3f2fd;
    border: 2px solid #2196f3;
    border-radius: 8px;
    font-weight: 600;
  }

  .breakdown-item .label,
  .breakdown-total .label {
    font-size: 11px;
    color: #495057;
    font-weight: 500;
  }

  .breakdown-item .value,
  .breakdown-total .value {
    font-size: 12px;
    color: #212529;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .hex-value {
    font-size: 10px;
    background: #f8f9fa;
    padding: 2px 4px;
    border-radius: 3px;
    color: #6c757d;
    border: 1px solid #dee2e6;
  }

  .algorithm-info {
    background: #fff3cd;
    border: 1px solid #ffeaa7;
    border-radius: 8px;
    padding: 12px;
  }

  .algorithm-info h5 {
    margin: 0 0 8px 0;
    color: #856404;
    font-size: 12px;
    font-weight: 600;
  }

  .formula {
    font-size: 10px;
    line-height: 1.4;
  }

  .formula code {
    color: #856404;
    background: transparent;
    padding: 0;
    word-break: break-all;
  }

  @media (max-width: 768px) {
    .calculator-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .header-info {
      width: 100%;
      justify-content: space-between;
    }

    .breakdown-item,
    .breakdown-total {
      flex-direction: column;
      align-items: flex-start;
      gap: 4px;
    }

    .formula {
      font-size: 9px;
    }
  }
`;

export default GasCalculatorAdvanced;