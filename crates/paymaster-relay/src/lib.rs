pub mod api_docs;
pub mod api_handlers;
pub mod api_schemas;
pub mod api_server;
pub mod error;
pub mod kms;
pub mod metrics;
pub mod policy;
pub mod rpc;
pub mod schemas;
pub mod service;
pub mod signer;
pub mod swagger;
pub mod validation;

// Re-export commonly used types
pub use api_server::{create_api_router, start_api_server, AppState};
pub use error::PaymasterError;
pub use kms::{KmsConfig, KmsError, MockKmsProvider, SigningContext};
pub use rpc::{PaymasterRelayApiServer, PaymasterRelayApiServerImpl};
pub use service::PaymasterRelayService;
pub use signer::{SignerBackend, SignerManager};
pub use swagger::{serve_swagger_ui, SwaggerState};
