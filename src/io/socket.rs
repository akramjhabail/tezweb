//! Socket wrapper with common operations

use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use crate::io::{TcpStream, TcpStreamExt, IoBuffer};

pub struct Socket {
    inner: TcpStream,
    peer_addr: SocketAddr,
}

impl Socket {
    pub fn new(stream: TcpStream, peer_addr: SocketAddr) -> Self {
        Self { inner: stream, peer_addr }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    pub fn inner(&self) -> &TcpStream {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut TcpStream {
        &mut self.inner
    }

    pub async fn read(&mut self, buf: &mut IoBuffer) -> io::Result<usize> {
        self.inner.read_buf(buf).await
    }

    pub async fn write(&mut self, buf: &mut IoBuffer) -> io::Result<usize> {
        self.inner.write_buf(buf).await
    }

    pub async fn read_exact(
        &mut self,
        buf: &mut IoBuffer,
        n: usize,
    ) -> io::Result<()> {
        self.inner.read_exact_buf(buf, n).await
    }

    pub async fn write_all(&mut self, buf: &mut IoBuffer) -> io::Result<()> {
        self.inner.write_all_buf(buf).await
    }

    // ✅ Graceful shutdown — connection reset fix!
    pub async fn shutdown(&mut self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::io::AsRawFd;
            unsafe {
                libc::shutdown(self.inner.as_raw_fd(), libc::SHUT_WR);
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            use tokio::io::AsyncWriteExt;
            let std_stream = self.inner
                .inner()
                .try_clone()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            std_stream.shutdown(std::net::Shutdown::Write)?;
        }

        Ok(())
    }

    // ✅ Keep-Alive check
    pub fn is_alive(&self) -> bool {
        #[cfg(not(target_os = "linux"))]
        {
            use socket2::SockRef;
            SockRef::from(self.inner.inner())
                .take_error()
                .map(|e| e.is_none())
                .unwrap_or(false)
        }

        #[cfg(target_os = "linux")]
        {
            true
        }
    }
}

pub struct SocketBuilder {
    nodelay:   bool,
    keepalive: Option<Duration>,
}

impl SocketBuilder {
    pub fn new() -> Self {
        Self {
            nodelay:   true,
            keepalive: Some(Duration::from_secs(60)), // ✅ 60s (was 30s)
        }
    }

    pub fn nodelay(mut self, enabled: bool) -> Self {
        self.nodelay = enabled;
        self
    }

    pub fn keepalive(mut self, duration: Option<Duration>) -> Self {
        self.keepalive = duration;
        self
    }

    pub fn configure(&self, _stream: &TcpStream) -> io::Result<()> {
        #[cfg(not(target_os = "linux"))]
        {
            use socket2::{SockRef, TcpKeepalive};
            let sock_ref = SockRef::from(_stream);
            sock_ref.set_nodelay(self.nodelay)?;
            if let Some(dur) = self.keepalive {
                let ka = TcpKeepalive::new()
                    .with_time(dur)
                    .with_interval(Duration::from_secs(10)); // ✅ Interval add
                sock_ref.set_tcp_keepalive(&ka)?;
            }
        }
        Ok(())
    }
}

impl Default for SocketBuilder {
    fn default() -> Self {
        Self::new()
    }
}