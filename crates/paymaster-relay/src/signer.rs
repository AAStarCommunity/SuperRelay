// paymaster-relay/src/signer.rs
// This file will implement the SignerManager for handling private keys.

use std::str::FromStr;

use ethers::{
    signers::{LocalWallet, Signer},
    types::{Address, Signature},
};
use eyre::Result;
use secrecy::{ExposeSecret, SecretString};

#[derive(Clone, Debug)]
pub struct SignerManager {
    signer: LocalWallet,
}

impl SignerManager {
    pub fn new(private_key: SecretString) -> Result<Self> {
        let signer = LocalWallet::from_str(private_key.expose_secret())?;
        Ok(Self { signer })
    }

    pub async fn sign_hash(&self, hash: [u8; 32]) -> Result<Signature> {
        let signature = self.signer.sign_hash(hash.into())?;
        Ok(signature)
    }

    pub fn address(&self) -> Address {
        self.signer.address()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethers::types::H256;

    use super::*;

    #[tokio::test]
    async fn test_signer_manager() {
        // A common test private key.
        // Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
        let private_key =
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string();
        let secret_key = SecretString::new(private_key.into());

        // 1. Create a new SignerManager
        let signer_manager =
            SignerManager::new(secret_key).expect("Failed to create signer manager");

        // 2. Check if the address is correct
        let expected_address =
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        assert_eq!(signer_manager.address(), expected_address);

        // 3. Sign a sample hash
        let hash = H256::random().to_fixed_bytes();
        let signature = signer_manager
            .sign_hash(hash)
            .await
            .expect("Failed to sign hash");

        // 4. Verify the signature
        signature
            .verify(hash, expected_address)
            .expect("Signature verification failed");
    }
}
