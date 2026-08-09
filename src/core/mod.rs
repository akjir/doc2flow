//! Core architecture abstractions and engine for Doc2Flow.

#[path = "parsing/arguments.rs"]
pub mod args;
pub mod builder;
pub mod components;
pub mod constants;
pub mod converter;
pub mod feature;
pub mod id;
pub mod image;
pub mod locales;

pub use crate::lib::{base64, error, hasher, io, mime, uri};
pub use args::{Args, help_message, parse_args};
pub use builder::{
    SCRIPT_CORE, STYLE_CORE, assemble_html, assemble_scripts, assemble_styles,
    format_iso8601_utc, generate_template_markdown, render, render_finish_box,
    render_lightbox, render_progress_bar, render_scripts, render_styles, substitute_template,
};
pub use components::*;
pub use constants::*;
pub use converter::*;
pub use feature::{DocumentContext, Feature, resolve_enabled_features};
pub use id::generate_d2f_id;
pub use image::{
    MAX_IMAGE_SIZE_BYTES, embed_images_as_base64, embed_images_as_base64_with_source, load_logo,
    resolve_logo_path,
};
pub use locales::{Locale, validate_locale_coverage};
