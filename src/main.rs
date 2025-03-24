
mod models;
mod handlers;
mod state;
mod blockchain;
mod trezor;
mod auth;

use actix_web::{web, App, HttpServer, middleware};
use actix_web_httpauth::middleware::HttpAuthentication;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{info, warn};

use state::AppState;

// Simple rate limiter
struct RateLimiter {
    // IP address -> (request_count, last_reset)
    requests: Mutex<HashMap<String, (u32, Instant)>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window: Duration::from_secs(window_seconds),
        }
    }

    fn is_rate_limited(&self, ip: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        let entry = requests.entry(ip.to_string()).or_insert((0, now));
        
        // If window has passed, reset count
        if now.duration_since(entry.1) >= self.window {
            *entry = (1, now);
            false
        } else {
            // Increment counter and check limit
            entry.0 += 1;
            if entry.0 > self.max_requests {
                warn!("Rate limit exceeded for IP: {}", ip);
                true
            } else {
                false
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting BTC Pay Server...");

    // JWT secret - in production, use environment variables or secure storage
    let jwt_secret = web::Data::new("your_jwt_secret_key_here".to_string());

    // Initialize application state with database
    let app_state = web::Data::new(AppState::new("btc_pay_server.db"));
    
    // Create rate limiter - 100 requests per minute
    let rate_limiter = Arc::new(RateLimiter::new(100, 60));

    // Start HTTP server (use rustls for HTTPS in production)
    HttpServer::new(move || {
        // Clone rate limiter for this thread
        let rate_limiter = Arc::clone(&rate_limiter);
        
        // Rate limiting middleware
        let rate_limit = move |req: actix_web::dev::ServiceRequest| {
            let ip = req
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("unknown")
                .to_owned();
                
            if rate_limiter.is_rate_limited(&ip) {
                return futures::future::err(
                    actix_web::error::ErrorTooManyRequests("Rate limit exceeded")
                );
            }
            
            futures::future::ok(req)
        };
        // Public routes don't require authentication
        let public_scope = web::scope("/api/public")
            .route("/invoice", web::post().to(handlers::create_invoice))
            .route("/invoice/{id}", web::get().to(handlers::get_invoice))
            .route("/invoice/{id}/check", web::get().to(handlers::check_payment_status));
            
        // Protected routes require authentication
        let bearer_auth = HttpAuthentication::bearer(auth::validator);
        let private_scope = web::scope("/api/private")
            .wrap(bearer_auth)
            .route("/transaction/sign", web::post().to(handlers::sign_transaction))
            .route("/auth/token", web::post().to(handlers::generate_token));
            
        App::new()
            .app_data(jwt_secret.clone())
            .app_data(app_state.clone())
            .wrap_fn(rate_limit)
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .service(public_scope)
            .service(private_scope)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
