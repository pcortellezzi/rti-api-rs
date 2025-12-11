use prost::Message;

use crate::{
    connection_info::AccountInfo,
    rti::{self, *},
    types::{
        AccountRmsUpdateBits, BracketOrderParams, EasyToBorrowListRequestType,
        InstrumentType, MarketDataField, MarketDataRequestType,
        ModifyOrderParams, OcoOrderParams, OrderParams, SearchPattern,
        SysInfraType, TickBarReplayBarType, TickBarReplayBarSubType, TickBarReplayDirection,
        TickBarReplayTimeOrder, TickBarUpdateBarType, TickBarUpdateBarSubType,
        TickBarUpdateRequest, TimeBarReplayBarType, TimeBarReplayDirection, TimeBarReplayTimeOrder,
        TimeBarUpdateBarType, TimeBarUpdateRequest,
    },
};

pub const TRADE_ROUTE_LIVE: &str = "globex";
pub const TRADE_ROUTE_DEMO: &str = "simulator";
pub const USER_TYPE: i32 = 3;

#[derive(Debug, Clone)]
pub struct RithmicSenderApi {
    message_id_counter: u64,
}

impl Default for RithmicSenderApi {
    fn default() -> Self {
        Self::new()
    }
}

impl RithmicSenderApi {
    pub fn new() -> Self {
        RithmicSenderApi {
            message_id_counter: 0,
        }
    }

    fn get_next_message_id(&mut self) -> String {
        self.message_id_counter += 1;
        self.message_id_counter.to_string()
    }

    fn request_to_buf(&self, req: impl Message, id: String) -> (Vec<u8>, String) {
        let mut buf = Vec::new();
        let len = req.encoded_len() as u32;
        let header = len.to_be_bytes();

        buf.reserve((len + 4) as usize);
        req.encode(&mut buf).unwrap();
        buf.splice(0..0, header.iter().cloned());

        (buf, id)
    }

