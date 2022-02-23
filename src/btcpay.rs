use async_trait::async_trait;
use redis::Commands;
use serde::Deserialize;
use std::{error::Error, fmt};
use tide::convert::json;
extern crate log;
use super::invoice::{InvoiceCommands, InvoiceError};
use super::state::State;

pub async fn handle_btcpay<T: InvoiceCommands + std::clone::Clone>(
    mut req: tide::Request<State<T>>,
) -> tide::Result<tide::Response> {
    log::trace!("{}", "Handling invoice update");

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
    let btcpay_sig = match req.header("BTCPAY-SIG") {
        Some(sig) => sig,
        None => {
            return Ok(tide::Response::builder(400)
                .body(json!({"detail": "missing BTCPAY-SIG header"}))
                .build())
        }
    };

    // Verify signature
    let sig_string = btcpay_sig[0].to_string();
    let sig_parts: Vec<&str> = sig_string.split('=').collect(); // Expects sha256=somekey
                                                                // Assert format of signature header is something=something
    if sig_parts.len() != 2 {
        return Ok(tide::Response::builder(401)
            .body(json!({"detail": "invalid BTCPAY-SIG"}))
            .build());
    }

    // Assert hash func is supported
    if sig_parts[0] != "sha256" {
        log::debug!("{}", sig_string);
        return Ok(tide::Response::builder(401)
            .body(json!({"detail": "invalid hmac operation"}))
            .build());
    }

    log::trace!("{}", sig_parts[1].to_string());

    if &req.state().verify_hmac(body_str, sig_parts[1].to_string()) == &false {
        return Ok(tide::Response::builder(401)
            .body(json!({"detail": "invalid hmac"}))
            .build());
    }

    {
        let mut db = req.state().db.lock().await;
        match db
            .set_invoice_status(update.invoice_id.clone(), update.status.clone())
            .await
        {
            Err(_) => Ok(tide::Response::builder(500).build()),
            _ => Ok(tide::Response::builder(200)
                .body(json!({"message": "update synced"}))
                .build()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct InvoiceUpdate {
    #[serde(rename = "type")]
    status: String,
    #[serde(rename = "invoiceId")]
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
        match connection.set::<String, String, String>(self.invoice_id.clone(), self.status.clone())
        {
            Ok(..) => Ok(()),
            Err(..) => Err(RedisError::Connection),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::sync::{Arc, Mutex};
    use hmac::{Hmac, Mac, NewMac};
    use std::{collections::HashMap, future::Future};
    use tide_testing::TideTestingExt;

    #[derive(Clone, Debug)]
    struct MockDb {
        invoices: HashMap<String, String>,
    }

    #[async_trait]
    impl InvoiceCommands for MockDb {
        async fn get_invoice_status(&self, invoice_id: String) -> Result<String, InvoiceError> {
            match self.invoices.get(&invoice_id) {
                Some(invoice) => Ok(invoice.clone()),
                None => Err(InvoiceError::DoesNotExist),
            }
        }

        async fn set_invoice_status(
            &mut self,
            invoice_id: String,
            status: String,
        ) -> Result<(), InvoiceError> {
            match self.invoices.insert(invoice_id, status) {
                Some(invoice) => Ok(()),
                None => Ok(()),
            }
        }
    }

    pub type HmacSha256 = Hmac<sha2::Sha256>;

    #[actix_rt::test]
    async fn test_btcpay() {
        let hashmap: HashMap<String, String> = HashMap::new();
        let mut state = State {
            db: Arc::new(Mutex::new(MockDb {
                invoices: hashmap.clone(),
            })),
            hmac: "bob".to_string(),
        };

        let mut app = tide::with_state(state);
        app.at("/btcpay").post(handle_btcpay);

        let update: serde_json::value::Value =
            json!({"invoiceId": "bob", "type": "InvoiceCreated"});

        let mut hmac_sig = HmacSha256::new_varkey("bob".to_string().as_bytes()).unwrap();
        hmac_sig.update(update.to_string().as_bytes());

        let sig_string = hex::encode(hmac_sig.finalize().into_bytes());

        let response: serde_json::value::Value = match app
            .post("/btcpay")
            .body(json!({ "invoiceId": "bob", "type": "InvoiceCreated" }))
            .header("BTCPAY-SIG", format!("sha256={}", sig_string))
            .recv_json()
            .await
        {
            Ok(res) => res,
            Err(_) => {
                assert!(false);
                return;
            }
        };

        assert_eq!(response, json!({"message": "update synced"}));

        {
            // Check Status Matches Change
            let invoices = app.state().db.lock().await;
            assert_eq!(
                invoices
                    .get_invoice_status("bob".to_string())
                    .await
                    .unwrap(),
                "InvoiceCreated"
            );
        }
    }
}
