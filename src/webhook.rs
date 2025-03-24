
use reqwest::Client;
use serde_json::json;
use log::{info, error};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex::encode;

use crate::models::{Invoice, WebhookConfig, WebhookEvent};

pub struct WebhookManager {
    client: Client,
}

type HmacSha256 = Hmac<Sha256>;

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    // Calculate signature for webhook payload
    fn calculate_signature(&self, payload: &str, secret: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        encode(result.into_bytes())
    }

    // Send webhook notification about invoice status change
    pub async fn notify_payment_status(&self, invoice: &Invoice, webhook: &WebhookConfig) -> Result<(), String> {
        info!("Sending webhook notification for invoice {}", invoice.id);
        
        let event = WebhookEvent {
            event_type: "invoice.updated".to_string(),
            invoice_id: invoice.id.clone(),
            timestamp: Utc::now(),
            data: json!({
                "status": format!("{:?}", invoice.status),
                "address": invoice.address,
                "amount": invoice.amount,
                "description": invoice.description,
            }),
        };
        
        let payload = serde_json::to_string(&event)
            .map_err(|e| format!("JSON serialization error: {}", e))?;
        
        let signature = self.calculate_signature(&payload, &webhook.secret);
        
        // Send the webhook request
        match self.client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-BTC-Pay-Signature", &signature)
            .body(payload)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("Webhook notification sent successfully");
                        Ok(())
                    } else {
                        let status = response.status();
                        error!("Webhook notification failed with status {}", status);
                        Err(format!("Webhook notification failed with status {}", status))
                    }
                },
                Err(e) => {
                    error!("Failed to send webhook notification: {}", e);
                    Err(format!("Failed to send webhook notification: {}", e))
                }
            }
    }
}
