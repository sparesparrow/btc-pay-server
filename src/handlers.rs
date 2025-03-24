
use actix_web::{web, HttpResponse, Responder};
use bitcoin::{Address, Network};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::key::{PublicKey, PrivateKey};
use rand;
use chrono::Utc;
use log::info;
use uuid::Uuid;

use crate::models::{Invoice, InvoiceStatus, PaymentRequest};
use crate::state::AppState;

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
        // Simulate blockchain verification
        let now = Utc::now();
        if now > invoice.expires_at {
            invoice.status = InvoiceStatus::Expired;
            HttpResponse::Ok().json(invoice.clone())
        } else {
            HttpResponse::Ok().json(invoice.clone())
        }
    } else {
        HttpResponse::NotFound().body("Invoice not found")
    }
}

pub async fn sign_transaction() -> impl Responder {
    // Trezor integration placeholder
    HttpResponse::Ok().body("Transaction signed with Trezor and broadcast")
}
