#!/usr/bin/env python3
"""
QEMU OP-TEE Monitor Script
Monitors QEMU VM status and SuperRelay health inside the OP-TEE environment
"""

import time
import socket
import subprocess
import json
import logging
import signal
import sys
from datetime import datetime
from typing import Dict, Any, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/opt/superrelay/logs/monitor.log'),
        logging.StreamHandler(sys.stdout)
    ]
)

logger = logging.getLogger(__name__)

class QemuOpteeMonitor:
    """Monitor for QEMU VM running OP-TEE and SuperRelay"""
    
    def __init__(self):
        self.qemu_console_port = 54320
        self.qemu_monitor_port = 54321
        self.superrelay_health_url = "http://localhost:3000/health"
        self.running = True
        self.stats = {
            'start_time': datetime.now(),
            'qemu_restarts': 0,
            'health_check_failures': 0,
            'last_health_check': None,
            'tee_operations': 0
        }
        
        # Setup signal handlers
        signal.signal(signal.SIGTERM, self._signal_handler)
        signal.signal(signal.SIGINT, self._signal_handler)
    
    def _signal_handler(self, signum, frame):
        """Handle shutdown signals gracefully"""
        logger.info(f"Received signal {signum}, shutting down monitor...")
        self.running = False
    
    def check_qemu_process(self) -> bool:
        """Check if QEMU process is running"""
        try:
            result = subprocess.run(
                ["pgrep", "-f", "qemu-system-aarch64"],
                capture_output=True,
                text=True
            )
            return result.returncode == 0
        except Exception as e:
            logger.error(f"Error checking QEMU process: {e}")
            return False
    
    def check_console_connection(self) -> bool:
        """Check if we can connect to QEMU console"""
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
                sock.settimeout(5)
                result = sock.connect_ex(('localhost', self.qemu_console_port))
                return result == 0
        except Exception as e:
            logger.debug(f"Console connection check failed: {e}")
            return False
    
    def send_console_command(self, command: str, timeout: int = 10) -> Optional[str]:
        """Send command to QEMU console and get response"""
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
                sock.settimeout(timeout)
                sock.connect(('localhost', self.qemu_console_port))
                
                # Send command
                sock.send(f"{command}\n".encode())
                
                # Receive response
                response = sock.recv(4096).decode()
                return response
                
        except Exception as e:
            logger.debug(f"Console command failed: {e}")
            return None
    
    def check_superrelay_health(self) -> Dict[str, Any]:
        """Check SuperRelay health endpoint"""
        try:
            result = subprocess.run([
                "curl", "-s", "-f", "--max-time", "10",
                self.superrelay_health_url
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                try:
                    health_data = json.loads(result.stdout)
                    self.stats['last_health_check'] = datetime.now()
                    return {
                        'status': 'healthy',
                        'data': health_data
                    }
                except json.JSONDecodeError:
                    return {'status': 'unhealthy', 'error': 'Invalid JSON response'}
            else:
                self.stats['health_check_failures'] += 1
                return {
                    'status': 'unhealthy', 
                    'error': f'HTTP error: {result.stderr}'
                }
                
        except Exception as e:
            self.stats['health_check_failures'] += 1
            return {'status': 'error', 'error': str(e)}
    
    def check_optee_status(self) -> Dict[str, Any]:
        """Check OP-TEE status inside VM"""
        # Check if tee-supplicant is running
        response = self.send_console_command("ps aux | grep tee-supplicant | grep -v grep")
        
        if response and "tee-supplicant" in response:
            return {'status': 'running', 'details': 'tee-supplicant active'}
        else:
            return {'status': 'error', 'details': 'tee-supplicant not found'}
    
    def check_ta_status(self) -> Dict[str, Any]:
        """Check SuperRelay TA status"""
        # Check if TA file exists
        response = self.send_console_command("ls -la /lib/optee_armtz/12345678-5b69-11d4-9fee-00c04f4c3456.ta")
        
        if response and "12345678-5b69-11d4-9fee-00c04f4c3456.ta" in response:
            return {'status': 'installed', 'details': 'SuperRelay TA found'}
        else:
            return {'status': 'missing', 'details': 'SuperRelay TA not found'}
    
    def get_system_stats(self) -> Dict[str, Any]:
        """Get system statistics from inside VM"""
        stats = {}
        
        # Memory usage
        mem_response = self.send_console_command("free -m")
        if mem_response:
            lines = mem_response.split('\n')
            for line in lines:
                if 'Mem:' in line:
                    parts = line.split()
                    if len(parts) >= 4:
                        stats['memory'] = {
                            'total': parts[1] + 'MB',
                            'used': parts[2] + 'MB',
                            'free': parts[3] + 'MB'
                        }
        
        # CPU load
        load_response = self.send_console_command("uptime")
        if load_response:
            stats['load'] = load_response.strip()
        
        # Disk usage
        disk_response = self.send_console_command("df -h /")
        if disk_response:
            lines = disk_response.split('\n')
            for line in lines:
                if '/' in line and '%' in line:
                    parts = line.split()
                    if len(parts) >= 5:
                        stats['disk'] = {
                            'size': parts[1],
                            'used': parts[2],
                            'available': parts[3],
                            'usage': parts[4]
                        }
        
        return stats
    
    def generate_status_report(self) -> Dict[str, Any]:
        """Generate comprehensive status report"""
        return {
            'timestamp': datetime.now().isoformat(),
            'uptime': str(datetime.now() - self.stats['start_time']),
            'qemu': {
                'process_running': self.check_qemu_process(),
                'console_accessible': self.check_console_connection()
            },
            'optee': self.check_optee_status(),
            'trusted_application': self.check_ta_status(),
            'superrelay': self.check_superrelay_health(),
            'system_stats': self.get_system_stats(),
            'monitor_stats': self.stats.copy()
        }
    
    def log_status_report(self, report: Dict[str, Any]):
        """Log status report with appropriate log levels"""
        # Log QEMU status
        if report['qemu']['process_running']:
            logger.debug("QEMU process is running")
        else:
            logger.error("QEMU process is not running!")
        
        # Log OP-TEE status  
        optee_status = report['optee']['status']
        if optee_status == 'running':
            logger.debug(f"OP-TEE status: {optee_status}")
        else:
            logger.warning(f"OP-TEE status: {optee_status}")
        
        # Log SuperRelay status
        sr_status = report['superrelay']['status']
        if sr_status == 'healthy':
            logger.debug("SuperRelay health check passed")
        else:
            logger.warning(f"SuperRelay health check failed: {sr_status}")
        
        # Log system stats
        if 'system_stats' in report and report['system_stats']:
            logger.debug(f"System stats: {report['system_stats']}")
    
    def save_status_report(self, report: Dict[str, Any]):
        """Save detailed status report to file"""
        try:
            with open('/opt/superrelay/logs/status_report.json', 'w') as f:
                json.dump(report, f, indent=2, default=str)
        except Exception as e:
            logger.error(f"Failed to save status report: {e}")
    
    def run(self):
        """Main monitoring loop"""
        logger.info("🔍 Starting QEMU OP-TEE Monitor...")
        logger.info(f"Monitor configuration:")
        logger.info(f"  - QEMU Console Port: {self.qemu_console_port}")
        logger.info(f"  - SuperRelay Health URL: {self.superrelay_health_url}")
        
        check_interval = 30  # seconds
        detailed_report_interval = 300  # 5 minutes
        last_detailed_report = 0
        
        while self.running:
            try:
                current_time = time.time()
                
                # Quick health checks every 30 seconds
                qemu_running = self.check_qemu_process()
                console_available = self.check_console_connection()
                
                if not qemu_running:
                    logger.error("QEMU process not running!")
                    break
                
                if not console_available:
                    logger.warning("QEMU console not accessible")
                
                # SuperRelay health check
                sr_health = self.check_superrelay_health()
                if sr_health['status'] != 'healthy':
                    logger.warning(f"SuperRelay health issue: {sr_health}")
                
                # Detailed report every 5 minutes
                if current_time - last_detailed_report > detailed_report_interval:
                    logger.info("📊 Generating detailed status report...")
                    report = self.generate_status_report()
                    self.log_status_report(report)
                    self.save_status_report(report)
                    last_detailed_report = current_time
                    
                    # Log uptime milestone
                    uptime_hours = (datetime.now() - self.stats['start_time']).total_seconds() / 3600
                    if uptime_hours > 1 and int(uptime_hours) % 6 == 0:  # Every 6 hours
                        logger.info(f"🕐 System uptime: {uptime_hours:.1f} hours")
                
                time.sleep(check_interval)
                
            except KeyboardInterrupt:
                logger.info("Monitor interrupted by user")
                break
            except Exception as e:
                logger.error(f"Monitor error: {e}")
                time.sleep(5)  # Brief pause before retrying
        
        logger.info("🔍 QEMU OP-TEE Monitor shutdown complete")

def main():
    """Main entry point"""
    monitor = QemuOpteeMonitor()
    monitor.run()

if __name__ == "__main__":
    main()