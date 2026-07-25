//! Dynamic `d2f_id` generator module for Doc2Flow.

use crate::converter::Frontmatter;
use crate::error::print_warning;
use crate::hasher::sha256;
use anyhow::{Result, bail};

/// Generates a dynamic, deterministic, and collision-free `d2f_id` for a document.
///
/// Combines the `title`, `version`, and `date` fields from the document's frontmatter.
/// Each field is trimmed and lowercased. If the `version` field exceeds 12 characters,
/// it is truncated and a warning is printed to `stderr`.
///
/// # Errors
///
/// Returns an error if 2 or 3 of the identity fields (`title`, `version`, `date`) are missing.
///
/// # Examples
///
/// ```
/// use doc2flow::converter::Frontmatter;
/// use doc2flow::d2f_id::generate_d2f_id;
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
/// assert_eq!(id.len(), 7 + 16); // "d2f_id_" + 16 hex chars
/// ```
pub fn generate_d2f_id(frontmatter: &Frontmatter) -> Result<String> {
    let norm_title = frontmatter.title.trim().to_lowercase();
    let norm_date = frontmatter.date.trim().to_lowercase();
    let raw_version = frontmatter.version.trim().to_lowercase();

    let missing_count = [&norm_title, &norm_date, &raw_version]
        .iter()
        .filter(|f| f.is_empty())
        .count();

    if missing_count >= 2 {
        bail!(
            "Fatal: At least 2 of the required identity fields (title, version, date) are missing in frontmatter."
        );
    }

    if missing_count == 1 {
        print_warning(
            "One of the identity fields (title, version, date) is missing in frontmatter. Generating d2f_id with available metadata.",
        );
    }

    let norm_version = if raw_version.chars().count() > 12 {
        print_warning("'version' field exceeds 12 characters. Truncating to 12 characters.");
        raw_version.chars().take(12).collect::<String>()
    } else {
        raw_version
    };

    let composite_key = format!("{}:{}:{}", norm_title, norm_version, norm_date);
    let hash_hex = sha256(composite_key.as_bytes());
    let id_suffix = &hash_hex[..16];

    Ok(format!("d2f_id_{}", id_suffix))
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
