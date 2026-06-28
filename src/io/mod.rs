//! IO module - Zero-copy networking

mod buffer;
mod tcp;
mod socket;
mod event;

pub use buffer::IoBuffer;
pub use tcp::{TcpListener, TcpStream, TcpStreamExt, accept, create_listener};
pub use socket::{Socket, SocketBuilder};
pub use event::EventLoop;

/// Default read buffer size (8KB)
pub const READ_BUFFER_SIZE: usize = 8192;
/// Default write buffer size
pub const WRITE_BUFFER_SIZE: usize = 8192;