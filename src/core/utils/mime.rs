//! Extension-based MIME type guessing without external database lookups.

use std::path::Path;

/// Guesses the MIME type based on a file path extension without heap allocations.
///
/// Returns `application/octet-stream` as a safe fallback when the extension is unknown.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use doc2flow::guess_mime_type;
///
/// assert_eq!(guess_mime_type(Path::new("image.png")), "image/png");
/// assert_eq!(guess_mime_type(Path::new("file.unknown")), "application/octet-stream");
/// ```
#[inline]
pub fn guess_mime_type(path: &Path) -> &'static str {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return "application/octet-stream";
    };

    match ext {
        e if e.eq_ignore_ascii_case("png") => "image/png",
        e if e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg") => "image/jpeg",
        e if e.eq_ignore_ascii_case("webp") => "image/webp",
        e if e.eq_ignore_ascii_case("svg") => "image/svg+xml",
        e if e.eq_ignore_ascii_case("gif") => "image/gif",
        e if e.eq_ignore_ascii_case("bmp") => "image/bmp",
        e if e.eq_ignore_ascii_case("ico") => "image/x-icon",
        e if e.eq_ignore_ascii_case("avif") => "image/avif",
        e if e.eq_ignore_ascii_case("tiff") || e.eq_ignore_ascii_case("tif") => "image/tiff",
        e if e.eq_ignore_ascii_case("pdf") => "application/pdf",
        e if e.eq_ignore_ascii_case("zip") => "application/zip",
        e if e.eq_ignore_ascii_case("html") || e.eq_ignore_ascii_case("htm") => "text/html",
        e if e.eq_ignore_ascii_case("css") => "text/css",
        e if e.eq_ignore_ascii_case("js") => "text/javascript",
        e if e.eq_ignore_ascii_case("json") => "application/json",
        e if e.eq_ignore_ascii_case("txt") => "text/plain",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_mime_type_images() {
        assert_eq!(guess_mime_type(Path::new("test.png")), "image/png");
        assert_eq!(guess_mime_type(Path::new("test.PNG")), "image/png");
        assert_eq!(guess_mime_type(Path::new("test.jpg")), "image/jpeg");
        assert_eq!(guess_mime_type(Path::new("test.jpeg")), "image/jpeg");
        assert_eq!(guess_mime_type(Path::new("test.webp")), "image/webp");
        assert_eq!(guess_mime_type(Path::new("test.svg")), "image/svg+xml");
        assert_eq!(guess_mime_type(Path::new("test.gif")), "image/gif");
        assert_eq!(guess_mime_type(Path::new("test.bmp")), "image/bmp");
        assert_eq!(guess_mime_type(Path::new("test.ico")), "image/x-icon");
        assert_eq!(guess_mime_type(Path::new("test.avif")), "image/avif");
        assert_eq!(guess_mime_type(Path::new("test.tiff")), "image/tiff");
        assert_eq!(guess_mime_type(Path::new("test.tif")), "image/tiff");
    }

    #[test]
    fn test_guess_mime_type_assets_and_fallbacks() {
        assert_eq!(guess_mime_type(Path::new("doc.pdf")), "application/pdf");
        assert_eq!(guess_mime_type(Path::new("archive.zip")), "application/zip");
        assert_eq!(guess_mime_type(Path::new("index.html")), "text/html");
        assert_eq!(guess_mime_type(Path::new("style.css")), "text/css");
        assert_eq!(guess_mime_type(Path::new("app.js")), "text/javascript");
        assert_eq!(guess_mime_type(Path::new("data.json")), "application/json");
        assert_eq!(guess_mime_type(Path::new("notes.txt")), "text/plain");

        assert_eq!(
            guess_mime_type(Path::new("file.unknown_extension")),
            "application/octet-stream"
        );
        assert_eq!(
            guess_mime_type(Path::new("no_extension")),
            "application/octet-stream"
        );
    }
}
