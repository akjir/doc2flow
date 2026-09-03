//! Generic utilities and helper functions for core processing.

pub mod base64;
pub mod hasher;
pub mod mime;
pub mod time;
pub mod uri;

pub use base64::{base64_encode, base64_encode_into};
pub use hasher::{Sha256, sha256, sha256_bytes};
pub use mime::guess_mime_type;
pub use time::{format_iso8601_utc, format_iso8601_utc_into};
pub use uri::{file_to_data_uri, to_base64_data_uri, to_base64_data_uri_into};

pub mod html;
pub mod image;
pub mod svg;

pub use html::extract_attribute;
pub use image::{
    MAX_IMAGE_SIZE_BYTES, is_image_source, is_remote_or_data_uri, process_and_encode_image_as_webp,
    resolve_or_encode_image,
};
pub use svg::clean_svg;
