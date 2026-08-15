//! Language dictionary and internationalization module.

/// Returns the serialized JSON dictionary string for the specified language code.
///
/// # Examples
///
/// ```
/// use doc2flow::core::language::get_language_json;
///
/// assert_eq!(get_language_json("en"), "{}");
/// assert_eq!(get_language_json("de"), "{}");
/// ```
pub fn get_language_json(_code: &str) -> &'static str {
    "{}"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_json_returns_empty_map() {
        assert_eq!(get_language_json("en"), "{}");
        assert_eq!(get_language_json("de"), "{}");
        assert_eq!(get_language_json("fr"), "{}");
        assert_eq!(get_language_json(""), "{}");
    }
}
