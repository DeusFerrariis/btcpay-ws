use serde::Deserialize;
use redis::Commands;
use super::db::InvoiceCommands;
use tide::convert::json;
use std::time::Duration;
use async_std::task;

#[derive(Deserialize)]
struct InvoiceQuery {
    invoice_id: String,
}

pub async fn websocket<T: InvoiceCommands>(
    req: tide::Request<T>,
    stream: tide_websockets::WebSocketConnection,
) -> tide::Result<()> {
    let query = req.query::<InvoiceQuery>()?;
    let state = req.state();

    let mut previous_status: String = match state.get_invoice_status(query.invoice_id.clone()) {
        Ok(status) => status,
        Err(_) => {
            stream.send_json(&json!({
                "message": "status not found"
            })).await?;
            return Ok(());
        }
    };

    loop {
        task::sleep(Duration::from_secs(1)).await;

        match state.get_invoice_status(query.invoice_id.clone()) {
            Ok(status) => {
                if status == previous_status {
                    continue;
                }

                log::trace!("sending status");
                stream.send_json(&json!({
                    "message": { "invoiceStatus": status[..] }
                })).await?;

                match &status[..] {
                    "InvoiceExpired" | "InvoicePayed" => {
                        break;
                    },
                    "InvoiceRecievedPayment" | "InvoiceCreated" => {},
                    _ => {
                        log::error!("Non supported status {} on invoice {}", &status[..], query.invoice_id.clone());
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
