use std::{error::Error, fmt};
use serde::Deserialize;
use tide::convert::json;
use redis::Commands;
use colored::Colorize;
extern crate log;
use super::state::State;

#[derive(Debug, Deserialize)]
struct InvoiceUpdate {
    r#type: String,
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
        match connection.set::<String, String, String>(self.invoice_id.clone(), self.r#type.clone()) {
            Ok(..) => Ok(()),
            Err(..) => Err(RedisError::Connection),
        }
    }
}

pub async fn handle_btcpay(mut req: tide::Request<State>) -> tide::Result<tide::Response> {
    log::trace!("{}", "Handling invoice update");
    let state: State = req.state().clone();

    let mut body_str: String = match req.body_string().await {
        Ok(body) => body,
        Err(..) => {
            log::trace!("request missing body");
            return Ok(tide::Response::builder(400)
                .body(json!({"message": "missing body"}))
                .build());
        },
    };

    log::debug!("{}", body_str);

    let mut update = match req.body_json::<InvoiceUpdate>().await {
        Ok(update) => update,
        Err(..) => {
            log::trace!("request contains invalid/bad body");
            return Ok(tide::Response::builder(400)
                .body(json!({"message": "invalid body"}))
                .build());
        },
    };

    let sig: Result<String, ()> = {
        let sig_header = req.header("BTCPAY_SIG").unwrap();
        let sig_val = sig_header.get(0).unwrap();
        Ok(sig_val.as_str().to_string().clone())
    };

    match sig {
        Ok(signature) => {
            log::trace!("verifying hmac from sig '{}'", &signature);
            if !state.verify_hmac(body_str, &signature.as_bytes()) {
                return Ok(tide::Response::builder(401)
                    .body(json!({"detail": "invalid hmac"}))
                    .build()
                )
            }
        },
        Err(e) => {
            log::trace!("bad request on handle_btcpay, missing hmac header");
            return Ok(tide::Response::builder(401)
                .body(json!({"detail": "failed to get hmac header"}))
                .build()
            );
        }
    };

    let mut connection = state.new_connection().unwrap().get_connection().unwrap();
    update.sync_update(connection);

    Ok(tide::Response::builder(200).body(json!({"message": "nice"})).build())
}
