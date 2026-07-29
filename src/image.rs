use crate::error::{DiagnosticError, Doc2FlowError, Result, print_warning};
use crate::io;
use crate::template::DEFAULT_LOGO_SVG;
use crate::utils::{base64_encode, file_to_data_uri, guess_mime_type};
use image::{GenericImageView, ImageFormat, imageops::FilterType};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Maximum allowed size in bytes for a local image embedded into HTML (250 KB).
pub const MAX_IMAGE_SIZE_BYTES: u64 = 250 * 1024;

/// Resolves a logo image file path relative to `base_dir` if specified and relative.
pub fn resolve_logo_path(path: &Path, base_dir: Option<&Path>) -> PathBuf {
    io::resolve_logo_path(path, base_dir)
}

/// Loads and processes a custom logo image (SVG or raster), or falls back to the default embedded SVG logo.
///
/// If `logo_path` is `None` or points to an empty path, returns [`DEFAULT_LOGO_SVG`].
/// If the custom logo file cannot be found or read, outputs a user-friendly warning message to `stderr`
/// via [`print_warning`] and falls back to [`DEFAULT_LOGO_SVG`].
///
/// Relative paths are resolved against `base_dir` if provided.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use doc2flow::image::load_logo;
///
/// let logo_html = load_logo(Some(Path::new("custom_logo.svg")), Some(Path::new("docs")));
/// assert!(logo_html.contains("<svg") || logo_html.contains("<img"));
/// ```
pub fn load_logo(logo_path: Option<&Path>, base_dir: Option<&Path>) -> String {
    let path = match logo_path {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => return DEFAULT_LOGO_SVG.to_string(),
    };

    let resolved_path = resolve_logo_path(path, base_dir);

    if !io::path_exists(&resolved_path) {
        print_warning(&format!(
            "Custom logo file '{}' not found. Falling back to default logo.",
            resolved_path.display()
        ));
        return DEFAULT_LOGO_SVG.to_string();
    }

    let mime = guess_mime_type(&resolved_path);

    if mime == "image/svg+xml" {
        match io::read_file_to_string(&resolved_path) {
            Ok(content) => {
                let cleaned = clean_svg(&content);
                if cleaned.contains("<svg") {
                    cleaned
                } else {
                    print_warning(&format!(
                        "Custom logo file '{}' does not contain valid SVG markup. Falling back to default logo.",
                        resolved_path.display()
                    ));
                    DEFAULT_LOGO_SVG.to_string()
                }
            }
            Err(e) => {
                print_warning(&format!(
                    "Failed to read custom logo file '{}': {}. Falling back to default logo.",
                    resolved_path.display(),
                    e
                ));
                DEFAULT_LOGO_SVG.to_string()
            }
        }
    } else {
        match file_to_data_uri(&resolved_path) {
            Ok(data_uri) => {
                format!("<img src=\"{data_uri}\" alt=\"Logo\">")
            }
            Err(e) => {
                print_warning(&format!(
                    "Failed to process custom logo image '{}': {}. Falling back to default logo.",
                    resolved_path.display(),
                    e
                ));
                DEFAULT_LOGO_SVG.to_string()
            }
        }
    }
}

/// Embeds local image references in HTML as Base64 `data:` URIs and converts non-image tags to links.
///
/// Scans `<img ... src="..." ...>` tags in the input HTML. If a `src` attribute points to a local
/// file, the file is read, Base64 encoded, and replaced with a `data:<mime>;base64,<encoded>` URI.
/// Remote images (`http://`, `https://`) are preserved as-is. Non-image resources (e.g. `.pdf`, `.zip`)
/// are converted to external link elements (`<a>`).
///
/// Local images must not exceed `MAX_IMAGE_SIZE_BYTES` (250 KB).
///
/// # Errors
///
/// Returns a compiler-style `DiagnosticError` if a local image exceeds the 250 KB size limit.
pub fn embed_images_as_base64(html: &str, base_dir: Option<&Path>) -> Result<String> {
    embed_images_as_base64_with_source(html, None, None, base_dir, false)
}

