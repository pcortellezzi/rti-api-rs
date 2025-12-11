use prost::Message;
use rti_api_rs::{
    api::sender_api::RithmicSenderApi,
    connection_info::AccountInfo,
    rti::{
        RequestAccountList, RequestBracketOrder, RequestCancelAllOrders,
        RequestCancelOrder, RequestFrontMonthContract, RequestHeartbeat, RequestLogin,
        RequestLogout, RequestMarketDataUpdate, RequestModifyOrder, RequestNewOrder,
        RequestOcoOrder, RequestPnLPositionSnapshot,
        RequestPnLPositionUpdates, RequestReferenceData, RequestSearchSymbols,
        RequestShowOrderHistory, RequestShowOrders, RequestSubscribeForOrderUpdates,
        RequestTickBarReplay, RequestTimeBarReplay, RequestTradeRoutes,
    },
    types::{
        BracketOrderParams, BracketType, InstrumentType, MarketDataField, MarketDataRequestType,
        ModifyOrderParams, OcoLegParams, OcoOrderParams, OrderDuration, OrderParams,
        PnlPositionUpdateRequest, PriceType, SearchPattern, SysInfraType, TickBarReplayBarType,
        TickBarReplayBarSubType, TickBarReplayDirection, TickBarReplayTimeOrder, TimeBarReplayBarType,
        TimeBarReplayDirection, TimeBarReplayTimeOrder, TransactionType,
    },
};

// --- System & Auth ---

#[test]
fn test_request_login() {
    let mut api = RithmicSenderApi::new();
    let (buf, id) = api.request_login("System", SysInfraType::TickerPlant, "user", "pass");

    assert_eq!(id, "1");
    let req = RequestLogin::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 10);
    assert_eq!(req.user.unwrap(), "user");
    assert_eq!(req.password.unwrap(), "pass");
    assert_eq!(req.system_name.unwrap(), "System");
    assert_eq!(req.infra_type.unwrap(), SysInfraType::TickerPlant as i32); // Adjusted
    assert_eq!(req.app_name.unwrap(), "rti-api-rs");
}

#[test]
fn test_request_logout() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_logout();
    let req = RequestLogout::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 12);
}

#[test]
fn test_request_heartbeat() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_heartbeat();
    let req = RequestHeartbeat::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 18);
}

// --- Market Data ---

#[test]
fn test_request_market_data_update() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_market_data_update(
        "ESZ5",
        "CME",
        vec![MarketDataField::LastTrade, MarketDataField::Bbo], // Adjusted
        MarketDataRequestType::Subscribe,                       // Adjusted
    );
    let req = RequestMarketDataUpdate::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 100);
    assert_eq!(req.symbol.unwrap(), "ESZ5");
    assert_eq!(req.exchange.unwrap(), "CME");
    assert_eq!(
        req.request.unwrap(),
        MarketDataRequestType::Subscribe as i32
    ); // Adjusted
    // 1 (LastTrade) | 2 (BBO) = 3
    assert_eq!(
        req.update_bits.unwrap(),
        (MarketDataField::LastTrade as u32) | (MarketDataField::Bbo as u32)
    );
}

#[test]
fn test_request_search_symbols() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_search_symbols(
        "ES",
        "CME",
        "?",
        Some(InstrumentType::Future),
        Some(SearchPattern::Contains),
    );
    let req = RequestSearchSymbols::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 109);
    assert_eq!(req.search_text.unwrap(), "ES");
    assert_eq!(req.instrument_type.unwrap(), InstrumentType::Future as i32); // Adjusted
    assert_eq!(req.pattern.unwrap(), SearchPattern::Contains as i32); // Adjusted
}

#[test]
fn test_request_reference_data() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_reference_data("ESZ5", "CME");
    let req = RequestReferenceData::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 14);
    assert_eq!(req.symbol.unwrap(), "ESZ5");
    assert_eq!(req.exchange.unwrap(), "CME");
}

#[test]
fn test_request_front_month_contract() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_front_month_contract("ES", "CME", true);
    let req = RequestFrontMonthContract::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 113);
    assert_eq!(req.symbol.unwrap(), "ES");
    assert_eq!(req.exchange.unwrap(), "CME");
    assert_eq!(req.need_updates.unwrap(), true);
}

// --- History ---

