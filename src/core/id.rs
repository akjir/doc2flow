//! Dynamic `d2f_id` generator module for Doc2Flow.

use crate::converter::Frontmatter;
use crate::error::{Doc2FlowError, Result, print_warning};
use std::borrow::Cow;

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// Generates a dynamic, deterministic, and collision-free `d2f_id` for a document.
///
/// Combines the `title`, `version`, and `date` fields from the given frontmatter.
/// Each field is trimmed and lowercased. If the `version` field exceeds 12 characters,
/// it is truncated to 12 characters and a warning is logged.
///
/// # Errors
///
/// Returns an error if 2 or 3 of the identity fields (`title`, `version`, `date`) are missing.
///
/// # Examples
///
/// ```
/// use doc2flow::converter::Frontmatter;
/// use doc2flow::id::generate_d2f_id;
///
/// let mut fm = Frontmatter::new();
/// fm.title = Some("Server Guide".into());
/// fm.version = Some("1.0.0".into());
/// fm.date = Some("2026-07-25".into());
///
/// let id = generate_d2f_id(&fm).unwrap();
/// assert!(id.starts_with("d2f_id_"));
/// assert_eq!(id.len(), 23);
/// ```
pub fn generate_d2f_id(frontmatter: &Frontmatter) -> Result<String> {
    let norm_title = normalize_field(frontmatter.title.as_deref());
    let norm_date = normalize_field(frontmatter.date.as_deref());
    let raw_version = normalize_field(frontmatter.version.as_deref());

    let missing_count = [norm_title.is_empty(), norm_date.is_empty(), raw_version.is_empty()]
        .into_iter()
        .filter(|&empty| empty)
        .count();

    match missing_count {
        0 => (),
        1 => print_warning(
            "One of the identity fields (title, version, date) is missing in frontmatter. Generating d2f_id with available metadata.",
        ),
        _ => return Err(Doc2FlowError::MissingIdentityFields),
    }

    let norm_version = if let Some((idx, _)) = raw_version.char_indices().nth(12) {
        print_warning("'version' field exceeds 12 characters. Truncating to 12 characters.");
        &raw_version[..idx]
    } else {
        &raw_version
    };

    let key_len = norm_title.len() + norm_version.len() + norm_date.len() + 2;
    let mut composite_key = String::with_capacity(key_len);
    composite_key.push_str(&norm_title);
    composite_key.push(':');
    composite_key.push_str(norm_version);
    composite_key.push(':');
    composite_key.push_str(&norm_date);

    let digest = crate::hasher::sha256_bytes(composite_key.as_bytes());

    let mut result = String::with_capacity(23);
    result.push_str("d2f_id_");
    for b in &digest[..8] {
        result.push(HEX_CHARS[(b >> 4) as usize] as char);
        result.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    Ok(result)
}

/// Normalizes an optional string field by trimming whitespace and lowercasing.
///
/// Returns `Cow::Borrowed` if the string requires no lowercasing, avoiding heap allocations.
fn normalize_field<'a>(input: Option<&'a str>) -> Cow<'a, str> {
    match input {
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.chars().any(char::is_uppercase) {
                Cow::Owned(trimmed.to_lowercase())
            } else {
                Cow::Borrowed(trimmed)
            }
        }
        None => Cow::Borrowed(""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_d2f_id_success() {
        let mut fm = Frontmatter::new();
        fm.title = Some("Maintenance Protocol".into());
        fm.version = Some("v1.2.3".into());
        fm.date = Some("2026-07-25".into());

        let result = generate_d2f_id(&fm).expect("d2f_id generation failed");
        assert!(result.starts_with("d2f_id_"));
        assert_eq!(result.len(), 23);
    }

    #[test]
    fn test_generate_d2f_id_determinism() {
        let mut fm1 = Frontmatter::new();
        fm1.title = Some("  Maintenance Protocol  ".into());
        fm1.version = Some("V1.2.3".into());
        fm1.date = Some("2026-07-25".into());

        let mut fm2 = Frontmatter::new();
        fm2.title = Some("maintenance protocol".into());
        fm2.version = Some("v1.2.3".into());
        fm2.date = Some("2026-07-25".into());

        let id1 = generate_d2f_id(&fm1).unwrap();
        let id2 = generate_d2f_id(&fm2).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_d2f_id_version_truncation() {
        let mut fm = Frontmatter::new();
        fm.title = Some("System Spec".into());
        fm.version = Some("1.0.0-beta.release.99".into());
        fm.date = Some("2026-07-25".into());

        let mut fm_truncated_manually = Frontmatter::new();
        fm_truncated_manually.title = Some("System Spec".into());
        fm_truncated_manually.version = Some("1.0.0-beta.r".into());
        fm_truncated_manually.date = Some("2026-07-25".into());

        let id1 = generate_d2f_id(&fm).unwrap();
        let id2 = generate_d2f_id(&fm_truncated_manually).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_d2f_id_version_truncation_unicode() {
        let mut fm = Frontmatter::new();
        fm.title = Some("System Spec".into());
        fm.version = Some("1.0.0-beta.äöü.99".into());
        fm.date = Some("2026-07-25".into());

        let id = generate_d2f_id(&fm).unwrap();
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), 23);
    }

    #[test]
    fn test_normalize_field_borrowed_vs_owned() {
        let borrowed = normalize_field(Some("  clean_string  "));
        assert!(matches!(borrowed, Cow::Borrowed("clean_string")));

        let owned = normalize_field(Some("  UPPER_STRING  "));
        assert!(matches!(owned, Cow::Owned(_)));
        assert_eq!(owned, "upper_string");
    }

    #[test]
    fn test_generate_d2f_id_one_missing_field_allowed() {
        let mut fm = Frontmatter::new();
        fm.title = Some("System Spec".into());
        fm.version = Some("1.0".into());

        let result = generate_d2f_id(&fm);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_d2f_id_two_missing_fields_fatal() {
        let mut fm = Frontmatter::new();
        fm.title = Some("System Spec".into());

        let result = generate_d2f_id(&fm);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("At least 2 of the required identity fields")
        );
    }

    #[test]
    fn test_generate_d2f_id_three_missing_fields_fatal() {
        let fm = Frontmatter::new();
        let result = generate_d2f_id(&fm);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_d2f_id_exact_12_char_version() {
        let mut fm = Frontmatter::new();
        fm.title = Some("System Spec".into());
        fm.version = Some("123456789012".into());
        fm.date = Some("2026-07-26".into());

        let result = generate_d2f_id(&fm).unwrap();
        assert!(result.starts_with("d2f_id_"));
    }

    #[test]
    fn test_generate_d2f_id_emoji_unicode_truncation() {
        let mut fm = Frontmatter::new();
        fm.title = Some("Unicode Test".into());
        fm.version = Some("v1.0.0-🚀🌟✨🎉🎈🎊".into());
        fm.date = Some("2026-07-26".into());

        let result = generate_d2f_id(&fm);
        assert!(result.is_ok(), "d2f_id generation should handle multi-byte emoji truncation safely");
        let id = result.unwrap();
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), 23);
    }

    #[test]
    fn test_generate_d2f_id_missing_title_allowed() {
        let mut fm = Frontmatter::new();
        fm.version = Some("v1.0".into());
        fm.date = Some("2026-07-26".into());

        let result = generate_d2f_id(&fm);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_d2f_id_missing_version_allowed() {
        let mut fm = Frontmatter::new();
        fm.title = Some("Title Only".into());
        fm.date = Some("2026-07-26".into());

        let result = generate_d2f_id(&fm);
        assert!(result.is_ok());
    }
}
