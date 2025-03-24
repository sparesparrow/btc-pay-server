
use rusqlite::{params, Connection, Result, Error as SqliteError};
use log::{info, error};
use std::path::Path;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::{Invoice, InvoiceStatus};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self, SqliteError> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<(), SqliteError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS invoices (
                id TEXT PRIMARY KEY,
                address TEXT NOT NULL,
                amount INTEGER NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            )",
            [],
        )?;
        
        info!("Database initialized successfully");
        Ok(())
    }

    pub fn save_invoice(&self, invoice: &Invoice) -> Result<(), SqliteError> {
        self.conn.execute(
            "INSERT INTO invoices (
                id, address, amount, description, status, created_at, expires_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                invoice.id,
                invoice.address,
                invoice.amount,
                invoice.description,
                format!("{:?}", invoice.status),
                invoice.created_at.to_rfc3339(),
                invoice.expires_at.to_rfc3339()
            ],
        )?;
        
        info!("Invoice {} saved to database", invoice.id);
        Ok(())
    }

    pub fn get_invoice(&self, id: &str) -> Result<Option<Invoice>, SqliteError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, address, amount, description, status, created_at, expires_at 
             FROM invoices WHERE id = ?"
        )?;
        
        let invoice_result = stmt.query_row(params![id], |row| {
            let status_str: String = row.get(4)?;
            let created_at_str: String = row.get(5)?;
            let expires_at_str: String = row.get(6)?;
            
            let status = match status_str.as_str() {
                "Pending" => InvoiceStatus::Pending,
                "Paid" => InvoiceStatus::Paid,
                "Expired" => InvoiceStatus::Expired,
                _ => InvoiceStatus::Pending, // Default
            };
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
                
            let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(6, "expires_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            Ok(Invoice {
                id: row.get(0)?,
                address: row.get(1)?,
                amount: row.get(2)?,
                description: row.get(3)?,
                status,
                created_at,
                expires_at,
            })
        });
        
        match invoice_result {
            Ok(invoice) => Ok(Some(invoice)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn update_invoice_status(&self, id: &str, status: InvoiceStatus) -> Result<(), SqliteError> {
        self.conn.execute(
            "UPDATE invoices SET status = ? WHERE id = ?",
            params![format!("{:?}", status), id],
        )?;
        
        info!("Invoice {} status updated to {:?}", id, status);
        Ok(())
    }

    pub fn get_pending_invoices(&self) -> Result<Vec<Invoice>, SqliteError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, address, amount, description, status, created_at, expires_at 
             FROM invoices WHERE status = 'Pending'"
        )?;
        
        let invoice_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(5)?;
            let expires_at_str: String = row.get(6)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
                
            let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(6, "expires_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            Ok(Invoice {
                id: row.get(0)?,
                address: row.get(1)?,
                amount: row.get(2)?,
                description: row.get(3)?,
                status: InvoiceStatus::Pending,
                created_at,
                expires_at,
            })
        })?;
        
        let mut invoices = Vec::new();
        for invoice in invoice_iter {
            invoices.push(invoice?);
        }
        
        Ok(invoices)
    }
}
