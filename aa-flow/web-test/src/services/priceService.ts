// 价格获取服务
import axios from 'axios';

export interface TokenPrice {
  usd: number;
  usd_24h_change: number;
  last_updated_at: number;
}

export interface PriceData {
  ethereum: TokenPrice;
}

export class PriceService {
  private static readonly COINGECKO_API = import.meta.env.DEV ? '/api/coingecko/api/v3' : 'https://api.coingecko.com/api/v3';
  private static readonly ALCHEMY_PRICE_API = 'https://api.g.alchemy.com/prices/v1';

  // 缓存价格数据，避免频繁请求
  private static priceCache: { [key: string]: { price: number; timestamp: number } } = {};
  private static readonly CACHE_DURATION = 60000; // 1分钟缓存

  // 从 CoinGecko 获取 ETH 价格
  static async getEthPriceFromCoinGecko(): Promise<number> {
    try {
      const response = await axios.get(`${this.COINGECKO_API}/simple/price`, {
        params: {
          ids: 'ethereum',
          vs_currencies: 'usd',
          include_24hr_change: true
        },
        timeout: 10000
      });

      if (response.data?.ethereum?.usd) {
        return response.data.ethereum.usd;
      }
      throw new Error('Invalid response from CoinGecko');
    } catch (error) {
      console.error('CoinGecko price fetch failed:', error);
      throw error;
    }
  }

  // 从 Alchemy 获取 ETH 价格 (备用)
  static async getEthPriceFromAlchemy(): Promise<number> {
    try {
      const alchemyApiKey = import.meta.env.VITE_ALCHEMY_API_KEY;
      if (!alchemyApiKey) {
        throw new Error('Alchemy API key not configured');
      }

      const response = await axios.get(`${this.ALCHEMY_PRICE_API}/${alchemyApiKey}/tokens/by-symbol`, {
        params: {
          symbols: 'ETH'
        },
        timeout: 10000
      });

      if (response.data?.data?.[0]?.prices?.[0]?.value) {
        return parseFloat(response.data.data[0].prices[0].value);
      }
      throw new Error('Invalid response from Alchemy');
    } catch (error) {
      console.error('Alchemy price fetch failed:', error);
      throw error;
    }
  }

  // 获取ETH价格（带缓存和多数据源）
  static async getEthPrice(): Promise<number> {
    const cacheKey = 'ETH';
    const now = Date.now();

    // 检查缓存
    if (this.priceCache[cacheKey] && (now - this.priceCache[cacheKey].timestamp) < this.CACHE_DURATION) {
      return this.priceCache[cacheKey].price;
    }

    // 尝试多个数据源
    const sources = [
      () => this.getEthPriceFromCoinGecko(),
      () => this.getEthPriceFromAlchemy()
    ];

    for (const getPrice of sources) {
      try {
        const price = await getPrice();
        if (price > 0) {
          // 更新缓存
          this.priceCache[cacheKey] = { price, timestamp: now };
          return price;
        }
      } catch (error) {
        console.warn('Price source failed, trying next:', error);
        continue;
      }
    }

    // 如果所有数据源都失败，返回缓存值或默认值
    if (this.priceCache[cacheKey]) {
      console.warn('All price sources failed, using cached value');
      return this.priceCache[cacheKey].price;
    }

    console.warn('All price sources failed, using fallback price');
    return 3000; // 默认价格作为后备
  }

  // 格式化价格显示
  static formatPrice(price: number): string {
    if (price >= 1000) {
      return `$${price.toLocaleString('en-US', {
        minimumFractionDigits: 0,
        maximumFractionDigits: 0
      })}`;
    } else {
      return `$${price.toFixed(2)}`;
    }
  }

  // 计算gas费用的USD价值
  static calculateGasCostUSD(gasUsed: number, gasPriceGwei: number, ethPriceUSD: number): string {
    const gasCostEth = (gasUsed * gasPriceGwei) / 1e9; // 转换为ETH
    const gasCostUSD = gasCostEth * ethPriceUSD;

    if (gasCostUSD < 0.01) {
      return `$${gasCostUSD.toFixed(4)}`;
    } else {
      return `$${gasCostUSD.toFixed(2)}`;
    }
  }

  // 清除价格缓存
  static clearCache(): void {
    this.priceCache = {};
  }
}