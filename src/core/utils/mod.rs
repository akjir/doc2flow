//! Generic utilities and helper functions for core processing.

pub mod base64;
pub mod hasher;
pub mod io;
pub mod mime;
pub mod time;
pub mod uri;

pub use base64::{base64_encode, base64_encode_into};
pub use hasher::{sha256, sha256_bytes};
pub use io::{
    create_dir_all, get_file_size, path_exists, prompt_user_yes_no, read_file_bytes,
    read_file_to_string, remove_dir_all, resolve_path, write_file,
};
pub use mime::guess_mime_type;
pub use time::{format_iso8601_utc, format_iso8601_utc_into};
pub use uri::{file_to_data_uri, to_base64_data_uri, to_base64_data_uri_into};
