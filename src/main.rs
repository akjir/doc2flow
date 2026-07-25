use anyhow::{Context, Result};
use clap::Parser;
use doc2flow::converter;
use doc2flow::i18n::Locale;
use doc2flow::template;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Path to the input Markdown file")]
    input: PathBuf,

    #[arg(short, long, help = "Path to the output HTML file (optional)")]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_path = args.input;
    let output_path = args.output.unwrap_or_else(|| {
        let mut p = input_path.clone();
        p.set_extension("html");
        p
    });

    let md_content = fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read input file: {}", input_path.display()))?;

    let (frontmatter, markdown_body) = converter::parse_frontmatter(&md_content);
    let locale = Locale::from_lang_code(&frontmatter.language);

    let html_content = converter::convert_markdown_to_html_with_locale(markdown_body, &locale)?;

    let d2f_id = doc2flow::d2f_id::generate_d2f_id(&frontmatter)?;
    let final_html = template::render(&frontmatter, &locale, &html_content, &d2f_id)?;

    fs::write(&output_path, final_html)
        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

    println!("Successfully generated {}", output_path.display());
    Ok(())
}
