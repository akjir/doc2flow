//! Pure-Rust zero-dependency implementation of the SHA-256 cryptographic hash algorithm.

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// Computes the SHA-256 digest of `data` and returns the 32-byte hash array directly.
pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut h = H0;

    let data_len = data.len();
    let bit_len = (data_len as u64) * 8;

    let mut padded_len = data_len + 1 + 8;
    let rem = padded_len % 64;
    if rem != 0 {
        padded_len += 64 - rem;
    }

    let mut bytes = Vec::with_capacity(padded_len);
    bytes.extend_from_slice(data);
    bytes.push(0x80);
    bytes.resize(padded_len - 8, 0x00);
    bytes.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in bytes.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_var = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h_var
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_var = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_var);
    }

    let mut out = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

/// Computes the SHA-256 hash of input bytes and returns a 64-character hexadecimal string.
///
/// Uses pre-allocated exact buffer capacity to eliminate re-allocations during padding.
///
/// # Examples
///
/// ```
/// use doc2flow::hasher::sha256;
///
/// let digest = sha256(b"abc");
/// assert_eq!(digest, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
/// ```
pub fn sha256(data: &[u8]) -> String {
    let digest = sha256_bytes(data);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    s
}

/// Generates a compact `doc_id` for browser `localStorage` in the format `"doc_<HEX_PREFIX>"`.
///
/// Computes the SHA-256 hash of the input string and returns the prefix `"doc_"`
/// followed by the first 16 hexadecimal characters of the hash.
///
/// # Examples
///
/// ```
/// use doc2flow::hasher::generate_doc_id;
///
/// let doc_id = generate_doc_id("test_input");
/// assert!(doc_id.starts_with("doc_"));
/// assert_eq!(doc_id.len(), 20);
/// ```
pub fn generate_doc_id(input: &str) -> String {
    let digest = sha256_bytes(input.as_bytes());
    let mut doc_id = String::with_capacity(20);
    doc_id.push_str("doc_");
    for b in &digest[..8] {
        doc_id.push(HEX_CHARS[(b >> 4) as usize] as char);
        doc_id.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    doc_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty_string() {
        let digest = sha256(b"");
        assert_eq!(
            digest,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_abc() {
        let digest = sha256(b"abc");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_sha256_longer_text() {
        let digest = sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            digest,
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn test_generate_doc_id() {
        let doc_id = generate_doc_id("sample_document");
        assert!(doc_id.starts_with("doc_"));
        assert_eq!(doc_id.len(), 20);
        // Verify exact 16-hex suffix matches sha256("sample_document")[..16]
        let full_hash = sha256(b"sample_document");
        assert_eq!(doc_id, format!("doc_{}", &full_hash[..16]));
    }

    #[test]
    fn test_sha256_padding_boundary_55_bytes() {
        let data = b"1234567890123456789012345678901234567890123456789012345";
        assert_eq!(data.len(), 55);
        let digest = sha256(data);
        assert_eq!(digest.len(), 64);
    }

    #[test]
    fn test_sha256_padding_overflow_56_bytes() {
        // 56 bytes + 1 byte (0x80) + 8 bytes (len) = 65 bytes > 64 -> forces second 64-byte block
        let data = b"12345678901234567890123456789012345678901234567890123456";
        assert_eq!(data.len(), 56);
        let digest = sha256(data);
        assert_eq!(digest.len(), 64);
    }

    #[test]
    fn test_sha256_exact_64_bytes() {
        let data = b"1234567890123456789012345678901234567890123456789012345678901234";
        assert_eq!(data.len(), 64);
        let digest = sha256(data);
        assert_eq!(digest.len(), 64);
    }

    #[test]
    fn test_sha256_large_input_1000_bytes() {
        let data = vec![b'a'; 1000];
        let digest = sha256(&data);
        assert_eq!(digest.len(), 64);
        // Multi-call determinism check
        assert_eq!(sha256(&data), digest);
    }

    #[test]
    fn test_generate_doc_id_determinism() {
        let id1 = generate_doc_id("test_doc_id_input");
        let id2 = generate_doc_id("test_doc_id_input");
        assert_eq!(id1, id2);
        assert_ne!(id1, generate_doc_id("different_input"));
    }
}

