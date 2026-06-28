use criterion::{criterion_group, criterion_main, Criterion};
use std::net::TcpStream;
use std::io::{Write, Read};

fn bench_tezweb(c: &mut Criterion) {
    // Start server in background (for manual run only)
    c.bench_function("tezweb_hello_world", |b| {
        b.iter(|| {
            if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
                let req = b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
                let _ = stream.write(req);
                let mut res = [0u8; 1024];
                let _ = stream.read(&mut res);
            }
        });
    });
}

criterion_group!(benches, bench_tezweb);
criterion_main!(benches);
