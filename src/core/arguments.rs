//! Command line argument parsing and validation for Doc2Flow.
//!
//! Provides zero-dependency command line argument parsing, structured options,
//! and help message generation for the `d2f` executable.

use std::ffi::OsStr;
use std::path::PathBuf;

use super::error::CliError;

/// Default filename used when creating starter templates via `--init`.
const DEFAULT_TEMPLATE_NAME: &str = "template.md";

/// Parsed command line arguments for the `d2f` executable.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Args {
    /// Automatically resize local images exceeding 250 KB to WebP.
    pub auto_scale: bool,
    /// Optional target path for generating a starter template Markdown file.
    pub init: Option<PathBuf>,
    /// Path to the input Markdown file.
    pub input: Option<PathBuf>,
    /// Whether to run using the legacy pipeline.
    pub legacy: bool,
    /// Optional path to a custom logo image file (SVG, PNG, JPG, WebP).
    pub logo: Option<PathBuf>,
    /// Path to the output HTML file (optional).
    pub output: Option<PathBuf>,
    /// Whether the user requested help information.
    pub show_help: bool,
    /// Whether the user requested version information.
    pub show_version: bool,
}

/// Returns the formatted CLI help text for `d2f`.
///
/// # Examples
///
/// ```
/// use doc2flow::core::arguments::help_message;
///
/// assert!(help_message().contains("Doc2Flow (d2f)"));
/// ```
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
        "        --legacy                Run using the legacy processing pipeline\n",
        "    -h, --help                  Print help information\n",
        "    -V, --version               Print version information\n"
    )
}

/// Parses raw command-line arguments into a structured [`Args`] struct.
///
/// # Examples
///
/// ```
/// use doc2flow::core::arguments::parse_args;
///
/// let args = parse_args(["doc.md", "-s"]).unwrap();
/// assert_eq!(args.input.unwrap().to_str().unwrap(), "doc.md");
/// assert!(args.auto_scale);
/// ```
///
/// # Errors
///
/// Returns a [`CliError`] if an invalid option, missing parameter value,
/// or unexpected positional argument is provided.
pub fn parse_args<I, T>(args: I) -> Result<Args, CliError>
where
    I: IntoIterator<Item = T>,
    T: AsRef<OsStr>,
{
    let mut parsed = Args::default();
    let mut iter = args.into_iter().peekable();

    while let Some(arg) = iter.next() {
        let arg_os = arg.as_ref();
        if let Some(arg_str) = arg_os.to_str() {
            if arg_str.starts_with('-') {
                let (key, inline_val) = arg_str
                    .split_once('=')
                    .map_or((arg_str, None), |(k, v)| (k, Some(OsStr::new(v))));

                match key {
                    "-h" | "--help" if inline_val.is_none() => parsed.show_help = true,
                    "-V" | "--version" if inline_val.is_none() => parsed.show_version = true,
                    "-s" | "--auto-scale" if inline_val.is_none() => parsed.auto_scale = true,
                    "--legacy" if inline_val.is_none() => parsed.legacy = true,
                    "-o" | "--output" => {
                        parsed.output =
                            Some(resolve_required_path("--output", inline_val, &mut iter)?);
                    }
                    "-l" | "--logo" => {
                        parsed.logo =
                            Some(resolve_required_path("--logo", inline_val, &mut iter)?);
                    }
                    "-i" | "--init" => {
                        parsed.init = Some(resolve_init_path(inline_val, &mut iter));
                    }
                    _ => {
                        return Err(CliError::InvalidArgument(format!(
                            "Unrecognized option '{arg_str}'"
                        )));
                    }
                }
            } else {
                if parsed.input.is_some() {
                    return Err(CliError::UnexpectedPositional(format!(
                        "Unexpected positional argument '{arg_str}'"
                    )));
                }
                parsed.input = Some(PathBuf::from(arg_os));
            }
        } else {
            if parsed.input.is_some() {
                return Err(CliError::UnexpectedPositional(format!(
                    "Unexpected positional argument '{}'",
                    arg_os.to_string_lossy()
                )));
            }
            parsed.input = Some(PathBuf::from(arg_os));
        }
    }

    Ok(parsed)
}