/// Embeds local image references in HTML as Base64 `data:` URIs with Markdown source context for error diagnostics.
///
/// Scans `<img ... src="..." ...>` tags in the input HTML. Checks image sizes against `MAX_IMAGE_SIZE_BYTES` (250 KB).
/// If an image exceeds this limit, offers interactive scaling/conversion to WebP or returns a compiler-style `DiagnosticError`.
///
/// # Errors
///
/// Returns a compiler-style `DiagnosticError` if a local image exceeds the 250 KB size limit and is not scaled.
pub fn embed_images_as_base64_with_source(
    html: &str,
    md_content: Option<&str>,
    file_name: Option<&str>,
    base_dir: Option<&Path>,
    auto_scale: bool,
) -> Result<String> {
    let mut out = String::with_capacity(html.len());
    let mut cache: HashMap<PathBuf, String> = HashMap::new();
    let mut cursor = 0;

    while let Some(img_start_rel) = html[cursor..].find("<img") {
        let img_start = cursor + img_start_rel;
        out.push_str(&html[cursor..img_start]);

        let img_end = match html[img_start..].find('>') {
            Some(rel_end) => img_start + rel_end + 1,
            None => {
                out.push_str(&html[img_start..]);
                cursor = html.len();
                break;
            }
        };

        let tag_slice = &html[img_start..img_end];

        if let Some((attr_start, attr_end, src_val)) = extract_attribute(tag_slice, "src") {
            if !is_image_source(src_val, base_dir) {
                let alt_text = extract_attribute(tag_slice, "alt")
                    .map(|(_, _, val)| val)
                    .unwrap_or(src_val);

                if let Some(next_cursor) = strip_img_item_wrapper(&mut out, html, img_end) {
                    let comment_icon = crate::template::COMMENT_ICON_SVG;
                    let _ = out.write_str("<div class=\"check-item text-item\">\n  <span class=\"text-content\"><a href=\"");
                    let _ = out.write_str(src_val);
                    let _ = out.write_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
                    let _ = out.write_str(alt_text);
                    let _ = out.write_str("</a></span>\n  ");
                    let _ = out.write_str(comment_icon);
                    let _ = out.write_str("\n</div>\n");
                    cursor = next_cursor;
                } else {
                    let _ = out.write_str("<a href=\"");
                    let _ = out.write_str(src_val);
                    let _ = out.write_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
                    let _ = out.write_str(alt_text);
                    let _ = out.write_str("</a>");
                    cursor = img_end;
                }
                continue;
            }

            let is_remote_or_data = src_val.starts_with("data:")
                || src_val.starts_with("http://")
                || src_val.starts_with("https://");

            if !is_remote_or_data {
                let path = Path::new(src_val);
                if let Some(resolved_path) = resolve_image_path(path, base_dir) {
                    if !cache.contains_key(&resolved_path) {
                        match io::read_file_bytes(&resolved_path) {
                            Ok(bytes) => {
                                let size = bytes.len() as u64;
                                if size > MAX_IMAGE_SIZE_BYTES {
                                    let should_scale =
                                        auto_scale || prompt_user_for_resizing(src_val, size);

                                    let mut scaled = false;
                                    if let Some(Ok(data_uri)) =
                                        should_scale.then(|| process_and_encode_image_as_webp(&resolved_path))
                                    {
                                        cache.insert(resolved_path.clone(), data_uri);
                                        scaled = true;
                                    }

                                    if !scaled {
                                        let (line_no, col_no, line_snippet) =
                                            find_markdown_location(md_content, src_val);
                                        let f_name = file_name.unwrap_or("input.md");
                                        return Err(DiagnosticError::image_too_large(
                                            f_name,
                                            line_no,
                                            col_no,
                                            line_snippet,
                                            src_val,
                                            size,
                                        ));
                                    }
                                } else {
                                    let mime = guess_mime_type(&resolved_path);
                                    let bytes_to_encode = if mime == "image/svg+xml" {
                                        if let Ok(utf8_str) = std::str::from_utf8(&bytes) {
                                            clean_svg(utf8_str).into_bytes()
                                        } else {
                                            bytes
                                        }
                                    } else {
                                        bytes
                                    };
                                    let b64 = base64_encode(&bytes_to_encode);
                                    cache.insert(
                                        resolved_path.clone(),
                                        format!("data:{mime};base64,{b64}"),
                                    );
                                }
                            }
                            Err(_) => {
                                cache.insert(resolved_path.clone(), src_val.to_string());
                            }
                        }
                    }

                    if let Some(data_uri) = cache.get(&resolved_path).filter(|&d| d != src_val) {
                        out.push_str(&tag_slice[..attr_start]);
                        out.push_str("src=\"");
                        out.push_str(data_uri);
                        out.push('"');
                        out.push_str(&tag_slice[attr_end..]);
                        cursor = img_end;
                        continue;
                    }
                }
            }

            out.push_str(tag_slice);
        } else {
            out.push_str(tag_slice);
        }

        cursor = img_end;
    }

    out.push_str(&html[cursor..]);
    Ok(out)
}

