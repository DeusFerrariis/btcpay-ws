pub fn get_args() -> clap::App<'static, 'static> {
    clap::App::new("BTCPay WebSocket Service")
        .version("0.0.1")
        .author("Will C. <cleghornw@gmail.com>")
        .about("Provides a WebSocket Interface for BTCPay Invoice Webhooks")
        .arg(
            clap::Arg::with_name("redis-host")
                .short("h")
                .long("host")
                .value_name("REDIS_HOST")
                .help("Sets Redis Host for Invoice Status Tracking")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("redis-port")
                .short("p")
                .long("port")
                .value_name("REDIS_PORT")
                .help("Sets Redis Port for Invoice Status Tracking")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("btcpay-hmac")
                .short("b")
                .long("hmac")
                .value_name("BTCPAY_HMAC")
                .help("BTCPay HMAC to Verify Incoming Updates")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("redis-password")
                .short("a")
                .long("pass")
                .value_name("REDIS_PASSWORD")
                .help("Password for Redis")
                .takes_value(true),
        )
}
