use crate::error::{DiagnosticError, Doc2FlowError, Result, print_warning};
use crate::io;
use crate::template::DEFAULT_LOGO_SVG;
use crate::utils::{guess_mime_type, to_base64_data_uri};
use image::{GenericImageView, ImageFormat, imageops::FilterType};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub use crate::utils::to_base64_data_uri_into;

/// Maximum allowed size in bytes for a local image embedded into HTML (250 KB).
pub const MAX_IMAGE_SIZE_BYTES: u64 = 250 * 1024;

/// Resolves a logo image file path relative to `base_dir` if specified and relative.
pub fn resolve_logo_path(path: &Path, base_dir: Option<&Path>) -> PathBuf {
    io::resolve_logo_path(path, base_dir)
}

/// Loads and processes a custom logo image (SVG or raster), or falls back to default.
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
        match crate::utils::file_to_data_uri(&resolved_path) {
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

/// Embeds local image references in HTML as Base64 `data:` URIs with Markdown source context.
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

    while let Some((img_start, img_end)) = find_next_img_tag(html, cursor) {
        out.push_str(&html[cursor..img_start]);
        let tag_slice = &html[img_start..img_end];

        let Some((attr_start, attr_end, src_val)) = extract_attribute(tag_slice, "src") else {
            out.push_str(tag_slice);
            cursor = img_end;
            continue;
        };

        if !is_image_source(src_val, base_dir) {
            let alt_text = extract_attribute(tag_slice, "alt")
                .map(|(_, _, val)| val)
                .unwrap_or(src_val);
            cursor = render_non_image_link(&mut out, html, img_end, src_val, alt_text);
            continue;
        }

        if is_remote_or_data_uri(src_val) {
            out.push_str(tag_slice);
            cursor = img_end;
            continue;
        }

        let path = Path::new(src_val);
        let Some(resolved_path) = resolve_image_path(path, base_dir) else {
            out.push_str(tag_slice);
            cursor = img_end;
            continue;
        };

        let data_uri = match cache.entry(resolved_path.clone()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let uri = resolve_or_encode_image(
                    &resolved_path,
                    src_val,
                    auto_scale,
                    md_content,
                    file_name,
                )?;
                entry.insert(uri).clone()
            }
        };

        if data_uri != src_val {
            replace_img_src(&mut out, tag_slice, attr_start, attr_end, &data_uri);
        } else {
            out.push_str(tag_slice);
        }

        cursor = img_end;
    }

    out.push_str(&html[cursor..]);
    Ok(out)
}

/// Finds start and end byte offsets of the next `<img` tag in `html` from `cursor`.
#[inline]
fn find_next_img_tag(html: &str, cursor: usize) -> Option<(usize, usize)> {
    let img_start_rel = html[cursor..].find("<img")?;
    let img_start = cursor + img_start_rel;
    let rel_end = html[img_start..].find('>')?;
    let img_end = img_start + rel_end + 1;
    Some((img_start, img_end))
}

/// Checks if an image source is a remote URL or existing Base64 Data URI.
#[inline]
fn is_remote_or_data_uri(src: &str) -> bool {
    src.starts_with("data:") || src.starts_with("http://") || src.starts_with("https://")
}

/// Renders a non-image attachment link into the buffer.
fn render_non_image_link(
    out: &mut String,
    html: &str,
    img_end: usize,
    src_val: &str,
    alt_text: &str,
) -> usize {
    if let Some(next_cursor) = strip_img_item_wrapper(out, html, img_end) {
        let comment_icon = crate::template::COMMENT_ICON_SVG;
        out.push_str("<div class=\"doc-item text-item\">\n  <span class=\"text-content\"><a href=\"");
        out.push_str(src_val);
        out.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
        out.push_str(alt_text);
        out.push_str("</a></span>\n  ");
        out.push_str(comment_icon);
        out.push_str("\n</div>\n");
        next_cursor
    } else {
        out.push_str("<a href=\"");
        out.push_str(src_val);
        out.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
        out.push_str(alt_text);
        out.push_str("</a>");
        img_end
    }
}

