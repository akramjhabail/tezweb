//! Integration Tests
use std::time::Duration;
use std::process::{Command, Child};
fn find_port() -> u16 { std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port() }
fn build_bin() -> String { let s = Command::new("cargo").args(["build","--example","hello_world"]).output().unwrap(); assert!(s.status.success()); String::from("./target/debug/examples/hello_world") }
fn start(port: u16) -> Child { Command::new(build_bin()).env("PORT", port.to_string()).spawn().unwrap() }
fn wait(port: u16) { for _ in 0..200 { if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() { return; } std::thread::sleep(Duration::from_millis(10)); } panic!("Port {} not bound", port); }
fn stop(mut c: Child) { let _ = c.kill(); let _ = c.wait(); }
fn cli() -> reqwest::Client { reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap() }
fn url(p: u16, path: &str) -> String { format!("http://127.0.0.1:{}{}", p, path) }
#[tokio::test]
async fn test_hello() { let p=find_port(); let s=start(p); wait(p); let r=cli().get(&url(p,"/")).send().await.unwrap(); assert_eq!(r.status(),200); stop(s); }

#[tokio::test]
async fn test_health() { let p=find_port(); let s=start(p); wait(p); let r=cli().get(&url(p,"/health")).send().await.unwrap(); assert_eq!(r.status(),200); stop(s); }

#[tokio::test]
async fn test_404() { let p=find_port(); let s=start(p); wait(p); let r=cli().get(&url(p,"/nope")).send().await.unwrap(); assert_eq!(r.status(),404); stop(s); }

#[tokio::test]
async fn test_keepalive() { let p=find_port(); let s=start(p); wait(p); let c=cli(); assert_eq!(c.get(&url(p,"/")).send().await.unwrap().status(),200); assert_eq!(c.get(&url(p,"/")).send().await.unwrap().status(),200); stop(s); }

#[tokio::test]
async fn test_concurrent() { let p=find_port(); let s=start(p); wait(p); let c=cli(); let h:Vec<_>=(0..10).map(|_|{let c=c.clone();let u=url(p,"/");tokio::spawn(async move{c.get(&u).send().await.unwrap().status()})}).collect(); for h in h{assert_eq!(h.await.unwrap(),200);} stop(s); }

#[tokio::test]
async fn test_headers() { let p=find_port(); let s=start(p); wait(p); let r=cli().get(&url(p,"/")).send().await.unwrap(); assert!(r.headers().get("connection").unwrap().to_str().unwrap().contains("keep-alive")); stop(s); }
