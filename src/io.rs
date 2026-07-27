//! Centralized filesystem and I/O abstraction module for Doc2Flow.

use crate::error::{Doc2FlowError, Result};
use std::fs;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

/// Reads the complete content of a UTF-8 encoded text file from disk into a `String`.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use doc2flow::io::read_file_to_string;
///
/// let content = read_file_to_string(Path::new("document.md")).unwrap();
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`] if the file cannot be opened or read.
pub fn read_file_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| Doc2FlowError::Io {
        path: Some(path.to_path_buf()),
        source,
    })
}

/// Reads the raw binary bytes of a file from disk into a `Vec<u8>`.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use doc2flow::io::read_file_bytes;
///
/// let bytes = read_file_bytes(Path::new("image.png")).unwrap();
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`] if the file cannot be opened or read.
pub fn read_file_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|source| Doc2FlowError::Io {
        path: Some(path.to_path_buf()),
        source,
    })
}

/// Writes byte sequence or string data to a file on disk, creating or truncating it.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use doc2flow::io::write_file;
///
/// write_file(Path::new("output.html"), "<h1>Header</h1>").unwrap();
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`] if the file cannot be created or written to.
pub fn write_file(path: &Path, content: impl AsRef<[u8]>) -> Result<()> {
    fs::write(path, content).map_err(|source| Doc2FlowError::Io {
        path: Some(path.to_path_buf()),
        source,
    })
}

/// Retrieves the size in bytes of a target file.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use doc2flow::io::get_file_size;
///
/// let size = get_file_size(Path::new("large_image.png")).unwrap();
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`] if metadata cannot be queried for the target path.
pub fn get_file_size(path: &Path) -> Result<u64> {
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|source| Doc2FlowError::Io {
            path: Some(path.to_path_buf()),
            source,
        })
}

/// Checks whether a given filesystem path exists on disk.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use doc2flow::io::path_exists;
///
/// assert!(!path_exists(Path::new("non_existent_file_xyz.txt")));
/// ```
pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

/// Resolves a relative path against an optional base directory using platform-native `PathBuf`.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use doc2flow::io::resolve_relative_path;
///
/// let resolved = resolve_relative_path(Path::new("img.png"), Some(Path::new("docs")));
/// assert_eq!(resolved, Path::new("docs").join("img.png"));
/// ```
pub fn resolve_relative_path(path: &Path, base_dir: Option<&Path>) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(base) = base_dir {
        base.join(path)
    } else {
        path.to_path_buf()
    }
}

/// Resolves a logo file path against `base_dir` or working directory, checking file existence.
///
/// Returns the prioritized existing path combination or defaults to joining with `base_dir`.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use doc2flow::io::resolve_logo_path;
///
/// let resolved = resolve_logo_path(Path::new("logo.svg"), None);
/// assert_eq!(resolved, Path::new("logo.svg"));
/// ```
pub fn resolve_logo_path(path: &Path, base_dir: Option<&Path>) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Some(base) = base_dir {
        let combined = base.join(path);
        if path_exists(&combined) {
            return combined;
        }
    }
    if path_exists(path) {
        return path.to_path_buf();
    }
    if let Some(base) = base_dir {
        base.join(path)
    } else {
        path.to_path_buf()
    }
}

/// Resolves an image path relative to an optional base directory, returning `Some` if the file exists.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use doc2flow::io::resolve_image_path;
///
/// assert_eq!(resolve_image_path(Path::new("non_existent.png"), None), None);
/// ```
pub fn resolve_image_path(path: &Path, base_dir: Option<&Path>) -> Option<PathBuf> {
    if path.is_absolute() {
        if path_exists(path) {
            return Some(path.to_path_buf());
        }
        return None;
    }

    if let Some(base) = base_dir {
        let combined = base.join(path);
        if path_exists(&combined) {
            return Some(combined);
        }
    }

    if path_exists(path) {
        return Some(path.to_path_buf());
    }

    None
}

/// Recursively creates a directory and all missing parent directories.
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`] if directory creation fails.
pub fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| Doc2FlowError::Io {
        path: Some(path.to_path_buf()),
        source,
    })
}

/// Recursively deletes a directory and all of its contents.
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`] if directory deletion fails.
pub fn remove_dir_all(path: &Path) -> Result<()> {
    fs::remove_dir_all(path).map_err(|source| Doc2FlowError::Io {
        path: Some(path.to_path_buf()),
        source,
    })
}

/// Interactively prompts the user via stderr/stdin with a yes/no question.
///
/// Returns `false` automatically if standard input is not an interactive terminal context.
pub fn prompt_user_yes_no(prompt_msg: &str) -> bool {
    if !std::io::stdin().is_terminal() {
        return false;
    }

    eprint!("{prompt_msg}");
    let _ = std::io::stderr().flush();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim().to_lowercase();
        return trimmed == "y" || trimmed == "yes";
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_read_write_string() {
        let temp_dir = std::env::temp_dir().join("d2f_test_io_string");
        let _ = create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.txt");

        write_file(&test_file, "Hello Doc2Flow I/O").unwrap();
        assert!(path_exists(&test_file));

        let content = read_file_to_string(&test_file).unwrap();
        assert_eq!(content, "Hello Doc2Flow I/O");

        let size = get_file_size(&test_file).unwrap();
        assert_eq!(size, 18);

        let _ = remove_dir_all(&temp_dir);
        assert!(!path_exists(&test_file));
    }

    #[test]
    fn test_io_read_write_bytes() {
        let temp_dir = std::env::temp_dir().join("d2f_test_io_bytes");
        let _ = create_dir_all(&temp_dir);
        let test_file = temp_dir.join("data.bin");

        let payload = vec![0x00, 0x01, 0x02, 0xFF];
        write_file(&test_file, &payload).unwrap();

        let bytes = read_file_bytes(&test_file).unwrap();
        assert_eq!(bytes, payload);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_io_non_existent_file_errors() {
        let missing = Path::new("non_existent_file_xyz_123.tmp");
        let err_str = read_file_to_string(missing).unwrap_err();
        match err_str {
            Doc2FlowError::Io { path, .. } => assert_eq!(path, Some(missing.to_path_buf())),
            _ => panic!("Expected Doc2FlowError::Io error variant"),
        }

        let err_bytes = read_file_bytes(missing).unwrap_err();
        match err_bytes {
            Doc2FlowError::Io { path, .. } => assert_eq!(path, Some(missing.to_path_buf())),
            _ => panic!("Expected Doc2FlowError::Io error variant"),
        }
    }

    #[test]
    fn test_resolve_relative_and_logo_path() {
        let temp_dir = std::env::temp_dir().join("d2f_test_io_path_res");
        let _ = create_dir_all(&temp_dir);
        let target_file = temp_dir.join("sub/logo.svg");
        let _ = create_dir_all(target_file.parent().unwrap());
        write_file(&target_file, "<svg></svg>").unwrap();

        let rel_path = Path::new("sub/logo.svg");
        let resolved_logo = resolve_logo_path(rel_path, Some(&temp_dir));
        assert_eq!(resolved_logo, target_file);

        let resolved_img = resolve_image_path(rel_path, Some(&temp_dir));
        assert_eq!(resolved_img, Some(target_file));

        let non_existent_img = resolve_image_path(Path::new("missing.png"), Some(&temp_dir));
        assert_eq!(non_existent_img, None);

        let _ = remove_dir_all(&temp_dir);
    }
}
