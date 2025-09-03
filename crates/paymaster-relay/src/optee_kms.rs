// SPDX-License-Identifier: MIT
// Copyright (c) 2025, AAStarCommunity
// OpteKmsProvider: OP-TEE Key Management Service Provider

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::{c_char, c_int, c_uchar, c_uint, c_void},
    ptr,
    sync::{Arc, Mutex},
};

use ethers::types::{Address, Signature, H256, U256};
use eyre::{eyre, Result};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};

use crate::kms::KmsError;
// TODO: Fix missing traits - KmsProvider, KmsSigningRequest, SigningContext need to be defined
// use crate::kms::{KmsProvider, KmsSigningRequest, SigningContext};

// OP-TEE Client API Constants
const TEEC_SUCCESS: u32 = 0x00000000;
const TEEC_ERROR_GENERIC: u32 = 0xFFFF0000;
const TEEC_ERROR_BAD_PARAMETERS: u32 = 0xFFFF0006;
const TEEC_ERROR_ITEM_NOT_FOUND: u32 = 0xFFFF0008;
const TEEC_ERROR_NOT_IMPLEMENTED: u32 = 0xFFFF0009;
const TEEC_ERROR_SHORT_BUFFER: u32 = 0xFFFF0010;

// Parameter types
const TEEC_PARAM_TYPE_NONE: u32 = 0x0;
const TEEC_PARAM_TYPE_VALUE_INPUT: u32 = 0x1;
const TEEC_PARAM_TYPE_VALUE_OUTPUT: u32 = 0x2;
const TEEC_PARAM_TYPE_MEMREF_INPUT: u32 = 0x5;
const TEEC_PARAM_TYPE_MEMREF_OUTPUT: u32 = 0x6;

// SuperRelay TA Constants
const SUPER_RELAY_TA_UUID: [u32; 4] = [0x12345678, 0x5b69, 0x11d4, 0x9fee00c0];
const TA_SUPER_RELAY_CMD_GENERATE_KEY: u32 = 0;
const TA_SUPER_RELAY_CMD_SIGN_MESSAGE: u32 = 2;
const TA_SUPER_RELAY_CMD_GET_PUBLIC_KEY: u32 = 3;
const TA_SUPER_RELAY_CMD_LIST_KEYS: u32 = 5;
const TA_SUPER_RELAY_CMD_HEALTH_CHECK: u32 = 7;

// Key constants
const SR_MAX_KEY_ID_SIZE: usize = 64;
const SR_SECP256K1_SIGNATURE_SIZE: usize = 64;
const SR_ETHEREUM_ADDRESS_SIZE: usize = 20;
const SR_MESSAGE_HASH_SIZE: usize = 32;

/// OP-TEE Client API bindings
#[link(name = "teec")]
extern "C" {
    fn TEEC_InitializeContext(name: *const c_char, context: *mut TEECContext) -> c_uint;

    fn TEEC_FinalizeContext(context: *mut TEECContext);

    fn TEEC_OpenSession(
        context: *mut TEECContext,
        session: *mut TEECSession,
        destination: *const TEECUUID,
        connection_method: c_uint,
        connection_data: *const c_void,
        operation: *mut TEECOperation,
        return_origin: *mut c_uint,
    ) -> c_uint;

    fn TEEC_CloseSession(session: *mut TEECSession);

    fn TEEC_InvokeCommand(
        session: *mut TEECSession,
        command_id: c_uint,
        operation: *mut TEECOperation,
        return_origin: *mut c_uint,
    ) -> c_uint;

    fn TEEC_AllocateSharedMemory(
        context: *mut TEECContext,
        shared_mem: *mut TEECSharedMemory,
    ) -> c_uint;

    fn TEEC_ReleaseSharedMemory(shared_mem: *mut TEECSharedMemory);
}

/// OP-TEE Context structure
#[repr(C)]
#[derive(Debug)]
pub struct TEECContext {
    fd: c_int,
    reg_mem: c_int,
}

/// OP-TEE Session structure
#[repr(C)]
#[derive(Debug)]
pub struct TEECSession {
    ctx: *mut TEECContext,
    session_id: c_uint,
}

/// OP-TEE UUID structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TEECUUID {
    time_low: c_uint,
    time_mid: u16,
    time_hi_and_version: u16,
    clock_seq_and_node: [c_uchar; 8],
}

/// OP-TEE Shared Memory structure
#[repr(C)]
#[derive(Debug)]
pub struct TEECSharedMemory {
    buffer: *mut c_void,
    size: usize,
    flags: c_uint,
    id: c_int,
    allocated: bool,
}

