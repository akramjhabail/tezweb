use tezweb::http::h3::{H3Listener, make_tls_config};
use http::StatusCode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    quinn::rustls::crypto::ring::default_provider().install_default().ok();
    // Wahi cert.pem/key.pem use karo jo TLS step mein bane
    let tls = make_tls_config("cert.pem", "key.pem")?;

    let addr = "0.0.0.0:4433".parse()?;
    let server = H3Listener::new(addr, tls)?;

    server
        .run(|req| async move {
            println!("HTTP/3 request: {} {}", req.method(), req.uri());
            (StatusCode::OK, format!("TezWeb HTTP/3! Path: {}", req.uri().path()))
        })
        .await?;

    Ok(())
}