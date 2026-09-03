//! Image vertical slice feature module.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::error::Result;
use crate::core::feature::FeatureModule;
use crate::core::format::{escape_html_into, push_indent};
use crate::core::io;
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};
use crate::utils::{
    extract_attribute, is_image_source, is_remote_or_data_uri, resolve_or_encode_image,
};

/// Embedded image CSS stylesheet.
pub const CSS: &str = include_str!("image.css");

/// Supported document element identifiers for image elements.
const IMAGE_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::Image];

/// Embedded image JavaScript client script.
pub const JS: &str = include_str!("image.js");

/// Image feature renderer handling image embedding and lightbox preview.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImageFeature;

impl ImageFeature {
    /// Creates a new image feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::image::ImageFeature;
    ///
    /// let feature = ImageFeature::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for ImageFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &IMAGE_SUPPORTED
    }

    fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        out: &mut String,
        _renderer: &HtmlRenderer,
    ) {
        if let DocumentElement::Image { alt, url } = element {
            push_indent(out, indent);
            out.push_str("<div class=\"image-item\">\n");

            push_indent(out, indent + 1);
            out.push_str("<img src=\"");
            escape_html_into(out, url);
            out.push_str("\" alt=\"");
            escape_html_into(out, alt);
            out.push_str("\" />\n");

            push_indent(out, indent);
            out.push_str("</div>\n");
        }
    }
}

impl FeatureModule for ImageFeature {
    fn name(&self) -> &'static str {
        "image"
    }

    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    fn javascript(&self) -> &[&'static str] {
        &[JS]
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
/// Returns a compiler-style [`DiagnosticError`](crate::core::error::DiagnosticError) if a local image exceeds the 250 KB size limit.
pub fn embed_images_as_base64(html: &str, base_dir: Option<&Path>) -> Result<String> {
    embed_images_as_base64_with_source(html, None, None, base_dir, false)
}

/// Embeds local image references in HTML as Base64 `data:` URIs with Markdown source context.
/// Context parameters for batch image tag processing.
struct ImageProcessContext<'a> {
    auto_scale: bool,
    base_dir: Option<&'a Path>,
    cache: &'a mut HashMap<PathBuf, String>,
    file_name: Option<&'a str>,
    md_content: Option<&'a str>,
}

/// Post-processes rendered HTML, identifying local `<img>` tags and embedding them as Base64.
///
/// Converts large images exceeding 250 KB to WebP when `auto_scale` is true or prompts interactively.
///
/// # Errors
///
/// Returns an [`Error`] if an image exceeds the size limit and is not scaled, or if encoding fails.
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
    let mut ctx = ImageProcessContext {
        auto_scale,
        base_dir,
        cache: &mut cache,
        file_name,
        md_content,
    };

    while let Some((img_start, img_end)) = find_next_img_tag(html, cursor) {
        out.push_str(&html[cursor..img_start]);
        cursor = process_single_image_tag(&mut out, html, img_start, img_end, &mut ctx)?;
    }

    out.push_str(&html[cursor..]);
    Ok(out)
}

/// Processes a single `<img>` tag slice in the HTML stream and advances the cursor.
fn process_single_image_tag(
    out: &mut String,
    html: &str,
    img_start: usize,
    img_end: usize,
    ctx: &mut ImageProcessContext<'_>,
) -> Result<usize> {
    let tag_slice = &html[img_start..img_end];
    let Some((attr_start, attr_end, src_val)) = extract_attribute(tag_slice, "src") else {
        out.push_str(tag_slice);
        return Ok(img_end);
    };

    if !is_image_source(src_val, ctx.base_dir) {
        let alt_text = extract_attribute(tag_slice, "alt")
            .map(|(_, _, val)| val)
            .unwrap_or(src_val);
        return Ok(render_non_image_link(out, html, img_end, src_val, alt_text));
    }

    if is_remote_or_data_uri(src_val) {
        out.push_str(tag_slice);
        return Ok(img_end);
    }

    let path = Path::new(src_val);
    let Some(resolved_path) = io::resolve_path(path, ctx.base_dir) else {
        out.push_str(tag_slice);
        return Ok(img_end);
    };

    let data_uri = if let Some(cached) = ctx.cache.get(&resolved_path) {
        cached.clone()
    } else {
        let uri =
            resolve_or_encode_image(&resolved_path, src_val, ctx.auto_scale, ctx.md_content, ctx.file_name)?;
        ctx.cache.insert(resolved_path, uri.clone());
        uri
    };

    if data_uri != src_val {
        replace_img_src(out, tag_slice, attr_start, attr_end, &data_uri);
    } else {
        out.push_str(tag_slice);
    }

    Ok(img_end)
}

