//! Command line argument parsing and validation for Doc2Flow.
//!
//! Provides zero-dependency command line argument parsing, structured options,
//! and help message generation for the `d2f` executable.

use std::path::PathBuf;

const DEFAULT_TEMPLATE_NAME: &str = "template.md";

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
/// # Examples
///
/// ```
/// use doc2flow::args::parse_args;
///
/// let args = parse_args(&["d2f", "doc.md", "-s"]).unwrap();
/// assert_eq!(args.input.unwrap().to_str().unwrap(), "doc.md");
/// assert!(args.auto_scale);
/// ```
///
/// # Errors
///
/// Returns an error string if an invalid option or missing parameter value is provided.
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
                    .ok_or_else(|| String::from("Option '--output' requires a path value"))?;
                if val.as_ref().is_empty() {
                    return Err(String::from("Option '--output' requires a non-empty path value"));
                }
                parsed.output = Some(PathBuf::from(val.as_ref()));
            }
            "-l" | "--logo" => {
                let val = iter
                    .next()
                    .ok_or_else(|| String::from("Option '--logo' requires a path value"))?;
                if val.as_ref().is_empty() {
                    return Err(String::from("Option '--logo' requires a non-empty path value"));
                }
                parsed.logo = Some(PathBuf::from(val.as_ref()));
            }
            "-i" | "--init" => {
                if let Some(next_arg) = iter.peek() {
                    let next_str = next_arg.as_ref();
                    if next_str.starts_with('-') {
                        parsed.init = Some(PathBuf::from(DEFAULT_TEMPLATE_NAME));
                    } else {
                        let val = iter.next().expect("peeked value must be present");
                        if val.as_ref().is_empty() {
                            parsed.init = Some(PathBuf::from(DEFAULT_TEMPLATE_NAME));
                        } else {
                            parsed.init = Some(PathBuf::from(val.as_ref()));
                        }
                    }
                } else {
                    parsed.init = Some(PathBuf::from(DEFAULT_TEMPLATE_NAME));
                }
            }
            opt if opt.starts_with('-') => {
                if let Some((key, val)) = opt.split_once('=') {
                    match key {
                        "-o" | "--output" => {
                            if val.is_empty() {
                                return Err(String::from(
                                    "Option '--output' requires a non-empty path value",
                                ));
                            }
                            parsed.output = Some(PathBuf::from(val));
                        }
                        "-l" | "--logo" => {
                            if val.is_empty() {
                                return Err(String::from(
                                    "Option '--logo' requires a non-empty path value",
                                ));
                            }
                            parsed.logo = Some(PathBuf::from(val));
                        }
                        "-i" | "--init" => {
                            if val.is_empty() {
                                parsed.init = Some(PathBuf::from(DEFAULT_TEMPLATE_NAME));
                            } else {
                                parsed.init = Some(PathBuf::from(val));
                            }
                        }
                        _ => return Err(format!("Unrecognized option '{arg_str}'")),
                    }
                } else {
                    return Err(format!("Unrecognized option '{arg_str}'"));
                }
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
///
/// # Examples
///
/// ```
/// use doc2flow::args::help_message;
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
        "    -h, --help                  Print help information\n",
        "    -V, --version               Print version information\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let args3 = parse_args(&["d2f", "-i="]).unwrap();
        assert_eq!(args3.init, Some(PathBuf::from("template.md")));

        let args4 = parse_args(&["d2f", "--init="]).unwrap();
        assert_eq!(args4.init, Some(PathBuf::from("template.md")));

        let args5 = parse_args(&["d2f", "-i", ""]).unwrap();
        assert_eq!(args5.init, Some(PathBuf::from("template.md")));

        let args6 = parse_args(&["d2f", "--init", ""]).unwrap();
        assert_eq!(args6.init, Some(PathBuf::from("template.md")));
    }

    #[test]
    fn test_parse_args_help_and_version() {
        let args = parse_args(&["d2f", "-h"]).unwrap();
        assert!(args.show_help);

        let args_long_h = parse_args(&["d2f", "--help"]).unwrap();
        assert!(args_long_h.show_help);

        let args_v = parse_args(&["d2f", "--version"]).unwrap();
        assert!(args_v.show_version);

        let args_v_short = parse_args(&["d2f", "-V"]).unwrap();
        assert!(args_v_short.show_version);
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
        assert!(parse_args(&["d2f", "--output="]).is_err());
        assert!(parse_args(&["d2f", "-o="]).is_err());
        assert!(parse_args(&["d2f", "--output", ""]).is_err());
        assert!(parse_args(&["d2f", "-o", ""]).is_err());
        assert!(parse_args(&["d2f", "--logo="]).is_err());
        assert!(parse_args(&["d2f", "-l="]).is_err());
        assert!(parse_args(&["d2f", "--logo", ""]).is_err());
        assert!(parse_args(&["d2f", "-l", ""]).is_err());
        assert!(parse_args(&["d2f", "input1.md", "input2.md"]).is_err());
        assert!(parse_args(&["d2f", "--unknown=value"]).is_err());
    }

    #[test]
    fn test_help_message_content() {
        let msg = help_message();
        assert!(msg.contains("Doc2Flow (d2f)"));
        assert!(msg.contains("--output"));
        assert!(msg.contains("--logo"));
        assert!(msg.contains("--init"));
        assert!(msg.contains("--auto-scale"));
        assert!(msg.contains("--help"));
        assert!(msg.contains("--version"));
    }
}