    pub fn request_rithmic_system_info(&mut self) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestRithmicSystemInfo {
            template_id: 16,
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_rithmic_system_gateway_info(
        &mut self,
        system_name: String,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestRithmicSystemGatewayInfo {
            template_id: 20,
            user_msg: vec![id.clone()],
            system_name: Some(system_name),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_login(
        &mut self,
        system_name: &str,
        infra_type: SysInfraType,
        user: &str,
        password: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestLogin {
            template_id: 10,
            template_version: Some("5.34".into()),
            user: Some(user.to_string()),
            password: Some(password.to_string()),
            app_name: Some("rti-api-rs".to_string()),
            app_version: Some("0.1.0".into()),
            system_name: Some(system_name.to_string()),
            infra_type: Some(request_login::SysInfraType::from(infra_type).into()),
            user_msg: vec![id.clone()],
            ..RequestLogin::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_logout(&mut self) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestLogout {
            template_id: 12,
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_heartbeat(&mut self) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestHeartbeat {
            template_id: 18,
            user_msg: vec![id.clone()],
            ..RequestHeartbeat::default()
        };

        self.request_to_buf(req, id)
    }

    // --- Market Data ---

    pub fn request_market_data_update(
        &mut self,
        symbol: &str,
        exchange: &str,
        fields: Vec<MarketDataField>,
        request_type: MarketDataRequestType,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let mut bits = 0;
        for field in fields {
            bits |= u32::from(field);
        }

        let req = RequestMarketDataUpdate {
            template_id: 100,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
            request: Some(request_market_data_update::Request::from(request_type).into()),
            update_bits: Some(bits),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_search_symbols(
        &mut self,
        search_text: &str,
        exchange: &str,
        product_code: &str,
        instrument_type: Option<InstrumentType>,
        pattern: Option<SearchPattern>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestSearchSymbols {
            template_id: 109,
            user_msg: vec![id.clone()],
            search_text: Some(search_text.into()),
            exchange: Some(exchange.into()),
            product_code: Some(product_code.into()),
            instrument_type: instrument_type
                .map(|it| request_search_symbols::InstrumentType::from(it).into()),
            pattern: pattern.map(|p| request_search_symbols::Pattern::from(p).into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_get_instrument_by_underlying(
        &mut self,
        underlying_symbol: &str,
        exchange: &str,
        expiration_date: Option<&str>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestGetInstrumentByUnderlying {
            template_id: 102,
            user_msg: vec![id.clone()],
            underlying_symbol: Some(underlying_symbol.into()),
            exchange: Some(exchange.into()),
            expiration_date: expiration_date.map(|s| s.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_market_data_update_by_underlying(
        &mut self,
        underlying_symbol: &str,
        exchange: &str,
        expiration_date: Option<&str>,
        fields: Vec<MarketDataField>,
        request_type: MarketDataRequestType,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let mut bits = 0;
        for field in fields {
            bits |= u32::from(field);
        }
        let req = RequestMarketDataUpdateByUnderlying {
            template_id: 105,
            user_msg: vec![id.clone()],
            underlying_symbol: Some(underlying_symbol.into()),
            exchange: Some(exchange.into()),
            expiration_date: expiration_date.map(|s| s.into()),
            request: Some(rti::request_market_data_update_by_underlying::Request::from(request_type).into()),
            update_bits: Some(bits),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_give_tick_size_type_table(
        &mut self,
        tick_size_type: Option<&str>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestGiveTickSizeTypeTable {
            template_id: 107,
            user_msg: vec![id.clone()],
            tick_size_type: tick_size_type.map(|s| s.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_product_codes(
        &mut self,
        exchange: Option<&str>,
        give_toi_products_only: Option<bool>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestProductCodes {
            template_id: 111,
            user_msg: vec![id.clone()],
            exchange: exchange.map(|s| s.into()),
            give_toi_products_only,
        };

        self.request_to_buf(req, id)
    }

    pub fn request_depth_by_order_snapshot(
        &mut self,
        symbol: &str,
        exchange: &str,
        depth_price: Option<f64>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestDepthByOrderSnapshot {
            template_id: 115,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
            depth_price,
        };

        self.request_to_buf(req, id)
    }

    pub fn request_depth_by_order_updates(
        &mut self,
        request_type: rti::request_depth_by_order_updates::Request,
        symbol: &str,
        exchange: &str,
        depth_price: Option<f64>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestDepthByOrderUpdates {
            template_id: 117,
            user_msg: vec![id.clone()],
            request: Some(request_type.into()),
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
            depth_price,
        };

        self.request_to_buf(req, id)
    }

    pub fn request_get_volume_at_price(
        &mut self,
        symbol: &str,
        exchange: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestGetVolumeAtPrice {
            template_id: 119,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_auxilliary_reference_data(
        &mut self,
        symbol: &str,
        exchange: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestAuxilliaryReferenceData {
            template_id: 121,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_reference_data(&mut self, symbol: &str, exchange: &str) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestReferenceData {
            template_id: 14,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_front_month_contract(
        &mut self,
        symbol: &str,
        exchange: &str,
        subscribe: bool,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestFrontMonthContract {
            template_id: 113,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
            need_updates: Some(subscribe),
        };

        self.request_to_buf(req, id)
    }

    // --- History ---

    #[allow(clippy::too_many_arguments)]
    pub fn request_tick_bar_replay(
        &mut self,
        exchange: String,
        symbol: String,
        start_index_sec: i32,
        finish_index_sec: i32,
        bar_type: TickBarReplayBarType,
        bar_sub_type: TickBarReplayBarSubType,
        direction: TickBarReplayDirection,
        time_order: TickBarReplayTimeOrder,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestTickBarReplay {
            template_id: 206,
            symbol: Some(symbol),
            exchange: Some(exchange),
            bar_type: Some(rti::request_tick_bar_replay::BarType::from(bar_type).into()),
            bar_sub_type: Some(rti::request_tick_bar_replay::BarSubType::from(bar_sub_type).into()),
            bar_type_specifier: Some("1".to_string()),
            start_index: Some(start_index_sec),
            finish_index: Some(finish_index_sec),
            direction: Some(rti::request_tick_bar_replay::Direction::from(direction).into()),
            time_order: Some(rti::request_tick_bar_replay::TimeOrder::from(time_order).into()),
            user_msg: vec![id.clone()],
            ..RequestTickBarReplay::default()
        };

        self.request_to_buf(req, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_time_bar_replay(
        &mut self,
        exchange: String,
        symbol: String,
        bar_type: TimeBarReplayBarType,
        bar_type_period: i32,
        start_index_sec: i32,
        finish_index_sec: i32,
        direction: TimeBarReplayDirection,
        time_order: TimeBarReplayTimeOrder,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestTimeBarReplay {
            template_id: 202,
            exchange: Some(exchange),
            symbol: Some(symbol),
            bar_type: Some(rti::request_time_bar_replay::BarType::from(bar_type).into()),
            bar_type_period: Some(bar_type_period),
            start_index: Some(start_index_sec),
            finish_index: Some(finish_index_sec),
            direction: Some(rti::request_time_bar_replay::Direction::from(direction).into()),
            time_order: Some(rti::request_time_bar_replay::TimeOrder::from(time_order).into()),
            user_msg: vec![id.clone()],
            ..RequestTimeBarReplay::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_time_bar_update(
        &mut self,
        symbol: &str,
        exchange: &str,
        request_type: TimeBarUpdateRequest,
        bar_type: TimeBarUpdateBarType,
        bar_type_period: i32,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestTimeBarUpdate {
            template_id: 200,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
            request: Some(rti::request_time_bar_update::Request::from(request_type).into()),
            bar_type: Some(rti::request_time_bar_update::BarType::from(bar_type).into()),
            bar_type_period: Some(bar_type_period),
        };

        self.request_to_buf(req, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_tick_bar_update(
        &mut self,
        symbol: &str,
        exchange: &str,
        request_type: TickBarUpdateRequest,
        bar_type: TickBarUpdateBarType,
        bar_sub_type: TickBarUpdateBarSubType,
        bar_type_specifier: Option<&str>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestTickBarUpdate {
            template_id: 204,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
            request: Some(rti::request_tick_bar_update::Request::from(request_type).into()),
            bar_type: Some(rti::request_tick_bar_update::BarType::from(bar_type).into()),
            bar_sub_type: Some(rti::request_tick_bar_update::BarSubType::from(bar_sub_type).into()),
            bar_type_specifier: bar_type_specifier.map(|s| s.into()),
            ..RequestTickBarUpdate::default()
        };

        self.request_to_buf(req, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_volume_profile_minute_bars(
        &mut self,
        symbol: &str,
        exchange: &str,
        bar_type_period: i32,
        start_index: i32,
        finish_index: i32,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestVolumeProfileMinuteBars {
            template_id: 208,
            user_msg: vec![id.clone()],
            symbol: Some(symbol.into()),
            exchange: Some(exchange.into()),
            bar_type_period: Some(bar_type_period),
            start_index: Some(start_index),
            finish_index: Some(finish_index),
            ..RequestVolumeProfileMinuteBars::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_resume_bars(&mut self, request_key: &str) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestResumeBars {
            template_id: 210,
            user_msg: vec![id.clone()],
            request_key: Some(request_key.into()),
        };

        self.request_to_buf(req, id)
    }

    // --- Order Management ---

    pub fn request_login_info(&mut self) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestLoginInfo {
            template_id: 300,
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_account_list(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestAccountList {
            template_id: 302,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            user_type: Some(rti::request_account_list::UserType::Trader.into()),
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_account_rms_info(
        &mut self,
        account: &AccountInfo,
        user_type: rti::request_account_rms_info::UserType,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestAccountRmsInfo {
            template_id: 304,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            user_type: Some(user_type.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_product_rms_info(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestProductRmsInfo {
            template_id: 306,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_subscribe_for_order_updates(
        &mut self,
        account: &AccountInfo,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestSubscribeForOrderUpdates {
            template_id: 308,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_trade_routes(&mut self) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();
        // Removed #[allow(clippy::needless_update)]
        let req = RequestTradeRoutes {
            template_id: 310,
            user_msg: vec![id.clone()],
            subscribe_for_updates: Some(true),
        };
        self.request_to_buf(req, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_new_order(
        &mut self,
        account: &AccountInfo,
        params: OrderParams,
        trade_route: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestNewOrder {
            template_id: 312,
            user_msg: vec![id.clone()],
            user_tag: params.user_tag,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            symbol: Some(params.symbol),
            exchange: Some(params.exchange),
            quantity: Some(params.quantity),
            price: Some(params.price),
            transaction_type: Some(
                request_new_order::TransactionType::from(params.transaction_type).into(),
            ),
            duration: Some(request_new_order::Duration::from(params.duration).into()),
            price_type: Some(request_new_order::PriceType::from(params.price_type).into()),
            trade_route: Some(trade_route.into()),
            manual_or_auto: if params.auto {
                Some(request_new_order::OrderPlacement::Auto.into())
            } else {
                Some(request_new_order::OrderPlacement::Manual.into())
            },
            ..RequestNewOrder::default()

        };

        self.request_to_buf(req, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_modify_order(
        &mut self,
        account: &AccountInfo,
        params: ModifyOrderParams,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestModifyOrder {
            template_id: 314,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            basket_id: Some(params.basket_id),
            symbol: Some(params.symbol),
            exchange: Some(params.exchange),
            quantity: Some(params.quantity),
            price: Some(params.price),
            price_type: Some(rti::request_modify_order::PriceType::from(params.price_type).into()),
            manual_or_auto: if params.auto {
                Some(request_modify_order::OrderPlacement::Auto.into())
            } else {
                Some(request_modify_order::OrderPlacement::Manual.into())
            },
            ..RequestModifyOrder::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_cancel_order(
        &mut self,
        account: &AccountInfo,
        basket_id: &str,
        auto: bool,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestCancelOrder {
            template_id: 316,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            basket_id: Some(basket_id.into()),
            manual_or_auto: if auto {
                Some(request_cancel_order::OrderPlacement::Auto.into())
            } else {
                Some(request_cancel_order::OrderPlacement::Manual.into())
            },
            ..RequestCancelOrder::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_show_order_history_dates(&mut self) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestShowOrderHistoryDates {
            template_id: 318,
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_cancel_all_orders(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestCancelAllOrders {
            template_id: 346,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            ..RequestCancelAllOrders::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_show_orders(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestShowOrders {
            template_id: 320,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_show_order_history(
        &mut self,
        account: &AccountInfo,
        basket_id: Option<&str>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestShowOrderHistory {
            template_id: 322,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            user_msg: vec![id.clone()],
            basket_id: basket_id.map(|s| s.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_show_order_history_summary(
        &mut self,
        account: &AccountInfo,
        date: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestShowOrderHistorySummary {
            template_id: 324,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            date: Some(date.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_show_order_history_detail(
        &mut self,
        account: &AccountInfo,
        basket_id: Option<&str>,
        date: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestShowOrderHistoryDetail {
            template_id: 326,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            basket_id: basket_id.map(|s| s.into()),
            date: Some(date.into()),
        };

        self.request_to_buf(req, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_bracket_order(
        &mut self,
        account: &AccountInfo,
        params: BracketOrderParams,
        trade_route: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let mut req = RequestBracketOrder {
            template_id: 330,
            user_msg: vec![id.clone()],
            user_tag: params.user_tag,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            symbol: Some(params.symbol),
            exchange: Some(params.exchange),
            quantity: Some(params.quantity),
            price: Some(params.price),
            transaction_type: Some(
                rti::request_bracket_order::TransactionType::from(params.transaction_type).into(),
            ),
            duration: Some(rti::request_bracket_order::Duration::from(params.duration).into()),
            price_type: Some(rti::request_bracket_order::PriceType::from(params.price_type).into()),
            trade_route: Some(trade_route.into()),
            bracket_type: Some(
                rti::request_bracket_order::BracketType::from(params.bracket_type).into(),
            ),
            ..RequestBracketOrder::default()
        };

        if let Some(ticks) = params.target_ticks {
            req.target_ticks.push(ticks);
            req.target_quantity.push(params.quantity);
        }

        if let Some(ticks) = params.stop_ticks {
            req.stop_ticks.push(ticks);
            req.stop_quantity.push(params.quantity);
        }

        self.request_to_buf(req, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_oco_order(
        &mut self,
        account: &AccountInfo,
        params: OcoOrderParams,
        trade_route: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestOcoOrder {
            template_id: 328,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),

            symbol: vec![params.leg1.symbol, params.leg2.symbol],
            exchange: vec![params.leg1.exchange, params.leg2.exchange],
            quantity: vec![params.leg1.quantity, params.leg2.quantity],
            price: vec![params.leg1.price, params.leg2.price],
            transaction_type: vec![
                rti::request_oco_order::TransactionType::from(params.leg1.transaction_type).into(),
                rti::request_oco_order::TransactionType::from(params.leg2.transaction_type).into(),
            ], // Adjusted
            price_type: vec![
                rti::request_oco_order::PriceType::from(params.leg1.price_type).into(),
                rti::request_oco_order::PriceType::from(params.leg2.price_type).into(),
            ], // Adjusted
            duration: vec![
                rti::request_oco_order::Duration::from(params.duration).into(),
                rti::request_oco_order::Duration::from(params.duration).into(),
            ], // Adjusted
            trade_route: vec![trade_route.into(), trade_route.into()],
            manual_or_auto: vec![
                rti::request_oco_order::OrderPlacement::Auto.into(),
                rti::request_oco_order::OrderPlacement::Auto.into(),
            ],

            user_msg: vec![id.clone()],
            user_tag: params.user_tag.map(|t| vec![t]).unwrap_or_default(),
            ..RequestOcoOrder::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_update_target_bracket_level(
        &mut self,
        account: &AccountInfo,
        basket_id: &str,
        level: i32,
        target_ticks: i32,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestUpdateTargetBracketLevel {
            template_id: 332,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            basket_id: Some(basket_id.into()),
            level: Some(level),
            target_ticks: Some(target_ticks),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_update_stop_bracket_level(
        &mut self,
        account: &AccountInfo,
        basket_id: &str,
        level: i32,
        stop_ticks: i32,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestUpdateStopBracketLevel {
            template_id: 334,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            basket_id: Some(basket_id.into()),
            level: Some(level),
            stop_ticks: Some(stop_ticks),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_subscribe_to_bracket_updates(
        &mut self,
        account: &AccountInfo,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestSubscribeToBracketUpdates {
            template_id: 336,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_show_brackets(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestShowBrackets {
            template_id: 338,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_show_bracket_stops(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestShowBracketStops {
            template_id: 340,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_list_exchange_permissions(&mut self, user: Option<&str>) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestListExchangePermissions {
            template_id: 342,
            user_msg: vec![id.clone()],
            user: user.map(|s| s.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_link_orders(
        &mut self,
        account: &AccountInfo,
        basket_ids: Vec<String>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestLinkOrders {
            template_id: 344,
            user_msg: vec![id.clone()],
            fcm_id: vec![account.fcm_id.clone()],
            ib_id: vec![account.ib_id.clone()],
            account_id: vec![account.account_id.clone()],
            basket_id: basket_ids,
        };

        self.request_to_buf(req, id)
    }

    pub fn request_easy_to_borrow_list(
        &mut self,
        request_type: EasyToBorrowListRequestType,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestEasyToBorrowList {
            template_id: 348,
            user_msg: vec![id.clone()],
            request: Some(rti::request_easy_to_borrow_list::Request::from(request_type).into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_modify_order_reference_data(
        &mut self,
        account: &AccountInfo,
        basket_id: &str,
        user_tag: Option<String>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestModifyOrderReferenceData {
            template_id: 3500,
            user_msg: vec![id.clone()],
            user_tag,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            basket_id: Some(basket_id.into()),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_order_session_config(
        &mut self,
        should_defer_request: Option<bool>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestOrderSessionConfig {
            template_id: 3502,
            user_msg: vec![id.clone()],
            should_defer_request,
        };

        self.request_to_buf(req, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_exit_position(
        &mut self,
        account: &AccountInfo,
        window_name: Option<&str>,
        symbol: Option<&str>,
        exchange: Option<&str>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestExitPosition {
            template_id: 3504,
            user_msg: vec![id.clone()],
            window_name: window_name.map(|s| s.into()),
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            symbol: symbol.map(|s| s.into()),
            exchange: exchange.map(|s| s.into()),
            ..RequestExitPosition::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_replay_executions(
        &mut self,
        account: &AccountInfo,
        start_index: i32,
        finish_index: i32,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestReplayExecutions {
            template_id: 3506,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            start_index: Some(start_index),
            finish_index: Some(finish_index),
        };

        self.request_to_buf(req, id)
    }

    pub fn request_account_rms_updates(
        &mut self,
        account: &AccountInfo,
        subscribe: bool,
        update_bits: Vec<AccountRmsUpdateBits>,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let mut bits = 0;
        for bit in update_bits {
            bits |= i32::from(bit);
        }

        let req = RequestAccountRmsUpdates {
            template_id: 3508,
            user_msg: vec![id.clone()],
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            request: Some(if subscribe {
                "subscribe".to_string()
            } else {
                "unsubscribe".to_string()
            }),
            update_bits: Some(bits),
        };

        self.request_to_buf(req, id)
    }

    // --- PnL & Position ---

    pub fn request_pnl_position_updates(
        &mut self,
        account: &AccountInfo,
        subscribe: bool,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestPnLPositionUpdates {
            template_id: 400,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            request: Some(if subscribe {
                request_pn_l_position_updates::Request::Subscribe.into()
            } else {
                request_pn_l_position_updates::Request::Unsubscribe.into()
            }),
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_pnl_position_snapshot(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestPnLPositionSnapshot {
            template_id: 402,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    // --- Repository ---

    pub fn request_list_unaccepted_agreements(&mut self) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestListUnacceptedAgreements {
            template_id: 500,
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_list_accepted_agreements(&mut self) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestListAcceptedAgreements {
            template_id: 502,
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_show_agreement(&mut self, agreement_id: Option<&str>) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestShowAgreement {
            template_id: 506,
            user_msg: vec![id.clone()],
            agreement_id: agreement_id.map(|s| s.into()),
        };

        self.request_to_buf(req, id)
    }
}