/// Resizes a local image file and converts it to WebP format until its size is within `MAX_IMAGE_SIZE_BYTES` (250 KB).
///
/// # Errors
///
/// Returns an error if opening or encoding the image fails.
pub fn process_and_encode_image_as_webp(image_path: &Path) -> Result<String> {
    let img = image::open(image_path).map_err(|e| {
        Doc2FlowError::ImageProcess(format!(
            "Failed to open image '{}': {}",
            image_path.display(),
            e
        ))
    })?;

    let (orig_w, orig_h) = img.dimensions();
    let file_size = io::get_file_size(image_path).unwrap_or(MAX_IMAGE_SIZE_BYTES + 1);

    let scale_ratio = (MAX_IMAGE_SIZE_BYTES as f64 / file_size as f64)
        .sqrt()
        .min(0.95);
    let mut target_w = ((orig_w as f64 * scale_ratio) as u32).max(100);
    let mut target_h = ((orig_h as f64 * scale_ratio) as u32).max(100);

    let mut buffer = Vec::with_capacity(MAX_IMAGE_SIZE_BYTES as usize);

    let (final_w, final_h) = loop {
        let resized_img = if target_w < orig_w || target_h < orig_h {
            img.resize(target_w, target_h, FilterType::Triangle)
        } else {
            img.clone()
        };

        let dims = resized_img.dimensions();

        buffer.clear();
        let mut cursor = Cursor::new(&mut buffer);
        resized_img.write_to(&mut cursor, ImageFormat::WebP).map_err(|e| {
            Doc2FlowError::ImageProcess(format!("Failed to encode image to WebP format: {}", e))
        })?;

        if (buffer.len() as u64) <= MAX_IMAGE_SIZE_BYTES || (target_w <= 100 && target_h <= 100) {
            break dims;
        }

        target_w = ((target_w as f64 * 0.85) as u32).max(100);
        target_h = ((target_h as f64 * 0.85) as u32).max(100);
    };

    let webp_path = image_path.with_extension("webp");
    let _ = io::write_file(&webp_path, &buffer);

    let orig_kb = file_size as f64 / 1024.0;
    let new_kb = buffer.len() as f64 / 1024.0;
    println!(
        "Resized image '{}': {}x{} ({orig_kb:.1} KB) -> {}x{} WebP ({new_kb:.1} KB)",
        image_path.display(),
        orig_w,
        orig_h,
        final_w,
        final_h
    );

    let b64 = base64_encode(&buffer);
    Ok(format!("data:image/webp;base64,{b64}"))
}

/// Asks user interactively via stderr/stdin whether to resize/convert an image that exceeds 250 KB.
fn prompt_user_for_resizing(src_val: &str, size_bytes: u64) -> bool {
    let size_kb = size_bytes as f64 / 1024.0;
    io::prompt_user_yes_no(&format!(
        "\nWarning: Image '{src_val}' ({size_kb:.1} KB) exceeds the 250 KB limit.\nDo you want to resize and convert it to WebP? [y/N]: "
    ))
}

/// Finds the line number (1-based), column number (1-based), and line snippet in Markdown for an image source string.
fn find_markdown_location<'a>(
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

const IMG_ITEM_OPEN: &str = "<div class=\"img-item\">";
const IMG_ITEM_CLOSE: &str = "</div>";

