//! Image resolution, Base64 embedding, and non-image link conversion module.

use base64::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Embeds local image references in HTML as Base64 `data:` URIs and converts non-image tags to links.
///
/// Scans `<img ... src="..." ...>` tags in the input HTML. If a `src` attribute points to a local
/// file, the file is read, Base64 encoded, and replaced with a `data:<mime>;base64,<encoded>` URI.
/// Remote images (`http://`, `https://`) are preserved as-is. Non-image resources (e.g. `.pdf`, `.zip`)
/// are converted to external link elements (`<a>`).
///
/// Images are cached by resolved path so each unique image file is read and encoded only once.
pub fn embed_images_as_base64(html: &str, base_dir: Option<&Path>) -> String {
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
                use std::fmt::Write;
                let _ = write!(
                    out,
                    "<a href=\"{src_val}\" target=\"_blank\" rel=\"noopener noreferrer\">{alt_text}</a>"
                );
                cursor = img_end;
                continue;
            }

            let is_remote_or_data = src_val.starts_with("data:")
                || src_val.starts_with("http://")
                || src_val.starts_with("https://");

            if !is_remote_or_data {
                let path = Path::new(src_val);
                if let Some(resolved_path) = resolve_image_path(path, base_dir) {
                    let data_uri = cache.entry(resolved_path.clone()).or_insert_with(|| {
                        match fs::read(&resolved_path) {
                            Ok(bytes) => {
                                let mime = mime_guess::from_path(&resolved_path)
                                    .first_raw()
                                    .unwrap_or("image/jpeg");
                                let b64 = BASE64_STANDARD.encode(&bytes);
                                format!("data:{mime};base64,{b64}")
                            }
                            Err(_) => src_val.to_string(),
                        }
                    });

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

            out.push_str(tag_slice);
        } else {
            out.push_str(tag_slice);
        }

        cursor = img_end;
    }

    out.push_str(&html[cursor..]);
    out
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
        let processed = embed_images_as_base64(html, None);
        assert!(processed.contains("<a href=\"https://example.com/manual.pdf\" target=\"_blank\" rel=\"noopener noreferrer\">Download Manual</a>"));
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
        let embedded = embed_images_as_base64(&html, Some(&dir));

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
        let embedded = embed_images_as_base64(html, Some(&dir));

        let _ = fs::remove_dir_all(&dir);

        let matches: Vec<_> = embedded.matches("data:image/jpeg;base64,").collect();
        assert_eq!(matches.len(), 2);
    }
}
