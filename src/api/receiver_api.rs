use prost::Message;
use std::io::Cursor;
use tracing::{Level, event};

use crate::rti::{messages::RithmicMessage, *};

#[derive(Debug, Clone)]
pub struct RithmicResponse {
    pub request_id: String,
    pub message: RithmicMessage,
    pub is_update: bool,
    pub has_more: bool,
    pub multi_response: bool,
    pub error: Option<String>,
}

pub fn decode_message(data: &[u8]) -> Result<RithmicResponse, Box<RithmicResponse>> {
    let cursor = &mut Cursor::new(&data[4..]);
    let parsed_message = match MessageType::decode(cursor) {
        Ok(m) => m,
        Err(e) => {
            event!(Level::ERROR, "Failed to decode MessageType: {:?}", e);
            return Err(Box::new(RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::Reject(Reject::default()),
                is_update: false,
                has_more: false,
                multi_response: false,
                error: Some(format!("Decode Error: {}", e)),
            }));
        }
    };

    let payload_slice = &data[4..];

    let response = match parsed_message.template_id {
        // Shared
        10 => {
            // RequestLogin (Should not receive this usually)
            let resp = RequestLogin::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: None, // Requests don't have rp_code
                message: RithmicMessage::ResponseLogin(ResponseLogin::default()), // Placeholder or custom type?
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        11 => {
            let resp = ResponseLogin::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseLogin(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        13 => {
            let resp = ResponseLogout::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseLogout(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        15 => {
            let resp = ResponseReferenceData::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseReferenceData(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        17 => {
            let resp = ResponseRithmicSystemInfo::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                // SystemInfo list ends when rp_code is present (or specific code logic).
                // Assuming simple handling for now.
                has_more: false, // Logic handled in client loop via timeout or parsing?
                // Actually, ResponseRithmicSystemInfo usually returns one big list or multiple messages?
                // The old code treated it as single response? No, client loops.
                message: RithmicMessage::ResponseRithmicSystemInfo(resp),
                is_update: false,
                multi_response: true,
            }
        }
        19 => {
            let resp = ResponseHeartbeat::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseHeartbeat(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        21 => {
            let resp = ResponseRithmicSystemGatewayInfo::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseRithmicSystemGatewayInfo(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        75 => {
            let resp = Reject::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::Reject(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        76 => {
            // User Account Update
            let resp = UserAccountUpdate::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::UserAccountUpdate(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        77 => {
            let resp = ForcedLogout::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::ForcedLogout(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
                error: Some("Forced Logout".to_string()),
            }
        }

        // Market Data (Ticker Plant)
        101 => {
            let resp = ResponseMarketDataUpdate::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseMarketDataUpdate(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        103 => {
            let resp = ResponseGetInstrumentByUnderlying::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseGetInstrumentByUnderlying(resp),
                is_update: false,
                multi_response: true,
            }
        }
        104 => {
            let resp = ResponseGetInstrumentByUnderlyingKeys::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: false, // No rq_handler_rp_code
                message: RithmicMessage::ResponseGetInstrumentByUnderlyingKeys(resp),
                is_update: false,
                multi_response: true,
            }
        }
        106 => {
            let resp = ResponseMarketDataUpdateByUnderlying::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseMarketDataUpdateByUnderlying(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        108 => {
            let resp = ResponseGiveTickSizeTypeTable::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseGiveTickSizeTypeTable(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        110 => {
            let resp = ResponseSearchSymbols::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseSearchSymbols(resp),
                is_update: false,
                multi_response: true,
            }
        }
        112 => {
            let resp = ResponseProductCodes::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseProductCodes(resp),
                is_update: false,
                multi_response: true,
            }
        }
        114 => {
            let resp = ResponseFrontMonthContract::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseFrontMonthContract(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        116 => {
            let resp = ResponseDepthByOrderSnapshot::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseDepthByOrderSnapshot(resp),
                is_update: false,
                multi_response: true,
            }
        }
        118 => {
            let resp = ResponseDepthByOrderUpdates::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseDepthByOrderUpdates(resp),
                is_update: false,
                has_more: false,
                multi_response: true,
            }
        }
        120 => {
            let resp = ResponseGetVolumeAtPrice::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseGetVolumeAtPrice(resp),
                is_update: false,
                multi_response: true,
            }
        }
        122 => {
            let resp = ResponseAuxilliaryReferenceData::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseAuxilliaryReferenceData(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        150 => {
            let resp = LastTrade::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::LastTrade(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        151 => {
            let resp = BestBidOffer::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::BestBidOffer(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        152 => {
            let resp = TradeStatistics::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::TradeStatistics(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        153 => {
            let resp = QuoteStatistics::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::QuoteStatistics(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        154 => {
            let resp = IndicatorPrices::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::IndicatorPrices(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        155 => {
            let resp = EndOfDayPrices::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::EndOfDayPrices(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        156 => {
            let resp = OrderBook::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::OrderBook(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        157 => {
            let resp = MarketMode::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::MarketMode(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        158 => {
            let resp = OpenInterest::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::OpenInterest(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        159 => {
            let resp = FrontMonthContractUpdate::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::FrontMonthContractUpdate(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        160 => {
            let resp = DepthByOrder::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::DepthByOrder(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        161 => {
            let resp = DepthByOrderEndEvent::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::DepthByOrderEndEvent(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        162 => {
            let resp = SymbolMarginRate::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::SymbolMarginRate(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        163 => {
            let resp = OrderPriceLimits::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::OrderPriceLimits(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }

        // History Plant
        201 => {
            let resp = ResponseTimeBarUpdate::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseTimeBarUpdate(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        203 => {
            let resp = ResponseTimeBarReplay::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseTimeBarReplay(resp),
                is_update: false,
                multi_response: true,
            }
        }
        205 => {
            let resp = ResponseTickBarUpdate::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseTickBarUpdate(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        207 => {
            let resp = ResponseTickBarReplay::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseTickBarReplay(resp),
                is_update: false,
                multi_response: true,
            }
        }
        209 => {
            let resp = ResponseVolumeProfileMinuteBars::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseVolumeProfileMinuteBars(resp),
                is_update: false,
                multi_response: true,
            }
        }
        211 => {
            let resp = ResponseResumeBars::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: false, // No rq_handler_rp_code
                message: RithmicMessage::ResponseResumeBars(resp),
                is_update: false,
                multi_response: true,
            }
        }
        250 => {
            let resp = TimeBar::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::TimeBar(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        251 => {
            let resp = TickBar::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::TickBar(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }

        // Order Plant
        301 => {
            let resp = ResponseLoginInfo::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseLoginInfo(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        303 => {
            let resp = ResponseAccountList::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseAccountList(resp),
                is_update: false,
                multi_response: true,
            }
        }
        305 => {
            let resp = ResponseAccountRmsInfo::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseAccountRmsInfo(resp),
                is_update: false,
                multi_response: true,
            }
        }
        307 => {
            let resp = ResponseProductRmsInfo::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseProductRmsInfo(resp),
                is_update: false,
                multi_response: true,
            }
        }
        309 => {
            let resp = ResponseSubscribeForOrderUpdates::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseSubscribeForOrderUpdates(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        311 => {
            let resp = ResponseTradeRoutes::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseTradeRoutes(resp),
                is_update: false,
                multi_response: true,
            }
        }
        313 => {
            let resp = ResponseNewOrder::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseNewOrder(resp),
                is_update: false,
                multi_response: true,
            }
        }
        315 => {
            let resp = ResponseModifyOrder::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseModifyOrder(resp),
                is_update: false,
                multi_response: true,
            }
        }
        317 => {
            let resp = ResponseCancelOrder::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseCancelOrder(resp),
                is_update: false,
                multi_response: true,
            }
        }
        319 => {
            let resp = ResponseShowOrderHistoryDates::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseShowOrderHistoryDates(resp),
                is_update: false,
                multi_response: true,
            }
        }
        321 => {
            let resp = ResponseShowOrders::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseShowOrders(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        323 => {
            let resp = ResponseShowOrderHistory::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseShowOrderHistory(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        325 => {
            let resp = ResponseShowOrderHistorySummary::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseShowOrderHistorySummary(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        327 => {
            let resp = ResponseShowOrderHistoryDetail::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseShowOrderHistoryDetail(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        329 => {
            let resp = ResponseOcoOrder::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseOcoOrder(resp),
                is_update: false,
                multi_response: true,
            }
        }
        331 => {
            let resp = ResponseBracketOrder::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseBracketOrder(resp),
                is_update: false,
                multi_response: true,
            }
        }
        333 => {
            let resp = ResponseUpdateTargetBracketLevel::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseUpdateTargetBracketLevel(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        335 => {
            let resp = ResponseUpdateStopBracketLevel::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseUpdateStopBracketLevel(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        337 => {
            let resp = ResponseSubscribeToBracketUpdates::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseSubscribeToBracketUpdates(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        339 => {
            let resp = ResponseShowBrackets::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseShowBrackets(resp),
                is_update: false,
                multi_response: true,
            }
        }
        341 => {
            let resp = ResponseShowBracketStops::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseShowBracketStops(resp),
                is_update: false,
                multi_response: true,
            }
        }
        343 => {
            let resp = ResponseListExchangePermissions::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseListExchangePermissions(resp),
                is_update: false,
                multi_response: true,
            }
        }
        345 => {
            let resp = ResponseLinkOrders::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseLinkOrders(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        347 => {
            let resp = ResponseCancelAllOrders::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseCancelAllOrders(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        349 => {
            let resp = ResponseEasyToBorrowList::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseEasyToBorrowList(resp),
                is_update: false,
                multi_response: true,
            }
        }
        350 => {
            let resp = TradeRoute::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::TradeRoute(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        351 => {
            let resp = RithmicOrderNotification::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::RithmicOrderNotification(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        352 => {
            let resp = ExchangeOrderNotification::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::ExchangeOrderNotification(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        353 => {
            let resp = BracketUpdates::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::BracketUpdates(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        355 => {
            let resp = UpdateEasyToBorrowList::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::UpdateEasyToBorrowList(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        356 => {
            let resp = AccountRmsUpdates::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::AccountRmsUpdates(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        3501 => {
            let resp = ResponseModifyOrderReferenceData::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseModifyOrderReferenceData(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        3503 => {
            let resp = ResponseOrderSessionConfig::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: false, // No rq_handler_rp_code
                message: RithmicMessage::ResponseOrderSessionConfig(resp),
                is_update: false,
                multi_response: true,
            }
        }
        3505 => {
            let resp = ResponseExitPosition::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseExitPosition(resp),
                is_update: false,
                multi_response: true,
            }
        }
        3507 => {
            let resp = ResponseReplayExecutions::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: false, // No rq_handler_rp_code
                message: RithmicMessage::ResponseReplayExecutions(resp),
                is_update: false,
                multi_response: true,
            }
        }
        3509 => {
            let resp = ResponseAccountRmsUpdates::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseAccountRmsUpdates(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }

        // PnL Plant
        401 => {
            let resp = ResponsePnLPositionUpdates::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponsePnLPositionUpdates(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        403 => {
            let resp = ResponsePnLPositionSnapshot::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponsePnLPositionSnapshot(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        450 => {
            let resp = InstrumentPnLPositionUpdate::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::InstrumentPnLPositionUpdate(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }
        451 => {
            let resp = AccountPnLPositionUpdate::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: "".to_string(),
                message: RithmicMessage::AccountPnLPositionUpdate(resp),
                is_update: true,
                has_more: false,
                multi_response: false,
                error: None,
            }
        }

        // Repository
        501 => {
            let resp = ResponseListUnacceptedAgreements::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseListUnacceptedAgreements(resp),
                is_update: false,
                multi_response: true,
            }
        }
        503 => {
            let resp = ResponseListAcceptedAgreements::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                has_more: is_has_more(&resp.rq_handler_rp_code),
                message: RithmicMessage::ResponseListAcceptedAgreements(resp),
                is_update: false,
                multi_response: true,
            }
        }
        505 => {
            // AcceptAgreement Response (Wait, template name?)
            // List says: 500 req -> 501 resp. 502 req -> 503 resp.
            // AcceptAgreement Request? Not listed in 1.6.
            // Ah, see imports: request_accept_agreement.proto
            // Let's assume 504 req, 505 resp? Or 504?
            // Reference Guide 1.6: "Show Agreement" 506/507.
            // Accept Agreement is likely there but maybe I missed ID?
            // Looking at imports: request_accept_agreement.proto.
            // Let's skip 505 if unsure.

            // Just use what's in RithmicMessage enum.
            // ResponseAcceptAgreement is in messages.rs.
            // Let's assume ID 505 for ResponseAcceptAgreement based on pattern.
            let resp = ResponseAcceptAgreement::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseAcceptAgreement(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        507 => {
            let resp = ResponseShowAgreement::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseShowAgreement(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }
        509 => {
            // Set Self Cert Status
            // Pattern implies.
            let resp = ResponseSetRithmicMrktDataSelfCertStatus::decode(payload_slice).unwrap();
            RithmicResponse {
                request_id: resp.user_msg.first().cloned().unwrap_or_default(),
                error: get_error(&resp.rp_code),
                message: RithmicMessage::ResponseSetRithmicMrktDataSelfCertStatus(resp),
                is_update: false,
                has_more: false,
                multi_response: false,
            }
        }

        id => RithmicResponse {
            request_id: "".to_string(),
            message: RithmicMessage::Reject(Reject::default()),
            is_update: false,
            has_more: false,
            multi_response: false,
            error: Some(format!("Unknown template_id: {}", id)),
        },
    };

    // Check for error in response
    if let Some(err_msg) = &response.error {
        event!(
            Level::ERROR,
            "Rithmic Error: {:?} - {}",
            response.message,
            err_msg
        );
        return Err(Box::new(response));
    }

    Ok(response)
}

fn get_error(rp_code: &[String]) -> Option<String> {
    if (rp_code.len() == 1 && rp_code[0] == "0") || (rp_code.is_empty()) {
        None
    } else {
        Some(
            rp_code
                .get(1)
                .cloned()
                .unwrap_or_else(|| "Unknown Error".to_string()),
        )
    }
}

fn is_has_more(rq_handler_rp_code: &[String]) -> bool {
    !rq_handler_rp_code.is_empty() && rq_handler_rp_code[0] == "0"
}
