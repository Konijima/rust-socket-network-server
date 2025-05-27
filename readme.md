# Socket Network Server

This is a simple, secure WebSocket server in Rust that authenticates clients using RSA keypairs (challenge-response with signed nonce). Only clients with registered public keys can connect.

## Features

- WebSocket server (port 8081 by default)
- RSA-based client authentication (PKCS#1 public keys)
- Broadcast incoming messages to all connected clients

## Requirements

- Rust (stable)
- OpenSSL (for key generation)

## Setup

### 1. Generate keys for each client

**On any device:**

```sh
mkdir -p keys
openssl genpkey -algorithm RSA -out keys/device123.p8.pem -pkeyopt rsa_keygen_bits:2048
openssl rsa -in keys/device123.p8.pem -out keys/device123.pem
rm keys/device123.p8.pem
openssl rsa -in keys/device123.pem -pubout -RSAPublicKey_out -out keys/device123.pub.pem
````

* Place **public key** (`device123.pub.pem`) in the server’s `keys/` directory.

### 2. Register the public key

By default, `main.rs` loads `keys/device123.pub.pem`.
To allow more clients, update `load_client_keys()`.

### 3. Build and Run

```sh
cargo build --release
cargo run
```

The server listens on `ws://0.0.0.0:8081`.

## Message Flow

1. Client connects; server sends a nonce (base64 challenge).
2. Client responds with an authentication message, including a signature.
3. If authentication succeeds, client can send/receive messages.
4. Messages from clients are broadcast to all connected clients.

## Troubleshooting

* If you see ASN1 or PEM errors, make sure the public key is in **PKCS#1** format (`-----BEGIN RSA PUBLIC KEY-----`).
* Signature verification errors: check that the client’s public/private keypair matches.

## Security Notes

* The server does not use TLS; run behind a reverse proxy if needed.
* Only pre-registered clients can connect.

## License

MIT
