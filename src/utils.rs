//! Utility module providing custom implementations for Base64 encoding,
//! MIME type guessing, file Data-URI conversion, and zero-dependency CLI argument parsing.

use crate::error::Result;
use crate::io;
use std::path::{Path, PathBuf};

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Static mapping of file extensions to standard MIME content types.
const MIME_TYPES: &[(&[&str], &str)] = &[
    (&["png"], "image/png"),
    (&["jpg", "jpeg"], "image/jpeg"),
    (&["webp"], "image/webp"),
    (&["svg"], "image/svg+xml"),
    (&["gif"], "image/gif"),
    (&["bmp"], "image/bmp"),
    (&["ico"], "image/x-icon"),
    (&["avif"], "image/avif"),
    (&["tiff", "tif"], "image/tiff"),
    (&["pdf"], "application/pdf"),
    (&["zip"], "application/zip"),
    (&["html", "htm"], "text/html"),
    (&["css"], "text/css"),
    (&["js"], "text/javascript"),
    (&["json"], "application/json"),
    (&["txt"], "text/plain"),
];

/// Encodes binary data into an RFC 4648 standard Base64 string representation.
///
/// Pre-allocates exact capacity and uses fast byte chunking to avoid heap reallocation.
///
/// # Examples
///
/// ```
/// use doc2flow::utils::base64_encode;
///
/// assert_eq!(base64_encode(b"foo"), "Zm9v");
/// ```
pub fn base64_encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let capacity = data.len().div_ceil(3) * 4;
    let mut buf = Vec::with_capacity(capacity);

    let chunks = data.chunks_exact(3);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];

        buf.push(BASE64_CHARS[(b0 >> 2) as usize]);
        buf.push(BASE64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize]);
        buf.push(BASE64_CHARS[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize]);
        buf.push(BASE64_CHARS[(b2 & 0x3F) as usize]);
    }

    match remainder.len() {
        1 => {
            let b0 = remainder[0];
            buf.push(BASE64_CHARS[(b0 >> 2) as usize]);
            buf.push(BASE64_CHARS[((b0 & 0x03) << 4) as usize]);
            buf.push(b'=');
            buf.push(b'=');
        }
        2 => {
            let b0 = remainder[0];
            let b1 = remainder[1];
            buf.push(BASE64_CHARS[(b0 >> 2) as usize]);
            buf.push(BASE64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize]);
            buf.push(BASE64_CHARS[((b1 & 0x0F) << 2) as usize]);
            buf.push(b'=');
        }
        _ => {}
    }

    String::from_utf8(buf).expect("Base64 output must be valid UTF-8")
}


/// Guesses the MIME type based on a file path extension without heap allocations.
///
/// Returns `application/octet-stream` as a safe fallback when the extension is unknown.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use doc2flow::utils::guess_mime_type;
///
/// assert_eq!(guess_mime_type(Path::new("image.png")), "image/png");
/// assert_eq!(guess_mime_type(Path::new("file.unknown")), "application/octet-stream");
/// ```
pub fn guess_mime_type(path: &Path) -> &'static str {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return "application/octet-stream";
    };

    MIME_TYPES
        .iter()
        .find(|(exts, _)| exts.iter().any(|&e| e.eq_ignore_ascii_case(ext)))
        .map_or("application/octet-stream", |(_, mime)| *mime)
}

/// Reads a local file and encodes its content into a Base64 Data URI string.
///
/// Allocates a single `String` buffer sized exactly to hold the Data URI header
/// and Base64 body without secondary buffer allocations.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use doc2flow::utils::file_to_data_uri;
///
/// let uri = file_to_data_uri(Path::new("test.png")).unwrap();
/// assert!(uri.starts_with("data:image/png;base64,"));
/// ```
///
/// # Errors
///
/// Returns [`Doc2FlowError::Io`] if the file cannot be read.
pub fn file_to_data_uri(path: &Path) -> Result<String> {
    let mime = guess_mime_type(path);
    let bytes = io::read_file_bytes(path)?;

    let b64_len = bytes.len().div_ceil(3) * 4;
    let prefix = "data:";
    let suffix = ";base64,";
    let capacity = prefix.len() + mime.len() + suffix.len() + b64_len;

    let mut out = String::with_capacity(capacity);
    out.push_str(prefix);
    out.push_str(mime);
    out.push_str(suffix);
    out.push_str(&base64_encode(&bytes));
    Ok(out)
}

