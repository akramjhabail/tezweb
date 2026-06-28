//! Static Files — Serve any file type
//! React, Vue, Angular, Plain HTML — sab kuch!

use std::path::{Path, PathBuf};
use crate::http::Response;

/// Extension se Content-Type detect karo
pub fn mime_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        // Web
        "html" | "htm" => "text/html; charset=utf-8",
        "css"           => "text/css",
        "js" | "mjs"    => "application/javascript",
        "json"          => "application/json",
        "xml"           => "application/xml",
        // Images
        "png"           => "image/png",
        "jpg" | "jpeg"  => "image/jpeg",
        "gif"           => "image/gif",
        "svg"           => "image/svg+xml",
        "ico"           => "image/x-icon",
        "webp"          => "image/webp",
        // Fonts
        "ttf"           => "font/ttf",
        "woff"          => "font/woff",
        "woff2"         => "font/woff2",
        // Media
        "mp4"           => "video/mp4",
        "mp3"           => "audio/mpeg",
        "wav"           => "audio/wav",
        // Docs
        "pdf"           => "application/pdf",
        "txt"           => "text/plain",
        // Data
        "csv"           => "text/csv",
        "zip"           => "application/zip",
        // Default
        _               => "application/octet-stream",
    }
}

/// File serve karo — std::fs use karo (sync, har runtime pe kaam karta hai)
pub async fn serve_file(file_path: &PathBuf) -> Response {
    match std::fs::read(file_path) {
        Ok(bytes) => {
            let path_str = file_path.to_string_lossy();
            let content_type = mime_type(&path_str);
            Response::ok()
                .header("Content-Type", content_type)
                .body(bytes)
        }
        Err(_) => {
            Response::not_found().text("404: File not found")
        }
    }
}

/// URL path → file path resolve karo
pub fn resolve_path(
    url_path: &str,
    route_prefix: &str,
    static_dir: &str,
) -> PathBuf {
    let relative = url_path
        .strip_prefix(route_prefix)
        .unwrap_or("")
        .trim_start_matches('/');

    let mut path = PathBuf::from(static_dir);

    if relative.is_empty() {
        path.push("index.html");
    } else {
        path.push(relative);
        if Path::new(relative).extension().is_none() {
            path.push("index.html");
        }
    }

    path
}