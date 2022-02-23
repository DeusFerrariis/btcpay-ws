use super::invoice::{InvoiceCommands, InvoiceError};
use async_trait::async_trait;
use redis::Commands;

#[derive(Clone)]
pub struct RedisDb {
    host: String,
    port: String,
    password: String,
}

impl RedisDb {
    pub fn get_connection(&self) -> Result<redis::Connection, ()> {
        match redis::Client::open(format!(
            "redis://:{}@{}:{}",
            &self.password, &self.host, &self.password
        )) {
            Ok(client) => match client.get_connection() {
                Ok(connection) => Ok(connection),
                Err(_) => return Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub fn new(host: String, port: String, password: String) -> RedisDb {
        RedisDb {
            host,
            port,
            password,
        }
    }
}

#[async_trait]
impl InvoiceCommands for RedisDb {
    async fn get_invoice_status(&self, invoice_id: String) -> Result<String, InvoiceError> {
        match self.get_connection() {
            Ok(mut connection) => match connection.get::<String, String>(invoice_id) {
                Ok(invoice_status) => Ok(invoice_status),
                Err(_) => Err(InvoiceError::DoesNotExist),
            },
            Err(e) => Err(InvoiceError::DbAuthentication),
        }
    }

    async fn set_invoice_status(
        &mut self,
        invoice_id: String,
        status: String,
    ) -> Result<(), InvoiceError> {
        match self.get_connection() {
            Ok(mut connection) => {
                match connection.set::<String, String, String>(invoice_id, status) {
                    Ok(invoice_status) => Ok(()),
                    Err(_) => Err(InvoiceError::DoesNotExist),
                }
            }
            Err(e) => Err(InvoiceError::DbConnection),
        }
    }
}
