// Debug logger utility to capture and display debug info
export class DebugLogger {
  private static logs: string[] = [];
  private static listeners: ((logs: string[]) => void)[] = [];

  static log(message: string) {
    console.log(message);
    this.logs.push(`${new Date().toLocaleTimeString()}: ${message}`);
    this.notifyListeners();
  }

  static error(message: string) {
    console.error(message);
    this.logs.push(`${new Date().toLocaleTimeString()}: ❌ ${message}`);
    this.notifyListeners();
  }

  static clear() {
    this.logs = [];
    this.notifyListeners();
  }

  static getLogs() {
    return [...this.logs];
  }

  static addListener(callback: (logs: string[]) => void) {
    this.listeners.push(callback);
  }

  static removeListener(callback: (logs: string[]) => void) {
    this.listeners = this.listeners.filter(l => l !== callback);
  }

  private static notifyListeners() {
    this.listeners.forEach(callback => callback([...this.logs]));
  }
}