//! Soroban and Stellar network integration.
//!
//! Provides a typed client for interacting with Soroban RPC nodes,
//! including contract deployment, transaction submission, event
//! querying, and account management.
//!
//! Ownership: Stellar Integration Team
//! Dependencies: config, models
//! Public API: `StellarClient`, `NetworkConfig`, `ContractDeployer`

pub mod client;
pub mod contract;
pub mod error;
pub mod event;
pub mod transaction;
