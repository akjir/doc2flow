//! Doc2Flow CLI entry point.

use doc2flow::core::build::builder;
use doc2flow::core::error::{Error, Result};
use doc2flow::core::parse::arguments::{help_message, parse_args};
use doc2flow::core::parse::parser;
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

    let document = parser::parse(&md_content)?;
    let content = builder::build(&document);

    io::write_file(&output_path, content)?;

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
