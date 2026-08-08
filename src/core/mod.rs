//! Core architecture abstractions and engine for Doc2Flow.

#[path = "parsing/arguments.rs"]
pub mod args;
#[path = "utils/base64.rs"]
pub mod base64;
pub mod components;
pub mod constants;
pub mod converter;
pub mod error;
pub mod feature;
pub mod generator;
#[path = "utils/hasher.rs"]
pub mod hasher;
pub mod id;
pub mod image;
pub mod io;
pub mod locales;
#[path = "utils/mime.rs"]
pub mod mime;
pub mod template;
#[path = "utils/uri.rs"]
pub mod uri;

pub use args::{Args, help_message, parse_args};
pub use base64::{base64_encode, base64_encode_into};
pub use components::*;
pub use constants::*;
pub use converter::*;
pub use error::{
    DiagnosticError, Doc2FlowError, IoResultExt, Result, build_caret_annotation, print_warning,
};
pub use feature::{DocumentContext, Feature, resolve_enabled_features};
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
pub use mime::guess_mime_type;
pub use template::{
    format_iso8601_utc, generate_template_markdown, render, render_scripts, substitute_template,
};
pub use uri::{file_to_data_uri, to_base64_data_uri, to_base64_data_uri_into};
