#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub name: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

/// Multipart body parse karo
/// Content-Type: multipart/form-data; boundary=----WebKitFormBoundary
pub fn parse_multipart(body: &[u8], boundary: &str) -> Vec<UploadedFile> {
    let mut files = Vec::new();
    let delimiter = format!("--{}", boundary);
    let body_str = String::from_utf8_lossy(body);

    for part in body_str.split(&delimiter) {
        if part.trim().is_empty() || part.trim() == "--" {
            continue;
        }

        // Header aur body alag karo
        if let Some(sep) = part.find("\r\n\r\n") {
            let headers = &part[..sep];
            let content = &part[sep + 4..];
            let content = content.trim_end_matches("\r\n");

            let name = extract_header_param(headers, "name");
            let filename = extract_header_param(headers, "filename");
            let content_type = extract_content_type(headers);

            if let Some(field_name) = name {
                files.push(UploadedFile {
                    name: field_name,
                    filename,
                    content_type,
                    data: content.as_bytes().to_vec(),
                });
            }
        }
    }

    files
}

/// Boundary extract karo Content-Type header se
/// "multipart/form-data; boundary=abc123" → "abc123"
pub fn extract_boundary(content_type: &str) -> Option<String> {
    content_type
        .split(';')
        .find(|s| s.trim().starts_with("boundary="))
        .map(|s| s.trim().trim_start_matches("boundary=").to_string())
}

fn extract_header_param(headers: &str, param: &str) -> Option<String> {
    let search = format!("{}=\"", param);
    let start = headers.find(&search)? + search.len();
    let end = headers[start..].find('"')? + start;
    Some(headers[start..end].to_string())
}

fn extract_content_type(headers: &str) -> Option<String> {
    headers.lines()
        .find(|l| l.to_lowercase().starts_with("content-type:"))
        .map(|l| l.split_once(':').map(|x| x.1).unwrap_or("").trim().to_string())
}