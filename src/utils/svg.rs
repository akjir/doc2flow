
/// Cleans and minifies SVG content by stripping declarations, comments, and editor metadata.
///
/// Strips XML processing instructions, DOCTYPE declarations, HTML/XML comments, and editor-specific
/// namespaces (`inkscape:*`, `sodipodi:*`, `<metadata>`, `<defs/>`) while streaming valid attributes
/// and CDATA blocks into the output buffer.
///
/// # Examples
///
/// ```
/// use doc2flow::utils::clean_svg;
///
/// let svg = r#"<?xml version="1.0"?><svg width="100" height="100"><circle cx="50" cy="50" r="40"/></svg>"#;
/// let cleaned = clean_svg(svg);
/// assert!(cleaned.starts_with("<svg"));
/// assert!(!cleaned.contains("<?xml"));
/// ```
#[must_use]
pub fn clean_svg(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut rest = input.trim_start();

    while !rest.is_empty() {
        if let Some(next) = skip_processing_instruction(rest) {
            rest = next.trim_start();
            continue;
        }
        if let Some(next) = skip_doctype(rest) {
            rest = next.trim_start();
            continue;
        }
        if let Some(next) = skip_comment(rest) {
            rest = next;
            continue;
        }
        if let Some((cdata, next)) = parse_cdata(rest) {
            result.push_str(cdata);
            rest = next;
            continue;
        }

        if let Some(tag_start) = rest.find('<') {
            let text_before = rest[..tag_start].trim();
            if !text_before.is_empty() {
                result.push_str(text_before);
            }

            let tag_rest = &rest[tag_start..];
            if tag_start > 0
                && (tag_rest.starts_with("<!--")
                    || tag_rest.starts_with("<?")
                    || tag_rest.starts_with("<!DOCTYPE")
                    || tag_rest.starts_with("<!doctype")
                    || tag_rest.starts_with("<![CDATA["))
            {
                rest = tag_rest;
                continue;
            }

            if let Some(tag_end) = tag_rest.find('>') {
                let full_tag = &tag_rest[..=tag_end];
                rest = &tag_rest[tag_end + 1..];

                if let Some(after_skip) = skip_editor_metadata_tag(full_tag, rest) {
                    rest = after_skip;
                    continue;
                }

                write_cleaned_tag(&mut result, full_tag);
            } else {
                result.push_str(tag_rest);
                break;
            }
        } else {
            let text = rest.trim();
            if !text.is_empty() {
                result.push_str(text);
            }
            break;
        }
    }

    result
}

/// Parses a CDATA block (`<![CDATA[ ... ]]>`), returning the block slice and remaining input.
pub(crate) fn parse_cdata(input: &str) -> Option<(&str, &str)> {
    if input.starts_with("<![CDATA[") {
        let end = input.find("]]>")?;
        Some((&input[..end + 3], &input[end + 3..]))
    } else {
        None
    }
}

fn skip_comment(input: &str) -> Option<&str> {
    if input.starts_with("<!--") {
        let end = input.find("-->")?;
        Some(&input[end + 3..])
    } else {
        None
    }
}

/// Skips DOCTYPE declarations (`<!DOCTYPE ... >` or `<!doctype ... >`).
fn skip_doctype(input: &str) -> Option<&str> {
    if input.starts_with("<!DOCTYPE") || input.starts_with("<!doctype") {
        let end = input.find('>')?;
        Some(&input[end + 1..])
    } else {
        None
    }
}

/// Skips editor-specific metadata tags like `<sodipodi:namedview>`, `<metadata>`, or self-closing `<defs/>`.
fn skip_editor_metadata_tag<'a>(full_tag: &str, rest: &'a str) -> Option<&'a str> {
    let is_closing = full_tag.starts_with("</");
    let is_self_closing = full_tag.ends_with("/>");
    let tag_inner = match (is_closing, is_self_closing) {
        (true, _) => full_tag.get(2..full_tag.len().saturating_sub(1))?.trim(),
        (false, true) => full_tag.get(1..full_tag.len().saturating_sub(2))?.trim(),
        (false, false) => full_tag.get(1..full_tag.len().saturating_sub(1))?.trim(),
    };

    let tag_name = tag_inner
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches('/');

    let is_editor_tag = tag_name.starts_with("sodipodi:") || tag_name == "metadata";

    if is_editor_tag {
        if !is_closing
            && !is_self_closing
            && let Some(pos) = rest.find("</")
        {
            let after = &rest[pos + 2..];
            if let Some(close_tag_end) = after.find('>') {
                let inside = after[..close_tag_end].trim();
                if inside == tag_name {
                    return Some(&after[close_tag_end + 1..]);
                }
            }
        }
        return Some(rest);
    }

    if !is_closing && tag_name == "defs" && is_self_closing {
        return Some(rest);
    }

    None
}