/// Parsed command line arguments for the `d2f` executable.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Args {
    /// Path to the input Markdown file.
    pub input: Option<PathBuf>,
    /// Path to the output HTML file (optional).
    pub output: Option<PathBuf>,
    /// Optional target path for generating a starter template Markdown file.
    pub init: Option<PathBuf>,
    /// Optional path to a custom logo image file (SVG, PNG, JPG, WebP).
    pub logo: Option<PathBuf>,
    /// Automatically resize local images exceeding 250 KB to WebP.
    pub auto_scale: bool,
    /// Whether the user requested help information.
    pub show_help: bool,
    /// Whether the user requested version information.
    pub show_version: bool,
}

/// Parses raw command-line arguments into a structured [`Args`] struct.
///
/// # Errors
///
/// Returns an error string if an invalid option or missing parameter value is provided.
///
/// # Examples
///
/// ```
/// use doc2flow::utils::parse_args;
///
/// let args = parse_args(&["d2f", "doc.md", "-s"]).unwrap();
/// assert_eq!(args.input.unwrap().to_str().unwrap(), "doc.md");
/// assert!(args.auto_scale);
/// ```
pub fn parse_args<I, T>(args: I) -> Result<Args, String>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let mut parsed = Args::default();
    let mut iter = args.into_iter().peekable();

    // Skip executable path if present
    let _exec = iter.next();

    while let Some(arg) = iter.next() {
        let arg_str = arg.as_ref();
        match arg_str {
            "-h" | "--help" => parsed.show_help = true,
            "-V" | "--version" => parsed.show_version = true,
            "-s" | "--auto-scale" => parsed.auto_scale = true,
            "-o" | "--output" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Option '--output' requires a path value".to_string())?;
                parsed.output = Some(PathBuf::from(val.as_ref()));
            }
            "-l" | "--logo" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "Option '--logo' requires a path value".to_string())?;
                parsed.logo = Some(PathBuf::from(val.as_ref()));
            }
            "-i" | "--init" => {
                if let Some(next_arg) = iter.peek() {
                    let next_str = next_arg.as_ref();
                    if next_str.starts_with('-') {
                        parsed.init = Some(PathBuf::from("template.md"));
                    } else {
                        let val = iter.next().expect("peeked value must be present");
                        parsed.init = Some(PathBuf::from(val.as_ref()));
                    }
                } else {
                    parsed.init = Some(PathBuf::from("template.md"));
                }
            }
            opt if opt.starts_with("--output=") => {
                let val = &opt["--output=".len()..];
                if val.is_empty() {
                    return Err("Option '--output' requires a non-empty path value".to_string());
                }
                parsed.output = Some(PathBuf::from(val));
            }
            opt if opt.starts_with("--logo=") => {
                let val = &opt["--logo=".len()..];
                if val.is_empty() {
                    return Err("Option '--logo' requires a non-empty path value".to_string());
                }
                parsed.logo = Some(PathBuf::from(val));
            }
            opt if opt.starts_with("-l=") => {
                let val = &opt["-l=".len()..];
                if val.is_empty() {
                    return Err("Option '--logo' requires a non-empty path value".to_string());
                }
                parsed.logo = Some(PathBuf::from(val));
            }
            opt if opt.starts_with("--init=") => {
                let val = &opt["--init=".len()..];
                if val.is_empty() {
                    parsed.init = Some(PathBuf::from("template.md"));
                } else {
                    parsed.init = Some(PathBuf::from(val));
                }
            }
            opt if opt.starts_with("-i=") => {
                let val = &opt["-i=".len()..];
                if val.is_empty() {
                    parsed.init = Some(PathBuf::from("template.md"));
                } else {
                    parsed.init = Some(PathBuf::from(val));
                }
            }
            opt if opt.starts_with('-') => {
                return Err(format!("Unrecognized option '{arg_str}'"));
            }
            _ => {
                if parsed.input.is_some() {
                    return Err(format!("Unexpected positional argument '{arg_str}'"));
                }
                parsed.input = Some(PathBuf::from(arg_str));
            }
        }
    }

    Ok(parsed)
}

