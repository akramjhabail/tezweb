use tezweb::server::{UnifiedServer, make_tls_config};

#[tokio::main]
async fn main() {
    // ✅ CryptoProvider install karo
    let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();

    let tls_config = make_tls_config("certs/cert.pem", "certs/key.pem");
    
    println!("🚀 TezWeb starting...");
    UnifiedServer::new(8443, tls_config)
        .run()
        .await
        .unwrap();
}
