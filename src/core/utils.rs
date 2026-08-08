//! Utility module providing custom implementations for Base64 encoding,
//! MIME type guessing, and file Data-URI conversion.

use crate::error::Result;
use crate::io;
use std::path::Path;

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes binary data into Base64 format, appending directly into the provided [`String`] buffer.
///
/// Pre-allocates buffer capacity and pushes ASCII bytes directly to avoid intermediate
/// string allocations or UTF-8 validation overhead.
///
/// # Examples
///
/// ```
/// use doc2flow::utils::base64_encode_into;
///
/// let mut buf = String::from("data:text/plain;base64,");
/// base64_encode_into(b"foo", &mut buf);
/// assert_eq!(buf, "data:text/plain;base64,Zm9v");
/// ```
#[inline]
pub fn base64_encode_into(data: &[u8], out: &mut String) {
    if data.is_empty() {
        return;
    }

    let capacity = data.len().div_ceil(3) * 4;
    out.reserve(capacity);

    let chunks = data.chunks_exact(3);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];

        out.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
        out.push(BASE64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(BASE64_CHARS[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        out.push(BASE64_CHARS[(b2 & 0x3F) as usize] as char);
    }

    match remainder.len() {
        1 => {
            let b0 = remainder[0];
            out.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
            out.push(BASE64_CHARS[((b0 & 0x03) << 4) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let b0 = remainder[0];
            let b1 = remainder[1];
            out.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
            out.push(BASE64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            out.push(BASE64_CHARS[((b1 & 0x0F) << 2) as usize] as char);
            out.push('=');
        }
        _ => {}
    }
}

/// Encodes binary data into an RFC 4648 standard Base64 string representation.
///
/// Pre-allocates exact capacity and uses fast byte chunking to avoid heap reallocation.
///
/// # Examples
///
/// ```
/// use doc2flow::utils::base64_encode;
///
/// assert_eq!(base64_encode(b"foo"), "Zm9v");
/// ```
#[inline]
pub fn base64_encode(data: &[u8]) -> String {
    let capacity = data.len().div_ceil(3) * 4;
    let mut out = String::with_capacity(capacity);
    base64_encode_into(data, &mut out);
    out
}

/// Guesses the MIME type based on a file path extension without heap allocations.
///
/// Returns `application/octet-stream` as a safe fallback when the extension is unknown.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use doc2flow::utils::guess_mime_type;
///
/// assert_eq!(guess_mime_type(Path::new("image.png")), "image/png");
/// assert_eq!(guess_mime_type(Path::new("file.unknown")), "application/octet-stream");
/// ```
#[inline]
pub fn guess_mime_type(path: &Path) -> &'static str {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return "application/octet-stream";
    };

    match ext {
        e if e.eq_ignore_ascii_case("png") => "image/png",
        e if e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg") => "image/jpeg",
        e if e.eq_ignore_ascii_case("webp") => "image/webp",
        e if e.eq_ignore_ascii_case("svg") => "image/svg+xml",
        e if e.eq_ignore_ascii_case("gif") => "image/gif",
        e if e.eq_ignore_ascii_case("bmp") => "image/bmp",
        e if e.eq_ignore_ascii_case("ico") => "image/x-icon",
        e if e.eq_ignore_ascii_case("avif") => "image/avif",
        e if e.eq_ignore_ascii_case("tiff") || e.eq_ignore_ascii_case("tif") => "image/tiff",
        e if e.eq_ignore_ascii_case("pdf") => "application/pdf",
        e if e.eq_ignore_ascii_case("zip") => "application/zip",
        e if e.eq_ignore_ascii_case("html") || e.eq_ignore_ascii_case("htm") => "text/html",
        e if e.eq_ignore_ascii_case("css") => "text/css",
        e if e.eq_ignore_ascii_case("js") => "text/javascript",
        e if e.eq_ignore_ascii_case("json") => "application/json",
        e if e.eq_ignore_ascii_case("txt") => "text/plain",
        _ => "application/octet-stream",
    }
}

/// Formats binary data as a Base64 `data:` URI directly into the provided [`String`] buffer.
///
/// Pre-allocates buffer capacity for the `data:<mime>;base64,<encoded>` payload without
/// intermediate string allocations.
///
/// # Examples
///
/// ```
/// use doc2flow::utils::to_base64_data_uri_into;
///
/// let mut buf = String::new();
/// to_base64_data_uri_into("image/png", b"foo", &mut buf);
/// assert_eq!(buf, "data:image/png;base64,Zm9v");
/// ```
#[inline]
pub fn to_base64_data_uri_into(mime: &str, bytes: &[u8], out: &mut String) {
    let b64_len = bytes.len().div_ceil(3) * 4;
    let prefix = "data:";
    let suffix = ";base64,";
    let capacity = prefix.len() + mime.len() + suffix.len() + b64_len;
    out.reserve(capacity);
    out.push_str(prefix);
    out.push_str(mime);
    out.push_str(suffix);
    base64_encode_into(bytes, out);
}

/// Formats binary data as a Base64 `data:<mime>;base64,<encoded>` URI string.
///
/// Allocates a single `String` buffer sized exactly to hold the Data URI header and Base64 body.
///
/// # Examples
///
/// ```
/// use doc2flow::utils::to_base64_data_uri;
///
/// let uri = to_base64_data_uri("image/png", b"foo");
/// assert_eq!(uri, "data:image/png;base64,Zm9v");
/// ```
#[inline]
pub fn to_base64_data_uri(mime: &str, bytes: &[u8]) -> String {
    let b64_len = bytes.len().div_ceil(3) * 4;
    let prefix = "data:";
    let suffix = ";base64,";
    let capacity = prefix.len() + mime.len() + suffix.len() + b64_len;
    let mut out = String::with_capacity(capacity);
    out.push_str(prefix);
    out.push_str(mime);
    out.push_str(suffix);
    base64_encode_into(bytes, &mut out);
    out
}

/// Reads a local file and encodes its content into a Base64 Data URI string.
///
/// Allocates a single `String` buffer sized exactly to hold the Data URI header
/// and Base64 body without secondary buffer allocations.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use doc2flow::utils::file_to_data_uri;
///
/// let uri = file_to_data_uri(Path::new("test.png")).unwrap();
/// assert!(uri.starts_with("data:image/png;base64,"));
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`] if the file cannot be read.
pub fn file_to_data_uri(path: &Path) -> Result<String> {
    let mime = guess_mime_type(path);
    let bytes = io::read_file_bytes(path)?;
    Ok(to_base64_data_uri(mime, &bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Doc2FlowError;

    #[test]
    fn test_base64_rfc4648_test_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_base64_binary_bytes() {
        let input = vec![0x00, 0x01, 0x02, 0xFE, 0xFF];
        let encoded = base64_encode(&input);
        assert_eq!(encoded, "AAEC/v8=");
    }

    #[test]
    fn test_base64_encode_into() {
        let mut out = String::from("prefix:");
        base64_encode_into(b"foo", &mut out);
        assert_eq!(out, "prefix:Zm9v");
    }

    #[test]
    fn test_guess_mime_type_images() {
        assert_eq!(guess_mime_type(Path::new("test.png")), "image/png");
        assert_eq!(guess_mime_type(Path::new("test.PNG")), "image/png");
        assert_eq!(guess_mime_type(Path::new("test.jpg")), "image/jpeg");
        assert_eq!(guess_mime_type(Path::new("test.jpeg")), "image/jpeg");
        assert_eq!(guess_mime_type(Path::new("test.webp")), "image/webp");
        assert_eq!(guess_mime_type(Path::new("test.svg")), "image/svg+xml");
        assert_eq!(guess_mime_type(Path::new("test.gif")), "image/gif");
        assert_eq!(guess_mime_type(Path::new("test.bmp")), "image/bmp");
        assert_eq!(guess_mime_type(Path::new("test.ico")), "image/x-icon");
        assert_eq!(guess_mime_type(Path::new("test.avif")), "image/avif");
        assert_eq!(guess_mime_type(Path::new("test.tiff")), "image/tiff");
        assert_eq!(guess_mime_type(Path::new("test.tif")), "image/tiff");
    }

    #[test]
    fn test_guess_mime_type_assets_and_fallbacks() {
        assert_eq!(guess_mime_type(Path::new("doc.pdf")), "application/pdf");
        assert_eq!(guess_mime_type(Path::new("archive.zip")), "application/zip");
        assert_eq!(guess_mime_type(Path::new("index.html")), "text/html");
        assert_eq!(guess_mime_type(Path::new("style.css")), "text/css");
        assert_eq!(guess_mime_type(Path::new("app.js")), "text/javascript");
        assert_eq!(guess_mime_type(Path::new("data.json")), "application/json");
        assert_eq!(guess_mime_type(Path::new("notes.txt")), "text/plain");

        assert_eq!(
            guess_mime_type(Path::new("file.unknown_extension")),
            "application/octet-stream"
        );
        assert_eq!(
            guess_mime_type(Path::new("no_extension")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_file_to_data_uri_success_and_error() {
        let temp_dir = std::env::temp_dir().join("d2f_test_data_uri");
        let _ = io::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("sample.png");
        io::write_file(&test_file, b"test image payload").unwrap();

        let data_uri = file_to_data_uri(&test_file).expect("should convert file to data uri");
        assert!(data_uri.starts_with("data:image/png;base64,"));
        let expected_b64 = base64_encode(b"test image payload");
        assert_eq!(data_uri, format!("data:image/png;base64,{expected_b64}"));

        let non_existent = temp_dir.join("does_not_exist.png");
        let err = file_to_data_uri(&non_existent).unwrap_err();
        match err {
            Doc2FlowError::Io { path, .. } => {
                assert_eq!(path, Some(non_existent));
            }
            _ => panic!("Expected Doc2FlowError::Io error variant"),
        }

        let _ = io::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_to_base64_data_uri_preallocation_and_formatting() {
        let payload = b"Hello, World!";
        let mime = "image/svg+xml";
        let uri = to_base64_data_uri(mime, payload);

        let expected_b64 = base64_encode(payload);
        assert_eq!(uri, format!("data:{mime};base64,{expected_b64}"));

        // Verify exact capacity pre-allocation
        let expected_b64_len = payload.len().div_ceil(3) * 4;
        let expected_cap = "data:".len() + mime.len() + ";base64,".len() + expected_b64_len;
        assert_eq!(uri.capacity(), expected_cap);

        // Test appending into existing buffer
        let mut buf = String::from("prefix_");
        to_base64_data_uri_into("text/plain", b"abc", &mut buf);
        assert_eq!(buf, "prefix_data:text/plain;base64,YWJj");
    }
}
