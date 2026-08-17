//! Image processing, SVG cleaning, and WebP compression engine.

use std::borrow::Cow;
use std::io::Cursor;
use std::path::Path;

use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat};

use crate::core::error::{DiagnosticError, Error, Result, build_caret_annotation};
use crate::core::io;
use crate::utils::{guess_mime_type, to_base64_data_uri};

/// Allowed image file extensions for image resource checking.
const ALLOWED_IMAGE_EXTENSIONS: [&str; 11] = [
    "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "tif", "tiff", "webp",
];

/// Maximum allowed size in bytes for a local image embedded into HTML (250 KB).
pub const MAX_IMAGE_SIZE_BYTES: u64 = 250 * 1024;

/// Cleans and minifies SVG content by stripping declarations, comments, and editor metadata.
///
/// Strips XML processing instructions, DOCTYPE declarations, HTML/XML comments, and editor-specific
/// namespaces (`inkscape:*`, `sodipodi:*`, `<metadata>`, `<defs/>`) while streaming valid attributes
/// and CDATA blocks into the output buffer.
///
/// # Examples
///
/// ```
/// use doc2flow::core::image::clean_svg;
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
/// use doc2flow::core::image::extract_attribute;
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

/// Finds line number, column number, and line snippet in Markdown for an image source string.
#[must_use]
pub fn find_markdown_location<'a>(
    md_content: Option<&'a str>,
    src_val: &'a str,
) -> (usize, usize, &'a str) {
    if let Some(md) = md_content {
        for (idx, line) in md.lines().enumerate() {
            if let Some(col_idx) = line.find(src_val) {
                return (idx + 1, col_idx + 1, line);
            }
        }
        if let Some(file_name) = Path::new(src_val).file_name().and_then(|f| f.to_str()) {
            for (idx, line) in md.lines().enumerate() {
                if let Some(col_idx) = line.find(file_name) {
                    return (idx + 1, col_idx + 1, line);
                }
            }
        }
    }
    (1, 1, "")
}

/// Checks if a file path or URL points to an image resource based on extension or MIME type.
#[must_use]
pub fn is_image_source(src: &str, base_dir: Option<&Path>) -> bool {
    if let Some(ext) = Path::new(src).extension().and_then(|e| e.to_str()) {
        if ALLOWED_IMAGE_EXTENSIONS
            .iter()
            .any(|&allowed| allowed.eq_ignore_ascii_case(ext))
        {
            return true;
        }
        if is_remote_or_data_uri(src) {
            return false;
        }
        let path = Path::new(src);
        let resolved = io::resolve_path(path, base_dir);
        return resolved
            .as_deref()
            .map(guess_mime_type)
            .is_some_and(|mime| mime.starts_with("image/"));
    }

    if src.starts_with("data:image/") {
        return true;
    }
    if src.starts_with("data:") {
        return false;
    }
    if src.starts_with("http://") || src.starts_with("https://") {
        return true;
    }

    let path = Path::new(src);
    let resolved = io::resolve_path(path, base_dir);
    resolved
        .as_deref()
        .map(guess_mime_type)
        .is_some_and(|mime| mime.starts_with("image/"))
}

/// Checks if an image source is a remote URL or existing Base64 Data URI.
#[must_use]
pub fn is_remote_or_data_uri(src: &str) -> bool {
    src.starts_with("data:") || src.starts_with("http://") || src.starts_with("https://")
}

/// Constructs a compiler-style [`DiagnosticError`] for local images exceeding the size limit.
#[must_use]
pub fn make_image_too_large_error<'a>(
    file_path: &'a str,
    line_no: usize,
    col_no: usize,
    line_snippet: &'a str,
    src_val: &'a str,
    size_bytes: u64,
) -> Error {
    let size_kb = size_bytes as f64 / 1024.0;
    let line_len = line_snippet.len().max(1);
    let carets = build_caret_annotation(col_no, src_val.len(), line_len);

    DiagnosticError {
        message: Cow::Owned(format!(
            "image '{src_val}' exceeds maximum allowed size of 250 KB ({size_kb:.1} KB)"
        )),
        file_path: Cow::Borrowed(file_path),
        line_number: line_no,
        col_number: col_no,
        line_snippet: Cow::Borrowed(line_snippet),
        annotation_carets: carets,
        annotation_text: Cow::Owned(format!(
            "local image size ({size_kb:.1} KB) exceeds 250 KB limit"
        )),
        help_text: Cow::Owned(format!(
            "reduce image resolution or compress '{src_val}' below 250 KB before embedding."
        )),
    }
    .into()
}

