
mod models;
mod handlers;
mod state;
mod blockchain;
mod trezor;

use actix_web::{web, App, HttpServer};
use std::collections::HashMap;
use std::sync::Mutex;
use log::info;

use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting BTC Pay Server...");

    // Initialize application state
    let app_state = web::Data::new(AppState {
        invoices: Mutex::new(HashMap::new()),
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/invoice", web::post().to(handlers::create_invoice))
            .route("/invoice/{id}", web::get().to(handlers::get_invoice))
            .route("/invoice/{id}/check", web::get().to(handlers::check_payment_status))
            .route("/transaction/sign", web::post().to(handlers::sign_transaction))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
