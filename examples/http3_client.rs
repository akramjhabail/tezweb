use std::net::SocketAddr;
use std::sync::Arc;
use std::future;
use bytes::Buf;

use h3_quinn::quinn;
use quinn::ClientConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut crypto = h3_quinn::quinn::rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipVerify))
        .with_no_client_auth();

    // ALPN set karo — server se match karna zaruri hai
    crypto.alpn_protocols = vec![b"h3".to_vec()];

    let crypto = quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?;
    let client_config = ClientConfig::new(Arc::new(crypto));

    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse::<SocketAddr>()?)?;
    endpoint.set_default_client_config(client_config);

    let addr: SocketAddr = "127.0.0.1:4433".parse()?;
    let conn = endpoint.connect(addr, "localhost")?.await?;
    println!("✅ QUIC connected!");

    let quinn_conn = h3_quinn::Connection::new(conn);
    let (mut driver, mut send) = h3::client::new(quinn_conn).await?;

    tokio::spawn(async move {
        let _ = future::poll_fn(|cx| driver.poll_close(cx)).await;
    });

    // ✅ FIX: "host" header hata diya — HTTP/3 mein forbidden hai
    // URI se h3 crate khud :authority pseudo-header set karta hai
    let req = http::Request::builder()
        .uri("https://localhost:4433/")
        .method("GET")
        .body(())?;

    let mut stream = send.send_request(req).await?;
    stream.finish().await?;

    let resp = stream.recv_response().await?;
    println!("✅ Response: {}", resp.status());

    while let Some(chunk) = stream.recv_data().await? {
        println!("📦 Body: {}", String::from_utf8_lossy(chunk.chunk()));
    }

    Ok(())
}

#[derive(Debug)]
struct SkipVerify;

impl h3_quinn::quinn::rustls::client::danger::ServerCertVerifier for SkipVerify {
    fn verify_server_cert(
        &self, _: &h3_quinn::quinn::rustls::pki_types::CertificateDer,
        _: &[h3_quinn::quinn::rustls::pki_types::CertificateDer],
        _: &h3_quinn::quinn::rustls::pki_types::ServerName,
        _: &[u8],
        _: h3_quinn::quinn::rustls::pki_types::UnixTime,
    ) -> Result<h3_quinn::quinn::rustls::client::danger::ServerCertVerified, h3_quinn::quinn::rustls::Error> {
        Ok(h3_quinn::quinn::rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(&self, _: &[u8], _: &h3_quinn::quinn::rustls::pki_types::CertificateDer, _: &h3_quinn::quinn::rustls::DigitallySignedStruct) -> Result<h3_quinn::quinn::rustls::client::danger::HandshakeSignatureValid, h3_quinn::quinn::rustls::Error> {
        Ok(h3_quinn::quinn::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(&self, _: &[u8], _: &h3_quinn::quinn::rustls::pki_types::CertificateDer, _: &h3_quinn::quinn::rustls::DigitallySignedStruct) -> Result<h3_quinn::quinn::rustls::client::danger::HandshakeSignatureValid, h3_quinn::quinn::rustls::Error> {
        Ok(h3_quinn::quinn::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<h3_quinn::quinn::rustls::SignatureScheme> {
        vec![
            h3_quinn::quinn::rustls::SignatureScheme::RSA_PSS_SHA256,
            h3_quinn::quinn::rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
        ]
    }
}