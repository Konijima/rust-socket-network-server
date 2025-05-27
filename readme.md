# Socket Network Server

A simple, secure Rust WebSocket server authenticating clients using RSA PKCS#1 keypairs (challenge-response with signed nonce). Only clients with registered public keys can connect.

## Features

- WebSocket server (port 8081 by default)
- RSA-based client authentication (PKCS#1 public keys)
- Broadcasts incoming messages to all connected clients

## Requirements

- Rust (stable)
- OpenSSL (for key generation)

## Setup

### 1. Generate an RSA Keypair for Each Client

**Always use this command to get a compatible PKCS#1 private key:**

```sh
mkdir -p keys
openssl genrsa -traditional -out keys/device123.pem 2048
````

* The private key will look like this:

  ```
  -----BEGIN RSA PRIVATE KEY-----
  ```

**Then extract the public key (for the server):**

```sh
openssl rsa -in keys/device123.pem -pubout -RSAPublicKey_out -out keys/device123.pub.pem
```

* The public key will look like this:

  ```
  -----BEGIN RSA PUBLIC KEY-----
  ```

### 2. Register the Public Key

Copy each client’s `device123.pub.pem` to the server’s `keys/` directory.

By default, `main.rs` loads `keys/device123.pub.pem`.
To allow more clients, update the `load_client_keys()` function in the source code.

### 3. Build and Run

```sh
cargo build --release
cargo run
```

The server listens on `ws://0.0.0.0:8081`.

## Message Flow

1. Client connects; server sends a nonce (base64 challenge).
2. Client responds with an authentication message, including a signature.
3. If authentication succeeds, the client can send/receive messages.
4. Messages from clients are broadcast to all connected clients.

## Troubleshooting

* **ASN1 or PEM errors:**
  Ensure all keys are in PKCS#1 PEM format (not PKCS#8).
  Private key: `-----BEGIN RSA PRIVATE KEY-----`
  Public key:  `-----BEGIN RSA PUBLIC KEY-----`
* **Signature verification errors:**
  Make sure the client’s public/private keypair matches.

## Security Notes

* The server does not use TLS; run behind a reverse proxy for encrypted transport if needed.
* Only pre-registered clients can connect.

## License

MIT
