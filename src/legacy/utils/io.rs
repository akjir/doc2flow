//! Centralized filesystem and I/O abstraction module.

use super::error::{IoResultExt, Result};
use std::fs;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

/// Reads the complete content of a UTF-8 encoded text file from disk into a `String`.
///
/// # Examples
///
/// ```no_run
/// use doc2flow::legacy::utils::io::read_file_to_string;
///
/// let content = read_file_to_string("document.md").unwrap();
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`](crate::legacy::utils::error::Doc2FlowError::Io) if the file cannot be opened or read.
pub fn read_file_to_string(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).with_path(path)
}

/// Reads the raw binary bytes of a file from disk into a `Vec<u8>`.
///
/// # Examples
///
/// ```no_run
/// use doc2flow::legacy::utils::io::read_file_bytes;
///
/// let bytes = read_file_bytes("image.png").unwrap();
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`](crate::legacy::utils::error::Doc2FlowError::Io) if the file cannot be opened or read.
pub fn read_file_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    fs::read(path).with_path(path)
}

/// Writes byte sequence or string data to a file on disk, creating or truncating it.
///
/// # Examples
///
/// ```no_run
/// use doc2flow::legacy::utils::io::write_file;
///
/// write_file("output.html", "<h1>Header</h1>").unwrap();
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`](crate::legacy::utils::error::Doc2FlowError::Io) if the file cannot be created or written to.
pub fn write_file(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    fs::write(path, content).with_path(path)
}

/// Retrieves the size in bytes of a target file.
///
/// # Examples
///
/// ```no_run
/// use doc2flow::legacy::utils::io::get_file_size;
///
/// let size = get_file_size("large_image.png").unwrap();
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`](crate::legacy::utils::error::Doc2FlowError::Io) if metadata cannot be queried for the target path.
pub fn get_file_size(path: impl AsRef<Path>) -> Result<u64> {
    let path = path.as_ref();
    fs::metadata(path)
        .map(|m| m.len())
        .with_path(path)
}

/// Checks whether a given filesystem path exists on disk.
///
/// # Examples
///
/// ```
/// use doc2flow::legacy::utils::io::path_exists;
///
/// assert!(!path_exists("non_existent_file_xyz.txt"));
/// ```
#[inline]
pub fn path_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// Resolves a file path against an optional base directory, returning `Some(PathBuf)` if the file exists.
///
/// If `path` is absolute, checks if it exists.
/// If `path` is relative, first checks `base_dir.join(path)` if `base_dir` is provided,
/// then falls back to checking `path` relative to the current working directory.
///
/// # Examples
///
/// ```
/// use doc2flow::legacy::utils::io::resolve_path;
///
/// assert_eq!(resolve_path("non_existent_file_xyz.txt", None::<&str>), None);
/// ```
pub fn resolve_path(
    path: impl AsRef<Path>,
    base_dir: Option<impl AsRef<Path>>,
) -> Option<PathBuf> {
    let path = path.as_ref();
    let base_dir = base_dir.as_ref().map(|b| b.as_ref());

    if path.is_absolute() {
        return path_exists(path).then(|| path.to_path_buf());
    }

    if let Some(base) = base_dir {
        let combined = base.join(path);
        if path_exists(&combined) {
            return Some(combined);
        }
    }

    path_exists(path).then(|| path.to_path_buf())
}

/// Recursively creates a directory and all missing parent directories.
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`](crate::legacy::utils::error::Doc2FlowError::Io) if directory creation fails.
pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path).with_path(path)
}

/// Recursively deletes a directory and all of its contents.
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`](crate::legacy::utils::error::Doc2FlowError::Io) if directory deletion fails.
pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    fs::remove_dir_all(path).with_path(path)
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
        let trimmed = input.trim();
        return trimmed.eq_ignore_ascii_case("y") || trimmed.eq_ignore_ascii_case("yes");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::utils::error::Doc2FlowError;

    struct TestTempDir {
        path: PathBuf,
    }

    impl TestTempDir {
        fn new(prefix: &str) -> Self {
            let path = std::env::temp_dir().join(format!("d2f_test_{prefix}_{}", std::process::id()));
            let _ = create_dir_all(&path);
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestTempDir {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_io_read_write_string() {
        let temp_dir = TestTempDir::new("io_string");
        let test_file = temp_dir.path().join("test.txt");

        write_file(&test_file, "Hello Doc2Flow I/O").unwrap();
        assert!(path_exists(&test_file));

        let content = read_file_to_string(&test_file).unwrap();
        assert_eq!(content, "Hello Doc2Flow I/O");

        let size = get_file_size(&test_file).unwrap();
        assert_eq!(size, 18);
    }

    #[test]
    fn test_io_read_write_bytes() {
        let temp_dir = TestTempDir::new("io_bytes");
        let test_file = temp_dir.path().join("data.bin");

        let payload = vec![0x00, 0x01, 0x02, 0xFF];
        write_file(&test_file, &payload).unwrap();

        let bytes = read_file_bytes(&test_file).unwrap();
        assert_eq!(bytes, payload);
    }

    #[test]
    fn test_io_non_existent_file_errors() {
        let missing = "non_existent_file_xyz_123.tmp";
        let err_str = read_file_to_string(missing).unwrap_err();
        match err_str {
            Doc2FlowError::Io { path, .. } => assert_eq!(path, Some(PathBuf::from(missing))),
            _ => panic!("Expected Doc2FlowError::Io error variant"),
        }

        let err_bytes = read_file_bytes(missing).unwrap_err();
        match err_bytes {
            Doc2FlowError::Io { path, .. } => assert_eq!(path, Some(PathBuf::from(missing))),
            _ => panic!("Expected Doc2FlowError::Io error variant"),
        }
    }

    #[test]
    fn test_resolve_path() {
        let temp_dir = TestTempDir::new("path_res");
        let target_file = temp_dir.path().join("sub/resource.svg");
        create_dir_all(target_file.parent().unwrap()).unwrap();
        write_file(&target_file, "<svg></svg>").unwrap();

        let rel_path = "sub/resource.svg";
        let resolved = resolve_path(rel_path, Some(temp_dir.path()));
        assert_eq!(resolved, Some(target_file));

        let non_existent = resolve_path("missing.png", Some(temp_dir.path()));
        assert_eq!(non_existent, None);
    }
}