/// Checks if `s` starts with `<tag_name` case-insensitively followed by a delimiter or end of string.
fn starts_with_tag_ignore_ascii_case(s: &str, tag_prefix: &str) -> bool {
    let Some(prefix_slice) = s.get(..tag_prefix.len()) else {
        return false;
    };
    if !prefix_slice.eq_ignore_ascii_case(tag_prefix) {
        return false;
    }
    let Some(after) = s.get(tag_prefix.len()..) else {
        return true;
    };
    after.is_empty() || after.starts_with(|c: char| c.is_whitespace() || c == '/' || c == '>')
}

/// Finds start and end byte offsets of the next `<img` tag in `html` from `cursor`.
///
/// Skips `<script>`, `<style>`, and HTML comments to avoid parsing markup inside scripts or styles.
fn find_next_img_tag(html: &str, mut cursor: usize) -> Option<(usize, usize)> {
    while cursor < html.len() {
        let tag_start_rel = html[cursor..].find('<')?;
        let tag_start = cursor + tag_start_rel;
        let rest = &html[tag_start..];

        if rest.starts_with("<!--") {
            let comment_end = rest.find("-->")?;
            cursor = tag_start + comment_end + 3;
            continue;
        }

        if starts_with_tag_ignore_ascii_case(rest, "<script") {
            if let Some(script_end) = find_closing_tag(rest, "script") {
                cursor = tag_start + script_end;
                continue;
            }
            return None;
        }

        if starts_with_tag_ignore_ascii_case(rest, "<style") {
            if let Some(style_end) = find_closing_tag(rest, "style") {
                cursor = tag_start + style_end;
                continue;
            }
            return None;
        }

        if starts_with_tag_ignore_ascii_case(rest, "<img") {
            let rel_end = rest.find('>')?;
            let img_end = tag_start + rel_end + 1;
            return Some((tag_start, img_end));
        }

        cursor = tag_start + 1;
    }

    None
}