/// OP-TEE Parameter Value
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TEECValue {
    a: c_uint,
    b: c_uint,
}

/// OP-TEE Parameter Memory Reference
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TEECTempMemoryReference {
    buffer: *mut c_void,
    size: usize,
}

/// OP-TEE Parameter (union in C, struct in Rust)
#[repr(C)]
#[derive(Clone, Copy)]
pub union TEECParameter {
    value: TEECValue,
    tmpref: TEECTempMemoryReference,
}

impl std::fmt::Debug for TEECParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TEECParameter {{ .. }}")
    }
}

/// OP-TEE Operation structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TEECOperation {
    started: c_uint,
    param_types: c_uint,
    params: [TEECParameter; 4],
    session: *mut TEECSession,
    cancel_flag: *mut bool,
}

/// Signature result from TA
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignatureResult {
    signature: [u8; SR_SECP256K1_SIGNATURE_SIZE],
    signature_len: u32,
    recovery_id: u8,
    _reserved: [u8; 3],
}

/// Public key result from TA
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PublicKeyResult {
    public_key: [u8; 64], // secp256k1 uncompressed
    public_key_len: u32,
    ethereum_address: [u8; SR_ETHEREUM_ADDRESS_SIZE],
    _reserved: [u8; 8],
}

/// Key information from TA
#[repr(C)]
#[derive(Debug, Clone)]
pub struct KeyInfo {
    key_id: [c_char; SR_MAX_KEY_ID_SIZE],
    key_type: u32,
    status: u32,
    created_time: u64,
    last_used_time: u64,
    usage_count: u32,
    ethereum_address: [u8; SR_ETHEREUM_ADDRESS_SIZE],
}

/// Health check result from TA
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HealthResult {
    status: u32,
    active_sessions: u32,
    total_operations: u32,
    storage_usage: u32,
    uptime: u64,
}

/// OP-TEE KMS Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpteeKmsConfig {
    pub device_path: String,
    pub ta_uuid: String,
    pub session_timeout: u64,
    pub max_retries: u32,
    pub audit_logging: bool,
}

impl Default for OpteeKmsConfig {
    fn default() -> Self {
        Self {
            device_path: "/dev/teepriv0".to_string(),
            ta_uuid: "12345678-5b69-11d4-9fee-00c04f4c3456".to_string(),
            session_timeout: 300,
            max_retries: 3,
            audit_logging: true,
        }
    }
}

/// Thread-safe OP-TEE session wrapper
struct OpteeSession {
    context: TEECContext,
    session: TEECSession,
    uuid: TEECUUID,
}

unsafe impl Send for OpteeSession {}
unsafe impl Sync for OpteeSession {}

impl OpteeSession {
    fn new() -> Result<Self> {
        let uuid = TEECUUID {
            time_low: 0x12345678,
            time_mid: 0x5b69,
            time_hi_and_version: 0x11d4,
            clock_seq_and_node: [0x9f, 0xee, 0x00, 0xc0, 0x4f, 0x4c, 0x34, 0x56],
        };

        let mut context: TEECContext = unsafe { std::mem::zeroed() };
        let mut session: TEECSession = unsafe { std::mem::zeroed() };

        // Initialize OP-TEE context
        let ret = unsafe { TEEC_InitializeContext(ptr::null(), &mut context) };

        if ret != TEEC_SUCCESS {
            return Err(eyre!("Failed to initialize OP-TEE context: 0x{:x}", ret));
        }

        // Open session with SuperRelay TA
        let ret = unsafe {
            TEEC_OpenSession(
                &mut context,
                &mut session,
                &uuid,
                0, // TEEC_LOGIN_PUBLIC
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };

        if ret != TEEC_SUCCESS {
            unsafe {
                TEEC_FinalizeContext(&mut context);
            }
            return Err(eyre!("Failed to open TA session: 0x{:x}", ret));
        }

        Ok(Self {
            context,
            session,
            uuid,
        })
    }

    fn invoke_command(&mut self, cmd_id: u32, operation: &mut TEECOperation) -> Result<()> {
        let mut return_origin: u32 = 0;

        let ret =
            unsafe { TEEC_InvokeCommand(&mut self.session, cmd_id, operation, &mut return_origin) };

        if ret != TEEC_SUCCESS {
            return Err(eyre!(
                "TA command {} failed: 0x{:x} (origin: {})",
                cmd_id,
                ret,
                return_origin
            ));
        }

        Ok(())
    }
}

impl Drop for OpteeSession {
    fn drop(&mut self) {
        unsafe {
            TEEC_CloseSession(&mut self.session);
            TEEC_FinalizeContext(&mut self.context);
        }
    }
}

/// OP-TEE KMS Provider implementation
pub struct OpteKmsProvider {
    session: Arc<Mutex<OpteeSession>>,
    config: OpteeKmsConfig,
    key_cache: Arc<Mutex<HashMap<String, Address>>>,
}

impl OpteKmsProvider {
    /// Create new OP-TEE KMS Provider
    pub fn new(config: OpteeKmsConfig) -> Result<Self> {
        info!("Initializing OP-TEE KMS Provider");
        debug!("OP-TEE device path: {}", config.device_path);

        let session = OpteeSession::new()?;

        let provider = Self {
            session: Arc::new(Mutex::new(session)),
            config,
            key_cache: Arc::new(Mutex::new(HashMap::new())),
        };

        // Perform initial health check
        provider.health_check()?;

        info!("OP-TEE KMS Provider initialized successfully");
        Ok(provider)
    }

