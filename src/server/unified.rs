// src/server/unified.rs — Per-core workers + SO_REUSEPORT

use std::net::SocketAddr;
use std::sync::Arc;
use std::io;

use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;


pub const ALT_SVC: &str = r#"h3=":443"; ma=86400"#;

pub struct UnifiedServer {
    port: u16,
    tls_config: Arc<ServerConfig>,
}

impl UnifiedServer {
    pub fn new(port: u16, tls_config: Arc<ServerConfig>) -> Self {
        Self { port, tls_config }
    }

    pub async fn run(self) -> io::Result<()> {
        let cpus = num_cpus::get();
        println!("🚀 TezWeb Unified Server on port {}", self.port);
        println!("   📡 TCP :{} → HTTP/1.1 + HTTP/2", self.port);
        println!("   ⚡ UDP :{} → HTTP/3 QUIC", self.port);
        println!("   🔒 TLS + ALPN + Alt-Svc enabled");
        println!("   🧠 {} CPU cores — SO_REUSEPORT per core
", cpus);

        let port = self.port;
        let tls_config = self.tls_config.clone();

        // QUIC thread
        let quic_tls = tls_config.clone();
        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            if let Err(e) = run_quic_server(addr, quic_tls).await {
                eprintln!("QUIC error: {e}");
            }
        });

        // TCP listeners — one per core
        let addr_str = format!("0.0.0.0:{}", port);
        let mut tasks = vec![];
        for core_id in 0..cpus {
            let tls = tls_config.clone();
            let addr = addr_str.clone();
            tasks.push(tokio::spawn(async move {
                let std_listener = crate::io::create_listener(&addr).unwrap();
                let listener = tokio::net::TcpListener::from_std(std_listener).unwrap();
                let acceptor = TlsAcceptor::from(tls);
                println!("   ✅ Core {} TCP listening on {}", core_id, addr);
                run_tcp_loop(listener, acceptor).await;
            }));
        }

        for t in tasks { let _ = t.await; }
        Ok(())
    }
}

async fn run_tcp_loop(
    listener: tokio::net::TcpListener,
    acceptor: TlsAcceptor,
) {
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                eprintln!("TCP connection from {}", peer);
                let acceptor = acceptor.clone();
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let protocol = tls_stream
                            .get_ref().1
                            .alpn_protocol()
                            .map(|p| String::from_utf8_lossy(p).to_string())
                            .unwrap_or_else(|| "http/1.1".to_string());
                        eprintln!("TLS OK, protocol: {}", protocol);
                        match protocol.as_str() {
                            "h2" => handle_h2(tls_stream).await,
                            _    => handle_h1(tls_stream).await,
                        }
                    }
                    Err(e) => eprintln!("TLS error: {}", e),
                }
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}

pub fn make_tls_config(cert_path: &str, key_path: &str) -> Arc<ServerConfig> {
    use tokio_rustls::rustls::pki_types::CertificateDer;
    use std::fs::File;
    use std::io::BufReader;

    let certs: Vec<CertificateDer> =
        rustls_pemfile::certs(&mut BufReader::new(File::open(cert_path).expect("cert nahi mila")))
            .map(|c| c.unwrap())
            .collect();

    let key = rustls_pemfile::private_key(
        &mut BufReader::new(File::open(key_path).expect("key nahi mili"))
    ).unwrap().expect("private key nahi mili");

    let mut cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("TLS config error");

    cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Arc::new(cfg)
}

async fn handle_h1<S>(mut stream: tokio_rustls::server::TlsStream<S>)
where S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut buf = vec![0u8; 8192];
    match stream.read(&mut buf).await {
        Ok(0) | Err(_) => return,
        Ok(n) => eprintln!("H1 request: {} bytes", n),
    }
    let body = b"TezWeb HTTP/1.1 over TLS!";
    let response = format!(
        "HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: {}
Connection: close

",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.write_all(body).await;
    let _ = stream.flush().await;
    eprintln!("H1 response sent");
}

#[derive(Clone)]
struct TokioExec;

impl<F> hyper::rt::Executor<F> for TokioExec
where F: std::future::Future + Send + 'static, F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::spawn(fut);
    }
}

async fn handle_h2<S>(stream: tokio_rustls::server::TlsStream<S>)
where S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let svc = hyper::service::service_fn(|_req| async move {
        Ok::<_, hyper::Error>(
            hyper::Response::builder()
                .status(200)
                .header("content-type", "text/plain")
                .body(hyper::Body::from("TezWeb HTTP/2! 🚀"))
                .unwrap()
        )
    });
    let _ = hyper::server::conn::Http::new()
        .with_executor(TokioExec)
        .http2_only(true)
        .serve_connection(stream, svc)
        .await;
}

async fn run_quic_server(
    addr: SocketAddr,
    _tls_config: Arc<ServerConfig>,
) -> io::Result<()> {
    use tokio_rustls::rustls::pki_types::CertificateDer;
    use std::fs::File;
    use std::io::BufReader;

    let certs: Vec<CertificateDer> =
        rustls_pemfile::certs(&mut BufReader::new(
            File::open("certs/cert.pem").map_err(io::Error::other)?
        )).map(|c| c.unwrap()).collect();

    let key = rustls_pemfile::private_key(
        &mut BufReader::new(
            File::open("certs/key.pem").map_err(io::Error::other)?
        )
    ).unwrap().expect("key nahi mili");

    let mut quic_rustls = quinn::rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(io::Error::other)?;

    quic_rustls.alpn_protocols = vec![b"h3".to_vec()];

    let quic_crypto = quinn::crypto::rustls::QuicServerConfig::try_from(quic_rustls)
        .map_err(io::Error::other)?;

    let endpoint = quinn::Endpoint::server(
        quinn::ServerConfig::with_crypto(Arc::new(quic_crypto)),
        addr,
    ).map_err(io::Error::other)?;

    println!("✅ QUIC listening on {}", addr);

    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(async move {
            match incoming.await {
                Ok(conn) => handle_h3(conn).await,
                Err(e)   => eprintln!("QUIC conn error: {e}"),
            }
        });
    }
    Ok(())
}

async fn handle_h3(conn: quinn::Connection) {
    let mut h3_conn = match h3::server::Connection::new(
        h3_quinn::Connection::new(conn)
    ).await {
        Ok(c)  => c,
        Err(e) => { eprintln!("H3 conn error: {e}"); return; }
    };
    loop {
        match h3_conn.accept().await {
            Ok(Some(resolver)) => {
                let (req, mut stream) = match resolver.resolve_request().await {
                    Ok(v)  => v,
                    Err(e) => { eprintln!("H3 resolve: {e}"); continue; }
                };
                println!("   [H3] {} {}", req.method(), req.uri());
                tokio::spawn(async move {
                    let resp = http::Response::builder()
                        .status(200)
                        .header("content-type", "text/plain")
                        .header("Alt-Svc", ALT_SVC)
                        .body(()).unwrap();
                    let _ = stream.send_response(resp).await;
                    let _ = stream.send_data(bytes::Bytes::from("TezWeb HTTP/3! ⚡")).await;
                    let _ = stream.finish().await;
                });
            }
            Ok(None) => break,
            Err(e)   => { eprintln!("H3 accept: {e}"); break; }
        }
    }
}