/// Parses CDATA sections (`<![CDATA[ ... ]]>`), returning the section content and the remainder.
fn parse_cdata(input: &str) -> Option<(&str, &str)> {
    if input.starts_with("<![CDATA[") {
        let end = input.find("]]>")?;
        Some((&input[..end + 3], &input[end + 3..]))
    } else {
        None
    }
}

/// Resizes a local image file and converts it to WebP format to stay within the 250 KB target.
///
/// Calculates target dimensions mathematically based on area ratio, encodes to WebP in a maximum
/// of two passes using triangle filtering, and saves a `.webp` copy alongside the original image.
///
/// # Errors
///
/// Returns an [`Error`] if opening or encoding the image fails.
pub fn process_and_encode_image_as_webp(image_path: &Path) -> Result<String> {
    let img = image::open(image_path)?;
    let (orig_w, orig_h) = img.dimensions();
    let file_size = io::get_file_size(image_path).unwrap_or(MAX_IMAGE_SIZE_BYTES + 1);

    let scale_ratio = (MAX_IMAGE_SIZE_BYTES as f64 / file_size as f64)
        .sqrt()
        .min(0.95);
    let mut target_w = ((orig_w as f64 * scale_ratio) as u32).max(100);
    let mut target_h = ((orig_h as f64 * scale_ratio) as u32).max(100);

    let mut buffer = Vec::with_capacity(MAX_IMAGE_SIZE_BYTES as usize);

    // Pass 1: Initial scaling based on file size estimation
    let resized_img = if target_w < orig_w || target_h < orig_h {
        img.resize(target_w, target_h, FilterType::Triangle)
    } else {
        img.clone()
    };

    let mut final_w = resized_img.width();
    let mut final_h = resized_img.height();

    let mut cursor = Cursor::new(&mut buffer);
    resized_img.write_to(&mut cursor, ImageFormat::WebP)?;

    // Pass 2: Direct area-to-bytes ratio recalculation if Pass 1 exceeds target size (max 2 passes)
    if (buffer.len() as u64) > MAX_IMAGE_SIZE_BYTES && (target_w > 100 || target_h > 100) {
        let pass2_scale =
            ((MAX_IMAGE_SIZE_BYTES as f64 / buffer.len() as f64).sqrt() * 0.92).min(0.95);
        target_w = ((target_w as f64 * pass2_scale) as u32).max(100);
        target_h = ((target_h as f64 * pass2_scale) as u32).max(100);

        let pass2_img = img.resize(target_w, target_h, FilterType::Triangle);
        final_w = pass2_img.width();
        final_h = pass2_img.height();

        buffer.clear();
        let mut cursor = Cursor::new(&mut buffer);
        pass2_img.write_to(&mut cursor, ImageFormat::WebP)?;
    }

    let webp_path = image_path.with_extension("webp");
    let _ = io::write_file(&webp_path, &buffer);

    let orig_kb = file_size as f64 / 1024.0;
    let new_kb = buffer.len() as f64 / 1024.0;
    eprintln!(
        "Resized image '{}': {orig_w}x{orig_h} ({orig_kb:.1} KB) -> {final_w}x{final_h} WebP ({new_kb:.1} KB)",
        image_path.display(),
    );

    Ok(to_base64_data_uri("image/webp", &buffer))
}

/// Asks user interactively via stderr/stdin whether to resize/convert an image that exceeds 250 KB.
#[must_use]
pub fn prompt_user_for_resizing(src_val: &str, size_bytes: u64) -> bool {
    let size_kb = size_bytes as f64 / 1024.0;
    io::prompt_user_yes_no(&format!(
        "\nWarning: Image '{src_val}' ({size_kb:.1} KB) exceeds the 250 KB limit.\nDo you want to resize and convert it to WebP? [y/N]: "
    ))
}