    /// Create TEEC operation helper
    fn create_operation(param_types: u32) -> TEECOperation {
        TEECOperation {
            started: 0,
            param_types,
            params: [TEECParameter {
                value: TEECValue { a: 0, b: 0 },
            }; 4],
            session: ptr::null_mut(),
            cancel_flag: ptr::null_mut(),
        }
    }

    /// Helper to create parameter types
    fn teec_param_types(p0: u32, p1: u32, p2: u32, p3: u32) -> u32 {
        (p0 & 0xF) | ((p1 & 0xF) << 4) | ((p2 & 0xF) << 8) | ((p3 & 0xF) << 12)
    }

    /// Generate a new key in TEE
    #[instrument(skip(self), fields(key_id = %key_id))]
    pub fn generate_key(&self, key_id: &str, key_type: u32) -> Result<Address> {
        info!("Generating key in TEE: {}", key_id);

        if key_id.len() >= SR_MAX_KEY_ID_SIZE {
            return Err(eyre!("Key ID too long: {} bytes", key_id.len()));
        }

        let key_id_cstring = CString::new(key_id)?;
        let mut address_buffer = [0u8; SR_ETHEREUM_ADDRESS_SIZE];

        let param_types = Self::teec_param_types(
            TEEC_PARAM_TYPE_MEMREF_INPUT,
            TEEC_PARAM_TYPE_VALUE_INPUT,
            TEEC_PARAM_TYPE_MEMREF_OUTPUT,
            TEEC_PARAM_TYPE_NONE,
        );

        let mut operation = Self::create_operation(param_types);

        // Set parameters
        unsafe {
            operation.params[0].tmpref = TEECTempMemoryReference {
                buffer: key_id_cstring.as_ptr() as *mut c_void,
                size: key_id_cstring.as_bytes().len(),
            };

            operation.params[1].value = TEECValue { a: key_type, b: 0 };

            operation.params[2].tmpref = TEECTempMemoryReference {
                buffer: address_buffer.as_mut_ptr() as *mut c_void,
                size: address_buffer.len(),
            };
        }

        // Invoke TA command
        {
            let mut session = self.session.lock().unwrap();
            session.invoke_command(TA_SUPER_RELAY_CMD_GENERATE_KEY, &mut operation)?;
        }

        // Parse result
        let address = Address::from_slice(&address_buffer);

        // Cache the address
        {
            let mut cache = self.key_cache.lock().unwrap();
            cache.insert(key_id.to_string(), address);
        }

        info!(
            "Successfully generated key {} with address: {:?}",
            key_id, address
        );
        Ok(address)
    }