/// Reads, validates size, and encodes a local image file to a Base64 Data URI.
fn resolve_or_encode_image(
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
            let f_name = file_name.unwrap_or("input.md");
            Err(DiagnosticError::image_too_large(
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

/// Replaces the `src="..."` attribute within a tag slice and writes the result to `out`.
#[inline]
fn replace_img_src(
    out: &mut String,
    tag_slice: &str,
    attr_start: usize,
    attr_end: usize,
    data_uri: &str,
) {
    out.push_str(&tag_slice[..attr_start]);
    out.push_str("src=\"");
    out.push_str(data_uri);
    out.push('"');
    out.push_str(&tag_slice[attr_end..]);
}

/// Resizes a local image file and converts it to WebP format until its size is within 250 KB.
///
/// # Errors
///
/// Returns a [`Doc2FlowError::ImageProcess`] if opening or encoding the image fails.
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
    eprintln!(
        "Resized image '{}': {}x{} ({orig_kb:.1} KB) -> {}x{} WebP ({new_kb:.1} KB)",
        image_path.display(),
        orig_w,
        orig_h,
        final_w,
        final_h
    );

    Ok(to_base64_data_uri("image/webp", &buffer))
}

/// Asks user interactively via stderr/stdin whether to resize/convert an image that exceeds 250 KB.
fn prompt_user_for_resizing(src_val: &str, size_bytes: u64) -> bool {
    let size_kb = size_bytes as f64 / 1024.0;
    io::prompt_user_yes_no(&format!(
        "\nWarning: Image '{src_val}' ({size_kb:.1} KB) exceeds the 250 KB limit.\nDo you want to resize and convert it to WebP? [y/N]: "
    ))
}

/// Finds line number, column number, and line snippet in Markdown for an image source string.
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

/// Extracts attribute bounds and unquoted value from an HTML/XML tag slice without heap allocations.
///
/// Returns `Some((attr_start, attr_end, attr_value))` where:
/// - `attr_start`: byte offset in `tag` where the attribute begins.
/// - `attr_end`: byte offset in `tag` immediately following the attribute definition.
/// - `attr_value`: unquoted value slice of the attribute.
///
/// Supports double quotes, single quotes, whitespace around `=`, unquoted values, boolean
/// attributes, multiline attributes, and escaped quotes. Loop bounds advance systematically
/// without redundant scanning passes.
///
/// # Examples
///
/// ```
/// use doc2flow::image::extract_attribute;
///
/// let tag = r#"<img src="photo.png" alt="Demo">"#;
/// let (start, end, val) = extract_attribute(tag, "src").unwrap();
/// assert_eq!(&tag[start..end], r#"src="photo.png""#);
/// assert_eq!(val, "photo.png");
/// ```
#[inline]
pub fn extract_attribute<'a>(tag: &'a str, attr_name: &str) -> Option<(usize, usize, &'a str)> {
    let bytes = tag.as_bytes();
    let mut cursor = 0;

    if cursor < bytes.len() && bytes[cursor] == b'<' {
        cursor += 1;
        if cursor < bytes.len() && bytes[cursor] == b'/' {
            cursor += 1;
        }
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'/'
            && bytes[cursor] != b'>'
        {
            cursor += 1;
        }
    }

    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] == b'/' || bytes[cursor] == b'>' {
            break;
        }

        let attr_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'='
            && bytes[cursor] != b'/'
            && bytes[cursor] != b'>'
        {
            cursor += 1;
        }

        let name = &tag[attr_start..cursor];
        if name.is_empty() {
            cursor += 1;
            continue;
        }

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        let is_target = name.eq_ignore_ascii_case(attr_name);

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
                    while cursor < bytes.len() {
                        if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
                            cursor += 2;
                        } else if bytes[cursor] == quote {
                            break;
                        } else {
                            cursor += 1;
                        }
                    }
                    let val_end = cursor;
                    if cursor < bytes.len() && bytes[cursor] == quote {
                        cursor += 1;
                    }
                    let attr_end = cursor;
                    if is_target {
                        return Some((attr_start, attr_end, &tag[val_start..val_end]));
                    }
                } else {
                    let val_start = cursor;
                    while cursor < bytes.len()
                        && !bytes[cursor].is_ascii_whitespace()
                        && bytes[cursor] != b'/'
                        && bytes[cursor] != b'>'
                    {
                        cursor += 1;
                    }
                    let val_end = cursor;
                    let attr_end = cursor;
                    if is_target {
                        return Some((attr_start, attr_end, &tag[val_start..val_end]));
                    }
                }
            } else if is_target {
                return Some((attr_start, cursor, ""));
            }
        } else if is_target {
            return Some((attr_start, cursor, ""));
        }
    }

    None
}

