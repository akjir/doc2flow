//! RFC 4648 Base64 encoding utilities without external dependencies.

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes binary data into Base64 format, appending directly into the provided [`String`] buffer.
///
/// Pre-allocates buffer capacity and pushes ASCII bytes directly to avoid intermediate
/// string allocations or UTF-8 validation overhead.
///
/// # Examples
///
/// ```
/// use doc2flow::base64_encode_into;
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
/// use doc2flow::base64_encode;
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
