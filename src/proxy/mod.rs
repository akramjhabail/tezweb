//! Reverse Proxy — TezWeb
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::http::{Request, Response};
use crate::TezError;

#[derive(Clone)]
pub struct ProxyRule {
    pub prefix: String,
    pub target: String,
}

pub async fn handle_proxy(req: Request, target: &str, prefix: &str) -> Result<Response, TezError> {
    let target = target.trim_end_matches('/').to_string();

    let stripped_path = if req.path.starts_with(prefix) {
        let p = &req.path[prefix.len()..];
        if p.is_empty() { "/".to_string() } else { p.to_string() }
    } else {
        req.path.clone()
    };

    let query = if req.query.is_empty() {
        String::new()
    } else {
        format!("?{}", req.query)
    };

    let target_no_scheme = target
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .to_string();

    let (host, port) = if target_no_scheme.contains(':') {
        let mut parts = target_no_scheme.splitn(2, ':');
        let h = parts.next().unwrap_or("localhost").to_string();
        let p = parts.next().unwrap_or("80").parse::<u16>().unwrap_or(80);
        (h, p)
    } else {
        (target_no_scheme.clone(), 80u16)
    };

    let addr = format!("{}:{}", host, port);
    eprintln!("Proxy: {} -> {}{}", req.path, addr, stripped_path);

    let std_stream = std::net::TcpStream::connect(&addr)
        .map_err(TezError::Io)?;
    std_stream.set_nonblocking(true).ok();
    let mut stream = TcpStream::from_std(std_stream)
        .map_err(TezError::Io)?;

    let method = format!("{:?}", req.method);
    let http_req = format!(
        "{} {}{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: TezWeb-Proxy/0.1\r\n\r\n",
        method, stripped_path, query, host
    );

    stream.write_all(http_req.as_bytes()).await
        .map_err(TezError::Io)?;

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).await
        .map_err(TezError::Io)?;

    if response_bytes.is_empty() {
        return Ok(Response::new(502).text("Proxy: empty response"));
    }

    let response_str = String::from_utf8_lossy(&response_bytes);
    let status = response_str
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(200);

    let body_start = response_bytes
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(0);

    let body = response_bytes[body_start..].to_vec();
    Ok(Response::new(status).body(body))
}