/// Reads, validates size, and encodes a local image file to a Base64 Data URI.
///
/// # Errors
///
/// Returns a [`DiagnosticError`] if the image exceeds the maximum size limit and is not scaled.
pub fn resolve_or_encode_image(
    resolved: &Path,
    src_val: &str,
    auto_scale: bool,
    md_content: Option<&str>,
    file_name: Option<&str>,
) -> Result<String> {
    let bytes = match io::read_file_bytes(resolved) {
        Ok(b) => b,
        Err(_) => return Ok(src_val.to_string()),
    };

    let size = bytes.len() as u64;
    if size > MAX_IMAGE_SIZE_BYTES {
        let should_scale = auto_scale || prompt_user_for_resizing(src_val, size);
        let scaled_uri = if should_scale {
            process_and_encode_image_as_webp(resolved).ok()
        } else {
            None
        };

        if let Some(u) = scaled_uri {
            Ok(u)
        } else {
            let (line_no, col_no, line_snippet) = find_markdown_location(md_content, src_val);
            let f_name = file_name.unwrap_or("<input>");
            Err(make_image_too_large_error(
                f_name,
                line_no,
                col_no,
                line_snippet,
                src_val,
                size,
            ))
        }
    } else {
        let mime = guess_mime_type(resolved);
        let bytes_to_encode = if mime == "image/svg+xml" {
            if let Ok(utf8_str) = std::str::from_utf8(&bytes) {
                clean_svg(utf8_str).into_bytes()
            } else {
                bytes
            }
        } else {
            bytes
        };

        Ok(to_base64_data_uri(mime, &bytes_to_encode))
    }
}

