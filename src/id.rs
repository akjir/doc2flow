//! Dynamic `d2f_id` generator module for Doc2Flow.

use crate::converter::Frontmatter;
use crate::error::print_warning;
use crate::hasher::sha256;
use anyhow::{Result, bail};
use std::borrow::Cow;

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
/// let fm = Frontmatter {
///     title: "Server Guide".into(),
///     version: "1.0.0".into(),
///     date: "2026-07-25".into(),
///     ..Default::default()
/// };
///
/// let id = generate_d2f_id(&fm).unwrap();
/// assert!(id.starts_with("d2f_id_"));
/// assert_eq!(id.len(), 23);
/// ```
pub fn generate_d2f_id(frontmatter: &Frontmatter) -> Result<String> {
    let norm_title = normalize_field(&frontmatter.title);
    let norm_date = normalize_field(&frontmatter.date);
    let raw_version = normalize_field(&frontmatter.version);

    let missing_count = [norm_title.is_empty(), norm_date.is_empty(), raw_version.is_empty()]
        .into_iter()
        .filter(|&empty| empty)
        .count();

    match missing_count {
        0 => (),
        1 => print_warning(
            "One of the identity fields (title, version, date) is missing in frontmatter. Generating d2f_id with available metadata.",
        ),
        _ => bail!(
            "Fatal: At least 2 of the required identity fields (title, version, date) are missing in frontmatter."
        ),
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

    let hash_hex = sha256(composite_key.as_bytes());

    let mut result = String::with_capacity(23);
    result.push_str("d2f_id_");
    result.push_str(&hash_hex[..16]);
    Ok(result)
}

/// Normalizes a string field by trimming whitespace and lowercasing.
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
    fn test_generate_d2f_id_success() {
        let fm = Frontmatter {
            title: "Maintenance Protocol".into(),
            version: "v1.2.3".into(),
            date: "2026-07-25".into(),
            ..Default::default()
        };

        let result = generate_d2f_id(&fm).expect("d2f_id generation failed");
        assert!(result.starts_with("d2f_id_"));
        assert_eq!(result.len(), 23);
    }

    #[test]
    fn test_generate_d2f_id_determinism() {
        let fm1 = Frontmatter {
            title: "  Maintenance Protocol  ".into(),
            version: "V1.2.3".into(),
            date: "2026-07-25".into(),
            ..Default::default()
        };

        let fm2 = Frontmatter {
            title: "maintenance protocol".into(),
            version: "v1.2.3".into(),
            date: "2026-07-25".into(),
            ..Default::default()
        };

        let id1 = generate_d2f_id(&fm1).unwrap();
        let id2 = generate_d2f_id(&fm2).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_d2f_id_version_truncation() {
        let fm = Frontmatter {
            title: "System Spec".into(),
            version: "1.0.0-beta.release.99".into(), // > 12 chars
            date: "2026-07-25".into(),
            ..Default::default()
        };

        let fm_truncated_manually = Frontmatter {
            title: "System Spec".into(),
            version: "1.0.0-beta.r".into(), // exact 12 chars
            date: "2026-07-25".into(),
            ..Default::default()
        };

        let id1 = generate_d2f_id(&fm).unwrap();
        let id2 = generate_d2f_id(&fm_truncated_manually).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_d2f_id_version_truncation_unicode() {
        let fm = Frontmatter {
            title: "System Spec".into(),
            version: "1.0.0-beta.äöü.99".into(), // > 12 unicode chars
            date: "2026-07-25".into(),
            ..Default::default()
        };

        let id = generate_d2f_id(&fm).unwrap();
        assert!(id.starts_with("d2f_id_"));
        assert_eq!(id.len(), 23);
    }

    #[test]
    fn test_normalize_field_borrowed_vs_owned() {
        let borrowed = normalize_field("  clean_string  ");
        assert!(matches!(borrowed, Cow::Borrowed("clean_string")));

        let owned = normalize_field("  UPPER_STRING  ");
        assert!(matches!(owned, Cow::Owned(_)));
        assert_eq!(owned, "upper_string");
    }

    #[test]
    fn test_generate_d2f_id_one_missing_field_allowed() {
        let fm = Frontmatter {
            title: "System Spec".into(),
            version: "1.0".into(),
            date: "".into(), // 1 missing field
            ..Default::default()
        };

        let result = generate_d2f_id(&fm);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_d2f_id_two_missing_fields_fatal() {
        let fm = Frontmatter {
            title: "System Spec".into(),
            version: "".into(), // missing
            date: "".into(),    // missing
            ..Default::default()
        };

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
        let fm = Frontmatter::default();
        let result = generate_d2f_id(&fm);
        assert!(result.is_err());
    }
}
