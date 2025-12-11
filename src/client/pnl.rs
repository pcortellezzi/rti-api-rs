use eyre::{Report, Result};
use tracing::info;

impl super::RithmicClient {
    /// Subscribes to PnL and position updates.
    pub async fn subscribe_pnl(&self) -> Result<(), Report> {
        info!("Subscribing to PnL and position updates.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_pnl_position_updates(
            &self.account_info,
            true,
        );
        drop(sender);
        self.send_single_command_to_plant(&self.pnl_tx, "PnL", buf, req_id)
            .await
    }

    /// Unsubscribes from PnL and position updates.
    pub async fn unsubscribe_pnl(&self) -> Result<(), Report> {
        info!("Unsubscribing from PnL and position updates.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_pnl_position_updates(
            &self.account_info,
            false,
        );
        drop(sender);
        self.send_single_command_to_plant(&self.pnl_tx, "PnL", buf, req_id)
            .await
    }

    /// Requests a snapshot of PnL and position information.
    pub async fn request_pnl_snapshot(&self) -> Result<(), Report> {
        info!("Requesting PnL and position snapshot.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_pnl_position_snapshot(&self.account_info);
        drop(sender);
        self.send_single_command_to_plant(&self.pnl_tx, "PnL", buf, req_id)
            .await
    }
}