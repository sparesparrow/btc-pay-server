
use std::collections::HashMap;
use std::sync::Mutex;
use crate::models::Invoice;
use crate::database::Database;
use crate::blockchain::BlockchainClient;
use crate::trezor::TrezorClient;

pub struct AppState {
    pub invoices: Mutex<HashMap<String, Invoice>>,
    pub db: Database,
    pub blockchain_client: BlockchainClient,
    pub trezor_client: TrezorClient,
}

impl AppState {
    pub fn new(db_path: &str) -> Self {
        let db = Database::new(db_path).expect("Failed to initialize database");
        let blockchain_client = BlockchainClient::new("https://blockstream.info/testnet/api".to_string());
        let trezor_client = TrezorClient::new();
        
        Self {
            invoices: Mutex::new(HashMap::new()),
            db,
            blockchain_client,
            trezor_client,
        }
    }
}
