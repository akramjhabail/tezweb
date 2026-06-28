
//! HTTP/1.1 module

pub mod method;
pub mod parser;
pub mod writer;

pub use method::Method;
pub use parser::Request;
pub use writer::Response;