/// Skips XML processing instructions (`<? ... ?>`).
fn skip_processing_instruction(input: &str) -> Option<&str> {
    if input.starts_with("<?") {
        let end = input.find("?>")?;
        Some(&input[end + 2..])
    } else {
        None
    }
}

/// Writes a cleaned tag and its filtered attributes into the output buffer.
fn write_cleaned_tag(out: &mut String, full_tag: &str) {
    let is_closing = full_tag.starts_with("</");
    let is_self_closing = full_tag.ends_with("/>");
    let tag_inner = match (is_closing, is_self_closing) {
        (true, _) => full_tag
            .get(2..full_tag.len().saturating_sub(1))
            .unwrap_or("")
            .trim(),
        (false, true) => full_tag
            .get(1..full_tag.len().saturating_sub(2))
            .unwrap_or("")
            .trim(),
        (false, false) => full_tag
            .get(1..full_tag.len().saturating_sub(1))
            .unwrap_or("")
            .trim(),
    };

    let tag_name = tag_inner
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches('/');

    if is_closing {
        out.push_str("</");
        out.push_str(tag_name);
        out.push('>');
    } else {
        out.push('<');
        out.push_str(tag_name);
        write_cleaned_tag_attributes(out, tag_name, tag_inner);
        if is_self_closing {
            out.push_str("/>");
        } else {
            out.push('>');
        }
    }
}

/// Helper function to clean attributes of an XML tag, streaming valid attributes into buffer without allocations.
fn write_cleaned_tag_attributes(out: &mut String, tag_name: &str, tag_inner: &str) {
    let tag_name_len = tag_name.len();
    let attr_str = if tag_inner.len() > tag_name_len {
        tag_inner[tag_name_len..].trim_start()
    } else {
        ""
    };

    let mut current_offset = 0;

    while current_offset < attr_str.len() {
        let remainder = &attr_str[current_offset..];
        let trimmed = remainder.trim_start();
        let ws_len = remainder.len() - trimmed.len();
        current_offset += ws_len;

        if current_offset >= attr_str.len() {
            break;
        }

        let slice = &attr_str[current_offset..];
        if slice.starts_with('>') || slice.starts_with('/') {
            break;
        }

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

        let after_name = &attr_str[current_offset..];
        let after_name_trimmed = after_name.trim_start();
        current_offset += after_name.len() - after_name_trimmed.len();

        let mut val = "";
        let mut has_equals = false;

        if current_offset < attr_str.len() && attr_str[current_offset..].starts_with('=') {
            has_equals = true;
            current_offset += 1;
            let after_eq = &attr_str[current_offset..];
            let after_eq_trimmed = after_eq.trim_start();
            current_offset += after_eq.len() - after_eq_trimmed.len();

            if current_offset < attr_str.len() {
                let val_slice = &attr_str[current_offset..];
                if let Some(quote) = val_slice.chars().next().filter(|&c| c == '"' || c == '\'') {
                    let quote_len = quote.len_utf8();
                    let val_start_offset = current_offset + quote_len;
                    let inner_slice = &attr_str[val_start_offset..];

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
                        val = &inner_slice[..end_idx];
                        current_offset = val_start_offset + end_idx + quote_len;
                    } else {
                        val = inner_slice;
                        current_offset = attr_str.len();
                    }
                } else {
                    let val_len = val_slice
                        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
                        .unwrap_or(val_slice.len());
                    val = &val_slice[..val_len];
                    current_offset += val_len;
                }
            }
        }

        let is_generic_svg_id = name == "id"
            && val.starts_with("svg")
            && val.len() > 3
            && val[3..].bytes().all(|b| b.is_ascii_digit());

        let should_remove = name.starts_with("inkscape:")
            || name.starts_with("sodipodi:")
            || name.starts_with("xmlns:inkscape")
            || name.starts_with("xmlns:sodipodi")
            || name == "xmlns:svg"
            || (tag_name == "svg" && (name == "version" || is_generic_svg_id))
            || (tag_name == "g" && name == "id" && val.starts_with("layer"));

        if !should_remove {
            out.push(' ');
            out.push_str(name);
            if !val.is_empty() || has_equals {
                out.push_str("=\"");
                out.push_str(val);
                out.push('"');
            }
        }
    }
}
