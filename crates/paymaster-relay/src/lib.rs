//! SuperRelay Paymaster Service
//!
//! This crate provides ERC-4337 paymaster functionality as a non-invasive extension
//! to the Rundler bundler. It implements gas sponsorship with configurable policies.

#![warn(missing_docs)]
#![deny(unused_must_use, rust_2018_idioms)]
#![allow(unused_crate_dependencies)]

pub mod config;
pub mod error;
pub mod policy;
pub mod rpc;
pub mod service;
pub mod signer;

pub use config::Config;
pub use error::PaymasterError;
pub use rpc::PaymasterRelayApiServer;
pub use service::PaymasterRelayService;
