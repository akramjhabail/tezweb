use std::net::SocketAddr;
use std::sync::Arc;

use h3_quinn::quinn;
use quinn::ClientConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Self-signed cert ke liye TLS verification skip
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipVerify))
        .with_no_client_auth();

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
        let _ = futures::future::poll_fn(|cx| driver.poll_close(cx)).await;
    });

    let req = http::Request::builder()
        .uri("https://localhost:4433/")
        .method("GET")
        .header("host", "localhost")
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

// TLS verification skip karne ke liye
#[derive(Debug)]
struct SkipVerify;

impl rustls::client::danger::ServerCertVerifier for SkipVerify {
    fn verify_server_cert(
        &self, _: &rustls::pki_types::CertificateDer,
        _: &[rustls::pki_types::CertificateDer],
        _: &rustls::pki_types::ServerName,
        _: &[u8],
        _: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(&self, _: &[u8], _: &rustls::pki_types::CertificateDer, _: &rustls::DigitallySignedStruct) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(&self, _: &[u8], _: &rustls::pki_types::CertificateDer, _: &rustls::DigitallySignedStruct) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
        ]
    }
}
