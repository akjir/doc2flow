//! Zero-dependency JSON string map parser for Doc2Flow core engine.

use std::collections::HashMap;

use crate::core::error::JsonError;

/// Validates whether a byte sequence conforms to a valid JSON number according to RFC 8259.
fn is_valid_json_number(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let mut idx = 0;

    // Optional leading minus sign
    if bytes[idx] == b'-' {
        idx += 1;
        if idx == bytes.len() {
            return false;
        }
    }

    // Integer component
    if bytes[idx] == b'0' {
        idx += 1;
        // Leading zero cannot be followed by another digit (e.g., "01" is invalid)
        if idx < bytes.len() && bytes[idx].is_ascii_digit() {
            return false;
        }
    } else if bytes[idx].is_ascii_digit() {
        // Must start with 1-9 (0 handled above)
        idx += 1;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
    } else {
        return false;
    }

    // Optional fractional component
    if idx < bytes.len() && bytes[idx] == b'.' {
        idx += 1;
        if idx == bytes.len() || !bytes[idx].is_ascii_digit() {
            return false;
        }
        idx += 1;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
    }

    // Optional exponent component
    if idx < bytes.len() && (bytes[idx] == b'e' || bytes[idx] == b'E') {
        idx += 1;
        if idx < bytes.len() && (bytes[idx] == b'+' || bytes[idx] == b'-') {
            idx += 1;
        }
        if idx == bytes.len() || !bytes[idx].is_ascii_digit() {
            return false;
        }
        idx += 1;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
    }

    idx == bytes.len()
}

/// Parses a 4-hex-digit Unicode codepoint from the byte slice starting at `cursor`.
fn parse_hex4(bytes: &[u8], cursor: &mut usize) -> Result<u16, JsonError> {
    if *cursor + 4 > bytes.len() {
        return Err(JsonError::UnexpectedEof);
    }
    let hex_slice = &bytes[*cursor..*cursor + 4];
    *cursor += 4;

    let mut codepoint: u16 = 0;
    for &b in hex_slice {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as u16,
            b'a'..=b'f' => (b - b'a' + 10) as u16,
            b'A'..=b'F' => (b - b'A' + 10) as u16,
            _ => return Err(JsonError::InvalidUnicodeEscape),
        };
        codepoint = (codepoint << 4) | digit;
    }

    Ok(codepoint)
}

/// Parses a JSON object mapping string keys to string values.
///
/// Supports string values, numbers, booleans, and null entries converted to their text representations.
///
/// # Examples
///
/// ```
/// use doc2flow::core::json::parse_json_map;
///
/// let json = r#"{"export_pdf": "Export as PDF", "lang": "en"}"#;
/// let map = parse_json_map(json).unwrap();
/// assert_eq!(map.get("export_pdf").unwrap(), "Export as PDF");
/// assert_eq!(map.get("lang").unwrap(), "en");
/// ```
///
/// # Errors
///
/// Returns [`JsonError`] if the input is malformed, contains invalid escape sequences, or is not a JSON object.
pub fn parse_json_map(src: &str) -> Result<HashMap<String, String>, JsonError> {
    let mut cursor = 0;
    skip_whitespace(src, &mut cursor);

    let bytes = src.as_bytes();
    if cursor >= bytes.len() || bytes[cursor] != b'{' {
        return Err(JsonError::ExpectedObject);
    }
    cursor += 1;

    let mut map = HashMap::with_capacity(32);

    loop {
        skip_whitespace(src, &mut cursor);
        if cursor >= bytes.len() {
            return Err(JsonError::UnexpectedEof);
        }

        if bytes[cursor] == b'}' {
            cursor += 1;
            break;
        }

        let key = parse_string(src, &mut cursor)?;
        skip_whitespace(src, &mut cursor);

        if cursor >= bytes.len() || bytes[cursor] != b':' {
            return Err(JsonError::ExpectedColon);
        }
        cursor += 1;
        skip_whitespace(src, &mut cursor);

        if cursor >= bytes.len() {
            return Err(JsonError::UnexpectedEof);
        }

        let value = if bytes[cursor] == b'"' {
            parse_string(src, &mut cursor)?
        } else {
            let start = cursor;
            while cursor < bytes.len()
                && !bytes[cursor].is_ascii_whitespace()
                && bytes[cursor] != b','
                && bytes[cursor] != b'}'
            {
                cursor += 1;
            }
            if start == cursor {
                return Err(JsonError::ExpectedValue);
            }
            let token = &src[start..cursor];
            match token {
                "true" => "true".to_string(),
                "false" => "false".to_string(),
                "null" => "null".to_string(),
                num if is_valid_json_number(num.as_bytes()) => num.to_string(),
                _ => return Err(JsonError::InvalidNumber),
            }
        };

        map.insert(key, value);

        skip_whitespace(src, &mut cursor);
        if cursor >= bytes.len() {
            return Err(JsonError::UnexpectedEof);
        }

        if bytes[cursor] == b',' {
            cursor += 1;
        } else if bytes[cursor] == b'}' {
            cursor += 1;
            break;
        } else {
            return Err(JsonError::ExpectedCommaOrClosingBrace);
        }
    }

    skip_whitespace(src, &mut cursor);
    Ok(map)
}

