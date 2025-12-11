use eyre::{Report, Result};
use tokio::sync::mpsc;
use tracing::info;

use crate::api::receiver_api::RithmicResponse;
use crate::types::{
    TickBarReplayBarSubType, TickBarReplayBarType, TickBarReplayDirection, TickBarReplayTimeOrder,
    TickBarUpdateBarSubType, TickBarUpdateBarType, TickBarUpdateRequest, TimeBarReplayBarType,
    TimeBarReplayDirection, TimeBarReplayTimeOrder, TimeBarUpdateBarType, TimeBarUpdateRequest,
}; // Removed RithmicMessage

impl super::RithmicClient {
    #[allow(clippy::too_many_arguments)]
    pub async fn replay_tick_bars(
        &self,
        symbol: &str,
        exchange: &str,
        start_time: i32,
        end_time: i32,
        bar_type: TickBarReplayBarType,
        bar_sub_type: TickBarReplayBarSubType,
        direction: TickBarReplayDirection,
        time_order: TickBarReplayTimeOrder,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Replaying tick bars for {}/{} from {} to {}",
            symbol, exchange, start_time, end_time
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_tick_bar_replay(
            exchange.to_string(),
            symbol.to_string(),
            start_time,
            end_time,
            bar_type,
            bar_sub_type,
            direction,
            time_order,
        );
        drop(sender);
        self.send_stream_command_to_plant(&self.history_tx, "History", buf, req_id)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn replay_time_bars(
        &self,
        symbol: &str,
        exchange: &str,
        bar_type: TimeBarReplayBarType,
        period: i32,
        start_time: i32,
        end_time: i32,
        direction: TimeBarReplayDirection,
        time_order: TimeBarReplayTimeOrder,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Replaying time bars for {}/{} from {} to {}",
            symbol, exchange, start_time, end_time
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_time_bar_replay(
            exchange.to_string(),
            symbol.to_string(),
            bar_type,
            period,
            start_time,
            end_time,
            direction,
            time_order,
        );
        drop(sender);
        self.send_stream_command_to_plant(&self.history_tx, "History", buf, req_id)
            .await
    }

    /// Subscribes to time bar updates for a given symbol and exchange.
    pub async fn get_time_bar_updates(
        &self,
        symbol: &str,
        exchange: &str,
        request_type: TimeBarUpdateRequest,
        bar_type: TimeBarUpdateBarType,
        bar_type_period: i32,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Subscribing to time bar updates for {} {} with period {} and request type {:?}",
            symbol, exchange, bar_type_period, request_type
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_time_bar_update(
            symbol,
            exchange,
            request_type,
            bar_type,
            bar_type_period,
        );
        drop(sender);
        self.send_stream_command_to_plant(&self.history_tx, "History", buf, req_id)
            .await
    }

    /// Subscribes to tick bar updates for a given symbol and exchange.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_tick_bar_updates(
        &self,
        symbol: &str,
        exchange: &str,
        request_type: TickBarUpdateRequest,
        bar_type: TickBarUpdateBarType,
        bar_sub_type: TickBarUpdateBarSubType,
        bar_type_specifier: Option<&str>,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Subscribing to tick bar updates for {} {} with bar type {:?} and request type {:?}",
            symbol, exchange, bar_type, request_type
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_tick_bar_update(
            symbol,
            exchange,
            request_type,
            bar_type,
            bar_sub_type,
            bar_type_specifier,
        );
        drop(sender);
        self.send_stream_command_to_plant(&self.history_tx, "History", buf, req_id)
            .await
    }

    /// Retrieves volume profile minute bars for a given symbol and exchange.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_volume_profile_minute_bars(
        &self,
        symbol: &str,
        exchange: &str,
        bar_type_period: i32,
        start_index: i32,
        finish_index: i32,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Requesting volume profile minute bars for {} {} with period {}",
            symbol, exchange, bar_type_period
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_volume_profile_minute_bars(
            symbol,
            exchange,
            bar_type_period,
            start_index,
            finish_index,
        );
        drop(sender);
        self.send_stream_command_to_plant(&self.history_tx, "History", buf, req_id)
            .await
    }

    /// Resumes a paused bar data stream using the provided request key.
    pub async fn resume_bars(&self, request_key: &str) -> Result<(), Report> {
        info!("Resuming bars for request key: {}", request_key);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_resume_bars(request_key);
        drop(sender);
        self.send_single_command_to_plant(&self.history_tx, "History", buf, req_id)
            .await
    }
}