#[test]
fn test_request_tick_bar_replay() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_tick_bar_replay(
        "CME".into(),
        "ESZ5".into(),
        1000,
        2000,
        TickBarReplayBarType::TickBar,    // Adjusted
        TickBarReplayBarSubType::Regular, // Adjusted
        TickBarReplayDirection::First,    // Adjusted
        TickBarReplayTimeOrder::Forwards, // Adjusted
    );
    let req = RequestTickBarReplay::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 206);
    assert_eq!(req.start_index.unwrap(), 1000);
    assert_eq!(req.finish_index.unwrap(), 2000);
    assert_eq!(req.bar_type.unwrap(), TickBarReplayBarType::TickBar as i32); // Adjusted
    assert_eq!(
        req.bar_sub_type.unwrap(),
        TickBarReplayBarSubType::Regular as i32
    ); // Adjusted
    assert_eq!(req.direction.unwrap(), TickBarReplayDirection::First as i32); // Adjusted
    assert_eq!(
        req.time_order.unwrap(),
        TickBarReplayTimeOrder::Forwards as i32
    ); // Adjusted
}

#[test]
fn test_request_time_bar_replay() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_time_bar_replay(
        "CME".into(),
        "ESZ5".into(),
        TimeBarReplayBarType::MinuteBar,
        1,
        1000,
        2000,
        TimeBarReplayDirection::First,    // Adjusted
        TimeBarReplayTimeOrder::Forwards, // Adjusted
    );
    let req = RequestTimeBarReplay::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 202);
    assert_eq!(
        req.bar_type.unwrap(),
        TimeBarReplayBarType::MinuteBar as i32
    ); // Adjusted
    assert_eq!(req.bar_type_period.unwrap(), 1);
    assert_eq!(req.direction.unwrap(), TimeBarReplayDirection::First as i32); // Adjusted
    assert_eq!(
        req.time_order.unwrap(),
        TimeBarReplayTimeOrder::Forwards as i32
    ); // Adjusted
}

// --- Order Management ---

fn mock_account() -> AccountInfo {
    AccountInfo {
        fcm_id: "FCM".into(),
        ib_id: "IB".into(),
        account_id: "ACC".into(),
        ..Default::default()
    }
}

#[test]
fn test_request_account_list() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_account_list(&mock_account());
    let req = RequestAccountList::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 302);
    assert_eq!(req.fcm_id.unwrap(), "FCM");
}

#[test]
fn test_request_subscribe_order_updates() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_subscribe_for_order_updates(&mock_account());
    let req = RequestSubscribeForOrderUpdates::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 308);
    assert_eq!(req.account_id.unwrap(), "ACC");
}

#[test]
fn test_request_trade_routes() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_trade_routes();
    let req = RequestTradeRoutes::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 310);
    assert_eq!(req.subscribe_for_updates.unwrap(), true);
}

#[test]
fn test_request_new_order() {
    let mut api = RithmicSenderApi::new();

    let params = OrderParams {
        // Adjusted
        exchange: "CME".into(),
        symbol: "ESZ5".into(),
        quantity: 1,
        price: 5000.0,
        transaction_type: TransactionType::Buy,
        price_type: PriceType::Limit,
        duration: OrderDuration::Day,
        user_tag: Some("id123".into()),
        auto: true,
    };

    let (buf, _) = api.request_new_order(&mock_account(), params, "route");
    let req = RequestNewOrder::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 312);
    assert_eq!(req.symbol.unwrap(), "ESZ5");
    assert_eq!(req.quantity.unwrap(), 1);
    assert_eq!(req.price.unwrap(), 5000.0);
    assert_eq!(req.user_tag.unwrap(), "id123");
    assert_eq!(req.trade_route.unwrap(), "route");
}

#[test]
fn test_request_modify_order() {
    let mut api = RithmicSenderApi::new();

    let params = ModifyOrderParams {
        // Adjusted
        basket_id: "basket1".into(),
        exchange: "CME".into(),
        symbol: "ESZ5".into(),
        quantity: 2,
        price: 5001.0,
        price_type: PriceType::Limit,
        auto: true,
    };

    let (buf, _) = api.request_modify_order(&mock_account(), params);
    let req = RequestModifyOrder::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 314);
    assert_eq!(req.basket_id.unwrap(), "basket1");
    assert_eq!(req.quantity.unwrap(), 2);
    assert_eq!(req.price.unwrap(), 5001.0);
}