/// Parses a JSON quoted string literal with escape sequence and surrogate pair resolution.
fn parse_string(src: &str, cursor: &mut usize) -> Result<String, JsonError> {
    let bytes = src.as_bytes();
    if *cursor >= bytes.len() || bytes[*cursor] != b'"' {
        return Err(JsonError::ExpectedKey);
    }
    *cursor += 1;

    let mut out = String::with_capacity(32);
    while *cursor < bytes.len() {
        let chunk_start = *cursor;
        while *cursor < bytes.len() && bytes[*cursor] != b'"' && bytes[*cursor] != b'\\' {
            *cursor += 1;
        }

        if chunk_start < *cursor {
            out.push_str(&src[chunk_start..*cursor]);
        }

        if *cursor >= bytes.len() {
            return Err(JsonError::UnexpectedEof);
        }

        match bytes[*cursor] {
            b'"' => {
                *cursor += 1;
                return Ok(out);
            }
            b'\\' => {
                *cursor += 1;
                if *cursor >= bytes.len() {
                    return Err(JsonError::UnexpectedEof);
                }
                let esc = bytes[*cursor];
                *cursor += 1;
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\x08'),
                    b'f' => out.push('\x0C'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let cp = parse_hex4(bytes, cursor)?;
                        if (0xD800..=0xDBFF).contains(&cp) {
                            if *cursor + 2 > bytes.len()
                                || bytes[*cursor] != b'\\'
                                || bytes[*cursor + 1] != b'u'
                            {
                                return Err(JsonError::InvalidUnicodeEscape);
                            }
                            *cursor += 2;
                            let low_cp = parse_hex4(bytes, cursor)?;
                            if !(0xDC00..=0xDFFF).contains(&low_cp) {
                                return Err(JsonError::InvalidUnicodeEscape);
                            }
                            let scalar = 0x10000
                                + (((cp as u32 - 0xD800) << 10) | (low_cp as u32 - 0xDC00));
                            let ch = char::from_u32(scalar)
                                .ok_or(JsonError::InvalidUnicodeEscape)?;
                            out.push(ch);
                        } else if (0xDC00..=0xDFFF).contains(&cp) {
                            return Err(JsonError::InvalidUnicodeEscape);
                        } else {
                            let ch =
                                char::from_u32(cp as u32).ok_or(JsonError::InvalidUnicodeEscape)?;
                            out.push(ch);
                        }
                    }
                    _ => return Err(JsonError::InvalidEscapeSequence),
                }
            }
            _ => unreachable!(),
        }
    }

    Err(JsonError::UnexpectedEof)
}

