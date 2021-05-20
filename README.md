# THIS PROJECT IS NOT COMPLETE

While the project is publicly visible I have not finished porting over code from a personalized use case.

```
BTCPay WebSocket Service 0.0.1
Will C. <cleghornw@gmail.com>
Provides a WebSocket Interface for BTCPay Invoice Webhooks

USAGE:
    btcpay-ws [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --hmac <BTCPAY_HMAC>       BTCPay HMAC to Verify Incoming Updates
    -h, --host <REDIS_HOST>        Sets Redis Host for Invoice Status Tracking
    -a, --pass <REDIS_PASSWORD>    Password for Redis
    -p, --port <REDIS_PORT>        Sets Redis Port for Invoice Status Tracking
```

# Installing

`git clone https://github.com/DeusFerrariis/btcpay-ws.git && cd btcpay-ws`

`cargo install --path .`

Make sure to add your cargo folder to path if you havent already e.g. `$HOME/.cargo/bin`

Work tree test