#[test]
fn test_request_cancel_order() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_cancel_order(&mock_account(), "basket1", true);
    let req = RequestCancelOrder::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 316);
    assert_eq!(req.basket_id.unwrap(), "basket1");
}

#[test]
fn test_request_cancel_all_orders() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_cancel_all_orders(&mock_account());
    let req = RequestCancelAllOrders::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 346);
    assert_eq!(req.account_id.unwrap(), "ACC");
}

#[test]
fn test_request_show_orders() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_show_orders(&mock_account());
    let req = RequestShowOrders::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 320);
}

#[test]
fn test_request_show_order_history() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_show_order_history(&mock_account(), Some("basket1"));
    let req = RequestShowOrderHistory::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 322);
    assert_eq!(req.basket_id.unwrap(), "basket1");
}

#[test]
fn test_request_bracket_order() {
    let mut api = RithmicSenderApi::new();

    let params = BracketOrderParams {
        // Adjusted
        exchange: "CME".into(),
        symbol: "ESZ5".into(),
        quantity: 1,
        price: 5000.0,
        transaction_type: TransactionType::Buy,
        price_type: PriceType::Limit,
        duration: OrderDuration::Day,
        bracket_type: BracketType::Both, // Adjusted: TargetAndStop -> Both
        target_ticks: Some(10),
        stop_ticks: Some(20),
        user_tag: Some("my_tag".into()),
    };

    let (buf, _) = api.request_bracket_order(&mock_account(), params, "globex");

    let req = RequestBracketOrder::decode(&buf[4..]).expect("Failed to decode bracket");
    assert_eq!(req.template_id, 330);
    assert_eq!(req.bracket_type.unwrap(), BracketType::Both as i32); // Adjusted
    assert_eq!(req.target_ticks[0], 10);
    assert_eq!(req.stop_ticks[0], 20);
    assert_eq!(req.target_quantity[0], 1);
    assert_eq!(req.stop_quantity[0], 1);
}

#[test]
fn test_request_oco_order() {
    let mut api = RithmicSenderApi::new();

    let leg1 = OcoLegParams {
        // Adjusted
        symbol: "ESZ5".into(),
        exchange: "CME".into(),
        quantity: 1,
        price: 5000.0,
        transaction_type: TransactionType::Buy,
        price_type: PriceType::Limit,
    };

    let leg2 = OcoLegParams {
        // Adjusted
        symbol: "ESZ5".into(),
        exchange: "CME".into(),
        quantity: 1,
        price: 5010.0,
        transaction_type: TransactionType::Sell,
        price_type: PriceType::Limit,
    };

    let params = OcoOrderParams {
        // Adjusted
        leg1,
        leg2,
        duration: OrderDuration::Day,
        user_tag: Some("oco_tag".into()),
    };

    let (buf, _) = api.request_oco_order(&mock_account(), params, "globex");

    let req = RequestOcoOrder::decode(&buf[4..]).expect("Failed to decode OCO");
    assert_eq!(req.template_id, 328);
    assert_eq!(req.symbol.len(), 2);
    // Check conversion
    assert_eq!(req.transaction_type[0], TransactionType::Buy as i32);
    assert_eq!(req.transaction_type[1], TransactionType::Sell as i32);
}

// --- PnL ---

#[test]
fn test_request_pnl_updates() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_pnl_position_updates(
        &mock_account(),
        true,
    );
    let req = RequestPnLPositionUpdates::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 400);
    assert_eq!(
        req.request.unwrap(),
        PnlPositionUpdateRequest::Subscribe as i32
    ); // Subscribe

    let (buf, _) = api.request_pnl_position_updates(
        &mock_account(),
        false,
    );
    let req = RequestPnLPositionUpdates::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(
        req.request.unwrap(),
        PnlPositionUpdateRequest::Unsubscribe as i32
    ); // Unsubscribe
}

#[test]
fn test_request_pnl_snapshot() {
    let mut api = RithmicSenderApi::new();
    let (buf, _) = api.request_pnl_position_snapshot(&mock_account());
    let req = RequestPnLPositionSnapshot::decode(&buf[4..]).expect("Decode failed");
    assert_eq!(req.template_id, 402);
}