/// Skips ASCII whitespace characters advancing the cursor.
fn skip_whitespace(src: &str, cursor: &mut usize) {
    let bytes = src.as_bytes();
    while *cursor < bytes.len() && bytes[*cursor].is_ascii_whitespace() {
        *cursor += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_map_simple() {
        let json = r#"{"name": "Doc2Flow", "version": "0.9.4"}"#;
        let map = parse_json_map(json).expect("valid json object");
        assert_eq!(map.get("name").unwrap(), "Doc2Flow");
        assert_eq!(map.get("version").unwrap(), "0.9.4");
    }

    #[test]
    fn test_parse_json_map_empty() {
        let json = "{}";
        let map = parse_json_map(json).expect("empty json object");
        assert!(map.is_empty());

        let json_spaced = "   {   \n\t  }  ";
        let map_spaced = parse_json_map(json_spaced).expect("empty spaced object");
        assert!(map_spaced.is_empty());
    }

    #[test]
    fn test_parse_json_map_escapes() {
        let json = r#"{
            "escaped_quote": "He said \"hello\"",
            "escaped_backslash": "C:\\Program Files\\App",
            "escaped_newline": "Line 1\nLine 2",
            "escaped_tab": "Col1\tCol2",
            "escaped_slash": "http:\/\/example.com",
            "escaped_controls": "back\bform\ffeed\rcarriage",
            "unicode_escape": "Unicode sample \u2713",
            "embedded_null": "Null \u0000 byte"
        }"#;
        let map = parse_json_map(json).expect("valid escaped json");
        assert_eq!(map.get("escaped_quote").unwrap(), "He said \"hello\"");
        assert_eq!(
            map.get("escaped_backslash").unwrap(),
            "C:\\Program Files\\App"
        );
        assert_eq!(map.get("escaped_newline").unwrap(), "Line 1\nLine 2");
        assert_eq!(map.get("escaped_tab").unwrap(), "Col1\tCol2");
        assert_eq!(map.get("escaped_slash").unwrap(), "http://example.com");
        assert_eq!(
            map.get("escaped_controls").unwrap(),
            "back\x08form\x0Cfeed\rcarriage"
        );
        assert_eq!(map.get("unicode_escape").unwrap(), "Unicode sample ✓");
        assert_eq!(map.get("embedded_null").unwrap(), "Null \0 byte");
    }

    #[test]
    fn test_parse_json_map_utf16_surrogate_pairs() {
        let json = r#"{
            "emoji_face": "\uD83D\uDE00",
            "emoji_crab": "\uD83E\uDD80",
            "emoji_rocket": "\uD83D\uDE80",
            "emoji_party": "\uD83C\uDF89",
            "mixed": "Hello \uD83D\uDE00 World \uD83E\uDD80!",
            "min_surrogate": "\uD800\uDC00",
            "max_surrogate": "\uDBFF\uDFFF"
        }"#;
        let map = parse_json_map(json).expect("valid surrogate pairs");
        assert_eq!(map.get("emoji_face").unwrap(), "😀");
        assert_eq!(map.get("emoji_crab").unwrap(), "🦀");
        assert_eq!(map.get("emoji_rocket").unwrap(), "🚀");
        assert_eq!(map.get("emoji_party").unwrap(), "🎉");
        assert_eq!(map.get("mixed").unwrap(), "Hello 😀 World 🦀!");
        assert_eq!(map.get("min_surrogate").unwrap(), "\u{10000}");
        assert_eq!(map.get("max_surrogate").unwrap(), "\u{10FFFF}");
    }

    #[test]
    fn test_parse_json_map_primitive_values() {
        let json = r#"{
            "count": 42,
            "zero": 0,
            "neg_zero": -0,
            "negative": -100,
            "float": 3.14159,
            "neg_float": -0.001,
            "exp_upper": 1E6,
            "exp_lower": 1e6,
            "exp_pos": 1e+6,
            "exp_neg": 1e-6,
            "neg_exp": -2.5e+3,
            "enabled": true,
            "disabled": false,
            "extra": null
        }"#;
        let map = parse_json_map(json).expect("valid primitives");
        assert_eq!(map.get("count").unwrap(), "42");
        assert_eq!(map.get("zero").unwrap(), "0");
        assert_eq!(map.get("neg_zero").unwrap(), "-0");
        assert_eq!(map.get("negative").unwrap(), "-100");
        assert_eq!(map.get("float").unwrap(), "3.14159");
        assert_eq!(map.get("neg_float").unwrap(), "-0.001");
        assert_eq!(map.get("exp_upper").unwrap(), "1E6");
        assert_eq!(map.get("exp_lower").unwrap(), "1e6");
        assert_eq!(map.get("exp_pos").unwrap(), "1e+6");
        assert_eq!(map.get("exp_neg").unwrap(), "1e-6");
        assert_eq!(map.get("neg_exp").unwrap(), "-2.5e+3");
        assert_eq!(map.get("enabled").unwrap(), "true");
        assert_eq!(map.get("disabled").unwrap(), "false");
        assert_eq!(map.get("extra").unwrap(), "null");
    }

    #[test]
    fn test_parse_json_map_rejects_malformed_numbers() {
        assert_eq!(
            parse_json_map(r#"{"key": 12.34.56}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": 12e99e}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": +123}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": 0123}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": -0123}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": .5}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": 1.}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": 1.e2}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": 1e}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": 1e+}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": 1e-}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": -}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": --1}"#),
            Err(JsonError::InvalidNumber)
        );
    }

    #[test]
    fn test_parse_json_map_rejects_barewords() {
        assert_eq!(
            parse_json_map(r#"{"key": bareword}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": true_value}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": false_flag}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": null_pointer}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": undefined}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": NaN}"#),
            Err(JsonError::InvalidNumber)
        );
        assert_eq!(
            parse_json_map(r#"{"key": Infinity}"#),
            Err(JsonError::InvalidNumber)
        );
    }

    #[test]
    fn test_parse_json_map_errors() {
        assert_eq!(parse_json_map(""), Err(JsonError::ExpectedObject));
        assert_eq!(parse_json_map("[1, 2]"), Err(JsonError::ExpectedObject));
        assert_eq!(parse_json_map("{"), Err(JsonError::UnexpectedEof));
        assert_eq!(
            parse_json_map("{ 123: \"val\" }"),
            Err(JsonError::ExpectedKey)
        );
        assert_eq!(
            parse_json_map("{\"key\" \"val\"}"),
            Err(JsonError::ExpectedColon)
        );
        assert_eq!(
            parse_json_map("{\"key\": \"val\" \"key2\": \"val2\"}"),
            Err(JsonError::ExpectedCommaOrClosingBrace)
        );
        assert_eq!(
            parse_json_map(r#"{"key": "invalid \x escape"}"#),
            Err(JsonError::InvalidEscapeSequence)
        );
        assert_eq!(
            parse_json_map(r#"{"key": "invalid \u12G4 unicode"}"#),
            Err(JsonError::InvalidUnicodeEscape)
        );
        assert_eq!(
            parse_json_map(r#"{"key": "unpaired high \uD83D"}"#),
            Err(JsonError::InvalidUnicodeEscape)
        );
        assert_eq!(
            parse_json_map(r#"{"key": "unpaired high \uD83D followed by text"}"#),
            Err(JsonError::InvalidUnicodeEscape)
        );
        assert_eq!(
            parse_json_map(r#"{"key": "unpaired low \uDE00"}"#),
            Err(JsonError::InvalidUnicodeEscape)
        );
        assert_eq!(
            parse_json_map(r#"{"key": "invalid second surrogate \uD83D\u1234"}"#),
            Err(JsonError::InvalidUnicodeEscape)
        );
        assert_eq!(
            parse_json_map(r#"{"key": }"#),
            Err(JsonError::ExpectedValue)
        );
    }

    #[test]
    fn test_is_valid_json_number_edge_cases() {
        assert!(!is_valid_json_number(b""));
        assert!(!is_valid_json_number(b"-"));
        assert!(!is_valid_json_number(b"+1"));
        assert!(!is_valid_json_number(b"01"));
        assert!(!is_valid_json_number(b"-01"));
        assert!(!is_valid_json_number(b".5"));
        assert!(!is_valid_json_number(b"1."));
        assert!(!is_valid_json_number(b"1e"));
        assert!(!is_valid_json_number(b"1e+"));
        assert!(!is_valid_json_number(b"1e-"));
        assert!(!is_valid_json_number(b"1e+a"));

        assert!(is_valid_json_number(b"0"));
        assert!(is_valid_json_number(b"-0"));
        assert!(is_valid_json_number(b"123"));
        assert!(is_valid_json_number(b"-123"));
        assert!(is_valid_json_number(b"0.123"));
        assert!(is_valid_json_number(b"-0.123"));
        assert!(is_valid_json_number(b"123.456"));
        assert!(is_valid_json_number(b"1e10"));
        assert!(is_valid_json_number(b"1E10"));
        assert!(is_valid_json_number(b"1e+10"));
        assert!(is_valid_json_number(b"1e-10"));
        assert!(is_valid_json_number(b"-123.456e+78"));
    }
}