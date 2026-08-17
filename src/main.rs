//! Doc2Flow CLI entry point.

use std::env;
use std::process::ExitCode;

use doc2flow::core::arguments::{help_message, parse_args};
use doc2flow::core::builder;
use doc2flow::core::document::DocumentElement;
use doc2flow::core::error::Result;
use doc2flow::core::io;
use doc2flow::core::markdown::parse_d2f_markdown;

fn run() -> Result<()> {
    let args = parse_args(env::args_os().skip(1))?;

    if args.show_help {
        println!("{}", help_message());
        return Ok(());
    }

    if args.show_version {
        println!("d2f {}", env!("D2F_FULL_VERSION"));
        return Ok(());
    }

    let Some(input_path) = args.input else {
        return Err(
            "Missing input file. Specify input path or use --init to generate a template.".into(),
        );
    };

    let output_path = args
        .output
        .unwrap_or_else(|| input_path.with_extension("html"));

    let md_content = io::read_file_to_string(&input_path)?;

    let mut document = parse_d2f_markdown(&md_content)?;
    if let Some(ref custom_logo) = args.logo {
        let logo_str = custom_logo.display().to_string();
        document.parameters.logo = logo_str.clone();
        if let Some(DocumentElement::Header { ref mut logo, .. }) = document.head.header {
            *logo = Some(logo_str);
        }
    }
    let rendered_html = builder::build(&document);

    let base_dir = input_path.parent();
    let file_name = input_path.to_str();
    let final_html = doc2flow::features::image::embed_images_as_base64_with_source(
        &rendered_html,
        Some(&md_content),
        file_name,
        base_dir,
        args.auto_scale,
    )?;

    io::write_file(&output_path, final_html)?;

    println!("Successfully generated {}", output_path.display());
    Ok(())
}

fn main() -> ExitCode {
    if let Err(err) = run() {
        eprintln!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
