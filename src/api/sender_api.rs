use prost::Message;

use crate::{
    connection_info::AccountInfo,
    rti::{
        *,
        request_account_list::UserType,
        request_login::SysInfraType,
        request_market_data_update::{Request, UpdateBits},
        request_tick_bar_replay::{BarSubType, BarType, Direction, TimeOrder},
        request_time_bar_replay,
        request_search_symbols::InstrumentType, // Needed
        request_search_symbols::Pattern,        // Needed
    },
};


pub const TRADE_ROUTE_LIVE: &str = "globex";
pub const TRADE_ROUTE_DEMO: &str = "simulator";
pub const USER_TYPE: i32 = 3;

#[derive(Debug, Clone)]
pub struct RithmicSenderApi {
    message_id_counter: u64,
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

    pub fn request_rithmic_system_gateway_info(&mut self, system_name: String) -> (Vec<u8>, String) {
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
            app_name: Some("rithmic-rs".to_string()),
            app_version: Some("0.4.2".into()),
            system_name: Some(system_name.to_string()),
            infra_type: Some(infra_type.into()),
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
        fields: Vec<UpdateBits>,
        request_type: Request,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let mut req = RequestMarketDataUpdate {
            template_id: 100,
            user_msg: vec![id.clone()],
            ..RequestMarketDataUpdate::default()
        };

        let mut bits = 0;
        for field in fields {
            bits |= field as u32;
        }

        req.symbol = Some(symbol.into());
        req.exchange = Some(exchange.into());
        req.request = Some(request_type.into());
        req.update_bits = Some(bits);

        self.request_to_buf(req, id)
    }

    pub fn request_search_symbols(
        &mut self,
        search_text: &str,
        instrument_type: Option<InstrumentType>,
        pattern: Option<Pattern>
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestSearchSymbols {
            template_id: 109,
            user_msg: vec![id.clone()],
            search_text: Some(search_text.into()),
            instrument_type: instrument_type.map(|it| it.into()),
            pattern: pattern.map(|p| p.into()),
            ..RequestSearchSymbols::default()
        };

        self.request_to_buf(req, id)
    }

    // --- History ---

    pub fn request_tick_bar_replay(
        &mut self,
        exchange: String,
        symbol: String,
        start_index_sec: i32,
        finish_index_sec: i32,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestTickBarReplay {
            template_id: 206,
            exchange: Some(exchange),
            symbol: Some(symbol),
            bar_type: Some(BarType::TickBar.into()),
            bar_sub_type: Some(BarSubType::Regular.into()),
            bar_type_specifier: Some("1".to_string()),
            start_index: Some(start_index_sec),
            finish_index: Some(finish_index_sec),
            direction: Some(Direction::First.into()),
            time_order: Some(TimeOrder::Forwards.into()),
            user_msg: vec![id.clone()],
            ..Default::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_time_bar_replay(
        &mut self,
        exchange: String,
        symbol: String,
        bar_type: request_time_bar_replay::BarType,
        bar_type_period: i32,
        start_index_sec: i32,
        finish_index_sec: i32,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestTimeBarReplay {
            template_id: 202,
            exchange: Some(exchange),
            symbol: Some(symbol),
            bar_type: Some(bar_type.into()),
            bar_type_period: Some(bar_type_period),
            start_index: Some(start_index_sec),
            finish_index: Some(finish_index_sec),
            direction: Some(request_time_bar_replay::Direction::First.into()),
            time_order: Some(request_time_bar_replay::TimeOrder::Forwards.into()),
            user_msg: vec![id.clone()],
            ..Default::default()
        };

        self.request_to_buf(req, id)
    }

    // --- Order Management ---

    pub fn request_account_list(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestAccountList {
            template_id: 302,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            user_type: Some(UserType::Trader.into()),
            user_msg: vec![id.clone()],
        };

        self.request_to_buf(req, id)
    }

    pub fn request_subscribe_for_order_updates(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
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
        let req = RequestTradeRoutes {
            template_id: 310,
            user_msg: vec![id.clone()],
            subscribe_for_updates: Some(true),
            ..RequestTradeRoutes::default()
        };
        self.request_to_buf(req, id)
    }

    pub fn request_new_order(
        &mut self,
        account: &AccountInfo,
        exchange: &str,
        symbol: &str,
        qty: i32,
        price: f64,
        action: request_new_order::TransactionType,
        ordertype: request_new_order::PriceType,
        duration: request_new_order::Duration, // Added param
        localid: &str,
        trade_route: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestNewOrder {
            template_id: 312,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            trade_route: Some(trade_route.into()),
            exchange: Some(exchange.into()),
            symbol: Some(symbol.into()),
            quantity: Some(qty),
            price: Some(price),
            transaction_type: Some(action.into()),
            price_type: Some(ordertype.into()),
            duration: Some(duration.into()), // Use param
            manual_or_auto: Some(2),
            user_msg: vec![id.clone()],
            user_tag: Some(localid.into()),
            ..RequestNewOrder::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_modify_order(
        &mut self,
        account: &AccountInfo,
        basket_id: &str, // Order ID
        exchange: &str,
        symbol: &str,
        qty: i32,
        price: f64,
        ordertype: request_new_order::PriceType, 
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestModifyOrder {
            template_id: 314,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            basket_id: Some(basket_id.into()),
            exchange: Some(exchange.into()),
            symbol: Some(symbol.into()),
            quantity: Some(qty),
            price: Some(price),
            price_type: Some(ordertype.into()),
            manual_or_auto: Some(2),
            user_msg: vec![id.clone()],
            ..RequestModifyOrder::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_cancel_order(
        &mut self,
        account: &AccountInfo,
        basket_id: &str,
    ) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestCancelOrder {
            template_id: 316,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            basket_id: Some(basket_id.into()),
            manual_or_auto: Some(2),
            user_msg: vec![id.clone()],
            ..RequestCancelOrder::default()
        };

        self.request_to_buf(req, id)
    }

    pub fn request_cancel_all_orders(&mut self, account: &AccountInfo) -> (Vec<u8>, String) {
        let id = self.get_next_message_id();

        let req = RequestCancelAllOrders {
            template_id: 346,
            fcm_id: Some(account.fcm_id.clone()),
            ib_id: Some(account.ib_id.clone()),
            account_id: Some(account.account_id.clone()),
            manual_or_auto: Some(2),
            user_msg: vec![id.clone()],
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
}