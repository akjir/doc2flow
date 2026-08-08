//! Core architecture abstractions and engine for Doc2Flow.

pub mod args;
pub mod components;
pub mod converter;
pub mod error;
pub mod feature;
pub mod generator;
pub mod hasher;
pub mod id;
pub mod image;
pub mod io;
pub mod locales;
pub mod template;
pub mod utils;

pub use args::{Args, help_message, parse_args};
pub use components::*;
pub use converter::*;
pub use error::{DiagnosticError, Doc2FlowError, Result, print_warning};
pub use feature::{DocumentContext, Feature};
pub use generator::{SCRIPT_CORE, STYLE_CORE, assemble_html, assemble_scripts, assemble_styles};
pub use hasher::{generate_doc_id, sha256, sha256_bytes};
pub use id::generate_d2f_id;
pub use image::{
    MAX_IMAGE_SIZE_BYTES, embed_images_as_base64, embed_images_as_base64_with_source, load_logo,
};
pub use io::{
    get_file_size, path_exists, read_file_bytes, read_file_to_string, resolve_image_path,
    resolve_logo_path, resolve_relative_path, write_file,
};
pub use locales::{Locale, validate_locale_coverage};
pub use template::{
    APP_VERSION, LICENSE_TERMS, LICENSE_URL, REPOSITORY_URL, format_iso8601_utc,
    generate_template_markdown, render, render_scripts, substitute_template,
};
pub use utils::{base64_encode, base64_encode_into, file_to_data_uri, guess_mime_type};