    /// Sign a message hash with the specified key
    #[instrument(skip(self, message_hash), fields(key_id = %key_id))]
    pub fn sign_message(&self, key_id: &str, message_hash: H256) -> Result<Signature> {
        debug!("Signing message with key: {}", key_id);

        if key_id.len() >= SR_MAX_KEY_ID_SIZE {
            return Err(eyre!("Key ID too long: {} bytes", key_id.len()));
        }

        let key_id_cstring = CString::new(key_id)?;
        let message_bytes = message_hash.as_bytes();
        let mut signature_result = SignatureResult {
            signature: [0u8; SR_SECP256K1_SIGNATURE_SIZE],
            signature_len: 0,
            recovery_id: 0,
            _reserved: [0u8; 3],
        };

        let param_types = Self::teec_param_types(
            TEEC_PARAM_TYPE_MEMREF_INPUT,
            TEEC_PARAM_TYPE_MEMREF_INPUT,
            TEEC_PARAM_TYPE_MEMREF_OUTPUT,
            TEEC_PARAM_TYPE_NONE,
        );

        let mut operation = Self::create_operation(param_types);

        // Set parameters
        unsafe {
            operation.params[0].tmpref = TEECTempMemoryReference {
                buffer: key_id_cstring.as_ptr() as *mut c_void,
                size: key_id_cstring.as_bytes().len(),
            };

            operation.params[1].tmpref = TEECTempMemoryReference {
                buffer: message_bytes.as_ptr() as *mut c_void,
                size: message_bytes.len(),
            };

            operation.params[2].tmpref = TEECTempMemoryReference {
                buffer: &mut signature_result as *mut _ as *mut c_void,
                size: std::mem::size_of::<SignatureResult>(),
            };
        }

        // Invoke TA command
        {
            let mut session = self.session.lock().unwrap();
            session.invoke_command(TA_SUPER_RELAY_CMD_SIGN_MESSAGE, &mut operation)?;
        }

        // Parse signature result
        if signature_result.signature_len != 64 {
            return Err(eyre!(
                "Invalid signature length: {}",
                signature_result.signature_len
            ));
        }

        // Extract r and s components
        let mut r_bytes = [0u8; 32];
        let mut s_bytes = [0u8; 32];
        r_bytes.copy_from_slice(&signature_result.signature[0..32]);
        s_bytes.copy_from_slice(&signature_result.signature[32..64]);

        let signature = Signature {
            r: U256::from_big_endian(&r_bytes),
            s: U256::from_big_endian(&s_bytes),
            v: 27 + signature_result.recovery_id as u64, // Convert to Ethereum v
        };

        debug!("Successfully signed message with key: {}", key_id);
        Ok(signature)
    }

    /// Get public key information for a key
    #[instrument(skip(self), fields(key_id = %key_id))]
    pub fn get_public_key(&self, key_id: &str) -> Result<(Vec<u8>, Address)> {
        debug!("Getting public key for: {}", key_id);

        if key_id.len() >= SR_MAX_KEY_ID_SIZE {
            return Err(eyre!("Key ID too long: {} bytes", key_id.len()));
        }

        let key_id_cstring = CString::new(key_id)?;
        let mut pubkey_result = PublicKeyResult {
            public_key: [0u8; 64],
            public_key_len: 0,
            ethereum_address: [0u8; SR_ETHEREUM_ADDRESS_SIZE],
            _reserved: [0u8; 8],
        };

        let param_types = Self::teec_param_types(
            TEEC_PARAM_TYPE_MEMREF_INPUT,
            TEEC_PARAM_TYPE_MEMREF_OUTPUT,
            TEEC_PARAM_TYPE_NONE,
            TEEC_PARAM_TYPE_NONE,
        );

        let mut operation = Self::create_operation(param_types);

        // Set parameters
        unsafe {
            operation.params[0].tmpref = TEECTempMemoryReference {
                buffer: key_id_cstring.as_ptr() as *mut c_void,
                size: key_id_cstring.as_bytes().len(),
            };

            operation.params[1].tmpref = TEECTempMemoryReference {
                buffer: &mut pubkey_result as *mut _ as *mut c_void,
                size: std::mem::size_of::<PublicKeyResult>(),
            };
        }

        // Invoke TA command
        {
            let mut session = self.session.lock().unwrap();
            session.invoke_command(TA_SUPER_RELAY_CMD_GET_PUBLIC_KEY, &mut operation)?;
        }

        let public_key =
            pubkey_result.public_key[0..pubkey_result.public_key_len as usize].to_vec();
        let address = Address::from_slice(&pubkey_result.ethereum_address);

        debug!("Retrieved public key for: {} -> {:?}", key_id, address);
        Ok((public_key, address))
    }