/// Returns the formatted CLI help text for `d2f`.
pub fn help_message() -> &'static str {
    concat!(
        "Doc2Flow (d2f)\n",
        "Converts structured Markdown documents into standalone offline HTML flowcharts.\n\n",
        "USAGE:\n",
        "    d2f [OPTIONS] [INPUT]\n\n",
        "ARGS:\n",
        "    <INPUT>    Path to the input Markdown file\n\n",
        "OPTIONS:\n",
        "    -o, --output <PATH>         Path to the output HTML file (optional)\n",
        "    -l, --logo <PATH>           Path to a custom logo image (SVG, PNG, JPG, WebP)\n",
        "    -i, --init [PATH]           Generate a starter template Markdown file (default: template.md)\n",
        "    -s, --auto-scale            Automatically resize local images exceeding 250 KB to WebP\n",
        "    -h, --help                  Print help information\n",
        "    -V, --version               Print version information\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Doc2FlowError;

    #[test]
    fn test_base64_rfc4648_test_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_base64_binary_bytes() {
        let input = vec![0x00, 0x01, 0x02, 0xFE, 0xFF];
        let encoded = base64_encode(&input);
        assert_eq!(encoded, "AAEC/v8=");
    }

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

    #[test]
    fn test_file_to_data_uri_success_and_error() {
        let temp_dir = std::env::temp_dir().join("d2f_test_data_uri");
        let _ = io::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("sample.png");
        io::write_file(&test_file, b"test image payload").unwrap();

        let data_uri = file_to_data_uri(&test_file).expect("should convert file to data uri");
        assert!(data_uri.starts_with("data:image/png;base64,"));
        let expected_b64 = base64_encode(b"test image payload");
        assert_eq!(data_uri, format!("data:image/png;base64,{expected_b64}"));

        let non_existent = temp_dir.join("does_not_exist.png");
        let err = file_to_data_uri(&non_existent).unwrap_err();
        match err {
            Doc2FlowError::Io { path, .. } => {
                assert_eq!(path, Some(non_existent));
            }
            _ => panic!("Expected Doc2FlowError::Io error variant"),
        }

        let _ = io::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_parse_args_defaults_and_positional() {
        let args = parse_args(&["d2f", "input.md"]).unwrap();
        assert_eq!(args.input, Some(PathBuf::from("input.md")));
        assert_eq!(args.output, None);
        assert_eq!(args.init, None);
        assert!(!args.auto_scale);
        assert!(!args.show_help);
        assert!(!args.show_version);
    }

    #[test]
    fn test_parse_args_options() {
        let args = parse_args(&[
            "d2f",
            "input.md",
            "-o",
            "output.html",
            "-s",
            "--init=custom_tpl.md",
        ])
        .unwrap();
        assert_eq!(args.input, Some(PathBuf::from("input.md")));
        assert_eq!(args.output, Some(PathBuf::from("output.html")));
        assert_eq!(args.init, Some(PathBuf::from("custom_tpl.md")));
        assert!(args.auto_scale);
    }

    #[test]
    fn test_parse_args_init_defaults() {
        let args = parse_args(&["d2f", "-i"]).unwrap();
        assert_eq!(args.init, Some(PathBuf::from("template.md")));

        let args2 = parse_args(&["d2f", "--init"]).unwrap();
        assert_eq!(args2.init, Some(PathBuf::from("template.md")));
    }

    #[test]
    fn test_parse_args_help_and_version() {
        let args = parse_args(&["d2f", "-h"]).unwrap();
        assert!(args.show_help);

        let args_v = parse_args(&["d2f", "--version"]).unwrap();
        assert!(args_v.show_version);
    }

    #[test]
    fn test_parse_args_logo_options() {
        let args_l = parse_args(&["d2f", "input.md", "-l", "my_logo.png"]).unwrap();
        assert_eq!(args_l.logo, Some(PathBuf::from("my_logo.png")));

        let args_long = parse_args(&["d2f", "input.md", "--logo", "brand/logo.svg"]).unwrap();
        assert_eq!(args_long.logo, Some(PathBuf::from("brand/logo.svg")));

        let args_eq = parse_args(&["d2f", "input.md", "--logo=assets/logo.webp"]).unwrap();
        assert_eq!(args_eq.logo, Some(PathBuf::from("assets/logo.webp")));

        let args_short_eq = parse_args(&["d2f", "input.md", "-l=assets/logo.png"]).unwrap();
        assert_eq!(args_short_eq.logo, Some(PathBuf::from("assets/logo.png")));
    }

    #[test]
    fn test_parse_args_errors() {
        assert!(parse_args(&["d2f", "--unknown"]).is_err());
        assert!(parse_args(&["d2f", "-o"]).is_err());
        assert!(parse_args(&["d2f", "-l"]).is_err());
        assert!(parse_args(&["d2f", "--logo="]).is_err());
        assert!(parse_args(&["d2f", "-l="]).is_err());
        assert!(parse_args(&["d2f", "input1.md", "input2.md"]).is_err());
    }
}
