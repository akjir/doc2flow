//! Image processing, SVG cleaning, and WebP compression engine.

use std::borrow::Cow;
use std::io::Cursor;
use std::path::Path;

use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat};

use crate::core::error::{DiagnosticError, Error, Result, build_caret_annotation};
use crate::core::io;
use crate::utils::{guess_mime_type, to_base64_data_uri, svg::clean_svg};

/// Allowed image file extensions for image resource checking.
const ALLOWED_IMAGE_EXTENSIONS: [&str; 11] = [
    "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "tif", "tiff", "webp",
];

/// Maximum allowed size in bytes for a local image embedded into HTML (250 KB).
pub const MAX_IMAGE_SIZE_BYTES: u64 = 250 * 1024;

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
        if mime == "image/svg+xml"
            && let Ok(utf8_str) = std::str::from_utf8(&bytes)
        {
            let cleaned = clean_svg(utf8_str);
            return Ok(to_base64_data_uri(mime, cleaned.as_bytes()));
        }
        Ok(to_base64_data_uri(mime, &bytes))
    }
}

