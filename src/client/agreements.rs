use eyre::{Report, Result};
use tokio::sync::mpsc;
use tracing::info;

use crate::api::receiver_api::RithmicResponse;

impl super::RithmicClient {
    /// Retrieves a list of unaccepted agreements.
    pub async fn list_unaccepted_agreements(
        &self,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting list of unaccepted agreements.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_list_unaccepted_agreements();
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves a list of accepted agreements.
    pub async fn list_accepted_agreements(
        &self,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting list of accepted agreements.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_list_accepted_agreements();
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves the text of a specific agreement.
    pub async fn show_agreement(
        &self,
        agreement_id: &str,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting agreement text for ID: {}.", agreement_id);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_show_agreement(Some(agreement_id)); // Assuming sender_api expects Option<&str>
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }
}