/// Helper to unwrap `<div class="img-item">` container if present around a non-image tag.
fn strip_img_item_wrapper(out: &mut String, html: &str, img_end: usize) -> Option<usize> {
    let trimmed_out = out.trim_end();
    let pos = trimmed_out.rfind(IMG_ITEM_OPEN)?;
    if !trimmed_out[pos + IMG_ITEM_OPEN.len()..].trim().is_empty() {
        return None;
    }
    let rest = &html[img_end..];
    let rest_trimmed = rest.trim_start();
    if !rest_trimmed.starts_with(IMG_ITEM_CLOSE) {
        return None;
    }
    let leading_ws = rest.len() - rest_trimmed.len();
    let suffix_len = leading_ws + IMG_ITEM_CLOSE.len();
    out.truncate(pos);
    Some(img_end + suffix_len)
}

/// Helper to extract attribute bounds and value from an HTML tag slice without heap allocations.
fn extract_attribute<'a>(tag: &'a str, attr_name: &str) -> Option<(usize, usize, &'a str)> {
    let bytes = tag.as_bytes();
    let attr_bytes = attr_name.as_bytes();
    let attr_len = attr_bytes.len();
    let mut i = 0;

    while i + attr_len < bytes.len() {
        if bytes[i..i + attr_len].eq_ignore_ascii_case(attr_bytes) && bytes[i + attr_len] == b'=' {
            let abs_pos = i;
            let val_search_start = abs_pos + attr_len + 1;
            let rest = tag[val_search_start..].trim_start();
            let quote = rest.chars().next()?;

            if quote == '"' || quote == '\'' {
                let quote_offset = tag[val_search_start..].len() - rest.len();
                let val_start = val_search_start + quote_offset + 1;
                if let Some(val_end_rel) = tag[val_start..].find(quote) {
                    let val_end = val_start + val_end_rel;
                    let attr_end = val_end + 1;
                    return Some((abs_pos, attr_end, &tag[val_start..val_end]));
                }
            }
        }
        i += 1;
    }
    None
}

/// Checks if a file path or URL points to an image resource based on file extension or MIME type.
fn is_image_source(src: &str, base_dir: Option<&Path>) -> bool {
    if let Some(ext) = Path::new(src).extension().and_then(|e| e.to_str()) {
        const NON_IMAGE_EXTS: &[&str] = &[
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "zip", "tar", "gz", "7z",
            "txt", "csv", "json", "xml", "html", "htm", "mp4", "mp3", "avi", "mov", "wav",
        ];
        if NON_IMAGE_EXTS.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
            return false;
        }

        const IMAGE_EXTS: &[&str] = &[
            "png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico", "avif", "tiff",
        ];
        if IMAGE_EXTS.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
            return true;
        }
    }

    if !src.starts_with("http://") && !src.starts_with("https://") && !src.starts_with("data:") {
        let path = Path::new(src);
        let resolved = resolve_image_path(path, base_dir);
        let is_img = resolved
            .as_deref()
            .map(guess_mime_type)
            .is_some_and(|mime| mime.starts_with("image/"));
        if is_img {
            return true;
        }
    }

    true
}

/// Resolves an image path relative to base_dir or current working directory.
fn resolve_image_path(path: &Path, base_dir: Option<&Path>) -> Option<PathBuf> {
    io::resolve_image_path(path, base_dir)
}

