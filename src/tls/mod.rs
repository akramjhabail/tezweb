// src/tls/mod.rs

pub mod config;
pub use config::TlsConfig;


#[cfg(feature = "tls")]
use std::sync::Arc;

#[cfg(feature = "tls")]
pub struct TlsAcceptor {
    config: Arc<ServerConfig>,
}

#[cfg(feature = "tls")]
impl TlsAcceptor {
    pub fn new(tls_config: &TlsConfig) -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::BufReader;

        let cert_file = File::open(&tls_config.cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()?;

        let key_file = File::open(&tls_config.key_path)?;
        let mut key_reader = BufReader::new(key_file);
        let mut keys = pkcs8_private_keys(&mut key_reader)
            .collect::<Result<Vec<_>, _>>()?;

        if keys.is_empty() {
            return Err("Private key nahi mili!".into());
        }

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                certs,
                rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0))
            )?;

        Ok(Self {
            config: Arc::new(config),
        })
    }

    pub fn arc_config(&self) -> Arc<ServerConfig> {
        self.config.clone()
    }
}