/// Finds the byte offset immediately following `</tag_name>` in `html`, tolerating arbitrary whitespace.
fn find_closing_tag(html: &str, tag_name: &str) -> Option<usize> {
    let mut i = 0;
    while i < html.len() {
        if let Some(pos) = html[i..].find("</") {
            let after_slash_idx = i + pos + 2;
            let Some(after_slash) = html.get(after_slash_idx..) else {
                break;
            };
            let trimmed = after_slash.trim_start();
            let ws_before = after_slash.len() - trimmed.len();

            if let Some(name_slice) = trimmed.get(..tag_name.len())
                && name_slice.eq_ignore_ascii_case(tag_name)
                && let Some(after_name) = trimmed.get(tag_name.len()..)
            {
                let after_trimmed = after_name.trim_start();
                if after_trimmed.starts_with('>') {
                    let end_pos = after_slash_idx
                        + ws_before
                        + tag_name.len()
                        + (after_name.len() - after_trimmed.len())
                        + 1;
                    return Some(end_pos);
                }
            }
            i = after_slash_idx;
        } else {
            break;
        }
    }
    None
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
        out.push_str(
            "<div class=\"item text-item item-selectable\">\n  <span class=\"text-content\"><a href=\"",
        );
        out.push_str(src_val);
        out.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
        out.push_str(alt_text);
        out.push_str("</a></span>\n</div>\n");
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

/// Replaces the `src="..."` attribute within a tag slice preserving quote style and writes to `out`.
fn replace_img_src(
    out: &mut String,
    tag_slice: &str,
    attr_start: usize,
    attr_end: usize,
    data_uri: &str,
) {
    let attr_slice = &tag_slice[attr_start..attr_end];
    let quote = if attr_slice.contains('\'') { '\'' } else { '"' };

    out.push_str(&tag_slice[..attr_start]);
    out.push_str("src=");
    out.push(quote);
    out.push_str(data_uri);
    out.push(quote);
    out.push_str(&tag_slice[attr_end..]);
}

/// Helper to unwrap `<div class="image-item">` or `<div class="img-item">` container if present around a non-image tag.
fn strip_img_item_wrapper(out: &mut String, html: &str, img_end: usize) -> Option<usize> {
    let trimmed_out = out.trim_end();
    let div_start = trimmed_out.rfind("<div")?;
    let div_slice = &trimmed_out[div_start..];
    let div_tag_end_rel = div_slice.find('>')?;
    let div_tag = &div_slice[..=div_tag_end_rel];

    if !div_slice[div_tag_end_rel + 1..].trim().is_empty() {
        return None;
    }

    let (_, _, class_val) = extract_attribute(div_tag, "class")?;
    let is_image_item = class_val
        .split_whitespace()
        .any(|c| c.eq_ignore_ascii_case("image-item") || c.eq_ignore_ascii_case("img-item"));
    if !is_image_item {
        return None;
    }

    let rest = &html[img_end..];
    let rest_trimmed = rest.trim_start();
    let close_tag_len = parse_closing_div(rest_trimmed)?;

    let leading_ws = rest.len() - rest_trimmed.len();
    let suffix_len = leading_ws + close_tag_len;
    out.truncate(div_start);
    Some(img_end + suffix_len)
}

/// Parses a closing `</div>` tag allowing flexible whitespace (e.g. `</div >`, `</ div>`), returning its byte length.
fn parse_closing_div(s: &str) -> Option<usize> {
    let stripped = s.strip_prefix("</")?;
    let trimmed = stripped.trim_start();
    if trimmed.len() >= 3 && trimmed[..3].eq_ignore_ascii_case("div") {
        let after_div = &trimmed[3..];
        let after_trimmed = after_div.trim_start();
        if after_trimmed.starts_with('>') {
            let total_len = (s.len() - stripped.len())
                + (stripped.len() - trimmed.len())
                + 3
                + (after_div.len() - after_trimmed.len())
                + 1;
            return Some(total_len);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MAX_IMAGE_SIZE_BYTES;

    #[test]
    fn test_image_feature_constructor_new() {
        let feature = ImageFeature::new();
        assert_eq!(feature, ImageFeature);
    }

    #[test]
    fn test_image_feature_css() {
        let feature = ImageFeature::new();
        let css = feature.css().expect("image css should exist");
        assert!(css.contains("--image-radius:"));
        assert!(css.contains("--image-border:"));
        assert!(css.contains("--image-fallback-bg:"));
        assert!(css.contains("--image-lightbox-bg:"));
        assert!(css.contains(".image-item"));
        assert!(css.contains(".image-fallback"));
        assert!(css.contains(".image-lightbox"));
    }

    #[test]
    fn test_image_feature_javascript() {
        let feature = ImageFeature::new();
        let js = feature.javascript();
        assert_eq!(js.len(), 1);
        let script = js[0];
        assert!(script.contains("window.d2f.image"));
        assert!(script.contains("openLightbox"));
        assert!(script.contains("closeLightbox"));
        assert!(script.contains("applyImageFallback"));
        assert!(script.contains("image-lightbox"));
    }

    #[test]
    fn test_image_feature_renders_image_element() {
        let feature = ImageFeature::new();
        let element = DocumentElement::image("Architecture Diagram", "assets/arch.png");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "  <div class=\"image-item\">\n",
            "    <img src=\"assets/arch.png\" alt=\"Architecture Diagram\" />\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_image_feature_escapes_html_attributes() {
        let feature = ImageFeature::new();
        let element = DocumentElement::image(
            "Picture <with> \"quotes\" & symbols",
            "https://example.com/pic.png?a=1&b=2",
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            2,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "    <div class=\"image-item\">\n",
            "      <img src=\"https://example.com/pic.png?a=1&amp;b=2\" alt=\"Picture &lt;with&gt; &quot;quotes&quot; &amp; symbols\" />\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_image_feature_empty_for_unsupported_elements() {
        let feature = ImageFeature::new();
        let text = DocumentElement::text("Regular text");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &text,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn test_non_image_source_converted_to_link() {
        let html = "<p><img src=\"https://example.com/manual.pdf\" alt=\"Download Manual\"></p>";
        let processed = embed_images_as_base64(html, None).unwrap();
        assert!(processed.contains("<a href=\"https://example.com/manual.pdf\" target=\"_blank\" rel=\"noopener noreferrer\">Download Manual</a>"));
        assert!(!processed.contains("<img"));
    }

    #[test]
    fn test_non_image_source_in_image_item_wrapper_converted_to_text_item() {
        let html = "<div class=\"image-item\">\n  <img src=\"https://example.com/dateien/spezifikation.pdf\" alt=\"Systemspezifikation PDF herunterladen\" />\n</div>";
        let processed = embed_images_as_base64(html, None).unwrap();
        assert!(processed.contains("<div class=\"item text-item item-selectable\">"));
        assert!(processed.contains("<span class=\"text-content\"><a href=\"https://example.com/dateien/spezifikation.pdf\" target=\"_blank\" rel=\"noopener noreferrer\">Systemspezifikation PDF herunterladen</a></span>"));
        assert!(!processed.contains("class=\"image-item\""));
        assert!(!processed.contains("<img"));
    }

    #[test]
    fn test_non_image_source_in_legacy_img_item_wrapper_converted_to_text_item() {
        let html = "<div class=\"img-item\">\n  <img src=\"https://example.com/dateien/spezifikation.pdf\" alt=\"Systemspezifikation PDF herunterladen\">\n</div>";
        let processed = embed_images_as_base64(html, None).unwrap();
        assert!(processed.contains("<div class=\"item text-item item-selectable\">"));
        assert!(processed.contains("<span class=\"text-content\"><a href=\"https://example.com/dateien/spezifikation.pdf\" target=\"_blank\" rel=\"noopener noreferrer\">Systemspezifikation PDF herunterladen</a></span>"));
        assert!(!processed.contains("class=\"img-item\""));
        assert!(!processed.contains("<img"));
    }

    #[test]
    fn test_non_image_source_in_wrapper_with_flexible_whitespace_and_attributes() {
        let html = "<div   class=\"extra-class image-item\"  id=\"wrapper-1\" >\n  <img src='https://example.com/archive.zip' alt='Archive'  />\n</div  >";
        let processed = embed_images_as_base64(html, None).unwrap();
        assert!(processed.contains("<div class=\"item text-item item-selectable\">"));
        assert!(processed.contains("<span class=\"text-content\"><a href=\"https://example.com/archive.zip\" target=\"_blank\" rel=\"noopener noreferrer\">Archive</a></span>"));
        assert!(!processed.contains("image-item"));
        assert!(!processed.contains("<img"));
    }

    #[test]
    fn test_embed_images_as_base64_local_file() {
        let dir = std::env::temp_dir().join(format!("d2f_test_img_1_{}", std::process::id()));
        let _ = io::create_dir_all(&dir);
        let img_path = dir.join("test.png");
        io::write_file(&img_path, b"fake png content").unwrap();

        let html = "<p><img src=\"test.png\" alt=\"demo\"></p>";
        let embedded = embed_images_as_base64(html, Some(&dir)).unwrap();

        let _ = io::remove_dir_all(&dir);

        assert!(embedded.contains("src=\"data:image/png;base64,"));
        assert!(!embedded.contains("src=\"test.png\""));
    }

    #[test]
    fn test_embed_images_preserves_single_quotes() {
        let dir = std::env::temp_dir().join(format!("d2f_test_img_sq_{}", std::process::id()));
        let _ = io::create_dir_all(&dir);
        let img_path = dir.join("single.png");
        io::write_file(&img_path, b"single quote image data").unwrap();

        let html = "<p><img src='single.png' alt='Single Demo'></p>";
        let embedded = embed_images_as_base64(html, Some(&dir)).unwrap();

        let _ = io::remove_dir_all(&dir);

        assert!(embedded.contains("src='data:image/png;base64,"));
        assert!(!embedded.contains("src='single.png'"));
    }

    #[test]
    fn test_embed_images_deduplication() {
        let dir = std::env::temp_dir().join(format!("d2f_test_img_2_{}", std::process::id()));
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
        let dir = std::env::temp_dir().join(format!("d2f_test_auto_scale_{}", std::process::id()));
        let _ = io::create_dir_all(&dir);
        let img_path = dir.join("big_photo.png");

        let img_buf = image::RgbImage::new(1000, 1000);
        img_buf
            .save_with_format(&img_path, image::ImageFormat::Png)
            .unwrap();

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
    fn test_embed_images_skips_script_and_style_tags() {
        let html = "<script>const x = '<img src=\"\" alt=\"\" />';</script><style>/* <img src=\"test.png\"> */</style><img src=\"https://example.com/pic.png\">";
        let result = embed_images_as_base64(html, None).unwrap();
        assert!(result.contains("<script>const x = '<img src=\"\" alt=\"\" />';</script>"));
        assert!(result.contains("<style>/* <img src=\"test.png\"> */</style>"));
        assert!(result.contains("<img src=\"https://example.com/pic.png\">"));
    }

    #[test]
    fn test_embed_images_skips_script_and_style_whitespace_variations() {
        let html = "<script type=\"text/javascript\" >var s = '<img src=\"bad.pdf\">';</ script  ><style  >/* <img src='fail.pdf'> */</  style ><img src=\"https://example.com/pic.png\">";
        let result = embed_images_as_base64(html, None).unwrap();
        assert!(result.contains(
            "<script type=\"text/javascript\" >var s = '<img src=\"bad.pdf\">';</ script  >"
        ));
        assert!(result.contains("<style  >/* <img src='fail.pdf'> */</  style >"));
        assert!(result.contains("<img src=\"https://example.com/pic.png\">"));
    }

    #[test]
    fn test_embed_images_skips_html_comments() {
        let html = "<!-- <img src=\"manual.pdf\" alt=\"manual\"> --><img src=\"https://example.com/pic.png\">";
        let result = embed_images_as_base64(html, None).unwrap();
        assert!(result.contains("<!-- <img src=\"manual.pdf\" alt=\"manual\"> -->"));
        assert!(result.contains("<img src=\"https://example.com/pic.png\">"));
    }
}
