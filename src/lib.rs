//! # rithmic-rs
//!
//! `rithmic-rs` is a Rust client library for the Rithmic R | Protocol API.
//!
//! ## Features
//!
//! - Unified client interface for Rithmic API
//! - Async/Await support via Tokio
//! - Market Data (Ticker Plant)
//! - Order Management (Order Plant)
//! - Historical Data (History Plant)
//! - PnL & Position Tracking (PnL Plant)

pub mod api;
pub mod client;
pub mod connection_info;
pub mod rti;
pub mod ws;

// Internal modules
mod plants;

// Public Re-exports
pub use client::RithmicClient;
pub use connection_info::{AccountInfo, RithmicConnectionSystem};
pub use rti::messages::RithmicMessage;
