
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct PaymentRequest {
    amount: u64,
    description: String,
    expiry: u64,
}

#[derive(Debug, Serialize, Deserialize)]
enum InvoiceStatus {
    Pending,
    Paid,
    Expired,
}

#[derive(Debug, Serialize, Deserialize)]
struct Invoice {
    id: String,
    address: String,
    amount: u64,
    description: String,
    status: InvoiceStatus,
    created_at: String,
    expires_at: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Create a payment request
    let payment_request = PaymentRequest {
        amount: 50000, // 50,000 satoshis (0.0005 BTC)
        description: "Test payment".to_string(),
        expiry: 3600, // 1 hour
    };
    
    // Send the request to create an invoice
    let response = client.post("http://localhost:8080/invoice")
        .json(&payment_request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        println!("Error creating invoice: {}", response.status());
        return Ok(());
    }
    
    let invoice: Invoice = response.json().await?;
    println!("Created invoice:");
    println!("ID: {}", invoice.id);
    println!("Bitcoin Address: {}", invoice.address);
    println!("Amount: {} satoshis", invoice.amount);
    println!("Description: {}", invoice.description);
    println!("Status: {:?}", invoice.status);
    println!("Created at: {}", invoice.created_at);
    println!("Expires at: {}", invoice.expires_at);
    
    // Check payment status
    let status_url = format!("http://localhost:8080/invoice/{}/check", invoice.id);
    println!("\nChecking payment status...");
    
    let status_response = client.get(&status_url)
        .send()
        .await?;
    
    if !status_response.status().is_success() {
        println!("Error checking payment status: {}", status_response.status());
        return Ok(());
    }
    
    let payment_status: Invoice = status_response.json().await?;
    println!("Payment Status: {:?}", payment_status.status);
    
    Ok(())
}
