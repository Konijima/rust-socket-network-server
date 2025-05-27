use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpListener, sync::broadcast};
use futures::{SinkExt, StreamExt};
use dashmap::DashMap;
use base64::{engine::general_purpose, Engine as _};
use rsa::{pkcs1::DecodeRsaPublicKey, RsaPublicKey};
use rsa::pkcs1v15::Pkcs1v15Sign;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Incoming {
    #[serde(rename = "auth")]
    Auth {
        client_id: String,
        signature: String,
    },
    #[serde(rename = "event")]
    Event {
        data: String,
    },
}

#[derive(Serialize)]
struct Outgoing<'a> {
    from: &'a str,
    data: &'a str,
}

type ClientMap = Arc<DashMap<String, broadcast::Sender<(String, String)>>>;
type KeyMap = Arc<HashMap<String, RsaPublicKey>>;

fn load_client_keys() -> KeyMap {
    let mut map = HashMap::new();
    let pem = std::fs::read_to_string("keys/device123.pub.pem").expect("key file");
    let pubkey = RsaPublicKey::from_pkcs1_pem(&pem).expect("valid RSA PEM");
    map.insert("device123".to_string(), pubkey);
    Arc::new(map)
}

fn generate_nonce() -> [u8; 32] {
    let mut nonce = [0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut nonce);
    nonce
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8081").await?;
    let known_keys = load_client_keys();
    let clients: ClientMap = Arc::new(DashMap::new());

    println!("🔐 Server listening on ws://0.0.0.0:8081");

    while let Ok((stream, _)) = listener.accept().await {
        let ws = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(_) => continue,
        };
        let known_keys = known_keys.clone();
        let clients = clients.clone();

        tokio::spawn(async move {
            let (mut ws_tx, mut ws_rx) = ws.split();

            // Step 1: send nonce
            let nonce = generate_nonce();
            let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
            if ws_tx.send(Message::Text(format!("{{\"challenge\":\"{}\"}}", nonce_b64).into())).await.is_err() {
                return;
            }

            // Step 2: wait for auth message
            let Some(Ok(Message::Text(msg))) = ws_rx.next().await else { return; };
            let auth: Incoming = match serde_json::from_str(&msg) {
                Ok(a) => a,
                Err(_) => return,
            };
            let (client_id, signature) = match auth {
                Incoming::Auth { client_id, signature } => (client_id, signature),
                _ => return,
            };

            let public_key = match known_keys.get(&client_id) {
                Some(pk) => pk,
                None => return,
            };
            let sig_bytes = match general_purpose::STANDARD.decode(signature) {
                Ok(b) => b,
                Err(_) => return,
            };

            // Signature verification using unprefixed PKCS1v15 (works for this crate version)
            let verified = public_key.verify(
                Pkcs1v15Sign::new_unprefixed(),
                &nonce,
                &sig_bytes,
            ).is_ok();

            if !verified {
                let _ = ws_tx.send(Message::Close(None)).await;
                return;
            }

            println!("✅ Authenticated: {client_id}");
            let (tx, mut rx) = broadcast::channel::<(String, String)>(100);
            clients.insert(client_id.clone(), tx);

            // Spawn relay task for sending to this client
            let mut send_ws = ws_tx;
            let client_id_clone = client_id.clone();
            tokio::spawn(async move {
                while let Ok((from, data)) = rx.recv().await {
                    let out = Outgoing { from: &from, data: &data };
                    if let Ok(payload) = serde_json::to_string(&out) {
                        let _ = send_ws.send(Message::Text(payload.into())).await;
                    }
                }
            });

            // Step 3: handle messages from client
            while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
                match serde_json::from_str::<Incoming>(&msg) {
                    Ok(Incoming::Event { data }) => {
                        println!("📩 From {}: {}", client_id, data);
                        // Broadcast to all clients
                        for entry in clients.iter() {
                            let _ = entry.value().send((client_id.clone(), data.clone()));
                        }
                    }
                    _ => {}
                }
            }

            println!("🔌 Disconnected: {client_id_clone}");
            clients.remove(&client_id_clone);
        });
    }

    Ok(())
}
