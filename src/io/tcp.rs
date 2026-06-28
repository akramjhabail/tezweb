//! TCP listener and stream with optimized io_uring support

use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::future::Future;
use crate::io::IoBuffer;
use crate::io::READ_BUFFER_SIZE;

#[cfg(target_os = "linux")]
pub use monoio::net::{TcpListener, TcpStream};

#[cfg(not(target_os = "linux"))]
pub use tokio::net::{TcpListener, TcpStream};

// ✅ Buffer Pool — Linux
#[cfg(target_os = "linux")]
thread_local! {
    static BUFFER_POOL: std::cell::RefCell<Vec<Vec<u8>>> =
        std::cell::RefCell::new(
            (0..64).map(|_| vec![0u8; READ_BUFFER_SIZE * 2]).collect()
        );
}

// ✅ Buffer Pool — Mac + Windows (same trick!)
#[cfg(not(target_os = "linux"))]
thread_local! {
    static BUFFER_POOL: std::cell::RefCell<Vec<Vec<u8>>> =
        std::cell::RefCell::new(
            (0..64).map(|_| vec![0u8; READ_BUFFER_SIZE * 2]).collect()
        );
}

#[cfg(target_os = "linux")]
fn get_buffer() -> Vec<u8> {
    BUFFER_POOL.with(|pool| {
        pool.borrow_mut()
            .pop()
            .unwrap_or_else(|| vec![0u8; READ_BUFFER_SIZE * 2])
    })
}

#[cfg(target_os = "linux")]
fn return_buffer(mut buf: Vec<u8>) {
    buf.clear();
    BUFFER_POOL.with(|pool| {
        let mut p = pool.borrow_mut();
        if p.len() < 128 { p.push(buf); }
    });
}

#[cfg(not(target_os = "linux"))]
fn get_buffer() -> Vec<u8> {
    BUFFER_POOL.with(|pool| {
        pool.borrow_mut()
            .pop()
            .unwrap_or_else(|| vec![0u8; READ_BUFFER_SIZE * 2])
    })
}

#[cfg(not(target_os = "linux"))]
fn return_buffer(mut buf: Vec<u8>) {
    buf.clear();
    BUFFER_POOL.with(|pool| {
        let mut p = pool.borrow_mut();
        if p.len() < 128 { p.push(buf); }
    });
}

pub fn create_listener(addr: &str) -> io::Result<std::net::TcpListener> {
    use socket2::{Domain, Protocol, Socket, Type};
    let addr: SocketAddr = addr.parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let socket = Socket::new(
        if addr.is_ipv6() { Domain::IPV6 } else { Domain::IPV4 },
        Type::STREAM,
        Some(Protocol::TCP),
    )?;

    socket.set_reuse_address(true)?;

    #[cfg(unix)]
    socket.set_reuse_port(true)?;

    socket.set_nodelay(true)?;
    socket.set_recv_buffer_size(512 * 1024)?;
    socket.set_send_buffer_size(512 * 1024)?;
    socket.bind(&addr.into())?;
    socket.listen(4096)?;
    socket.set_nonblocking(true)?;

    Ok(socket.into())
}

#[cfg(target_os = "linux")]
pub trait TcpStreamExt {
    fn read_buf<'a>(&'a mut self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<usize>> + 'a>>;
    fn write_buf<'a>(&'a mut self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<usize>> + 'a>>;
    fn read_exact_buf<'a>(&'a mut self, buf: &'a mut IoBuffer, n: usize)
        -> Pin<Box<dyn Future<Output = io::Result<()>> + 'a>>;
    fn write_all_buf<'a>(&'a mut self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<()>> + 'a>>;
    fn into_std_tcp_stream(self) -> std::net::TcpStream;
}

// ✅ Mac + Windows — FIXED return types
#[cfg(not(target_os = "linux"))]
pub trait TcpStreamExt {
    fn read_buf<'a>(&'a self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>>;
    fn write_buf<'a>(&'a self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>>;
    fn read_exact_buf<'a>(&'a self, buf: &'a mut IoBuffer, n: usize)
        -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>;
    // ✅ FIXED: usize → ()
    fn write_all_buf<'a>(&'a self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>;
    fn into_std_tcp_stream(self) -> std::net::TcpStream;
}

