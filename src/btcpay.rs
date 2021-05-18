use colored::Colorize;
use redis::Commands;
use serde::Deserialize;
use std::{error::Error, fmt};
use tide::convert::json;
extern crate log;
use super::state::State;

#[derive(Debug, Deserialize)]
struct InvoiceUpdate {
    r#type: String,
    invoiceId: String,
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
        match connection.set::<String, String, String>(self.invoiceId.clone(), self.r#type.clone())
        {
            Ok(..) => Ok(()),
            Err(..) => Err(RedisError::Connection),
        }
    }
}

pub async fn handle_btcpay(mut req: tide::Request<State>) -> tide::Result<tide::Response> {
    log::trace!("{}", "Handling invoice update");
    let state: State = req.state().clone();

    let body = Box::new(req.take_body());

    let body_str: String = match body.into_string().await {
        Ok(body) => body.clone(),
        Err(..) => {
            log::trace!("request missing body");
            return Ok(tide::Response::builder(400)
                .body(json!({"message": "missing body"}))
                .build());
        }
    };

    log::debug!("{}", &body_str);

    let update: InvoiceUpdate = match serde_json::from_str::<InvoiceUpdate>(&body_str) {
        Ok(update) => update,
        Err(e) => {
            log::debug!("{}", e);
            log::trace!("request contains invalid/bad body");
            return Ok(tide::Response::builder(400)
                .body(json!({"message": "invalid body"}))
                .build());
        }
    };

    match req.header("BTCPAY-SIG") {
        Some(signature) => {
            match signature.get(0) {
                Some(sig) => {
                    let sig_string: String = sig.to_string();
                    let parts: Vec<&str> = sig_string.split('=').collect();

                    if parts.len() != 2 {
                        return Ok(tide::Response::builder(401)
                            .body(json!({"detail": "invalid BTCPAY-SIG"}))
                            .build());
                    }

                    if parts[0] != "sha256" {
                        return Ok(tide::Response::builder(401)
                            .body(json!({"detail": "invalid hmac operation"}))
                            .build());
                    }

                    log::debug!("e: {}", parts[1]);
                    log::debug!("e: {}", body_str);

                    if !state.verify_hmac(body_str, parts[1].as_bytes()) {
                        return Ok(tide::Response::builder(401)
                            .body(json!({"detail": "invalid hmac"}))
                            .build());
                    }
                },

                None => {
                    log::trace!("bad request on handle_btcpay, missing hmac header");
                    return Ok(tide::Response::builder(401)
                        .body(json!({"detail": "failed to get hmac header"}))
                        .build());
                }
            }
            log::trace!("verifying hmac from sig '{}'", &signature);
        }

        None => {
            log::trace!("bad request on handle_btcpay, missing hmac header");
            return Ok(tide::Response::builder(401)
                .body(json!({"detail": "failed to get hmac header"}))
                .build());
        }
    };

    match state.new_connection() {
        Ok(client) => {
            match client.get_connection() {
                Ok(connection) => {
                    match update.sync_update(connection) {
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("Error syncing update '{:?}''", e);
                            return Ok(tide::Response::builder(500).build());
                        }
                    }
                },
                Err(e) => {
                    log::error!("Error connecting to redis '{:?}'", e);
                    return Ok(tide::Response::builder(500).build());
                }
            }
        },
        Err(e) => {
            log::error!("Error connecting to redis '{:?}'", e);
            return Ok(tide::Response::builder(500).build());
        }
    }

    Ok(tide::Response::builder(200)
        .body(json!({"message": "nice"}))
        .build())
}


#[derive(Deserialize)]
struct InvoiceQuery {
    invoice_id: String,
}

pub async fn websocket(
    req: tide::Request<State>,
    stream: tide_websockets::WebSocketConnection,
) -> tide::Result<()> {
    let query = req.query::<InvoiceQuery>()?;
    let state = req.state();

    let mut connection: redis::Connection = match state.new_connection() {
        Ok(client) => {
            match client.get_connection() {
                Ok(connection) => connection,
                Err(e) => {
                    log::error!("Error connecting to redis '{:?}'", e);
                    stream.send_json(&json!({
                        "message": "An error occured"
                    })).await?;
                    return Ok(());
                }
            }
        },
        Err(e) => {
            log::error!("Error connecting to redis '{:?}'", e);
            stream.send_json(&json!({
                "message": "An error occured"
            })).await?;
            return Ok(());
        }
    };

    match connection.get::<String, String>(query.invoice_id.clone()) {
        Ok(status) => {
            match &status[..] {
                "InvoiceExpired" | "InvoicePayed" => {
                    stream.send_json(&json!({
                        "message": { "invoiceStatus": status[..] }
                    })).await?;
                    return Ok(());
                },
                "InvoiceRecievedPayment" | "InvoiceCreated" => {
                    stream.send_json(&json!({
                        "message": { "invoiceStatus": status[..] }
                    })).await?;
                },
                _ => {
                    log::error!("Non supported status {} on invoice {}", &status[..], query.invoice_id);
                    stream.send_json(&json!({
                        "message": "An error occured"
                    })).await?;
                    return Ok(());
                },
            }
        },
        Err(e) => {
            log::error!("Error connecting to redis '{:?}'", e);
            stream.send_json(&json!({
                "message": "An error occured"
            })).await?;
            return Ok(());
        }
    };

    loop {
        match connection.get::<String, String>(query.invoice_id.clone()) {
            Ok(status) => {
                match &status[..] {
                    "InvoiceExpired" | "InvoicePayed" => {
                        stream.send_json(&json!({
                            "message": { "invoiceStatus": status[..] }
                        })).await?;
                        break;
                    },
                    "InvoiceRecievedPayment" | "InvoiceCreated" => {
                        stream.send_json(&json!({
                            "message": { "invoiceStatus": status[..] }
                        })).await?;
                    },
                    _ => {
                        log::error!("Non supported status {} on invoice {}", &status[..], query.invoice_id);
                        stream.send_json(&json!({
                            "message": "An error occured"
                        })).await?;
                        return Ok(());
                    }
                };
            },
            Err(e) => {
                log::error!("Error connecting to redis '{:?}'", e);
                stream.send_json(&json!({
                    "message": "An error occured"
                })).await?;
                return Ok(());
            }
        };
    }
    return Ok(());
}
