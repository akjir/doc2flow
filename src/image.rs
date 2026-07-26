//! Image resolution, Base64 embedding, and non-image link conversion module.

use crate::error::DiagnosticError;
use anyhow::Result;
use base64::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum allowed size in bytes for a local image embedded into HTML (250 KB).
pub const MAX_IMAGE_SIZE_BYTES: u64 = 250 * 1024;

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
    embed_images_as_base64_with_source(html, None, None, base_dir)
}

/// Embeds local image references in HTML as Base64 `data:` URIs with Markdown source context for error diagnostics.
///
/// Scans `<img ... src="..." ...>` tags in the input HTML. Checks image sizes against `MAX_IMAGE_SIZE_BYTES` (250 KB).
/// If an image exceeds this limit, returns a compiler-style `DiagnosticError` pointing to the exact location in `md_content`.
///
/// # Errors
///
/// Returns a compiler-style `DiagnosticError` if a local image exceeds the 250 KB size limit.
pub fn embed_images_as_base64_with_source(
    html: &str,
    md_content: Option<&str>,
    file_name: Option<&str>,
    base_dir: Option<&Path>,
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
                    use std::fmt::Write;
                    let _ = writeln!(
                        out,
                        "<div class=\"check-item text-item\">\n  <span class=\"text-content\"><a href=\"{src_val}\" target=\"_blank\" rel=\"noopener noreferrer\">{alt_text}</a></span>\n</div>"
                    );
                    cursor = next_cursor;
                } else {
                    use std::fmt::Write;
                    let _ = write!(
                        out,
                        "<a href=\"{src_val}\" target=\"_blank\" rel=\"noopener noreferrer\">{alt_text}</a>"
                    );
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
                        match fs::read(&resolved_path) {
                            Ok(bytes) => {
                                let size = bytes.len() as u64;
                                if size > MAX_IMAGE_SIZE_BYTES {
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

                                let mime = mime_guess::from_path(&resolved_path)
                                    .first_raw()
                                    .unwrap_or("image/jpeg");
                                let b64 = BASE64_STANDARD.encode(&bytes);
                                cache.insert(
                                    resolved_path.clone(),
                                    format!("data:{mime};base64,{b64}"),
                                );
                            }
                            Err(_) => {
                                cache.insert(resolved_path.clone(), src_val.to_string());
                            }
                        }
                    }

                    if let Some(data_uri) = cache.get(&resolved_path) {
                        if data_uri != src_val {
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
    if let Some(pos) = trimmed_out.rfind(IMG_ITEM_OPEN) {
        if trimmed_out[pos + IMG_ITEM_OPEN.len()..].trim().is_empty() {
            let rest = &html[img_end..];
            let rest_trimmed = rest.trim_start();
            if rest_trimmed.starts_with(IMG_ITEM_CLOSE) {
                let leading_ws = rest.len() - rest_trimmed.len();
                let suffix_len = leading_ws + IMG_ITEM_CLOSE.len();
                out.truncate(pos);
                return Some(img_end + suffix_len);
            }
        }
    }
    None
}

/// Helper to extract attribute bounds and value from an HTML tag slice without heap allocations.
fn extract_attribute<'a>(tag: &'a str, attr_name: &str) -> Option<(usize, usize, &'a str)> {
    let bytes = tag.as_bytes();
    let attr_bytes = attr_name.as_bytes();
    let attr_len = attr_bytes.len();
    let mut i = 0;

    while i + attr_len + 1 <= bytes.len() {
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
        if let Some(resolved) = resolve_image_path(path, base_dir) {
            if let Some(mime) = mime_guess::from_path(&resolved).first_raw() {
                return mime.starts_with("image/");
            }
        }
    }

    true
}

/// Resolves an image path relative to base_dir or current working directory.
fn resolve_image_path(path: &Path, base_dir: Option<&Path>) -> Option<PathBuf> {
    if path.is_absolute() {
        if path.exists() {
            return Some(path.to_path_buf());
        }
        return None;
    }

    if let Some(base) = base_dir {
        let combined = base.join(path);
        if combined.exists() {
            return Some(combined);
        }
    }

    if path.exists() {
        return Some(path.to_path_buf());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

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
        let _ = fs::create_dir_all(&dir);
        let img_path = dir.join("test.png");
        let mut file = File::create(&img_path).unwrap();
        file.write_all(b"fake png content").unwrap();

        let html = "<p><img src=\"test.png\" alt=\"demo\"></p>".to_string();
        let embedded = embed_images_as_base64(&html, Some(&dir)).unwrap();

        let _ = fs::remove_dir_all(&dir);

        assert!(embedded.contains("src=\"data:image/png;base64,"));
        assert!(!embedded.contains("src=\"test.png\""));
    }

    #[test]
    fn test_embed_images_deduplication() {
        let dir = std::env::temp_dir().join("d2f_test_img_2");
        let _ = fs::create_dir_all(&dir);
        let img_path = dir.join("logo.jpg");
        let mut file = File::create(&img_path).unwrap();
        file.write_all(b"sample image data").unwrap();

        let html = "<img src=\"logo.jpg\"><p>text</p><img src=\"logo.jpg\">";
        let embedded = embed_images_as_base64(html, Some(&dir)).unwrap();

        let _ = fs::remove_dir_all(&dir);

        let matches: Vec<_> = embedded.matches("data:image/jpeg;base64,").collect();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_embed_images_exceeds_max_size_error() {
        let dir = std::env::temp_dir().join("d2f_test_img_large");
        let _ = fs::create_dir_all(&dir);
        let img_path = dir.join("large.png");
        let mut file = File::create(&img_path).unwrap();
        // Write 251 KB (251 * 1024 bytes)
        let large_buf = vec![0u8; 251 * 1024];
        file.write_all(&large_buf).unwrap();

        let md_content = "---\ntitle: \"Test\"\ncompany: \"Corp\"\n---\n## Section\n\n![Large Image](large.png)\n";
        let html = "<div class=\"img-item\">\n  <img src=\"large.png\" alt=\"Large Image\">\n</div>";

        let err = embed_images_as_base64_with_source(
            html,
            Some(md_content),
            Some("test_doc.md"),
            Some(&dir),
        )
        .unwrap_err();

        let _ = fs::remove_dir_all(&dir);

        let err_str = err.to_string();
        assert!(err_str.contains("error: image 'large.png' exceeds maximum allowed size of 250 KB (251.0 KB)"));
        assert!(err_str.contains("--> test_doc.md:7:16"));
        assert!(err_str.contains("7 | ![Large Image](large.png)"));
        assert!(err_str.contains("^^^^^^^^^ local image size (251.0 KB) exceeds 250 KB limit"));
        assert!(err_str.contains("= help: reduce image resolution or compress 'large.png' below 250 KB before embedding."));
    }
}
