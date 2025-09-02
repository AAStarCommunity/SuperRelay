pub mod airaccount_kms;
pub mod api_docs;
pub mod api_handlers;
pub mod api_schemas;
pub mod api_server;
pub mod error;
#[cfg(feature = "integration-tests")]
pub mod integration_tests;
pub mod key_manager;
pub mod kms;
pub mod metrics;
// TODO: Fix KmsProvider trait dependencies before enabling
// #[cfg(feature = "optee-kms")]
// pub mod optee_kms;
pub mod policy;
pub mod proxy_client;
pub mod proxy_server;
pub mod rpc;
pub mod schemas;
pub mod service;
pub mod signer;
pub mod swagger;
pub mod validation;

// Re-export commonly used types
pub use airaccount_kms::{AirAccountKmsClient, KmsDualSignRequest, KmsSignResponse};
pub use api_server::{create_api_router, start_api_server, AppState};
pub use error::PaymasterError;
pub use key_manager::{PaymasterKeyError, PaymasterKeyManager, PaymasterKeyStatus};
pub use kms::{KmsConfig, KmsError, MockKmsProvider, SigningContext};
// TODO: Re-enable when optee_kms module is fixed
// #[cfg(feature = "optee-kms")]
// pub use optee_kms::{OpteKmsProvider, OpteeKmsConfig};
pub use proxy_server::start_proxy_api_server;
pub use rpc::{PaymasterRelayApiServer, PaymasterRelayApiServerImpl};
pub use service::PaymasterRelayService;
pub use signer::{SignerBackend, SignerManager};
pub use swagger::{serve_swagger_ui, SwaggerState};
