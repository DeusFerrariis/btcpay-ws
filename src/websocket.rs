use serde::Deserialize;
use redis::Commands;
use super::state::State;
use tide::convert::json;
use std::time::Duration;
use async_std::task;

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

    let mut previous_status: String = String::from("");

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
                    previous_status = status.clone();
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
        log::trace!("Wooooo");
        task::sleep(Duration::from_secs(1)).await;

        match stream.send_string("pong".to_string()).await {
            Ok(_) => {},
            Err(_) => {
                break;
            }
        };

        match connection.get::<String, String>(query.invoice_id.clone()) {
            Ok(status) => {
                if status == previous_status {
                    continue;
                }
                match &status[..] {
                    "InvoiceExpired" | "InvoicePayed" => {
                        log::trace!("sending status");
                        stream.send_json(&json!({
                            "message": { "invoiceStatus": status[..] }
                        })).await?;
                        break;
                    },
                    "InvoiceRecievedPayment" | "InvoiceCreated" => {
                        log::trace!("sending status");
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
