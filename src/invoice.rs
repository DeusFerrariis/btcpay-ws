use std::{ error::Error, fmt, future::Future};
use async_trait::async_trait;

#[derive(Debug)]
pub enum InvoiceError {
    DbConnection,
    DbAuthentication,
    DoesNotExist,
    AlreadyExists,
    BadStatusUpdate,
    BadStatus,
}

impl Error for InvoiceError {}

impl fmt::Display for InvoiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvoiceError({:?})", self)
    }
}

pub enum InvoiceStatus {
    Created,
    PartiallyPayed,
    Payed,
    Expired,
}

impl fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvoiceStatus::Created => write!(f, "InvoiceCreated"),
            InvoiceStatus::Expired => write!(f, "InvoiceExpired"),
            InvoiceStatus::PartiallyPayed => write!(f, "InvoiceRecievedPayment"),
            InvoiceStatus::Payed => write!(f, "InvoicePayed"),
        }
    }
}

impl InvoiceStatus {
    fn from_str(status: &str) -> Result<InvoiceStatus, InvoiceError> {
        match status {
            "InvoiceCreated" => Ok(InvoiceStatus::Created),
            "InvoiceExpired" => Ok(InvoiceStatus::Expired),
            "InvoiceRecievedPayment" => Ok(InvoiceStatus::PartiallyPayed),
            "InvoicePayed" => Ok(InvoiceStatus::Payed),
            _ => Err(InvoiceError::BadStatus),
        }
    }
}

#[async_trait]
pub trait InvoiceCommands {
    async fn get_invoice_status(&self, invoice_id: String) -> Result<String, InvoiceError>;
    async fn set_invoice_status(&mut self, invoice_id: String, status: String) -> Result<(), InvoiceError>;
}
