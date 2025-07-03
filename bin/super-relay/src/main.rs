// SuperRelay - Enterprise Account Abstraction Paymaster Solution
// A wrapper around rundler with SuperPaymaster enhancements

use std::process::Command;

use clap::{Parser, Subcommand};
use eyre::Result;

#[derive(Parser)]
#[command(
    name = "super-relay",
    version = "0.1.4",
    about = "SuperPaymaster Enterprise Account Abstraction Relay Service",
    long_about = "SuperRelay provides enterprise-grade ERC-4337 Account Abstraction services\nwith integrated paymaster functionality, monitoring, and Swagger UI documentation.\n\nBuilt on Rundler v0.9.0 with SuperPaymaster Extensions"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the bundler node
    Node {
        /// Path to configuration file
        #[arg(long, default_value = "config/config.toml")]
        config: String,

        /// Enable JSON-RPC server
        #[arg(long)]
        rpc: bool,

        /// RPC port
        #[arg(long, default_value = "3000")]
        rpc_port: u16,

        /// Additional rundler arguments
        #[arg(last = true)]
        rundler_args: Vec<String>,
    },
    /// Pool operations
    Pool {
        #[arg(last = true)]
        rundler_args: Vec<String>,
    },
    /// Builder operations  
    Builder {
        #[arg(last = true)]
        rundler_args: Vec<String>,
    },
    /// Admin operations
    Admin {
        #[arg(last = true)]
        rundler_args: Vec<String>,
    },
    /// Show version information
    Version,
    /// Show service status
    Status,
}

impl Cli {
    async fn run(self) -> Result<()> {
        // Show SuperRelay branding
        self.show_banner();

        match self.command {
            Commands::Node {
                config,
                rpc,
                rpc_port: _,
                rundler_args,
            } => {
                println!("🚀 Starting SuperRelay Node...\n");

                let mut cmd = Command::new("cargo");
                cmd.args(["run", "--bin", "rundler", "--", "node", "--config", &config]);

                if rpc {
                    cmd.args(["--rpc"]);
                }

                if !rundler_args.is_empty() {
                    cmd.args(&rundler_args);
                }

                let status = cmd.status()?;
                if !status.success() {
                    eyre::bail!("rundler node failed with exit code: {:?}", status.code());
                }
            }
            Commands::Pool { rundler_args } => {
                let mut cmd = Command::new("cargo");
                cmd.args(["run", "--bin", "rundler", "--", "pool"]);
                cmd.args(&rundler_args);

                let status = cmd.status()?;
                if !status.success() {
                    eyre::bail!("rundler pool failed with exit code: {:?}", status.code());
                }
            }
            Commands::Builder { rundler_args } => {
                let mut cmd = Command::new("cargo");
                cmd.args(["run", "--bin", "rundler", "--", "builder"]);
                cmd.args(&rundler_args);

                let status = cmd.status()?;
                if !status.success() {
                    eyre::bail!("rundler builder failed with exit code: {:?}", status.code());
                }
            }
            Commands::Admin { rundler_args } => {
                let mut cmd = Command::new("cargo");
                cmd.args(["run", "--bin", "rundler", "--", "admin"]);
                cmd.args(&rundler_args);

                let status = cmd.status()?;
                if !status.success() {
                    eyre::bail!("rundler admin failed with exit code: {:?}", status.code());
                }
            }
            Commands::Version => {
                self.show_version();
            }
            Commands::Status => {
                self.check_status().await?;
            }
        }

        Ok(())
    }

    fn show_banner(&self) {
        println!("🚀 SuperRelay v0.1.4 - Enterprise Account Abstraction Service");
        println!("📊 Enhanced with PaymasterRelay, Monitoring & Swagger UI");
        println!("🌐 Swagger UI: http://localhost:9000/swagger-ui/");
        println!("📈 Monitoring: http://localhost:9000/health");
        println!("🔧 Built on Rundler v0.9.0 with SuperPaymaster Extensions");
        println!();
    }

    fn show_version(&self) {
        println!("SuperRelay v0.1.4");
        println!("Built on Rundler v0.9.0");
        println!("SuperPaymaster Extensions:");
        println!("  - PaymasterRelay module");
        println!("  - Prometheus monitoring");
        println!("  - Swagger UI documentation");
        println!("  - Enterprise-grade policies");
    }

    async fn check_status(&self) -> Result<()> {
        println!("🔍 Checking SuperRelay service status...\n");

        // Check main RPC service
        match self
            .check_endpoint("http://localhost:3000", "Main RPC Service")
            .await
        {
            Ok(_) => println!("✅ Main RPC Service: RUNNING"),
            Err(_) => println!("❌ Main RPC Service: NOT RUNNING"),
        }

        // Check Swagger UI
        match self
            .check_endpoint("http://localhost:9000/health", "Swagger UI & Monitoring")
            .await
        {
            Ok(_) => println!("✅ Swagger UI & Monitoring: RUNNING"),
            Err(_) => println!("❌ Swagger UI & Monitoring: NOT RUNNING"),
        }

        // Check Prometheus metrics
        match self
            .check_endpoint("http://localhost:8080/metrics", "Prometheus Metrics")
            .await
        {
            Ok(_) => println!("✅ Prometheus Metrics: RUNNING"),
            Err(_) => println!("❌ Prometheus Metrics: NOT RUNNING"),
        }

        println!("\n📋 Service URLs:");
        println!("  🌐 Swagger UI: http://localhost:9000/swagger-ui/");
        println!("  🏥 Health Check: http://localhost:9000/health");
        println!("  📊 Metrics: http://localhost:9000/metrics");
        println!("  📈 Prometheus: http://localhost:8080/metrics");
        println!("  🔧 Main RPC: http://localhost:3000");

        Ok(())
    }

    async fn check_endpoint(&self, url: &str, _service: &str) -> Result<()> {
        // Simple TCP connection check (avoiding external dependencies)
        use std::{net::TcpStream, time::Duration};

        let url_parts: Vec<&str> = url.split("://").collect();
        if url_parts.len() != 2 {
            return Err(eyre::eyre!("Invalid URL format"));
        }

        let host_port: Vec<&str> = url_parts[1].split('/').next().unwrap().split(':').collect();
        if host_port.len() != 2 {
            return Err(eyre::eyre!("Invalid host:port format"));
        }

        let host = host_port[0];
        let port: u16 = host_port[1].parse()?;

        match TcpStream::connect_timeout(
            &format!("{}:{}", host, port).parse()?,
            Duration::from_secs(3),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre::eyre!("Connection failed: {}", e)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}
