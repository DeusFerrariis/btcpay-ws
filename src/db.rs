use std::{ error::Error, fmt };
use redis::Commands;

#[derive(Debug)]
enum InvoiceError {
    Connection,
    Authentication,
    DoesNotExist,
}

impl Error for InvoiceError {}

impl fmt::Display for InvoiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvoiceError::Connection => write!(f, "There was an error connecting"),
            InvoiceError::Authentication => write!(f, "There was an error authenticating"),
            InvoiceError::DoesNotExist => write!(f, "The specified invoice does not exist"),
        }
    }
}

pub trait InvoiceCommands {
    fn get_invoice_status(&self, invoice_id: String) -> Result<String, InvoiceError>;
    fn set_invoice_status(&self, invoice_id: String, status: String) -> Result<(), InvoiceError>;
}

#[derive(Clone)]
pub struct RedisDb {
    host: String,
    port: String,
    password: String
}

impl RedisDb {
    pub fn get_connection(&self) -> Result<redis::Connection, ()> {
        match redis::Client::open(
            format!("redis://:{}@{}:{}", &self.password, &self.host, &self.password)
        ) {
            Ok(client) => {
                match client.get_connection() {
                    Ok(connection) => Ok(connection),
                    Err(_) => return Err(())
                }
            },
            Err(e) => Err(())
        }
    }

    pub fn new(host: String, port: String, password: String) -> RedisDb {
        RedisDb { host, port, password }
    }
}

impl InvoiceCommands for RedisDb {
    fn get_invoice_status(&self, invoice_id: String) -> Result<String, InvoiceError> {
        match self.get_connection() {
            Ok(connection) => {
                match connection.get::<String, String>(invoice_id) {
                    Ok(invoice_status) => Ok(invoice_status),
                    Err(_) => Err(InvoiceError::DoesNotExist),
                }
            }
            Err(e) => Err(InvoiceError::Connection),
        }
    }

    fn set_invoice_status(&self, invoice_id: String, status: String) -> Result<(), InvoiceError> {
        match self.get_connection() {
            Ok(connection) => {
                match connection.set::<String, String, String>(invoice_id, status) {
                    Ok(invoice_status) => Ok(()),
                    Err(_) => Err(InvoiceError::DoesNotExist),
                }
            }
            Err(e) => Err(InvoiceError::Connection),
        }
    }
}
