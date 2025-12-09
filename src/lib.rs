//! # rti-api-rs
//!
//! `rti-api-rs` is a Rust client library for the Rithmic R | Protocol API.
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
pub use client::{OrderParams, ModifyOrderParams, BracketOrderParams, OcoOrderParams, OcoLegParams};
pub use connection_info::AccountInfo;
pub use rti::messages::RithmicMessage;

// Type Aliases for better DX
pub use rti::request_new_order::TransactionType;
pub use rti::request_new_order::PriceType;
pub use rti::request_new_order::Duration as OrderDuration; // Renamed to avoid conflict with std::time::Duration
pub use rti::request_bracket_order::BracketType;
pub use rti::request_market_data_update::UpdateBits as MarketDataField; // More descriptive name
