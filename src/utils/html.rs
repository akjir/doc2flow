/// Extracts attribute bounds and unquoted value from an HTML/XML tag slice without heap allocations.
///
/// Returns `Some((attr_start, attr_end, attr_value))` where:
/// - `attr_start`: byte offset in `tag` where the attribute begins.
/// - `attr_end`: byte offset in `tag` immediately following the attribute definition.
/// - `attr_value`: unquoted value slice of the attribute.
///
/// Supports double quotes, single quotes, whitespace around `=`, unquoted values, boolean
/// attributes, multiline attributes, and escaped quotes. Loop bounds advance systematically
/// using standard string slice operations without redundant scanning passes.
///
/// # Examples
///
/// ```
/// use doc2flow::utils::extract_attribute;
///
/// let tag = r#"<img src="photo.png" alt="Demo">"#;
/// let (start, end, val) = extract_attribute(tag, "src").unwrap();
/// assert_eq!(&tag[start..end], r#"src="photo.png""#);
/// assert_eq!(val, "photo.png");
/// ```
#[must_use]
pub fn extract_attribute<'a>(tag: &'a str, attr_name: &str) -> Option<(usize, usize, &'a str)> {
    let tag_trimmed = tag.trim_start();
    if !tag_trimmed.starts_with('<') {
        return None;
    }

    let tag_content = tag_trimmed.strip_prefix('<')?;
    let tag_content = tag_content.strip_prefix('/').unwrap_or(tag_content);

    let tag_name_len = tag_content
        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .unwrap_or(tag_content.len());

    let mut current_offset = tag.len() - tag_content.len() + tag_name_len;

    while current_offset < tag.len() {
        let remainder = &tag[current_offset..];
        let trimmed_remainder = remainder.trim_start();
        let ws_len = remainder.len() - trimmed_remainder.len();
        current_offset += ws_len;

        if current_offset >= tag.len() {
            break;
        }

        let slice = &tag[current_offset..];
        if slice.starts_with('>') || slice.starts_with("/>") {
            break;
        }

        let attr_start = current_offset;

        let name_len = slice
            .find(|c: char| c.is_whitespace() || c == '=' || c == '/' || c == '>')
            .unwrap_or(slice.len());

        if name_len == 0 {
            let next_char_len = slice.chars().next().map_or(1, |c| c.len_utf8());
            current_offset += next_char_len;
            continue;
        }

        let name = &slice[..name_len];
        current_offset += name_len;

        let after_name = &tag[current_offset..];
        let after_name_trimmed = after_name.trim_start();
        current_offset += after_name.len() - after_name_trimmed.len();

        let is_target = name.eq_ignore_ascii_case(attr_name);

        if current_offset < tag.len() && tag[current_offset..].starts_with('=') {
            current_offset += 1;
            let after_eq = &tag[current_offset..];
            let after_eq_trimmed = after_eq.trim_start();
            current_offset += after_eq.len() - after_eq_trimmed.len();

            if current_offset < tag.len() {
                let val_slice = &tag[current_offset..];
                if let Some(quote) = val_slice.chars().next().filter(|&c| c == '"' || c == '\'') {
                    let quote_len = quote.len_utf8();
                    let val_start_offset = current_offset + quote_len;
                    let inner_slice = &tag[val_start_offset..];

                    let mut escaped = false;
                    let mut found_end = None;
                    for (idx, ch) in inner_slice.char_indices() {
                        if escaped {
                            escaped = false;
                        } else if ch == '\\' {
                            escaped = true;
                        } else if ch == quote {
                            found_end = Some(idx);
                            break;
                        }
                    }

                    if let Some(end_idx) = found_end {
                        let val = &inner_slice[..end_idx];
                        let attr_end = val_start_offset + end_idx + quote_len;
                        if is_target {
                            return Some((attr_start, attr_end, val));
                        }
                        current_offset = attr_end;
                    } else {
                        let val = inner_slice;
                        let attr_end = tag.len();
                        if is_target {
                            return Some((attr_start, attr_end, val));
                        }
                        current_offset = attr_end;
                    }
                } else {
                    let val_len = val_slice
                        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
                        .unwrap_or(val_slice.len());
                    let val = &val_slice[..val_len];
                    let attr_end = current_offset + val_len;
                    if is_target {
                        return Some((attr_start, attr_end, val));
                    }
                    current_offset = attr_end;
                }
            } else if is_target {
                return Some((attr_start, current_offset, ""));
            }
        } else if is_target {
            return Some((attr_start, current_offset, ""));
        }
    }

    None
}
