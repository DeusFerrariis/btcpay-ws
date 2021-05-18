#![feature(macro_rules)]
use redis::Commands;
use std::env;
use tide_websockets::{ Message, WebSocket };

mod args;
mod btcpay;
mod state;
mod websocket;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let matches = args::get_args().get_matches();

    let hmac = matches
        .value_of("btcpay-hmac")
        .expect("Missing argument hmac");
    let host = matches
        .value_of("redis-host")
        .expect("Missing argument host");
    let port = matches.value_of("redis-port").unwrap_or("6379");
    let pass = matches.value_of("redis-password").unwrap_or("");

    let state = state::State::new(pass.to_owned(), host.to_owned(), hmac.to_owned());
    let mut app = tide::with_state(state);
    app.at("/btcpay").post(btcpay::handle_btcpay);
    app
        .at("/ws")
        .with(WebSocket::new(websocket::websocket)) 
        .get(|_| async move { Ok("not a websocket request") });

    log::info!("Listening on {}:5000", host);
    app.listen(format!("{}:5000", host)).await?;

    Ok(())
}
