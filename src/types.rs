// src/types.rs
// Re-export key Rithmic API types for client-side use.
pub use crate::rti::messages::*;

// Client-facing enums for common types
#[derive(Debug, Clone, Copy)]
pub enum TransactionType {
    Buy = 1,  // Explicitly set to match rti::request_new_order::TransactionType::Buy
    Sell = 2, // Explicitly set to match rti::request_new_order::TransactionType::Sell
}

impl From<TransactionType> for crate::rti::request_new_order::TransactionType {
    fn from(val: TransactionType) -> Self {
        match val {
            TransactionType::Buy => crate::rti::request_new_order::TransactionType::Buy,
            TransactionType::Sell => crate::rti::request_new_order::TransactionType::Sell,
        }
    }
}

impl From<TransactionType> for crate::rti::request_bracket_order::TransactionType {
    fn from(val: TransactionType) -> Self {
        match val {
            TransactionType::Buy => crate::rti::request_bracket_order::TransactionType::Buy,
            TransactionType::Sell => crate::rti::request_bracket_order::TransactionType::Sell,
        }
    }
}

impl From<TransactionType> for crate::rti::request_oco_order::TransactionType {
    fn from(val: TransactionType) -> Self {
        match val {
            TransactionType::Buy => crate::rti::request_oco_order::TransactionType::Buy,
            TransactionType::Sell => crate::rti::request_oco_order::TransactionType::Sell,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PriceType {
    Limit = 1,      // Explicitly set
    Market = 2,     // Explicitly set
    StopLimit = 3,  // Explicitly set
    StopMarket = 4, // Explicitly set
}

impl From<PriceType> for crate::rti::request_new_order::PriceType {
    fn from(val: PriceType) -> Self {
        match val {
            PriceType::Limit => crate::rti::request_new_order::PriceType::Limit,
            PriceType::Market => crate::rti::request_new_order::PriceType::Market,
            PriceType::StopLimit => crate::rti::request_new_order::PriceType::StopLimit,
            PriceType::StopMarket => crate::rti::request_new_order::PriceType::StopMarket,
        }
    }
}

impl From<PriceType> for crate::rti::request_bracket_order::PriceType {
    fn from(val: PriceType) -> Self {
        match val {
            PriceType::Limit => crate::rti::request_bracket_order::PriceType::Limit,
            PriceType::Market => crate::rti::request_bracket_order::PriceType::Market,
            PriceType::StopLimit => crate::rti::request_bracket_order::PriceType::StopLimit,
            PriceType::StopMarket => crate::rti::request_bracket_order::PriceType::StopMarket,
        }
    }
}

impl From<PriceType> for crate::rti::request_oco_order::PriceType {
    fn from(val: PriceType) -> Self {
        match val {
            PriceType::Limit => crate::rti::request_oco_order::PriceType::Limit,
            PriceType::Market => crate::rti::request_oco_order::PriceType::Market,
            PriceType::StopLimit => crate::rti::request_oco_order::PriceType::StopLimit,
            PriceType::StopMarket => crate::rti::request_oco_order::PriceType::StopMarket,
        }
    }
}

impl From<PriceType> for crate::rti::request_modify_order::PriceType {
    fn from(val: PriceType) -> Self {
        match val {
            PriceType::Limit => crate::rti::request_modify_order::PriceType::Limit,
            PriceType::Market => crate::rti::request_modify_order::PriceType::Market,
            PriceType::StopLimit => crate::rti::request_modify_order::PriceType::StopLimit,
            PriceType::StopMarket => crate::rti::request_modify_order::PriceType::StopMarket,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OrderDuration {
    Day = 1, // Explicitly set
    Gtc = 2, // Explicitly set
    Ioc = 3, // Explicitly set
    Fok = 4, // Explicitly set
}

impl From<OrderDuration> for crate::rti::request_new_order::Duration {
    fn from(val: OrderDuration) -> Self {
        match val {
            OrderDuration::Day => crate::rti::request_new_order::Duration::Day,
            OrderDuration::Gtc => crate::rti::request_new_order::Duration::Gtc,
            OrderDuration::Ioc => crate::rti::request_new_order::Duration::Ioc,
            OrderDuration::Fok => crate::rti::request_new_order::Duration::Fok,
        }
    }
}

impl From<OrderDuration> for crate::rti::request_bracket_order::Duration {
    fn from(val: OrderDuration) -> Self {
        match val {
            OrderDuration::Day => crate::rti::request_bracket_order::Duration::Day,
            OrderDuration::Gtc => crate::rti::request_bracket_order::Duration::Gtc,
            OrderDuration::Ioc => crate::rti::request_bracket_order::Duration::Ioc,
            OrderDuration::Fok => crate::rti::request_bracket_order::Duration::Fok,
        }
    }
}

impl From<OrderDuration> for crate::rti::request_oco_order::Duration {
    fn from(val: OrderDuration) -> Self {
        match val {
            OrderDuration::Day => crate::rti::request_oco_order::Duration::Day,
            OrderDuration::Gtc => crate::rti::request_oco_order::Duration::Gtc,
            OrderDuration::Ioc => crate::rti::request_oco_order::Duration::Ioc,
            OrderDuration::Fok => crate::rti::request_oco_order::Duration::Fok,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BracketType {
    TakeProfit = 2, // Maps to TargetOnly
    StopLoss = 1,   // Maps to StopOnly
    Both = 3,       // Maps to TargetAndStop
}

impl From<BracketType> for crate::rti::request_bracket_order::BracketType {
    fn from(val: BracketType) -> Self {
        match val {
            BracketType::TakeProfit => crate::rti::request_bracket_order::BracketType::TargetOnly, // Mapped
            BracketType::StopLoss => crate::rti::request_bracket_order::BracketType::StopOnly, // Mapped
            BracketType::Both => crate::rti::request_bracket_order::BracketType::TargetAndStop, // Mapped
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketDataField {
    // Basic market data (Mapped to RTI UpdateBits)
    LastTrade = 1,
    Bbo = 2, // Best Bid Offer
    OrderBook = 4,
    Open = 8,
    OpeningIndicator = 16,
    HighLow = 32,
    HighBidLowAsk = 64,
    Close = 128,
    ClosingIndicator = 256,
    Settlement = 512,
    MarketMode = 1024,
    OpenInterest = 2048,
    MarginRate = 4096,
    HighPriceLimit = 8192,
    LowPriceLimit = 16384,
    ProjectedSettlement = 32768,
    // AdjustedClose removed, not found in request_market_data_update_by_underlying::UpdateBits
}

impl From<MarketDataField> for u32 {
    fn from(val: MarketDataField) -> Self {
        match val {
            MarketDataField::LastTrade => {
                crate::rti::request_market_data_update::UpdateBits::LastTrade as u32
            }
            MarketDataField::Bbo => crate::rti::request_market_data_update::UpdateBits::Bbo as u32,
            MarketDataField::OrderBook => {
                crate::rti::request_market_data_update::UpdateBits::OrderBook as u32
            }
            MarketDataField::Open => {
                crate::rti::request_market_data_update::UpdateBits::Open as u32
            }
            MarketDataField::OpeningIndicator => {
                crate::rti::request_market_data_update::UpdateBits::OpeningIndicator as u32
            }
            MarketDataField::HighLow => {
                crate::rti::request_market_data_update::UpdateBits::HighLow as u32
            }
            MarketDataField::HighBidLowAsk => {
                crate::rti::request_market_data_update::UpdateBits::HighBidLowAsk as u32
            }
            MarketDataField::Close => {
                crate::rti::request_market_data_update::UpdateBits::Close as u32
            }
            MarketDataField::ClosingIndicator => {
                crate::rti::request_market_data_update::UpdateBits::ClosingIndicator as u32
            }
            MarketDataField::Settlement => {
                crate::rti::request_market_data_update::UpdateBits::Settlement as u32
            }
            MarketDataField::MarketMode => {
                crate::rti::request_market_data_update::UpdateBits::MarketMode as u32
            }
            MarketDataField::OpenInterest => {
                crate::rti::request_market_data_update::UpdateBits::OpenInterest as u32
            }
            MarketDataField::MarginRate => {
                crate::rti::request_market_data_update::UpdateBits::MarginRate as u32
            }
            MarketDataField::HighPriceLimit => {
                crate::rti::request_market_data_update::UpdateBits::HighPriceLimit as u32
            }
            MarketDataField::LowPriceLimit => {
                crate::rti::request_market_data_update::UpdateBits::LowPriceLimit as u32
            }
            MarketDataField::ProjectedSettlement => {
                crate::rti::request_market_data_update::UpdateBits::ProjectedSettlement as u32
            }
        }
    }
}

impl From<MarketDataField> for crate::rti::request_market_data_update_by_underlying::UpdateBits {
    fn from(val: MarketDataField) -> Self {
        match val {
            MarketDataField::LastTrade => crate::rti::request_market_data_update_by_underlying::UpdateBits::LastTrade,
            MarketDataField::Bbo => crate::rti::request_market_data_update_by_underlying::UpdateBits::Bbo,
            MarketDataField::OrderBook => crate::rti::request_market_data_update_by_underlying::UpdateBits::OrderBook,
            MarketDataField::Open => crate::rti::request_market_data_update_by_underlying::UpdateBits::Open,
            MarketDataField::OpeningIndicator => crate::rti::request_market_data_update_by_underlying::UpdateBits::OpeningIndicator,
            MarketDataField::HighLow => crate::rti::request_market_data_update_by_underlying::UpdateBits::HighLow,
            MarketDataField::HighBidLowAsk => crate::rti::request_market_data_update_by_underlying::UpdateBits::HighBidLowAsk,
            MarketDataField::Close => crate::rti::request_market_data_update_by_underlying::UpdateBits::Close,
            MarketDataField::ClosingIndicator => crate::rti::request_market_data_update_by_underlying::UpdateBits::ClosingIndicator,
            MarketDataField::Settlement => crate::rti::request_market_data_update_by_underlying::UpdateBits::Settlement,
            MarketDataField::MarketMode => crate::rti::request_market_data_update_by_underlying::UpdateBits::MarketMode,
            MarketDataField::OpenInterest => crate::rti::request_market_data_update_by_underlying::UpdateBits::OpenInterest,
            MarketDataField::MarginRate => crate::rti::request_market_data_update_by_underlying::UpdateBits::MarginRate,
            MarketDataField::HighPriceLimit => crate::rti::request_market_data_update_by_underlying::UpdateBits::HighPriceLimit,
            MarketDataField::LowPriceLimit => crate::rti::request_market_data_update_by_underlying::UpdateBits::LowPriceLimit,
            MarketDataField::ProjectedSettlement => crate::rti::request_market_data_update_by_underlying::UpdateBits::ProjectedSettlement,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MarketDataRequestType {
    Subscribe = 1, // Explicitly set
    Unsubscribe = 2, // Explicitly set
                   // Snapshot removed as it's not a direct RTI Request enum variant
}

impl From<MarketDataRequestType> for crate::rti::request_market_data_update::Request {
    fn from(val: MarketDataRequestType) -> Self {
        match val {
            MarketDataRequestType::Subscribe => {
                crate::rti::request_market_data_update::Request::Subscribe
            }
            MarketDataRequestType::Unsubscribe => {
                crate::rti::request_market_data_update::Request::Unsubscribe
            }
        }
    }
}

impl From<MarketDataRequestType> for crate::rti::request_market_data_update_by_underlying::Request {
    fn from(val: MarketDataRequestType) -> Self {
        match val {
            MarketDataRequestType::Subscribe => {
                crate::rti::request_market_data_update_by_underlying::Request::Subscribe
            }
            MarketDataRequestType::Unsubscribe => {
                crate::rti::request_market_data_update_by_underlying::Request::Unsubscribe
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SysInfraType {
    TickerPlant = 1,
    OrderPlant = 2,
    HistoryPlant = 3,
    PnlPlant = 4,
    RepositoryPlant = 5,
}

impl From<SysInfraType> for crate::rti::request_login::SysInfraType {
    fn from(val: SysInfraType) -> Self {
        match val {
            SysInfraType::TickerPlant => crate::rti::request_login::SysInfraType::TickerPlant,
            SysInfraType::OrderPlant => crate::rti::request_login::SysInfraType::OrderPlant,
            SysInfraType::HistoryPlant => crate::rti::request_login::SysInfraType::HistoryPlant,
            SysInfraType::PnlPlant => crate::rti::request_login::SysInfraType::PnlPlant,
            SysInfraType::RepositoryPlant => {
                crate::rti::request_login::SysInfraType::RepositoryPlant
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InstrumentType {
    Future = 1,       // Explicitly set
    FutureOption = 2, // Explicitly set
    Equity = 4,       // Explicitly set
    EquityOption = 5, // Explicitly set
    Spread = 9,       // Explicitly set
    Synthetic = 10,   // Explicitly set
                      // Forex removed
}

impl From<InstrumentType> for crate::rti::request_search_symbols::InstrumentType {
    fn from(val: InstrumentType) -> Self {
        match val {
            InstrumentType::Future => crate::rti::request_search_symbols::InstrumentType::Future,
            InstrumentType::FutureOption => {
                crate::rti::request_search_symbols::InstrumentType::FutureOption
            } // Adjusted
            InstrumentType::Equity => crate::rti::request_search_symbols::InstrumentType::Equity,
            InstrumentType::EquityOption => {
                crate::rti::request_search_symbols::InstrumentType::EquityOption
            } // Adjusted
            InstrumentType::Spread => crate::rti::request_search_symbols::InstrumentType::Spread,
            InstrumentType::Synthetic => {
                crate::rti::request_search_symbols::InstrumentType::Synthetic
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SearchPattern {
    Equals = 1,   // Explicitly set
    Contains = 2, // Explicitly set
}

impl From<SearchPattern> for crate::rti::request_search_symbols::Pattern {
    fn from(val: SearchPattern) -> Self {
        match val {
            SearchPattern::Equals => crate::rti::request_search_symbols::Pattern::Equals,
            SearchPattern::Contains => crate::rti::request_search_symbols::Pattern::Contains,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TickBarReplayBarType {
    TickBar = 1,   // Explicitly set
    VolumeBar = 3, // Explicitly set
    RangeBar = 2,  // Explicitly set
}

impl From<TickBarReplayBarType> for crate::rti::request_tick_bar_replay::BarType {
    fn from(val: TickBarReplayBarType) -> Self {
        match val {
            TickBarReplayBarType::TickBar => crate::rti::request_tick_bar_replay::BarType::TickBar,
            TickBarReplayBarType::VolumeBar => {
                crate::rti::request_tick_bar_replay::BarType::VolumeBar
            }
            TickBarReplayBarType::RangeBar => {
                crate::rti::request_tick_bar_replay::BarType::RangeBar
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TickBarReplayBarSubType {
    Regular = 1, // Explicitly set
    Custom = 2,  // Explicitly set
}

impl From<TickBarReplayBarSubType> for crate::rti::request_tick_bar_replay::BarSubType {
    fn from(val: TickBarReplayBarSubType) -> Self {
        match val {
            TickBarReplayBarSubType::Regular => {
                crate::rti::request_tick_bar_replay::BarSubType::Regular
            }
            TickBarReplayBarSubType::Custom => {
                crate::rti::request_tick_bar_replay::BarSubType::Custom
            } // Added
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TickBarReplayDirection {
    First = 1, // Explicitly set
    Last = 2,  // Explicitly set
}

impl From<TickBarReplayDirection> for crate::rti::request_tick_bar_replay::Direction {
    fn from(val: TickBarReplayDirection) -> Self {
        match val {
            TickBarReplayDirection::First => crate::rti::request_tick_bar_replay::Direction::First,
            TickBarReplayDirection::Last => crate::rti::request_tick_bar_replay::Direction::Last,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TickBarReplayTimeOrder {
    Forwards = 1,  // Explicitly set
    Backwards = 2, // Explicitly set
}

impl From<TickBarReplayTimeOrder> for crate::rti::request_tick_bar_replay::TimeOrder {
    fn from(val: TickBarReplayTimeOrder) -> Self {
        match val {
            TickBarReplayTimeOrder::Forwards => {
                crate::rti::request_tick_bar_replay::TimeOrder::Forwards
            }
            TickBarReplayTimeOrder::Backwards => {
                crate::rti::request_tick_bar_replay::TimeOrder::Backwards
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeBarReplayBarType {
    SecondBar = 1, // Explicitly set
    MinuteBar = 2, // Explicitly set
    DailyBar = 3,  // Explicitly set
    WeeklyBar = 4, // Explicitly set
}

impl From<TimeBarReplayBarType> for crate::rti::request_time_bar_replay::BarType {
    fn from(val: TimeBarReplayBarType) -> Self {
        match val {
            TimeBarReplayBarType::SecondBar => {
                crate::rti::request_time_bar_replay::BarType::SecondBar
            }
            TimeBarReplayBarType::MinuteBar => {
                crate::rti::request_time_bar_replay::BarType::MinuteBar
            }
            TimeBarReplayBarType::DailyBar => {
                crate::rti::request_time_bar_replay::BarType::DailyBar
            }
            TimeBarReplayBarType::WeeklyBar => {
                crate::rti::request_time_bar_replay::BarType::WeeklyBar
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeBarReplayDirection {
    First = 1, // Explicitly set
    Last = 2,  // Explicitly set
}

impl From<TimeBarReplayDirection> for crate::rti::request_time_bar_replay::Direction {
    fn from(val: TimeBarReplayDirection) -> Self {
        match val {
            TimeBarReplayDirection::First => crate::rti::request_time_bar_replay::Direction::First,
            TimeBarReplayDirection::Last => crate::rti::request_time_bar_replay::Direction::Last,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeBarReplayTimeOrder {
    Forwards = 1,  // Explicitly set
    Backwards = 2, // Explicitly set
}

impl From<TimeBarReplayTimeOrder> for crate::rti::request_time_bar_replay::TimeOrder {
    fn from(val: TimeBarReplayTimeOrder) -> Self {
        match val {
            TimeBarReplayTimeOrder::Forwards => {
                crate::rti::request_time_bar_replay::TimeOrder::Forwards
            }
            TimeBarReplayTimeOrder::Backwards => {
                crate::rti::request_time_bar_replay::TimeOrder::Backwards
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeBarUpdateBarType {
    SecondBar = 1,
    MinuteBar = 2,
    DailyBar = 3,
    WeeklyBar = 4,
}

impl From<TimeBarUpdateBarType> for crate::rti::request_time_bar_update::BarType {
    fn from(val: TimeBarUpdateBarType) -> Self {
        match val {
            TimeBarUpdateBarType::SecondBar => {
                crate::rti::request_time_bar_update::BarType::SecondBar
            }
            TimeBarUpdateBarType::MinuteBar => {
                crate::rti::request_time_bar_update::BarType::MinuteBar
            }
            TimeBarUpdateBarType::DailyBar => {
                crate::rti::request_time_bar_update::BarType::DailyBar
            }
            TimeBarUpdateBarType::WeeklyBar => {
                crate::rti::request_time_bar_update::BarType::WeeklyBar
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeBarUpdateRequest {
    Subscribe = 1,
    Unsubscribe = 2,
}

impl From<TimeBarUpdateRequest> for crate::rti::request_time_bar_update::Request {
    fn from(val: TimeBarUpdateRequest) -> Self {
        match val {
            TimeBarUpdateRequest::Subscribe => {
                crate::rti::request_time_bar_update::Request::Subscribe
            }
            TimeBarUpdateRequest::Unsubscribe => {
                crate::rti::request_time_bar_update::Request::Unsubscribe
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TickBarUpdateBarType {
    TickBar = 1,
    VolumeBar = 3,
    RangeBar = 2,
}

impl From<TickBarUpdateBarType> for crate::rti::request_tick_bar_update::BarType {
    fn from(val: TickBarUpdateBarType) -> Self {
        match val {
            TickBarUpdateBarType::TickBar => crate::rti::request_tick_bar_update::BarType::TickBar,
            TickBarUpdateBarType::VolumeBar => {
                crate::rti::request_tick_bar_update::BarType::VolumeBar
            }
            TickBarUpdateBarType::RangeBar => {
                crate::rti::request_tick_bar_update::BarType::RangeBar
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TickBarUpdateBarSubType {
    Regular = 1,
    Custom = 2,
}

impl From<TickBarUpdateBarSubType> for crate::rti::request_tick_bar_update::BarSubType {
    fn from(val: TickBarUpdateBarSubType) -> Self {
        match val {
            TickBarUpdateBarSubType::Regular => {
                crate::rti::request_tick_bar_update::BarSubType::Regular
            }
            TickBarUpdateBarSubType::Custom => {
                crate::rti::request_tick_bar_update::BarSubType::Custom
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TickBarUpdateRequest {
    Subscribe = 1,
    Unsubscribe = 2,
}

impl From<TickBarUpdateRequest> for crate::rti::request_tick_bar_update::Request {
    fn from(val: TickBarUpdateRequest) -> Self {
        match val {
            TickBarUpdateRequest::Subscribe => {
                crate::rti::request_tick_bar_update::Request::Subscribe
            }
            TickBarUpdateRequest::Unsubscribe => {
                crate::rti::request_tick_bar_update::Request::Unsubscribe
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AccountRmsUpdateBits {
    AutoLiqThresholdCurrentValue = 1,
}

impl From<AccountRmsUpdateBits> for i32 {
    fn from(val: AccountRmsUpdateBits) -> Self {
        match val {
            AccountRmsUpdateBits::AutoLiqThresholdCurrentValue => {
                crate::rti::account_rms_updates::UpdateBits::AutoLiqThresholdCurrentValue as i32
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PnlPositionUpdateRequest {
    Subscribe = 1,
    Unsubscribe = 2,
}

impl From<PnlPositionUpdateRequest> for crate::rti::request_pn_l_position_updates::Request {
    fn from(val: PnlPositionUpdateRequest) -> Self {
        match val {
            PnlPositionUpdateRequest::Subscribe => {
                crate::rti::request_pn_l_position_updates::Request::Subscribe
            }
            PnlPositionUpdateRequest::Unsubscribe => {
                crate::rti::request_pn_l_position_updates::Request::Unsubscribe
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EasyToBorrowListRequestType {
    Subscribe = 1,
    Unsubscribe = 2,
}

impl From<EasyToBorrowListRequestType> for crate::rti::request_easy_to_borrow_list::Request {
    fn from(val: EasyToBorrowListRequestType) -> Self {
        match val {
            EasyToBorrowListRequestType::Subscribe => {
                crate::rti::request_easy_to_borrow_list::Request::Subscribe
            }
            EasyToBorrowListRequestType::Unsubscribe => {
                crate::rti::request_easy_to_borrow_list::Request::Unsubscribe
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExitPositionOrderPlacement {
    Manual = 1,
    Auto = 2,
}

impl From<ExitPositionOrderPlacement> for crate::rti::request_exit_position::OrderPlacement {
    fn from(val: ExitPositionOrderPlacement) -> Self {
        match val {
            ExitPositionOrderPlacement::Manual => {
                crate::rti::request_exit_position::OrderPlacement::Manual
            }
            ExitPositionOrderPlacement::Auto => {
                crate::rti::request_exit_position::OrderPlacement::Auto
            }
        }
    }
}

/// Parameters for submitting a standard new order
#[derive(Debug, Clone)]
pub struct OrderParams {
    pub symbol: String,
    pub exchange: String,
    pub quantity: i32,
    pub price: f64,
    pub transaction_type: TransactionType,
    pub price_type: PriceType,
    pub duration: OrderDuration,
    pub user_tag: Option<String>,
    pub auto: bool,
}

/// Parameters for modifying an existing order
#[derive(Debug, Clone)]
pub struct ModifyOrderParams {
    pub basket_id: String,
    pub symbol: String,
    pub exchange: String,
    pub quantity: i32,
    pub price: f64,
    pub price_type: PriceType,
    pub auto: bool,
}

/// Parameters for a Bracket Order (Entry + Take Profit + Stop Loss)
#[derive(Debug, Clone)]
pub struct BracketOrderParams {
    pub symbol: String,
    pub exchange: String,
    pub quantity: i32,
    pub price: f64,
    pub transaction_type: TransactionType,
    pub price_type: PriceType,
    pub duration: OrderDuration,
    pub bracket_type: BracketType,
    pub target_ticks: Option<i32>,
    pub stop_ticks: Option<i32>,
    pub user_tag: Option<String>,
}

/// Parameters for one leg of an OCO order
#[derive(Debug, Clone)]
pub struct OcoLegParams {
    pub symbol: String,
    pub exchange: String,
    pub quantity: i32,
    pub price: f64,
    pub transaction_type: TransactionType,
    pub price_type: PriceType,
}

/// Parameters for an OCO (One-Cancels-Other) Order
#[derive(Debug, Clone)]
pub struct OcoOrderParams {
    pub leg1: OcoLegParams,
    pub leg2: OcoLegParams,
    pub duration: OrderDuration,
    pub user_tag: Option<String>,
}
