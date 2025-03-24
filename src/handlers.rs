
use actix_web::{web, HttpResponse, Responder};
use bitcoin::{Address, Network};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::key::{PublicKey, PrivateKey};
use bitcoin::consensus::Decodable;
use rand;
use chrono::Utc;
use log::info;
use uuid::Uuid;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

use crate::models::{Invoice, InvoiceStatus, PaymentRequest};
use crate::state::AppState;
use crate::auth;

#[derive(Deserialize)]
pub struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    token: String,
}

pub async fn create_invoice(
    payment_req: web::Json<PaymentRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let payment_req = payment_req.into_inner();

    // Generate a new Bitcoin address
    let secp = Secp256k1::new();
    let mut rng = rand::thread_rng();
    // Generate a secret key first
    let secret_key = bitcoin::secp256k1::SecretKey::new(&mut rng);
    // Create private key with the secret key and network
    let private_key = PrivateKey::new(secret_key, Network::Testnet);
    let public_key = PublicKey::from_private_key(&secp, &private_key);
    let address = Address::p2pkh(&public_key, Network::Testnet);

    // Create a new invoice
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + chrono::Duration::seconds(payment_req.expiry as i64);

    let invoice = Invoice {
        id: id.clone(),
        address: address.to_string(),
        amount: payment_req.amount,
        description: payment_req.description,
        status: InvoiceStatus::Pending,
        created_at: now,
        expires_at,
    };

    // Store the invoice
    {
        let mut invoices = data.invoices.lock().unwrap();
        invoices.insert(id.clone(), invoice.clone());
    }

    info!("Created new invoice: {}", id);
    HttpResponse::Ok().json(invoice)
}

pub async fn get_invoice(
    id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let invoices = data.invoices.lock().unwrap();
    match invoices.get(&id.into_inner()) {
        Some(invoice) => HttpResponse::Ok().json(invoice),
        None => HttpResponse::NotFound().body("Invoice not found"),
    }
}

pub async fn check_payment_status(
    id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let invoice_id = id.into_inner();
    let mut invoices = data.invoices.lock().unwrap();
    
    if let Some(invoice) = invoices.get_mut(&invoice_id) {
        // Create blockchain client and check for transactions
        let blockchain_client = crate::blockchain::BlockchainClient::new("https://mempool.space/api".to_string());
        
        // Parse the invoice address
        match Address::from_str(&invoice.address) {
            Ok(unchecked_address) => {
                // Convert to checked address with the appropriate network
                let address = unchecked_address.require_network(Network::Testnet).unwrap();
                
                // Check for transactions to this address
                match blockchain_client.check_address_transactions(&address).await {
                    Ok(has_transactions) => {
                        if has_transactions {
                            invoice.status = InvoiceStatus::Paid;
                        } else {
                            // Check for expiry
                            let now = Utc::now();
                            if now > invoice.expires_at {
                                invoice.status = InvoiceStatus::Expired;
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Error checking transactions: {}", e);
                    }
                }
            },
            Err(e) => {
                log::error!("Error parsing address: {}", e);
            }
        }

pub async fn generate_token(
    req: web::Json<AuthRequest>,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // In a real application, validate credentials against a database
    // This is a simplified example for demonstration
    if req.username == "admin" && req.password == "secure_password" {
        match auth::generate_token(&req.username, jwt_secret.get_ref().as_bytes()) {
            Ok(token) => HttpResponse::Ok().json(TokenResponse { token }),
            Err(_) => HttpResponse::InternalServerError().body("Could not generate token"),
        }
    } else {
        HttpResponse::Unauthorized().body("Invalid credentials")
    }
}

        
        HttpResponse::Ok().json(invoice.clone())
    } else {
        HttpResponse::NotFound().body("Invoice not found")
    }
}

pub async fn sign_transaction(
    tx_data: web::Json<String>,
    _data: web::Data<AppState>,
) -> impl Responder {
    // Parse the transaction hex - extract the string from JSON first
    match bitcoin::Transaction::consensus_decode(&mut hex::decode(&tx_data.into_inner()).unwrap().as_slice()) {
        Ok(unsigned_tx) => {
            // Create Trezor client and sign transaction
            let trezor_client = crate::trezor::TrezorClient::new();
            
            match trezor_client.sign_transaction(&unsigned_tx).await {
                Ok(signed_tx) => {
                    // Create blockchain client to broadcast the transaction
                    let blockchain_client = crate::blockchain::BlockchainClient::new("https://mempool.space/api".to_string());
                    
                    // Serialize the signed transaction to hex
                    let tx_hex = hex::encode(bitcoin::consensus::encode::serialize(&signed_tx));
                    
                    // Broadcast the transaction
                    match blockchain_client.broadcast_transaction(&tx_hex).await {
                        Ok(txid) => HttpResponse::Ok().json(txid),
                        Err(e) => HttpResponse::InternalServerError().body(format!("Error broadcasting: {}", e))
                    }
                },
                Err(e) => HttpResponse::InternalServerError().body(format!("Error signing: {}", e))
            }
        },
        Err(e) => HttpResponse::BadRequest().body(format!("Invalid transaction: {}", e))
    }
}
