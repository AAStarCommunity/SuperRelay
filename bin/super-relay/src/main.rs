//! SuperRelay standalone paymaster service binary

use std::net::SocketAddr;

use anyhow::Context;
use clap::{Arg, Command};
use jsonrpsee::server::ServerBuilder;
use rundler_paymaster_relay::{Config, PaymasterRelayApiServer, PaymasterRelayService};
use tokio::signal;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let matches = Command::new("super-relay")
        .version(env!("CARGO_PKG_VERSION"))
        .about("SuperRelay - Non-invasive ERC-4337 Paymaster Service")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("config/super-relay.toml"),
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("Host to bind to")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to bind to")
                .default_value("3001"),
        )
        .get_matches();

    // Parse command line arguments
    let config_path = matches.get_one::<String>("config").unwrap();
    let host = matches.get_one::<String>("host").unwrap();
    let port: u16 = matches
        .get_one::<String>("port")
        .unwrap()
        .parse()
        .context("Invalid port number")?;

    info!("Starting SuperRelay v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration file: {}", config_path);
    info!("Server binding: {}:{}", host, port);

    // Load configuration
    let mut config = if std::path::Path::new(config_path).exists() {
        info!("Loading configuration from {}", config_path);
        Config::from_file(config_path)
            .await
            .context("Failed to load configuration")?
    } else {
        warn!(
            "Configuration file {} not found, using default configuration",
            config_path
        );
        Config::default()
    };

    // Override with command line arguments
    config.server.host = host.clone();
    config.server.port = port;

    // Validate configuration
    config
        .validate()
        .context("Configuration validation failed")?;

    // Create the paymaster service
    info!("Initializing PaymasterRelayService...");
    let service = PaymasterRelayService::new(config.clone())
        .await
        .context("Failed to create PaymasterRelayService")?;

    // Start the RPC server
    let server_addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("Invalid server address")?;

    info!("Starting RPC server on {}", server_addr);
    let server = ServerBuilder::default()
        .max_connections(config.server.max_connections)
        .build(server_addr)
        .await
        .context("Failed to start RPC server")?;

    // Register RPC methods
    let handle = server.start(service.into_rpc());

    // Print startup information
    info!("🚀 SuperRelay started successfully!");
    info!("📡 RPC server listening on http://{}", server_addr);
    info!(
        "🏥 Health check: http://{}/health (use pm_health RPC method)",
        server_addr
    );
    info!("📊 Chain ID: {}", config.paymaster.chain_id);
    info!(
        "🔑 EntryPoints supported: {:?}",
        config.paymaster.entry_points.keys().collect::<Vec<_>>()
    );
    info!("📋 Policy file: {}", config.policy.rules_path);
    info!("🔗 Rundler endpoint: {}", config.rundler.url);
    info!("");
    info!("Available RPC methods:");
    info!("  • pm_sponsorUserOperation    - Sponsor a UserOperation");
    info!("  • pm_getSupportedEntryPoints - Get supported EntryPoint addresses");
    info!("  • pm_getChainId              - Get chain ID");
    info!("  • pm_getStatistics           - Get service statistics");
    info!("  • pm_health                  - Health check");
    info!("  • pm_getPolicyInfo           - Get policy information");
    info!("");
    info!("Press Ctrl+C to shutdown");

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received shutdown signal, stopping server...");
        }
        Err(err) => {
            warn!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Graceful shutdown
    handle.stop()?;
    info!("SuperRelay stopped successfully");

    Ok(())
}

/// Additional utilities for the binary
mod utils {
    use std::path::Path;

    /// Check if a file exists and is readable
    #[allow(dead_code)]
    pub fn check_file_readable<P: AsRef<Path>>(path: P) -> bool {
        std::fs::metadata(path.as_ref()).is_ok()
    }

    /// Format uptime duration in human readable format
    #[allow(dead_code)]
    pub fn format_uptime(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, secs)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;

    #[test]
    fn test_uptime_formatting() {
        assert_eq!(format_uptime(30), "30s");
        assert_eq!(format_uptime(90), "1m 30s");
        assert_eq!(format_uptime(3661), "1h 1m 1s");
        assert_eq!(format_uptime(90061), "1d 1h 1m 1s");
    }
}
