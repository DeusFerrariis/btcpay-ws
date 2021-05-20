use colored::Colorize;
use redis::Commands;
use serde::Deserialize;
use std::{error::Error, fmt};
use tide::convert::json;
extern crate log;
use super::db::InvoiceCommands;

#[derive(Debug, Deserialize)]
struct InvoiceUpdate {
    #[serde(rename="type")]
    status: String,
    #[serde(rename="invoiceId")]
    invoice_id: String,
}

#[derive(Debug)]
enum RedisError {
    Connection,
}

impl Error for RedisError {}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error connection to redis")
    }
}

impl InvoiceUpdate {
    fn sync_update(&self, mut connection: redis::Connection) -> Result<(), RedisError> {
        match connection.set::<String, String, String>(self.invoice_id.clone(), self.status.clone()) {
            Ok(..) => Ok(()),
            Err(..) => Err(RedisError::Connection),
        }
    }
}

pub async fn handle_btcpay<T: InvoiceCommands>(mut req: tide::Request<T>) -> tide::Result<tide::Response> {
    log::trace!("{}", "Handling invoice update");
    let state = req.state().clone();

    let body_str: String = match req.body_string().await {
        Ok(body) => body.clone(),
        Err(..) => {
            log::trace!("request missing body");
            return Ok(tide::Response::builder(400)
                .body(json!({"message": "missing body"}))
                .build());
        }
    };

    // Get invoice update
    let update: InvoiceUpdate = match serde_json::from_str::<InvoiceUpdate>(&body_str) {
        Ok(update) => update,
        Err(e) => {
            log::trace!("request contains invalid/bad body");
            return Ok(tide::Response::builder(400)
                .body(json!({"message": "invalid body"}))
                .build());
        }
    };

    // Fetch btcpay sig
    let btcpay_sig = match req.header() {
        Ok(sig) => sig,
        Err(e) => return Ok(tide::Response::builder(400)
            .body(json!({"detail": "missing BTCPAY-SIG header"}))
            .build())
    };

    // Verify signature
    let sig_string = btcpay_sig.to_string();
    let sig_parts: Vec<&str> = sig_string.split('=').collect();  // Expects sha256=somekey

    // Assert format of signature header is something=something
    if sig_parts.len() != 2 {
        return Ok(tide::Response::builder(401)
            .body(json!({"detail": "invalid BTCPAY-SIG"}))
            .build());
    }

    // Assert hash func is supported
    if sig_parts[0] != "sha256" {
        return Ok(tide::Response::builder(401)
            .body(json!({"detail": "invalid hmac operation"}))
            .build());
    }

    if !state.verify_hmac(body_str, sig_parts[1].as_bytes()) {
        return Ok(tide::Response::builder(401)
            .body(json!({"detail": "invalid hmac"}))
            .build());
    }

    match state.set_invoice_status(update.invoice_id.clone(), update.status.clone()) {
        Err(e) => Ok(tide::Response::builder(500).build()),
        _ => Ok(tide::Response::builder(200)
            .body(json!({"message": "update synced"}))
            .build()),
    }
}