/// Checks if a file path or URL points to an image resource based on extension or MIME type.
fn is_image_source(src: &str, base_dir: Option<&Path>) -> bool {
    if let Some(ext) = Path::new(src).extension().and_then(|e| e.to_str()) {
        match ext {
            e if e.eq_ignore_ascii_case("png")
                || e.eq_ignore_ascii_case("jpg")
                || e.eq_ignore_ascii_case("jpeg")
                || e.eq_ignore_ascii_case("gif")
                || e.eq_ignore_ascii_case("svg")
                || e.eq_ignore_ascii_case("webp")
                || e.eq_ignore_ascii_case("bmp")
                || e.eq_ignore_ascii_case("ico")
                || e.eq_ignore_ascii_case("avif")
                || e.eq_ignore_ascii_case("tiff") =>
            {
                return true;
            }
            e if e.eq_ignore_ascii_case("pdf")
                || e.eq_ignore_ascii_case("doc")
                || e.eq_ignore_ascii_case("docx")
                || e.eq_ignore_ascii_case("xls")
                || e.eq_ignore_ascii_case("xlsx")
                || e.eq_ignore_ascii_case("ppt")
                || e.eq_ignore_ascii_case("pptx")
                || e.eq_ignore_ascii_case("zip")
                || e.eq_ignore_ascii_case("tar")
                || e.eq_ignore_ascii_case("gz")
                || e.eq_ignore_ascii_case("7z")
                || e.eq_ignore_ascii_case("txt")
                || e.eq_ignore_ascii_case("csv")
                || e.eq_ignore_ascii_case("json")
                || e.eq_ignore_ascii_case("xml")
                || e.eq_ignore_ascii_case("html")
                || e.eq_ignore_ascii_case("htm")
                || e.eq_ignore_ascii_case("mp4")
                || e.eq_ignore_ascii_case("mp3")
                || e.eq_ignore_ascii_case("avi")
                || e.eq_ignore_ascii_case("mov")
                || e.eq_ignore_ascii_case("wav") =>
            {
                return false;
            }
            _ => {}
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

/// Skips XML processing instructions (`<? ... ?>`).
#[inline]
fn skip_processing_instruction(input: &str) -> Option<&str> {
    if input.starts_with("<?") {
        let end = input.find("?>")?;
        Some(&input[end + 2..])
    } else {
        None
    }
}

/// Skips DOCTYPE declarations (`<!DOCTYPE ... >` or `<!doctype ... >`).
#[inline]
fn skip_doctype(input: &str) -> Option<&str> {
    if input.starts_with("<!DOCTYPE") || input.starts_with("<!doctype") {
        let end = input.find('>')?;
        Some(&input[end + 1..])
    } else {
        None
    }
}

/// Skips XML comments (`<!-- ... -->`).
#[inline]
fn skip_comment(input: &str) -> Option<&str> {
    if input.starts_with("<!--") {
        let end = input.find("-->")?;
        Some(&input[end + 3..])
    } else {
        None
    }
}

/// Parses CDATA sections (`<![CDATA[ ... ]]>`), returning the section content and the remainder.
#[inline]
fn parse_cdata(input: &str) -> Option<(&str, &str)> {
    if input.starts_with("<![CDATA[") {
        let end = input.find("]]>")?;
        Some((&input[..end + 3], &input[end + 3..]))
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
        if !is_closing && !is_self_closing
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

/// Cleans and minifies SVG content by stripping declarations, comments, and editor metadata.
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
            if tag_rest.starts_with("<!--")
                || tag_rest.starts_with("<?")
                || tag_rest.starts_with("<!DOCTYPE")
                || tag_rest.starts_with("<!doctype")
                || tag_rest.starts_with("<![CDATA[")
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

/// Helper function to clean attributes of an XML tag, streaming valid attributes into buffer without allocations.
fn write_cleaned_tag_attributes(out: &mut String, tag_name: &str, tag_inner: &str) {
    let tag_name_len = tag_name.len();
    let attr_str = if tag_inner.len() > tag_name_len {
        tag_inner[tag_name_len..].trim_start()
    } else {
        ""
    };

    let bytes = attr_str.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] == b'/' || bytes[cursor] == b'>' {
            break;
        }

        let name_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'='
            && bytes[cursor] != b'/'
            && bytes[cursor] != b'>'
        {
            cursor += 1;
        }
        let name = &attr_str[name_start..cursor];
        if name.is_empty() {
            cursor += 1;
            continue;
        }

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        let mut val = "";
        let mut has_equals = false;

        if cursor < bytes.len() && bytes[cursor] == b'=' {
            has_equals = true;
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() {
                let quote = bytes[cursor];
                if quote == b'"' || quote == b'\'' {
                    cursor += 1;
                    let val_start = cursor;
                    while cursor < bytes.len() {
                        if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
                            cursor += 2;
                        } else if bytes[cursor] == quote {
                            break;
                        } else {
                            cursor += 1;
                        }
                    }
                    val = &attr_str[val_start..cursor];
                    if cursor < bytes.len() && bytes[cursor] == quote {
                        cursor += 1;
                    }
                } else {
                    let val_start = cursor;
                    while cursor < bytes.len()
                        && !bytes[cursor].is_ascii_whitespace()
                        && bytes[cursor] != b'/'
                        && bytes[cursor] != b'>'
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
        let tag = "<img\n  src=\"images/multiline.png\"\n  alt=\"System\nDiagram with Newlines\"\n/>";
        let (start, end, src_val) = extract_attribute(tag, "src").unwrap();
        assert_eq!(&tag[start..end], "src=\"images/multiline.png\"");
        assert_eq!(src_val, "images/multiline.png");

        let (_, _, alt_val) = extract_attribute(tag, "alt").unwrap();
        assert_eq!(alt_val, "System\nDiagram with Newlines");
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
        assert!(processed.contains("<div class=\"doc-item text-item\">"));
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
    fn test_find_markdown_location() {
        let md = "Line 1\nLine 2\n![Alt](images/photo.png)\nLine 4";
        let (line_no, col_no, snippet) = find_markdown_location(Some(md), "images/photo.png");
        assert_eq!(line_no, 3);
        assert_eq!(col_no, 8);
        assert_eq!(snippet, "![Alt](images/photo.png)");
    }

    #[test]
    fn test_load_logo_default_and_custom() {
        let default_logo = load_logo(None, None);
        assert_eq!(default_logo, DEFAULT_LOGO_SVG);

        let empty_logo = load_logo(Some(Path::new("")), None);
        assert_eq!(empty_logo, DEFAULT_LOGO_SVG);

        let temp_dir = std::env::temp_dir().join("d2f_test_logo");
        let _ = io::create_dir_all(&temp_dir);

        let svg_path = temp_dir.join("test_logo.svg");
        let svg_content = "<?xml version=\"1.0\"?><svg width=\"100\" height=\"100\"><circle cx=\"50\" cy=\"50\" r=\"40\"/></svg>";
        io::write_file(&svg_path, svg_content).unwrap();

        let loaded_svg = load_logo(Some(&svg_path), None);
        assert!(loaded_svg.starts_with("<svg"));
        assert!(loaded_svg.contains("circle"));
        assert!(loaded_svg.ends_with("</svg>"));

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
}
