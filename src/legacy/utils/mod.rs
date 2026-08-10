//! Generic, project-unspecific library utilities and helpers.

pub mod base64;
pub mod error;
pub mod hasher;
pub mod io;
pub mod mime;
pub mod uri;

pub use base64::{base64_encode, base64_encode_into};
pub use error::{
    DiagnosticError, Doc2FlowError, IoResultExt, Result, build_caret_annotation, print_warning,
};
pub use hasher::{sha256, sha256_bytes};
pub use io::{
    create_dir_all, get_file_size, path_exists, prompt_user_yes_no, read_file_bytes,
    read_file_to_string, remove_dir_all, resolve_path, write_file,
};
pub use mime::guess_mime_type;
pub use uri::{file_to_data_uri, to_base64_data_uri, to_base64_data_uri_into};
