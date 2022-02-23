use super::invoice::InvoiceCommands;
use super::state::State;
use async_std::task;
use redis::Commands;
use serde::Deserialize;
use std::time::Duration;
use tide::convert::json;

#[derive(Deserialize)]
struct InvoiceQuery {
    invoice_id: String,
}

pub async fn websocket<T: InvoiceCommands + std::clone::Clone>(
    req: tide::Request<State<T>>,
    stream: tide_websockets::WebSocketConnection,
) -> tide::Result<()> {
    let query = req.query::<InvoiceQuery>()?;
    let state = req.state();

    let mut previous_string: String = {
        let db = state.db.lock().await;
        match db.get_invoice_status(query.invoice_id.clone()).await {
            Ok(status) => status,
            Err(_) => {
                stream
                    .send_json(&json!({
                        "message": "status not found"
                    }))
                    .await?;
                return Ok(());
            }
        }
    };

    loop {
        task::sleep(Duration::from_secs(1)).await;

        {
            let db = state.db.lock().await;
            match db.get_invoice_status(query.invoice_id.clone()).await {
                Ok(status) => {
                    if status == previous_string.clone() {
                        continue;
                    }

                    previous_string = status.clone();

                    log::trace!("sending status");
                    stream
                        .send_json(&json!({
                            "message": { "invoiceStatus": status[..] }
                        }))
                        .await?;

                    match &status[..] {
                        "InvoiceExpired" | "InvoicePayed" => {
                            break;
                        }
                        "InvoiceRecievedPayment" | "InvoiceCreated" => {}
                        _ => {
                            log::error!(
                                "Non supported status {} on invoice {}",
                                &status[..],
                                query.invoice_id.clone()
                            );
                            stream
                                .send_json(&json!({
                                    "message": "An error occured"
                                }))
                                .await?;
                            return Ok(());
                        }
                    };
                }
                Err(e) => {
                    log::error!("Error connecting to redis '{:?}'", e);
                    stream
                        .send_json(&json!({
                            "message": "An error occured"
                        }))
                        .await?;
                    return Ok(());
                }
            };
        }
    }

    return Ok(());
}