/// Cleans and minifies SVG content by removing XML headers, comments, editor metadata tags
/// (e.g. `<sodipodi:namedview>`, `<metadata>`), empty `<defs/>`, and Inkscape/Sodipodi namespace attributes.
pub fn clean_svg(input: &str) -> String {
    let mut s = input.trim();

    // 1. Strip XML declarations <?xml ...?> and <!DOCTYPE ...>
    while let Some(start) = s.find("<?") {
        if let Some(end) = s[start..].find("?>") {
            s = s[end + 2..].trim();
        } else {
            break;
        }
    }
    while let Some(start) = s.find("<!DOCTYPE") {
        if let Some(end) = s[start..].find('>') {
            s = s[end + 1..].trim();
        } else {
            break;
        }
    }

    // 2. Strip comments <!-- ... -->
    let mut no_comments = String::with_capacity(s.len());
    let mut cur = 0;
    while let Some(rel_start) = s[cur..].find("<!--") {
        let start = cur + rel_start;
        no_comments.push_str(&s[cur..start]);
        if let Some(rel_end) = s[start..].find("-->") {
            cur = start + rel_end + 3;
        } else {
            cur = s.len();
            break;
        }
    }
    no_comments.push_str(&s[cur..]);

    // 3. Process XML tags & elements
    let mut result = String::with_capacity(no_comments.len());
    let mut rest = no_comments.as_str();

    while let Some(tag_start) = rest.find('<') {
        let text_before = rest[..tag_start].trim();
        if !text_before.is_empty() {
            result.push_str(text_before);
        }

        let tag_rest = &rest[tag_start..];
        if let Some(tag_end) = tag_rest.find('>') {
            let full_tag = &tag_rest[..=tag_end];
            rest = &tag_rest[tag_end + 1..];

            let is_closing = full_tag.starts_with("</");
            let tag_inner = if is_closing {
                full_tag[2..full_tag.len() - 1].trim()
            } else if full_tag.ends_with("/>") {
                full_tag[1..full_tag.len() - 2].trim()
            } else {
                full_tag[1..full_tag.len() - 1].trim()
            };

            let tag_name = tag_inner
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('/');

            // Skip editor-specific metadata tags
            if tag_name.starts_with("sodipodi:") || tag_name == "metadata" {
                if !is_closing && !full_tag.ends_with("/>") {
                    let closing = format!("</{tag_name}>");
                    if let Some(close_pos) = rest.find(&closing) {
                        rest = &rest[close_pos + closing.len()..];
                    }
                }
                continue;
            }

            if is_closing && (tag_name.starts_with("sodipodi:") || tag_name == "metadata") {
                continue;
            }

            // Skip empty <defs .../> tags without children
            if !is_closing && tag_name == "defs" && full_tag.ends_with("/>") {
                continue;
            }

            if is_closing {
                let _ = write!(result, "</{tag_name}>");
            } else {
                let is_self_closing = full_tag.ends_with("/>");
                let cleaned_attrs = clean_tag_attributes(tag_name, tag_inner);
                if cleaned_attrs.is_empty() {
                    if is_self_closing {
                        let _ = write!(result, "<{tag_name}/>");
                    } else {
                        let _ = write!(result, "<{tag_name}>");
                    }
                } else if is_self_closing {
                    let _ = write!(result, "<{tag_name} {cleaned_attrs}/>");
                } else {
                    let _ = write!(result, "<{tag_name} {cleaned_attrs}>");
                }
            }
        } else {
            result.push_str(tag_rest);
            break;
        }
    }

    result
}

