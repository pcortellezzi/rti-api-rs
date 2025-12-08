use rithmic_rs::{
    api::decoder::decode_message,
    rti::{
        ResponseLogin, ResponseHeartbeat,
        ResponseSearchSymbols, ResponseShowOrders, 
        LastTrade, TickBar, RithmicOrderNotification,
        MessageType
    },
    RithmicMessage,
};
use prost::Message;

fn encode_msg<T: Message + Default>(template_id: i32, msg: T) -> Vec<u8> {
    let mut buf = Vec::new();
    let msg_type = MessageType { template_id };
    msg_type.encode(&mut buf).unwrap();
    msg.encode(&mut buf).unwrap();
    
    let len = buf.len() as u32;
    let mut final_buf = len.to_be_bytes().to_vec();
    final_buf.extend(buf);
    final_buf
}

#[test]
fn test_decode_response_login() {
    let resp = ResponseLogin {
        template_id: 11,
        user_msg: vec!["req1".to_string()],
        rp_code: vec!["0".to_string()],
        fcm_id: Some("FCM".into()),
        ..Default::default()
    };
    
    let bytes = encode_msg(11, resp);
    let result = decode_message(&bytes).expect("Decode failed");
    
    assert_eq!(result.request_id, "req1");
    match result.message {
        RithmicMessage::ResponseLogin(r) => assert_eq!(r.fcm_id.unwrap(), "FCM"),
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_decode_response_heartbeat() {
    let resp = ResponseHeartbeat {
        template_id: 19,
        rp_code: vec!["0".to_string()],
        ..Default::default()
    };
    
    let bytes = encode_msg(19, resp);
    let result = decode_message(&bytes).expect("Decode failed");
    
    match result.message {
        RithmicMessage::ResponseHeartbeat(_) => {},
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_decode_last_trade() {
    let msg = LastTrade {
        template_id: 150,
        symbol: Some("ESZ5".into()),
        trade_price: Some(5000.0),
        ..Default::default()
    };
    
    let bytes = encode_msg(150, msg);
    let result = decode_message(&bytes).expect("Decode failed");
    
    match result.message {
        RithmicMessage::LastTrade(t) => {
            assert_eq!(t.symbol.unwrap(), "ESZ5");
            assert_eq!(t.trade_price.unwrap(), 5000.0);
        },
        _ => panic!("Wrong message type"),
    }
    assert!(result.is_update);
}

#[test]
fn test_decode_response_search_symbols() {
    let resp = ResponseSearchSymbols {
        template_id: 110,
        user_msg: vec!["req2".into()],
        symbol_name: Some("ESZ5".into()),
        rp_code: vec!["0".into()],
        ..Default::default()
    };
    
    let bytes = encode_msg(110, resp);
    let result = decode_message(&bytes).expect("Decode failed");
    
    assert_eq!(result.request_id, "req2");
    match result.message {
        RithmicMessage::ResponseSearchSymbols(r) => assert_eq!(r.symbol_name.unwrap(), "ESZ5"),
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_decode_error_response() {
    let resp = ResponseLogin {
        template_id: 11,
        user_msg: vec!["req3".into()],
        rp_code: vec!["1".into(), "Invalid Password".into()],
        ..Default::default()
    };
    
    let bytes = encode_msg(11, resp);
    
    let result = decode_message(&bytes);
    assert!(result.is_err());
    
    let err_resp = result.unwrap_err();
    assert_eq!(err_resp.error, Some("Invalid Password".into()));
}

#[test]
fn test_decode_show_orders_ack() {
    let resp = ResponseShowOrders {
        template_id: 321,
        user_msg: vec!["req4".into()],
        rp_code: vec!["0".into()],
        ..Default::default()
    };
    
    let bytes = encode_msg(321, resp);
    let result = decode_message(&bytes).expect("Decode failed");
    
    // ResponseShowOrders is just an ACK/Header
    match result.message {
        RithmicMessage::ResponseShowOrders(r) => assert_eq!(r.template_id, 321),
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_decode_rithmic_order_notification() {
    let msg = RithmicOrderNotification {
        template_id: 351,
        basket_id: Some("ord1".into()),
        symbol: Some("ESZ5".into()),
        status: Some("Open".into()),
        quantity: Some(1),
        price: Some(5000.0),
        ..Default::default()
    };
    
    let bytes = encode_msg(351, msg);
    let result = decode_message(&bytes).expect("Decode failed");
    
    match result.message {
        RithmicMessage::RithmicOrderNotification(n) => {
            assert_eq!(n.basket_id.unwrap(), "ord1");
            assert_eq!(n.symbol.unwrap(), "ESZ5");
            assert_eq!(n.status.unwrap(), "Open");
        },
        _ => panic!("Wrong message type"),
    }
    assert!(result.is_update);
}

#[test]
fn test_decode_tick_bar() {
    let msg = TickBar {
        template_id: 251,
        symbol: Some("ESZ5".into()),
        close_price: Some(5005.0),
        ..Default::default()
    };
    
    let bytes = encode_msg(251, msg);
    let result = decode_message(&bytes).expect("Decode failed");
    
    match result.message {
        RithmicMessage::TickBar(t) => assert_eq!(t.close_price.unwrap(), 5005.0),
        _ => panic!("Wrong message type"),
    }
}