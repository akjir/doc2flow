//! Deterministic document identifier generation for client-side storage isolation.

use std::borrow::Cow;
use std::fmt::Write;

use crate::core::document::DocumentParameters;
use crate::utils::Sha256;

/// Maximum allowed character count for the version component in the composite key.
const MAX_VERSION_CHARS: usize = 12;

/// Total formatted length of the generated document identifier (`"d2f_id_" + 16 hex chars`).
pub const DOCUMENT_ID_LENGTH: usize = 23;

/// Generates a dynamic, deterministic, and collision-resistant document identifier (`d2f_id`).
///
/// Cryptographically separates fields via length-prefixed streaming into a SHA-256 state
/// accumulator, eliminating delimiter injection vulnerabilities and avoiding intermediate
/// string buffer allocations.
///
/// # Examples
///
/// ```
/// use doc2flow::core::document::DocumentParameters;
/// use doc2flow::core::id::generate_document_id;
///
/// let mut params = DocumentParameters::new();
/// params.title = "Maintenance Protocol".into();
/// params.version = "v1.2.3".into();
/// params.date = "2026-08-17".into();
///
/// let id = generate_document_id(&params);
/// assert!(id.starts_with("d2f_id_"));
/// assert_eq!(id.len(), 23);
/// ```
#[must_use]
pub fn generate_document_id(params: &DocumentParameters) -> String {
    let norm_title = normalize_field(&params.title);
    let norm_date = normalize_field(&params.date);
    let raw_version = normalize_field(&params.version);

    let norm_version = if let Some((idx, _)) = raw_version.char_indices().nth(MAX_VERSION_CHARS) {
        &raw_version[..idx]
    } else {
        &raw_version
    };

    let mut hasher = Sha256::new();

    // P-CRYPTO-KEY-SEP & P-STREAM-HASH: Length-prefixed sequential streaming
    hasher.update(&(norm_title.len() as u64).to_be_bytes());
    hasher.update(norm_title.as_bytes());

    hasher.update(&(norm_version.len() as u64).to_be_bytes());
    hasher.update(norm_version.as_bytes());

    hasher.update(&(norm_date.len() as u64).to_be_bytes());
    hasher.update(norm_date.as_bytes());

    let digest = hasher.finish();

    let mut result = String::with_capacity(DOCUMENT_ID_LENGTH);
    result.push_str("d2f_id_");
    for &b in &digest[..8] {
        let _ = write!(result, "{b:02x}");
    }
    result
}

/// Normalizes a string field by trimming whitespace and converting to lowercase.
///
/// Returns `Cow::Borrowed` if the string requires no lowercasing, avoiding heap allocations.
fn normalize_field(input: &str) -> Cow<'_, str> {
    let trimmed = input.trim();
    if trimmed.chars().any(char::is_uppercase) {
        Cow::Owned(trimmed.to_lowercase())
    } else {
        Cow::Borrowed(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_document_id_success() {
        let mut params = DocumentParameters::new();
        params.title = "Maintenance Protocol".into();
        params.version = "v1.2.3".into();
        params.date = "2026-08-17".into();

        let id = generate_document_id(&params);
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), DOCUMENT_ID_LENGTH);
    }

    #[test]
    fn test_generate_document_id_determinism() {
        let mut params1 = DocumentParameters::new();
        params1.title = "  Maintenance Protocol  ".into();
        params1.version = "V1.2.3".into();
        params1.date = "2026-08-17".into();

        let mut params2 = DocumentParameters::new();
        params2.title = "maintenance protocol".into();
        params2.version = "v1.2.3".into();
        params2.date = "2026-08-17".into();

        let id1 = generate_document_id(&params1);
        let id2 = generate_document_id(&params2);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_document_id_delimiter_injection() {
        let mut p1 = DocumentParameters::new();
        p1.title = "A:".into();
        p1.version = "B".into();

        let mut p2 = DocumentParameters::new();
        p2.title = "A".into();
        p2.version = ":B".into();

        let id1 = generate_document_id(&p1);
        let id2 = generate_document_id(&p2);

        assert_ne!(
            id1, id2,
            "Delimiter injection must produce different document IDs"
        );
    }

    #[test]
    fn test_generate_document_id_version_truncation() {
        let mut params = DocumentParameters::new();
        params.title = "System Spec".into();
        params.version = "1.0.0-beta.release.99".into();
        params.date = "2026-08-17".into();

        let mut params_truncated = DocumentParameters::new();
        params_truncated.title = "System Spec".into();
        params_truncated.version = "1.0.0-beta.r".into();
        params_truncated.date = "2026-08-17".into();

        let id1 = generate_document_id(&params);
        let id2 = generate_document_id(&params_truncated);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_document_id_exact_12_char_version() {
        let mut params = DocumentParameters::new();
        params.title = "System Spec".into();
        params.version = "123456789012".into();
        params.date = "2026-08-17".into();

        let id = generate_document_id(&params);
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), DOCUMENT_ID_LENGTH);
    }

    #[test]
    fn test_generate_document_id_unicode_version_truncation() {
        let mut params = DocumentParameters::new();
        params.title = "Unicode Doc".into();
        params.version = "v1.0.0-🚀🌟✨🎉🎈🎊".into();
        params.date = "2026-08-17".into();

        let id = generate_document_id(&params);
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), DOCUMENT_ID_LENGTH);
    }

    #[test]
    fn test_generate_document_id_empty_params_fallback() {
        let params = DocumentParameters::new();
        let id = generate_document_id(&params);
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), DOCUMENT_ID_LENGTH);

        let params2 = DocumentParameters::default();
        let id2 = generate_document_id(&params2);
        assert_eq!(id, id2);
    }

    #[test]
    fn test_generate_document_id_partial_metadata() {
        let mut p1 = DocumentParameters::new();
        p1.title = "Title Only".into();

        let mut p2 = DocumentParameters::new();
        p2.version = "1.0.0".into();

        let mut p3 = DocumentParameters::new();
        p3.date = "2026-08-17".into();

        let id1 = generate_document_id(&p1);
        let id2 = generate_document_id(&p2);
        let id3 = generate_document_id(&p3);

        assert_eq!(id1.len(), DOCUMENT_ID_LENGTH);
        assert_eq!(id2.len(), DOCUMENT_ID_LENGTH);
        assert_eq!(id3.len(), DOCUMENT_ID_LENGTH);

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_generate_document_id_special_characters() {
        let mut params = DocumentParameters::new();
        params.title = "Protocol: Section #1 / Sub-tier [Alpha]".into();
        params.version = "v2.0:rc-1".into();
        params.date = "2026/08/17 21:00".into();

        let id = generate_document_id(&params);
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), DOCUMENT_ID_LENGTH);
    }

    #[test]
    fn test_generate_document_id_extreme_lengths() {
        let mut params = DocumentParameters::new();
        params.title = "A".repeat(10_000);
        params.version = "1.0.0".into();
        params.date = "2026-08-17".into();

        let id = generate_document_id(&params);
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), DOCUMENT_ID_LENGTH);
    }

    #[test]
    fn test_normalize_field_borrowed_vs_owned() {
        let borrowed = normalize_field("  clean_string  ");
        assert!(matches!(borrowed, Cow::Borrowed("clean_string")));

        let owned = normalize_field("  UPPER_STRING  ");
        assert!(matches!(owned, Cow::Owned(_)));
        assert_eq!(owned, "upper_string");
    }
}
