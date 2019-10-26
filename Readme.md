# Sockethook

> NOTE: This is a Rust implementation/port of this [sockethook](https://github.com/fabianlindfors/sockethook) implementation.

Sockethook is a Webhook-to-WebSocket proxy written in Rust. It's designed for building real-time applications around third-party APIs which provide Webhooks. Sockethook could for example be used to create a live feed of [Github](https://developer.github.com/webhooks/) deployments or a real-time view of [Shopify](https://help.shopify.com/api/reference/events/webhook) orders.

## Usage

Sockethook is written in Rust and can be installed by running

`$ go get github.com/fabianlindfors/sockethook`

The tool is started with

```
$ sockethook
2019-10-26T17:15:56Z INFO  sockethook] Sockethook is ready and listening at 0.0.0.0:3300 ✅
```

Sockethook is now ready to start receiving Webhooks! WebSockets can be connected through `/socket` followed by the endpoint to which you want to subscribe, for example: `/socket/order/created`. Any Webhook requests sent to `/hook/order/created` will now be broadcast to all subscribers listening to the specific endpoint.

The broadcasted message will be JSON encoded and contain information about the Webhook request. The following is an example message from a Shopify Webhook:

```json
{
  "headers": {
    "accept": "*\/*",
    "accept-encoding": "gzip;q=1.0,deflate;q=0.6,identity;q=0.3",
    "connection": "close",
    "content-length": "4264",
    "content-type": "application\/json",
    "user-agent": "Ruby",
    "x-forwarded-for": "35.231.14.37",
    "x-forwarded-proto": "https",
    "x-shopify-hmac-sha256": "wa5ZVAJPjbtr4Oj8xVnt\/jLWwfD9JdGFcdrjY4VgORQ=",
    "x-shopify-order-id": "820982911946154508",
    "x-shopify-shop-domain": "fabianlindfors.myshopify.com",
    "x-shopify-test": "true",
    "x-shopify-topic": "orders\/create"
  },
  "endpoint": "\/order\/created",
  "data": {
    "app_id": null,
    "billing_address": {
      "address1": "123 Billing Street",
      "address2": null,
      "city": "Billtown",
      "company": "My Company",
      "country": "United States",
      "country_code": "US",
      "first_name": "Bob",
      "last_name": "Biller",
      "latitude": null,
      "longitude": null,
      "name": "Bob Biller",
      "phone": "555-555-BILL",
      "province": "Kentucky",
      "province_code": "KY",
      "zip": "K2P0B0"
    }
    ...
  }
}
```

If the request content type is JSON then the `data` field will contain the JSON body. Otherwise `data` will be a string of the body.

## Command-line options

Two possible options can be passed to Sockethook, `--port` and `--address`. `--port` specifies which port at which to listen (default is 1234) and `--address` sets a specific address to bind to.

```
$ sockethook --port 8000
$ sockethook --address 127.0.0.1
```

## Authentication

Sockethook doesn't include any authentication meaning all endpoints and sockets are publicly available by default. The recommended way to add authentication is to use a reverse proxy or similar, which lends a lot of flexibility. Examples include [nginx](https://www.nginx.com), [Caddy](https://caddyserver.com), and [Traefik](https://traefik.io).

## License

Sockethook is licensed under [MIT](https://github.com/perfectmak/sockethook-rust/blob/master/LICENSE).


## Improvements
Here is a list improvements in this implementation from the original [sockethooks](https://github.com/fabianlindfors/sockethook) written in Go:

- [x] Cleanup websocket connections on disconnection
- [x] Handle Cors
- [ ] RWLock around client sockets management
- [ ] Allow scaling sockets and hooks to multiple computer