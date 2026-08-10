//! Doc2Flow CLI entry point.

use doc2flow::core::dev_helper::document_to_json;
use doc2flow::core::error::{Error, Result};
use doc2flow::core::parsing::arguments::{help_message, parse_args};
use doc2flow::core::parsing::parse_d2f_markdown;
use doc2flow::core::utils::io;
use std::env;
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = parse_args(env::args())?;

    if args.show_help {
        println!("{}", help_message());
        return Ok(());
    }

    if args.show_version {
        println!("d2f {}", env!("D2F_FULL_VERSION"));
        return Ok(());
    }

    if args.legacy {
        doc2flow::legacy::run().map_err(|err| Error::Message(err.to_string()))?;
        return Ok(());
    }

    let Some(input_path) = args.input else {
        return Err(
            "Missing input file. Specify input path or use --init to generate a template.".into(),
        );
    };

    let output_path = args.output.unwrap_or_else(|| input_path.with_extension("html"));

    let md_content = io::read_file_to_string(&input_path)?;

    let document =
        parse_d2f_markdown(&md_content).map_err(|e| Error::Message(e.to_string()))?;
    let json = document_to_json(&document);

    io::write_file(&output_path, json)?;

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
