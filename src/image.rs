//! Image resolution and Base64 embedding module.

use base64::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Embeds local image references in HTML as Base64 `data:` URIs.
///
/// Scans `<img ... src="..." ...>` tags in the input HTML. If a `src` attribute points to a local
/// file, the file is read, Base64 encoded, and replaced with a `data:<mime>;base64,<encoded>` URI.
///
/// Images are cached by resolved path so each unique image file is read and encoded only once
/// to avoid redundant file operations and duplicate conversions.
pub fn embed_images_as_base64(html: &str, base_dir: Option<&Path>) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cache: HashMap<PathBuf, String> = HashMap::new();
    let mut cursor = 0;

    while let Some(img_start_rel) = html[cursor..].find("<img") {
        let img_start = cursor + img_start_rel;
        out.push_str(&html[cursor..img_start]);

        // Find closing tag '>'
        let img_end = match html[img_start..].find('>') {
            Some(rel_end) => img_start + rel_end + 1,
            None => {
                out.push_str(&html[img_start..]);
                cursor = html.len();
                break;
            }
        };

        let tag_slice = &html[img_start..img_end];
        let mut replaced_tag = tag_slice.to_string();

        if let Some((attr_start, attr_end, src_val)) = extract_src_attribute(tag_slice) {
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
                        let new_attr = format!("src=\"{}\"", data_uri);
                        replaced_tag.replace_range(attr_start..attr_end, &new_attr);
                    }
                }
            }
        }

        out.push_str(&replaced_tag);
        cursor = img_end;
    }

    out.push_str(&html[cursor..]);
    out
}

/// Helper to extract src attribute bounds and value from an `<img>` tag slice.
fn extract_src_attribute(tag: &str) -> Option<(usize, usize, &str)> {
    let lower_tag = tag.to_lowercase();
    let mut search_idx = 0;

    while let Some(pos) = lower_tag[search_idx..].find("src=") {
        let abs_pos = search_idx + pos;
        let rest = tag[abs_pos + 4..].trim_start();
        let quote = rest.chars().next()?;

        if quote == '"' || quote == '\'' {
            let val_start = abs_pos + 4 + (tag[abs_pos + 4..].len() - rest.len()) + 1;
            let val_rest = &tag[val_start..];
            if let Some(val_end_rel) = val_rest.find(quote) {
                let val_end = val_start + val_end_rel;
                let attr_end = val_end + 1;
                let src_val = &tag[val_start..val_end];
                return Some((abs_pos, attr_end, src_val));
            }
        }
        search_idx = abs_pos + 4;
    }
    None
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
    fn test_extract_src_attribute() {
        let tag = "<img src=\"images/pic.png\" alt=\"test\">";
        let (start, end, val) = extract_src_attribute(tag).unwrap();
        assert_eq!(&tag[start..end], "src=\"images/pic.png\"");
        assert_eq!(val, "images/pic.png");
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
