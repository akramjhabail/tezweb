//! HTTP/3 — QUIC protocol support
//! Enable with: cargo build --features http3

#![cfg(feature = "http3")]

use std::net::SocketAddr;
use std::sync::Arc;
use bytes::Bytes;
use http::{Response, StatusCode};
use quinn::{Endpoint, ServerConfig};
use tokio::task;

pub struct H3Listener {
    endpoint: Endpoint,
}

impl H3Listener {
    pub fn new(
        addr: SocketAddr,
        tls_config: Arc<quinn::rustls::ServerConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let quic_tls = quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)?;
        let server_cfg = ServerConfig::with_crypto(Arc::new(quic_tls));
        let endpoint = Endpoint::server(server_cfg, addr)?;
        Ok(Self { endpoint })
    }

    pub async fn run<F, Fut>(
        self,
        handler: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(http::Request<()>) -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = (StatusCode, String)> + Send + 'static,
    {
        let handler = Arc::new(handler);
        println!("🚀 HTTP/3 (QUIC) listening on {}", self.endpoint.local_addr()?);

        while let Some(incoming) = self.endpoint.accept().await {
            let handler = Arc::clone(&handler);

            task::spawn(async move {
                let conn = match incoming.await {
                    Ok(c)  => c,
                    Err(e) => { eprintln!("QUIC handshake failed: {e}"); return; }
                };

                let mut h3_conn = match h3::server::Connection::new(
                    h3_quinn::Connection::new(conn)
                ).await {
                    Ok(c)  => c,
                    Err(e) => { eprintln!("H3 connection error: {e}"); return; }
                };

                loop {
                    match h3_conn.accept().await {
                        Ok(Some(resolver)) => {
                            let handler = Arc::clone(&handler);
                            task::spawn(async move {
                                // ✅ New API: resolve karo pehle
                                let (req, mut stream) = match resolver.resolve_request().await {
                                    Ok(v)  => v,
                                    Err(e) => { eprintln!("H3 resolve error: {e}"); return; }
                                };

                                let (status, body) = handler(req).await;

                                let response = Response::builder()
                                    .status(status)
                                    .header("content-type", "text/plain")
                                    .header("server", "TezWeb/H3")
                                    .body(())
                                    .unwrap();

                                if let Err(e) = stream.send_response(response).await {
                                    eprintln!("H3 send_response error: {e}"); return;
                                }
                                if let Err(e) = stream.send_data(Bytes::from(body)).await {
                                    eprintln!("H3 send_data error: {e}"); return;
                                }
                                let _ = stream.finish().await;
                            });
                        }
                        Ok(None) => break,
                        Err(e)   => { eprintln!("H3 accept error: {e}"); break; }
                    }
                }
            });
        }
        Ok(())
    }
}

pub fn make_tls_config(
    cert_path: &str,
    key_path:  &str,
) -> Result<Arc<quinn::rustls::ServerConfig>, Box<dyn std::error::Error + Send + Sync>> {
    use quinn::rustls::pki_types::CertificateDer;
    use rustls_pemfile::{certs, private_key};
    use std::fs::File;
    use std::io::BufReader;

    let cert_file  = File::open(cert_path)?;
    let key_file   = File::open(key_path)?;

    let cert_chain: Vec<CertificateDer> = certs(&mut BufReader::new(cert_file))
        .map(|c| c.unwrap())
        .collect();

    let key = private_key(&mut BufReader::new(key_file))?.unwrap();

    let mut cfg = quinn::rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)?;

    cfg.alpn_protocols = vec![b"h3".to_vec()];
    Ok(Arc::new(cfg))
}