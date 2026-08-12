//! Global system metadata, CLI branding, and core defaults for Doc2Flow.

/// Application binary name.
pub const APP_NAME: &str = "doc2flow";

/// Short command-line interface alias.
pub const CLI_ALIAS: &str = "d2f";

/// Command-line interface banner text.
pub const CLI_BANNER: &str = "Doc2Flow (d2f)";

/// Application version with SemVer 2.0.0 build metadata dynamically generated at compile time.
pub const APP_VERSION: &str = env!("D2F_FULL_VERSION");

/// Official application repository URL.
pub const REPOSITORY_URL: &str = "https://github.com/akjir/doc2flow";

/// Application license terms.
pub const LICENSE_TERMS: &str = "GPL-3.0-or-later";

/// Official application license URL.
pub const LICENSE_URL: &str = "https://github.com/akjir/doc2flow/blob/main/LICENSE";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_system_constants() {
        assert_eq!(APP_NAME, "doc2flow");
        assert_eq!(CLI_ALIAS, "d2f");
        assert_eq!(CLI_BANNER, "Doc2Flow (d2f)");
        assert!(!APP_VERSION.is_empty());
        assert_eq!(REPOSITORY_URL, "https://github.com/akjir/doc2flow");
        assert_eq!(LICENSE_TERMS, "GPL-3.0-or-later");
        assert_eq!(
            LICENSE_URL,
            "https://github.com/akjir/doc2flow/blob/main/LICENSE"
        );
    }
}
