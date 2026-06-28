use std::io::Write;
use flate2::write::GzEncoder;
use flate2::Compression;

pub enum Encoding {
    Gzip,
    Brotli,
    None,
}

pub fn detect_encoding(accept_encoding: &str) -> Encoding {
    if accept_encoding.contains("br") {
        Encoding::Brotli
    } else if accept_encoding.contains("gzip") {
        Encoding::Gzip
    } else {
        Encoding::None
    }
}

pub fn compress_gzip(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap_or_default();
    encoder.finish().unwrap_or_default()
}

pub fn compress_brotli(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let params = brotli::enc::BrotliEncoderParams::default();
    brotli::BrotliCompress(
        &mut std::io::Cursor::new(data),
        &mut output,
        &params
    ).ok();
    output
}

pub fn compress(data: &[u8], accept_encoding: &str) -> (Vec<u8>, Option<&'static str>) {
    match detect_encoding(accept_encoding) {
        Encoding::Brotli => (compress_brotli(data), Some("br")),
        Encoding::Gzip   => (compress_gzip(data), Some("gzip")),
        Encoding::None   => (data.to_vec(), None),
    }
}