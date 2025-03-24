
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentRequest {
    pub amount: u64,        // Amount in satoshis
    pub description: String, // Payment description
    pub expiry: u64,        // Expiry in seconds
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InvoiceStatus {
    Pending,
    Paid,
    Expired,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Invoice {
    pub id: String,
    pub address: String,
    pub amount: u64,
    pub description: String,
    pub status: InvoiceStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