#[cfg(target_os = "linux")]
impl TcpStreamExt for TcpStream {
    fn read_buf<'a>(&'a mut self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<usize>> + 'a>>
    {
        Box::pin(async move {
            use monoio::io::AsyncReadRent;
            let vec = get_buffer();
            let (result, data) = self.read(vec).await;
            let n = result?;
            buf.inner.extend_from_slice(&data[..n]);
            return_buffer(data);
            Ok(n)
        })
    }

    fn write_buf<'a>(&'a mut self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<usize>> + 'a>>
    {
        Box::pin(async move {
            use monoio::io::AsyncWriteRent;
            let data = buf.to_vec();
            let (result, _) = self.write(data).await;
            let n = result?;
            buf.advance(n);
            Ok(n)
        })
    }

    fn read_exact_buf<'a>(&'a mut self, buf: &'a mut IoBuffer, n: usize)
        -> Pin<Box<dyn Future<Output = io::Result<()>> + 'a>>
    {
        Box::pin(async move {
            let mut total = 0;
            while total < n {
                let read = self.read_buf(buf).await?;
                if read == 0 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
                }
                total += read;
            }
            Ok(())
        })
    }

    fn write_all_buf<'a>(&'a mut self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<()>> + 'a>>
    {
        Box::pin(async move {
            while !buf.is_empty() {
                let written = self.write_buf(buf).await?;
                if written == 0 {
                    return Err(io::Error::new(io::ErrorKind::WriteZero, "write zero"));
                }
            }
            Ok(())
        })
    }

    fn into_std_tcp_stream(self) -> std::net::TcpStream {
        use std::os::unix::io::{FromRawFd, IntoRawFd};
        let fd = self.into_raw_fd();
        unsafe { std::net::TcpStream::from_raw_fd(fd) }
    }
}

// ✅ Mac + Windows implementation
#[cfg(not(target_os = "linux"))]
impl TcpStreamExt for TcpStream {
    fn read_buf<'a>(&'a self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>>
    {
        Box::pin(async move {
            // ✅ Pool se buffer!
            let mut pool_buf = get_buffer();
            self.readable().await?;
            match self.try_read(&mut pool_buf) {
                Ok(n) => {
                    buf.inner.extend_from_slice(&pool_buf[..n]);
                    return_buffer(pool_buf);
                    Ok(n)
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    return_buffer(pool_buf);
                    Ok(0)
                }
                Err(e) => {
                    return_buffer(pool_buf);
                    Err(e)
                }
            }
        })
    }

    fn write_buf<'a>(&'a self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>>
    {
        Box::pin(async move {
            self.writable().await?;
            match self.try_write(&buf.inner) {
                Ok(n) => {
                    buf.advance(n);
                    Ok(n)
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(0),
                Err(e) => Err(e),
            }
        })
    }

    fn read_exact_buf<'a>(&'a self, buf: &'a mut IoBuffer, n: usize)
        -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>
    {
        Box::pin(async move {
            while buf.len() < n {
                self.readable().await?;
                let mut pool_buf = get_buffer();
                match self.try_read(&mut pool_buf) {
                    Ok(read) => {
                        if read == 0 {
                            return_buffer(pool_buf);
                            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
                        }
                        buf.inner.extend_from_slice(&pool_buf[..read]);
                        return_buffer(pool_buf);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        return_buffer(pool_buf);
                    }
                    Err(e) => {
                        return_buffer(pool_buf);
                        return Err(e);
                    }
                }
            }
            Ok(())
        })
    }

    // ✅ FIXED: return () not usize
    fn write_all_buf<'a>(&'a self, buf: &'a mut IoBuffer)
        -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>
    {
        Box::pin(async move {
            let data = std::mem::take(&mut buf.inner);
            let mut pos = 0;
            let len = data.len();
            while pos < len {
                self.writable().await?;
                match self.try_write(&data[pos..]) {
                    Ok(n) => pos += n,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        })
    }

    fn into_std_tcp_stream(self) -> std::net::TcpStream {
        self.into_std().unwrap()
    }
}

pub async fn accept(listener: &TcpListener) -> io::Result<(TcpStream, SocketAddr)> {
    listener.accept().await
}