/// Parses an optional template path value for `--init`.
fn parse_init_path(raw_val: &OsStr) -> PathBuf {
    if raw_val.is_empty() {
        PathBuf::from(DEFAULT_TEMPLATE_NAME)
    } else {
        PathBuf::from(raw_val)
    }
}

/// Resolves the starter template path from an inline or peeked argument.
fn resolve_init_path<I, T>(
    inline_val: Option<&OsStr>,
    iter: &mut std::iter::Peekable<I>,
) -> PathBuf
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    match inline_val {
        Some(val) => parse_init_path(val),
        None => {
            if let Some(next_arg) = iter.peek() {
                let is_flag = next_arg.as_ref().to_str().is_some_and(|s| s.starts_with('-'));
                if is_flag {
                    PathBuf::from(DEFAULT_TEMPLATE_NAME)
                } else {
                    let next_val = iter.next().expect("peeked value must be present");
                    parse_init_path(next_val.as_ref())
                }
            } else {
                PathBuf::from(DEFAULT_TEMPLATE_NAME)
            }
        }
    }
}

/// Resolves a required non-empty path value from an inline or subsequent argument.
fn resolve_required_path<I, T>(
    flag: &str,
    inline_val: Option<&OsStr>,
    iter: &mut I,
) -> Result<PathBuf, CliError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    let val_buf = match inline_val {
        Some(val) => PathBuf::from(val),
        None => {
            let next_arg = iter.next().ok_or_else(|| {
                CliError::MissingArgument(format!("Option '{flag}' requires a path value"))
            })?;
            PathBuf::from(next_arg.as_ref())
        }
    };

    if val_buf.as_os_str().is_empty() {
        Err(CliError::InvalidArgument(format!(
            "Option '{flag}' requires a non-empty path value"
        )))
    } else {
        Ok(val_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_help_message_content() {
        let msg = help_message();
        assert!(msg.contains("Doc2Flow (d2f)"));
        assert!(msg.contains("--output"));
        assert!(msg.contains("--logo"));
        assert!(msg.contains("--init"));
        assert!(msg.contains("--auto-scale"));
        assert!(msg.contains("--legacy"));
        assert!(msg.contains("--help"));
        assert!(msg.contains("--version"));
    }

    #[test]
    fn test_parse_args_defaults_and_positional() {
        let args = parse_args(["input.md"]).unwrap();
        assert_eq!(args.input, Some(PathBuf::from("input.md")));
        assert_eq!(args.output, None);
        assert_eq!(args.init, None);
        assert_eq!(args.logo, None);
        assert!(!args.auto_scale);
        assert!(!args.legacy);
        assert!(!args.show_help);
        assert!(!args.show_version);
    }

    #[test]
    fn test_parse_args_empty_args() {
        let args = parse_args([] as [&str; 0]).unwrap();
        assert_eq!(args, Args::default());
    }

    #[test]
    fn test_parse_args_errors() {
        assert_eq!(
            parse_args(["--unknown"]).unwrap_err(),
            CliError::InvalidArgument("Unrecognized option '--unknown'".into())
        );
        assert_eq!(
            parse_args(["-o"]).unwrap_err(),
            CliError::MissingArgument("Option '--output' requires a path value".into())
        );
        assert_eq!(
            parse_args(["-l"]).unwrap_err(),
            CliError::MissingArgument("Option '--logo' requires a path value".into())
        );
        assert_eq!(
            parse_args(["--output="]).unwrap_err(),
            CliError::InvalidArgument(
                "Option '--output' requires a non-empty path value".into()
            )
        );
        assert_eq!(
            parse_args(["-o="]).unwrap_err(),
            CliError::InvalidArgument(
                "Option '--output' requires a non-empty path value".into()
            )
        );
        assert_eq!(
            parse_args(["--output", ""]).unwrap_err(),
            CliError::InvalidArgument(
                "Option '--output' requires a non-empty path value".into()
            )
        );
        assert_eq!(
            parse_args(["-o", ""]).unwrap_err(),
            CliError::InvalidArgument(
                "Option '--output' requires a non-empty path value".into()
            )
        );
        assert_eq!(
            parse_args(["--logo="]).unwrap_err(),
            CliError::InvalidArgument(
                "Option '--logo' requires a non-empty path value".into()
            )
        );
        assert_eq!(
            parse_args(["-l="]).unwrap_err(),
            CliError::InvalidArgument(
                "Option '--logo' requires a non-empty path value".into()
            )
        );
        assert_eq!(
            parse_args(["--logo", ""]).unwrap_err(),
            CliError::InvalidArgument(
                "Option '--logo' requires a non-empty path value".into()
            )
        );
        assert_eq!(
            parse_args(["-l", ""]).unwrap_err(),
            CliError::InvalidArgument(
                "Option '--logo' requires a non-empty path value".into()
            )
        );
        assert_eq!(
            parse_args(["input1.md", "input2.md"]).unwrap_err(),
            CliError::UnexpectedPositional(
                "Unexpected positional argument 'input2.md'".into()
            )
        );
        assert_eq!(
            parse_args(["--unknown=value"]).unwrap_err(),
            CliError::InvalidArgument("Unrecognized option '--unknown=value'".into())
        );
        assert_eq!(
            parse_args(["--help=invalid"]).unwrap_err(),
            CliError::InvalidArgument("Unrecognized option '--help=invalid'".into())
        );
    }

    #[test]
    fn test_parse_args_help_and_version() {
        let args = parse_args(["-h"]).unwrap();
        assert!(args.show_help);

        let args_long_h = parse_args(["--help"]).unwrap();
        assert!(args_long_h.show_help);

        let args_v = parse_args(["--version"]).unwrap();
        assert!(args_v.show_version);

        let args_v_short = parse_args(["-V"]).unwrap();
        assert!(args_v_short.show_version);
    }

    #[test]
    fn test_parse_args_init_custom_and_defaults() {
        let args_short = parse_args(["-i"]).unwrap();
        assert_eq!(args_short.init, Some(PathBuf::from("template.md")));

        let args_long = parse_args(["--init"]).unwrap();
        assert_eq!(args_long.init, Some(PathBuf::from("template.md")));

        let args_short_eq = parse_args(["-i="]).unwrap();
        assert_eq!(args_short_eq.init, Some(PathBuf::from("template.md")));

        let args_long_eq = parse_args(["--init="]).unwrap();
        assert_eq!(args_long_eq.init, Some(PathBuf::from("template.md")));

        let args_short_empty = parse_args(["-i", ""]).unwrap();
        assert_eq!(args_short_empty.init, Some(PathBuf::from("template.md")));

        let args_long_empty = parse_args(["--init", ""]).unwrap();
        assert_eq!(args_long_empty.init, Some(PathBuf::from("template.md")));

        let args_space_val = parse_args(["-i", "starter.md"]).unwrap();
        assert_eq!(args_space_val.init, Some(PathBuf::from("starter.md")));

        let args_next_flag = parse_args(["-i", "-s"]).unwrap();
        assert_eq!(args_next_flag.init, Some(PathBuf::from("template.md")));
        assert!(args_next_flag.auto_scale);

        let args_hyphen_val_eq = parse_args(["--init=-custom_tpl.md"]).unwrap();
        assert_eq!(args_hyphen_val_eq.init, Some(PathBuf::from("-custom_tpl.md")));

        let args_short_hyphen_val_eq = parse_args(["-i=-custom_tpl.md"]).unwrap();
        assert_eq!(
            args_short_hyphen_val_eq.init,
            Some(PathBuf::from("-custom_tpl.md"))
        );

        let args_dot_slash_hyphen = parse_args(["-i", "./-template.md"]).unwrap();
        assert_eq!(
            args_dot_slash_hyphen.init,
            Some(PathBuf::from("./-template.md"))
        );
    }

    #[test]
    fn test_parse_args_legacy() {
        let args = parse_args(["input.md", "--legacy"]).unwrap();
        assert!(args.legacy);
    }

    #[test]
    fn test_parse_args_logo_options() {
        let args_l = parse_args(["input.md", "-l", "my_logo.png"]).unwrap();
        assert_eq!(args_l.logo, Some(PathBuf::from("my_logo.png")));

        let args_long = parse_args(["input.md", "--logo", "brand/logo.svg"]).unwrap();
        assert_eq!(args_long.logo, Some(PathBuf::from("brand/logo.svg")));

        let args_eq = parse_args(["input.md", "--logo=assets/logo.webp"]).unwrap();
        assert_eq!(args_eq.logo, Some(PathBuf::from("assets/logo.webp")));

        let args_short_eq = parse_args(["input.md", "-l=assets/logo.png"]).unwrap();
        assert_eq!(args_short_eq.logo, Some(PathBuf::from("assets/logo.png")));
    }

    #[cfg(unix)]
    #[test]
    fn test_parse_args_non_utf8_paths() {
        use std::os::unix::ffi::OsStrExt;

        let non_utf8_input_bytes = b"input_\xFF\xFE.md";
        let non_utf8_input_os = OsStr::from_bytes(non_utf8_input_bytes);
        let args = parse_args([non_utf8_input_os]).unwrap();
        assert_eq!(
            args.input.unwrap().as_os_str().as_bytes(),
            non_utf8_input_bytes
        );

        let non_utf8_out_bytes = b"out_\xFF\xFE.html";
        let non_utf8_out_os = OsStr::from_bytes(non_utf8_out_bytes);
        let args_out = parse_args([OsStr::new("-o"), non_utf8_out_os]).unwrap();
        assert_eq!(
            args_out.output.unwrap().as_os_str().as_bytes(),
            non_utf8_out_bytes
        );

        let non_utf8_second_bytes = b"second_\xFF\xFE.md";
        let non_utf8_second_os = OsStr::from_bytes(non_utf8_second_bytes);
        let err = parse_args([non_utf8_input_os, non_utf8_second_os]).unwrap_err();
        assert!(matches!(err, CliError::UnexpectedPositional(_)));
    }

    #[test]
    fn test_parse_args_options() {
        let args = parse_args([
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
        assert!(!args.legacy);
    }

    #[test]
    fn test_parse_args_os_string() {
        let os_args = vec![
            OsString::from("input.md"),
            OsString::from("-o"),
            OsString::from("output.html"),
        ];
        let args = parse_args(&os_args).unwrap();
        assert_eq!(args.input, Some(PathBuf::from("input.md")));
        assert_eq!(args.output, Some(PathBuf::from("output.html")));
    }

    #[test]
    fn test_parse_args_pure_no_binary_assumption() {
        let args = parse_args(["input.md", "-o", "output.html"]).unwrap();
        assert_eq!(args.input, Some(PathBuf::from("input.md")));
        assert_eq!(args.output, Some(PathBuf::from("output.html")));
    }

    #[test]
    fn test_parse_init_path_direct() {
        assert_eq!(parse_init_path(OsStr::new("")), PathBuf::from("template.md"));
        assert_eq!(
            parse_init_path(OsStr::new("custom.md")),
            PathBuf::from("custom.md")
        );
    }

    #[test]
    fn test_resolve_required_path_direct() {
        let mut empty_iter = std::iter::empty::<&str>();
        assert!(resolve_required_path("--output", Some(OsStr::new("")), &mut empty_iter).is_err());
        assert_eq!(
            resolve_required_path("--output", Some(OsStr::new("out.html")), &mut empty_iter)
                .unwrap(),
            PathBuf::from("out.html")
        );
    }
}