/// Skips XML comments (`<!-- ... -->`).
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
        (true, _) => full_tag[2..full_tag.len() - 1].trim(),
        (false, true) => full_tag[1..full_tag.len() - 2].trim(),
        (false, false) => full_tag[1..full_tag.len() - 1].trim(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attribute_src_and_alt() {
        let tag = "<img src=\"images/pic.png\" alt=\"System Diagram\">";
        let (start, end, val) = extract_attribute(tag, "src").unwrap();
        assert_eq!(&tag[start..end], "src=\"images/pic.png\"");
        assert_eq!(val, "images/pic.png");

        let (_, _, alt_val) = extract_attribute(tag, "alt").unwrap();
        assert_eq!(alt_val, "System Diagram");
    }

    #[test]
    fn test_extract_attribute_single_quotes() {
        let tag = "<img src='images/pic.png' alt='System Diagram'>";
        let (start, end, val) = extract_attribute(tag, "src").unwrap();
        assert_eq!(&tag[start..end], "src='images/pic.png'");
        assert_eq!(val, "images/pic.png");
    }

    #[test]
    fn test_extract_attribute_quotes_whitespace_and_escapes() {
        let tag = r#"<img src="  images/pic.png  " alt="A \"complex\" diagram" data-extra='val\'s test' checked>"#;

        let (_, _, src_val) = extract_attribute(tag, "src").unwrap();
        assert_eq!(src_val, "  images/pic.png  ");

        let (_, _, alt_val) = extract_attribute(tag, "alt").unwrap();
        assert_eq!(alt_val, r#"A \"complex\" diagram"#);

        let (_, _, extra_val) = extract_attribute(tag, "data-extra").unwrap();
        assert_eq!(extra_val, r#"val\'s test"#);

        let (chk_start, chk_end, chk_val) = extract_attribute(tag, "checked").unwrap();
        assert_eq!(&tag[chk_start..chk_end], "checked");
        assert_eq!(chk_val, "");
    }

    #[test]
    fn test_extract_attribute_multiline_and_newlines_in_tag() {
        let tag =
            "<img\n  src=\"images/multiline.png\"\n  alt=\"System\nDiagram with Newlines\"\n/>";
        let (start, end, src_val) = extract_attribute(tag, "src").unwrap();
        assert_eq!(&tag[start..end], "src=\"images/multiline.png\"");
        assert_eq!(src_val, "images/multiline.png");

        let (_, _, alt_val) = extract_attribute(tag, "alt").unwrap();
        assert_eq!(alt_val, "System\nDiagram with Newlines");
    }

    #[test]
    fn test_extract_attribute_missing_and_empty() {
        let tag = "<img>";
        assert_eq!(extract_attribute(tag, "src"), None);

        let non_tag = "not a tag at all";
        assert_eq!(extract_attribute(non_tag, "src"), None);

        let empty = "";
        assert_eq!(extract_attribute(empty, "src"), None);
    }

    #[test]
    fn test_extract_attribute_malformed_tags() {
        let unclosed_quote = "<img src=\"images/unclosed.png";
        let (_, _, val) = extract_attribute(unclosed_quote, "src").unwrap();
        assert_eq!(val, "images/unclosed.png");

        let unquoted_val = "<img src=image.png alt=sample>";
        let (_, _, val) = extract_attribute(unquoted_val, "src").unwrap();
        assert_eq!(val, "image.png");
        let (_, _, alt) = extract_attribute(unquoted_val, "alt").unwrap();
        assert_eq!(alt, "sample");

        let tag_trailing_equals = "<img src=>";
        let (_, _, val) = extract_attribute(tag_trailing_equals, "src").unwrap();
        assert_eq!(val, "");
    }

    #[test]
    fn test_is_image_source_extensions() {
        assert!(is_image_source("image.png", None));
        assert!(is_image_source("photo.JPG", None));
        assert!(is_image_source("https://example.com/pic.webp", None));
        assert!(!is_image_source("manual.pdf", None));
        assert!(!is_image_source("archive.ZIP", None));
        assert!(!is_image_source("doc.docx", None));
        assert!(!is_image_source("music.mp3", None));
        assert!(is_image_source("data:image/png;base64,123", None));
        assert!(!is_image_source("data:application/pdf;base64,123", None));
        assert!(is_image_source("https://example.com/avatar", None));
    }

    #[test]
    fn test_is_remote_or_data_uri() {
        assert!(is_remote_or_data_uri("http://example.com/pic.png"));
        assert!(is_remote_or_data_uri("https://example.com/pic.png"));
        assert!(is_remote_or_data_uri("data:image/png;base64,abc"));
        assert!(!is_remote_or_data_uri("images/pic.png"));
        assert!(!is_remote_or_data_uri("../pic.png"));
    }

    #[test]
    fn test_find_markdown_location() {
        let md = "Line 1\nLine 2\n![Alt](images/photo.png)\nLine 4";
        let (line_no, col_no, snippet) = find_markdown_location(Some(md), "images/photo.png");
        assert_eq!(line_no, 3);
        assert_eq!(col_no, 8);
        assert_eq!(snippet, "![Alt](images/photo.png)");

        let (l0, c0, s0) = find_markdown_location(None, "images/photo.png");
        assert_eq!((l0, c0, s0), (1, 1, ""));
    }

    #[test]
    fn test_clean_svg_inkscape_clutter() {
        let raw_svg = r##"<?xml version="1.0" encoding="UTF-8"?>
<!-- Created with Inkscape (http://www.inkscape.org/) -->
<svg
   width="350"
   height="200"
   viewBox="0 0 175 100"
   version="1.1"
   id="svg1"
   inkscape:version="1.4.4"
   sodipodi:docname="drawing.svg"
   xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
   xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"
   xmlns="http://www.w3.org/2000/svg"
   xmlns:svg="http://www.w3.org/2000/svg">
  <sodipodi:namedview id="namedview1" pagecolor="#ffffff" />
  <metadata id="metadata1">Some metadata</metadata>
  <defs id="defs1" />
  <g inkscape:label="Layer 1" inkscape:groupmode="layer" id="layer1">
    <path d="M 10 10 L 20 20 Z" fill="#ffffff" id="path5" />
  </g>
</svg>"##;

        let cleaned = clean_svg(raw_svg);
        assert!(!cleaned.contains("inkscape"));
        assert!(!cleaned.contains("sodipodi"));
        assert!(!cleaned.contains("metadata"));
        assert!(!cleaned.contains("defs1"));
        assert!(!cleaned.contains("<?xml"));
        assert!(!cleaned.contains("<!--"));
        assert!(cleaned.contains("viewBox=\"0 0 175 100\""));
        assert!(cleaned.contains("fill=\"#ffffff\""));
        assert!(cleaned.contains("path"));
    }

    #[test]
    fn test_clean_svg_complex_cdata_multiline_and_escaped_quotes() {
        let complex_svg = r##"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<!-- Multi-line SVG with CDATA and escaped quotes -->
<svg viewBox="0 0 500 500" xmlns="http://www.w3.org/2000/svg">
  <style type="text/css">
    <![CDATA[
      .st0 { fill: #ff0000; stroke: #000000; }
    ]]>
  </style>
  <path
     d="M 10 10
        L 100 100
        Z"
     fill="#ff0000"
     aria-label="Escaped \"Quote\" Shape"
  />
  <text font-family='Arial, "Helvetica Neue", sans-serif'>Sample Text</text>
</svg>"##;

        let cleaned = clean_svg(complex_svg);
        assert!(!cleaned.contains("<?xml"));
        assert!(!cleaned.contains("<!DOCTYPE"));
        assert!(!cleaned.contains("<!-- Multi-line"));
        assert!(cleaned.contains("<![CDATA["));
        assert!(cleaned.contains(".st0 { fill: #ff0000; stroke: #000000; }"));
        assert!(cleaned.contains("]]>"));
        assert!(cleaned.contains("aria-label=\"Escaped \\\"Quote\\\" Shape\""));
        assert!(cleaned.contains("Sample Text"));
    }

    #[test]
    fn test_clean_svg_malformed_inputs() {
        let unclosed_comment = "<!-- unclosed comment without closing delimiter <svg></svg>";
        let cleaned1 = clean_svg(unclosed_comment);
        assert!(!cleaned1.is_empty());

        let unclosed_doctype = "<!DOCTYPE svg PUBLIC without closing";
        let cleaned2 = clean_svg(unclosed_doctype);
        assert_eq!(cleaned2, "<!DOCTYPE svg PUBLIC without closing");

        let unclosed_pi = "<?xml version='1.0' without question mark";
        let cleaned3 = clean_svg(unclosed_pi);
        assert_eq!(cleaned3, "<?xml version='1.0' without question mark");

        let unclosed_cdata = "<![CDATA[ some unclosed text";
        let cleaned4 = clean_svg(unclosed_cdata);
        assert_eq!(cleaned4, "<![CDATA[ some unclosed text");

        let unclosed_tag = "<svg width='100'";
        let cleaned5 = clean_svg(unclosed_tag);
        assert_eq!(cleaned5, "<svg width='100'");
    }

    #[test]
    fn test_make_image_too_large_error() {
        let err = make_image_too_large_error(
            "doc.md",
            12,
            16,
            "![Diagram](images/large_photo.png)",
            "images/large_photo.png",
            300 * 1024,
        );
        let err_str = err.to_string();
        assert!(err_str.contains("error: image 'images/large_photo.png' exceeds maximum allowed size of 250 KB (300.0 KB)"));
        assert!(err_str.contains("--> doc.md:12:16"));
        assert!(err_str.contains("12 | ![Diagram](images/large_photo.png)"));
        assert!(
            err_str
                .contains("^^^^^^^^^^^^^^^^^^^ local image size (300.0 KB) exceeds 250 KB limit")
        );
        assert!(err_str.contains("= help: reduce image resolution or compress 'images/large_photo.png' below 250 KB before embedding."));
    }

    #[test]
    fn test_process_and_encode_image_as_webp_under_target_size() {
        let temp_dir = std::env::temp_dir().join(format!("d2f_webp_test_{}", std::process::id()));
        let _ = io::create_dir_all(&temp_dir);
        let img_path = temp_dir.join("sample.png");

        let img_buf = image::RgbImage::new(100, 100);
        img_buf
            .save_with_format(&img_path, ImageFormat::Png)
            .expect("should save test png");

        let uri = process_and_encode_image_as_webp(&img_path).expect("encoding should succeed");
        assert!(uri.starts_with("data:image/webp;base64,"));

        let webp_path = temp_dir.join("sample.webp");
        assert!(io::path_exists(&webp_path));
        let webp_size = io::get_file_size(&webp_path).unwrap_or(0);
        assert!(webp_size > 0 && webp_size <= MAX_IMAGE_SIZE_BYTES);

        let _ = io::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_process_and_encode_image_as_webp_large_image_scaling() {
        let temp_dir =
            std::env::temp_dir().join(format!("d2f_webp_large_test_{}", std::process::id()));
        let _ = io::create_dir_all(&temp_dir);
        let img_path = temp_dir.join("large.png");

        let mut img_buf = image::RgbImage::new(800, 800);
        for (x, y, pixel) in img_buf.enumerate_pixels_mut() {
            *pixel = image::Rgb([(x % 255) as u8, (y % 255) as u8, ((x + y) % 255) as u8]);
        }
        img_buf
            .save_with_format(&img_path, ImageFormat::Png)
            .expect("should save large test png");

        let uri =
            process_and_encode_image_as_webp(&img_path).expect("large image encoding should work");
        assert!(uri.starts_with("data:image/webp;base64,"));

        let webp_path = temp_dir.join("large.webp");
        assert!(io::path_exists(&webp_path));
        let webp_size = io::get_file_size(&webp_path).unwrap_or(0);
        assert!(webp_size > 0 && webp_size <= MAX_IMAGE_SIZE_BYTES);

        let _ = io::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_process_and_encode_image_as_webp_non_existent_file() {
        let non_existent = Path::new("/path/that/definitely/does/not/exist_12345.png");
        let result = process_and_encode_image_as_webp(non_existent);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Image(_)));
    }

    #[test]
    fn test_prompt_user_for_resizing_text() {
        let prompt_res = prompt_user_for_resizing("dummy.png", 300 * 1024);
        assert!(!prompt_res);
    }
}
