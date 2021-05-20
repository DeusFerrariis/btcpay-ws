#![feature(macro_rules)]
use redis::Commands;
use tide_websockets::WebSocket;

mod args;
mod btcpay;
mod websocket;
mod db;

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

    let mut app = tide::with_state(db::RedisDb::new(
        host.to_string(),
        port.to_string(),
        pass.to_string(),
    ));

    app.at("/btcpay").post(btcpay::handle_btcpay);
    app.at("/ws")
        .with(WebSocket::new(websocket::websocket)) 
        .get(|_| async move { Ok("not a websocket request") });

    log::info!("Listening on {}:5000", host);
    app.listen(format!("{}:5000", host)).await?;

    Ok(())
}