    /// Perform health check on the TA
    #[instrument(skip(self))]
    pub fn health_check(&self) -> Result<()> {
        debug!("Performing OP-TEE TA health check");

        let mut health_result = HealthResult {
            status: 0,
            active_sessions: 0,
            total_operations: 0,
            storage_usage: 0,
            uptime: 0,
        };

        let param_types = Self::teec_param_types(
            TEEC_PARAM_TYPE_MEMREF_OUTPUT,
            TEEC_PARAM_TYPE_NONE,
            TEEC_PARAM_TYPE_NONE,
            TEEC_PARAM_TYPE_NONE,
        );

        let mut operation = Self::create_operation(param_types);

        // Set parameters
        unsafe {
            operation.params[0].tmpref = TEECTempMemoryReference {
                buffer: &mut health_result as *mut _ as *mut c_void,
                size: std::mem::size_of::<HealthResult>(),
            };
        }

        // Invoke TA command
        {
            let mut session = self.session.lock().unwrap();
            session.invoke_command(TA_SUPER_RELAY_CMD_HEALTH_CHECK, &mut operation)?;
        }

        if health_result.status != TEEC_SUCCESS {
            return Err(eyre!(
                "TA health check failed: 0x{:x}",
                health_result.status
            ));
        }

        info!(
            "OP-TEE TA health check passed - sessions: {}, operations: {}, uptime: {}s",
            health_result.active_sessions, health_result.total_operations, health_result.uptime
        );

        Ok(())
    }
}

impl Clone for OpteKmsProvider {
    fn clone(&self) -> Self {
        Self {
            session: Arc::clone(&self.session),
            config: self.config.clone(),
            key_cache: Arc::clone(&self.key_cache),
        }
    }
}

#[async_trait::async_trait]
// TODO: Implement KmsProvider trait once it's defined in kms.rs
// impl KmsProvider for OpteKmsProvider {
    #[instrument(skip(self, request))]
    async fn sign(&self, request: KmsSigningRequest) -> Result<Signature, KmsError> {
        debug!("KMS sign request for key: {}", request.key_id);

        self.sign_message(&request.key_id, request.message_hash)
            .map_err(|e| {
                error!("OP-TEE signing failed: {}", e);
                KmsError::SignatureFailed {
                    reason: e.to_string(),
                }
            })
    }

    #[instrument(skip(self))]
    async fn get_address(&self, key_id: &str) -> Result<Address, KmsError> {
        debug!("Getting address for key: {}", key_id);

        // Check cache first
        {
            let cache = self.key_cache.lock().unwrap();
            if let Some(address) = cache.get(key_id) {
                return Ok(*address);
            }
        }

        // Get from TA
        match self.get_public_key(key_id) {
            Ok((_, address)) => {
                // Update cache
                {
                    let mut cache = self.key_cache.lock().unwrap();
                    cache.insert(key_id.to_string(), address);
                }
                Ok(address)
            }
            Err(e) => {
                error!("Failed to get address for key {}: {}", key_id, e);
                Err(KmsError::KeyNotFound {
                    key_id: key_id.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optee_kms_config_default() {
        let config = OpteeKmsConfig::default();
        assert_eq!(config.device_path, "/dev/teepriv0");
        assert_eq!(config.ta_uuid, "12345678-5b69-11d4-9fee-00c04f4c3456");
        assert_eq!(config.session_timeout, 300);
        assert_eq!(config.max_retries, 3);
        assert!(config.audit_logging);
    }

    #[test]
    fn test_teec_param_types() {
        let param_types = OpteKmsProvider::teec_param_types(
            TEEC_PARAM_TYPE_MEMREF_INPUT,
            TEEC_PARAM_TYPE_VALUE_INPUT,
            TEEC_PARAM_TYPE_MEMREF_OUTPUT,
            TEEC_PARAM_TYPE_NONE,
        );

        // Expected: 0x0615 (MEMREF_INPUT=5, VALUE_INPUT=1, MEMREF_OUTPUT=6, NONE=0)
        assert_eq!(param_types, 0x0615);
    }

    // Note: Integration tests require OP-TEE environment
    #[cfg(feature = "integration-tests")]
    mod integration_tests {
        use tokio;

        use super::*;

        #[tokio::test]
        async fn test_optee_kms_provider_creation() {
            let config = OpteeKmsConfig::default();

            match OpteKmsProvider::new(config) {
                Ok(provider) => {
                    // Test health check
                    assert!(provider.health_check().is_ok());
                }
                Err(e) => {
                    // Expected if OP-TEE not available in test environment
                    println!("OP-TEE not available in test environment: {}", e);
                }
            }
        }

        #[tokio::test]
        async fn test_key_generation_and_signing() {
            let config = OpteeKmsConfig::default();

            if let Ok(provider) = OpteKmsProvider::new(config) {
                let key_id = "test-key-001";

                // Generate key
                match provider.generate_key(key_id, 0) {
                    Ok(address) => {
                        println!("Generated key with address: {:?}", address);

                        // Test signing
                        let message_hash = H256::random();
                        match provider.sign_message(key_id, message_hash) {
                            Ok(signature) => {
                                println!("Signature: {:?}", signature);
                                assert_ne!(signature.r, U256::zero());
                                assert_ne!(signature.s, U256::zero());
                            }
                            Err(e) => panic!("Signing failed: {}", e),
                        }
                    }
                    Err(e) => panic!("Key generation failed: {}", e),
                }
            }
        }
    }
}