/// Helper function to clean attributes of an XML tag, stripping editor namespaces and attributes.
fn clean_tag_attributes(tag_name: &str, tag_inner: &str) -> String {
    let mut attrs = Vec::new();
    let tag_name_len = tag_name.len();
    let attr_str = if tag_inner.len() > tag_name_len {
        tag_inner[tag_name_len..].trim()
    } else {
        ""
    };

    let mut cursor = 0;
    let bytes = attr_str.as_bytes();

    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }

        let name_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'='
            && bytes[cursor] != b'/'
        {
            cursor += 1;
        }
        let name = &attr_str[name_start..cursor];
        if name.is_empty() {
            break;
        }

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        let mut val = "";
        if cursor < bytes.len() && bytes[cursor] == b'=' {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() {
                let quote = bytes[cursor];
                if quote == b'"' || quote == b'\'' {
                    cursor += 1;
                    let val_start = cursor;
                    while cursor < bytes.len() && bytes[cursor] != quote {
                        cursor += 1;
                    }
                    val = &attr_str[val_start..cursor];
                    if cursor < bytes.len() {
                        cursor += 1;
                    }
                } else {
                    let val_start = cursor;
                    while cursor < bytes.len()
                        && !bytes[cursor].is_ascii_whitespace()
                        && bytes[cursor] != b'/'
                    {
                        cursor += 1;
                    }
                    val = &attr_str[val_start..cursor];
                }
            }
        }

        let is_generic_svg_id = name == "id"
            && val.starts_with("svg")
            && val.len() > 3
            && val[3..].chars().all(|c| c.is_ascii_digit());

        let should_remove = name.starts_with("inkscape:")
            || name.starts_with("sodipodi:")
            || name.starts_with("xmlns:inkscape")
            || name.starts_with("xmlns:sodipodi")
            || name == "xmlns:svg"
            || (tag_name == "svg" && (name == "version" || is_generic_svg_id))
            || (tag_name == "g" && name == "id" && val.starts_with("layer"));

        if !should_remove {
            if val.is_empty() && !attr_str[name_start..cursor].contains('=') {
                attrs.push(name.to_string());
            } else {
                attrs.push(format!("{name}=\"{val}\""));
            }
        }
    }

    attrs.join(" ")
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
    fn test_is_image_source_extensions() {
        assert!(is_image_source("image.png", None));
        assert!(is_image_source("photo.JPG", None));
        assert!(is_image_source("https://example.com/pic.webp", None));
        assert!(!is_image_source("manual.pdf", None));
        assert!(!is_image_source("archive.ZIP", None));
    }

    #[test]
    fn test_non_image_source_converted_to_link() {
        let html = "<p><img src=\"https://example.com/manual.pdf\" alt=\"Download Manual\"></p>";
        let processed = embed_images_as_base64(html, None).unwrap();
        assert!(processed.contains("<a href=\"https://example.com/manual.pdf\" target=\"_blank\" rel=\"noopener noreferrer\">Download Manual</a>"));
        assert!(!processed.contains("<img"));
    }

    #[test]
    fn test_non_image_source_in_img_item_wrapper_converted_to_text_item() {
        let html = "<div class=\"img-item\">\n  <img src=\"https://example.com/dateien/spezifikation.pdf\" alt=\"Systemspezifikation PDF herunterladen\">\n</div>";
        let processed = embed_images_as_base64(html, None).unwrap();
        assert!(processed.contains("<div class=\"check-item text-item\">"));
        assert!(processed.contains("<span class=\"text-content\"><a href=\"https://example.com/dateien/spezifikation.pdf\" target=\"_blank\" rel=\"noopener noreferrer\">Systemspezifikation PDF herunterladen</a></span>"));
        assert!(!processed.contains("class=\"img-item\""));
        assert!(!processed.contains("<img"));
    }

    #[test]
    fn test_embed_images_as_base64_local_file() {
        let dir = std::env::temp_dir().join("d2f_test_img_1");
        let _ = io::create_dir_all(&dir);
        let img_path = dir.join("test.png");
        io::write_file(&img_path, b"fake png content").unwrap();

        let html = "<p><img src=\"test.png\" alt=\"demo\"></p>".to_string();
        let embedded = embed_images_as_base64(&html, Some(&dir)).unwrap();

        let _ = io::remove_dir_all(&dir);

        assert!(embedded.contains("src=\"data:image/png;base64,"));
        assert!(!embedded.contains("src=\"test.png\""));
    }

    #[test]
    fn test_embed_images_deduplication() {
        let dir = std::env::temp_dir().join("d2f_test_img_2");
        let _ = io::create_dir_all(&dir);
        let img_path = dir.join("logo.jpg");
        io::write_file(&img_path, b"sample image data").unwrap();

        let html = "<img src=\"logo.jpg\"><p>text</p><img src=\"logo.jpg\">";
        let embedded = embed_images_as_base64(html, Some(&dir)).unwrap();

        let _ = io::remove_dir_all(&dir);

        let matches: Vec<_> = embedded.matches("data:image/jpeg;base64,").collect();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_auto_scale_large_image_to_webp() {
        let dir = std::env::temp_dir().join("d2f_test_auto_scale");
        let _ = io::create_dir_all(&dir);
        let img_path = dir.join("big_photo.png");

        let img_buf = image::RgbImage::new(1000, 1000);
        img_buf.save_with_format(&img_path, ImageFormat::Png).unwrap();

        let file_size = io::get_file_size(&img_path).unwrap();
        if file_size <= MAX_IMAGE_SIZE_BYTES {
            let mut existing = io::read_file_bytes(&img_path).unwrap();
            existing.resize((MAX_IMAGE_SIZE_BYTES + 50 * 1024) as usize, 0);
            io::write_file(&img_path, &existing).unwrap();
        }

        let html = "<img src=\"big_photo.png\">";
        let result = embed_images_as_base64_with_source(
            html,
            Some("![Big](big_photo.png)"),
            Some("doc.md"),
            Some(&dir),
            true,
        )
        .expect("auto scale should succeed");

        let _ = io::remove_dir_all(&dir);

        assert!(result.contains("src=\"data:image/webp;base64,"));
    }

    #[test]
    fn test_remote_http_and_https_urls_preserved() {
        let html = "<img src=\"https://example.com/logo.png\" alt=\"Remote Logo\"><img src=\"http://example.com/banner.jpg\">";
        let result = embed_images_as_base64(html, None).unwrap();
        assert!(result.contains("src=\"https://example.com/logo.png\""));
        assert!(result.contains("src=\"http://example.com/banner.jpg\""));
    }

    #[test]
    fn test_extract_attribute_single_quotes() {
        let tag = "<img src='images/pic.png' alt='System Diagram'>";
        let (start, end, val) = extract_attribute(tag, "src").unwrap();
        assert_eq!(&tag[start..end], "src='images/pic.png'");
        assert_eq!(val, "images/pic.png");
    }

    #[test]
    fn test_find_markdown_location() {
        let md = "Line 1\nLine 2\n![Alt](images/photo.png)\nLine 4";
        let (line_no, col_no, snippet) = find_markdown_location(Some(md), "images/photo.png");
        assert_eq!(line_no, 3);
        assert_eq!(col_no, 8);
        assert_eq!(snippet, "![Alt](images/photo.png)");
    }

    #[test]
    fn test_load_logo_default_and_custom() {
        // None or empty returns default logo
        let default_logo = load_logo(None, None);
        assert_eq!(default_logo, DEFAULT_LOGO_SVG);

        let empty_logo = load_logo(Some(Path::new("")), None);
        assert_eq!(empty_logo, DEFAULT_LOGO_SVG);

        // Temp dir for testing custom SVG & PNG logos
        let temp_dir = std::env::temp_dir().join("d2f_test_logo");
        let _ = io::create_dir_all(&temp_dir);

        // Custom SVG
        let svg_path = temp_dir.join("test_logo.svg");
        let svg_content = "<?xml version=\"1.0\"?><svg width=\"100\" height=\"100\"><circle cx=\"50\" cy=\"50\" r=\"40\"/></svg>";
        io::write_file(&svg_path, svg_content).unwrap();

        let loaded_svg = load_logo(Some(&svg_path), None);
        assert!(loaded_svg.starts_with("<svg"));
        assert!(loaded_svg.contains("circle"));
        assert!(loaded_svg.ends_with("</svg>"));

        // Custom PNG
        let png_path = temp_dir.join("test_logo.png");
        io::write_file(&png_path, b"fake png data").unwrap();

        let loaded_png = load_logo(Some(&png_path), None);
        assert!(loaded_png.starts_with("<img src=\"data:image/png;base64,"));
        assert!(loaded_png.contains("alt=\"Logo\""));

        let _ = io::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_logo_missing_fallback() {
        let missing_path = Path::new("non_existent_logo_12345.svg");
        let fallback = load_logo(Some(missing_path), None);
        assert_eq!(fallback, DEFAULT_LOGO_SVG);
    }

    #[test]
    fn test_resolve_logo_path() {
        let base_dir = std::env::temp_dir().join("d2f_test_resolve");
        let _ = io::create_dir_all(&base_dir);
        let rel_file = base_dir.join("sub/logo.png");
        let _ = io::create_dir_all(rel_file.parent().unwrap());
        io::write_file(&rel_file, b"data").unwrap();

        let rel_path = Path::new("sub/logo.png");
        let resolved = resolve_logo_path(rel_path, Some(&base_dir));
        assert_eq!(resolved, rel_file);

        let _ = io::remove_dir_all(&base_dir);
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
}

