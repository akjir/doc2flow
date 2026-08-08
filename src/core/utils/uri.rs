//! Base64 Data-URI formatting and file conversion utilities.

use super::base64::base64_encode_into;
use super::mime::guess_mime_type;
use crate::error::Result;
use crate::io;
use std::path::Path;

/// Formats binary data as a Base64 `data:` URI directly into the provided [`String`] buffer.
///
/// Pre-allocates buffer capacity for the `data:<mime>;base64,<encoded>` payload without
/// intermediate string allocations.
///
/// # Examples
///
/// ```
/// use doc2flow::to_base64_data_uri_into;
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
/// use doc2flow::to_base64_data_uri;
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
/// use doc2flow::file_to_data_uri;
///
/// let uri = file_to_data_uri(Path::new("test.png")).unwrap();
/// assert!(uri.starts_with("data:image/png;base64,"));
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`](crate::error::Doc2FlowError::Io) if the file cannot be read.
pub fn file_to_data_uri(path: &Path) -> Result<String> {
    let mime = guess_mime_type(path);
    let bytes = io::read_file_bytes(path)?;
    Ok(to_base64_data_uri(mime, &bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::base64::base64_encode;
    use crate::error::Doc2FlowError;

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
