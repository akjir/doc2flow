//! Doc2Flow CLI entry point.

use doc2flow::core::error::{Error, Result};
use doc2flow::core::parsing::arguments::{help_message, parse_args};
use std::env;
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = parse_args(env::args()).map_err(Error::Message)?;

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
