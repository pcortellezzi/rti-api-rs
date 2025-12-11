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
pub mod types; // Added types module
pub mod ws;

// Internal modules
mod plants;

// Public Re-exports
pub use crate::client::RithmicClient;
pub use crate::types::{
    AccountRmsUpdateBits,
    BracketOrderParams,
    BracketType,
    EasyToBorrowListRequestType,
    ExitPositionOrderPlacement,
    InstrumentType,
    MarketDataField,
    MarketDataRequestType,
    ModifyOrderParams,
    OcoLegParams,
    OcoOrderParams,
    OrderDuration,
    OrderParams,
    PnlPositionUpdateRequest,
    PriceType,
    RithmicMessage, // Re-export RithmicMessage from types
    SearchPattern,
    SysInfraType,
    TickBarReplayBarSubType,
    TickBarReplayBarType,
    TickBarReplayDirection,
    TickBarReplayTimeOrder,
    TickBarUpdateBarSubType,
    TickBarUpdateBarType,
    TickBarUpdateRequest,
    TimeBarReplayBarType,
    TimeBarReplayDirection,
    TimeBarReplayTimeOrder,
    TimeBarUpdateBarType,
    TimeBarUpdateRequest,
    TransactionType,
};

// Type Aliases for better DX
