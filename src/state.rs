
use std::collections::HashMap;
use std::sync::Mutex;
use crate::models::Invoice;

pub struct AppState {
    pub invoices: Mutex<HashMap<String, Invoice>>,
}
