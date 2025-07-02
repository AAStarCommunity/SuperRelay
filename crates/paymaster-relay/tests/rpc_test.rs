//! Integration tests for the paymaster relay RPC.
use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use ethers::{
    prelude::*,
    utils::{Anvil, AnvilInstance},
};
use jsonrpsee::{
    core::client::ClientT,
    rpc_params,
    ws_client::{WsClient, WsClientBuilder},
};
// 移除未使用的导入，测试将使用JSON格式
use serde::Deserialize;
use tempfile::tempdir;

const RUNDLER_WS_URL: &str = "ws://127.0.0.1:3000";
const PAYMASTER_SIGNER_KEY: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000002";
const ENTRY_POINT_ADDRESS: &str = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";

// Define the receipt struct locally as it's not public in rundler
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct UserOperationReceipt {
    pub user_op_hash: H256,
    pub sender: Address,
    pub nonce: U256,
    pub actual_gas_cost: U256,
    pub actual_gas_used: U256,
    pub success: bool,
}

struct TestContext {
    _anvil: AnvilInstance,
    _rundler: Child,
    ws_client: Arc<WsClient>,
}

async fn setup_test_context() -> Result<TestContext> {
    // 1. Start Anvil
    let anvil = Anvil::new().spawn();

    // Wait for anvil to be ready
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 2. Start Rundler
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("Could not determine workspace root"))?;
    let rundler_bin = workspace_root.join("target/debug/rundler");

    let temp_dir = tempdir()?;
    let policy_path = temp_dir.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        "
[policy]
type = \"allowlist\"
senders = [\"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266\"] # First anvil key
",
    )?;

    let mut rundler_handle = Command::new(rundler_bin)
        .arg("--rpc.listen")
        .arg("127.0.0.1:3000")
        .arg("--eth-client-address")
        .arg(anvil.ws_endpoint())
        .arg("--chain-id")
        .arg("31337")
        .arg("--entry-points")
        .arg(ENTRY_POINT_ADDRESS)
        .arg("--paymaster.enabled")
        .arg("true")
        .arg("--paymaster.policy-file")
        .arg(policy_path.to_str().unwrap())
        .env("RUNDLER__PAYMASTER__SIGNER__TYPE", "local_hot_wallet")
        .env(
            "RUNDLER__PAYMASTER__SIGNER__PRIVATE_KEY",
            PAYMASTER_SIGNER_KEY,
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Err(e) = wait_for_socket("127.0.0.1:3000").await {
        rundler_handle.kill()?;
        return Err(e);
    }

    let ws_client = WsClientBuilder::default().build(RUNDLER_WS_URL).await?;

    Ok(TestContext {
        _anvil: anvil,
        _rundler: rundler_handle,
        ws_client: Arc::new(ws_client),
    })
}

#[tokio::test]
#[ignore]
async fn test_pm_sponsor_user_operation_success() -> Result<()> {
    let context = setup_test_context().await?;
    let rpc_client = context.ws_client;

    let wallet: LocalWallet = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse()
        .unwrap();

    // 创建一个符合JSON格式的UserOperation，不直接序列化UserOperationVariant
    let uo_json = serde_json::json!({
        "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "nonce": "0x0",
        "initCode": "0x",
        "callData": "0x",
        "callGasLimit": "0x186A0",
        "verificationGasLimit": "0x186A0",
        "preVerificationGas": "0x5208",
        "maxFeePerGas": "0x3B9ACA00",
        "maxPriorityFeePerGas": "0x3B9ACA00",
        "paymasterAndData": "0x",
        "signature": "0x"
    });

    let params = rpc_params![uo_json, ENTRY_POINT_ADDRESS];

    let user_op_hash: H256 = rpc_client
        .request("pm_sponsorUserOperation", params)
        .await?;

    assert_ne!(user_op_hash, H256::zero());

    // Poll for the receipt
    let mut receipt: Option<UserOperationReceipt> = None;
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let params = rpc_params![user_op_hash];
        receipt = rpc_client
            .request("eth_getUserOperationReceipt", params)
            .await?;
        if receipt.is_some() {
            break;
        }
    }

    let receipt = receipt.expect("Failed to get user operation receipt");
    assert!(receipt.success, "User operation should succeed");
    assert_eq!(receipt.sender, wallet.address());

    Ok(())
}

/// Helper to wait for a service to open a TCP port.
async fn wait_for_socket(addr_str: &str) -> Result<()> {
    let socket_addr: SocketAddr = addr_str.parse()?;
    let mut attempts = 0;
    loop {
        if socket_addr.to_socket_addrs().is_ok()
            && tokio::net::TcpStream::connect(socket_addr).await.is_ok()
        {
            println!("Connection successful to {}", addr_str);
            return Ok(());
        }

        if attempts > 20 {
            anyhow::bail!("Timed out waiting for socket at {}", addr_str);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        attempts += 1;
    }
}
