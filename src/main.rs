use anyhow::{Context, Result};
use clap::Parser;
use doc2flow::converter;
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

    let html_content = converter::convert_markdown_to_html(markdown_body)?;

    let base_html = include_str!("../templates/base.html");
    let style_css = include_str!("../templates/style.css");
    let script_js = include_str!("../templates/script.js");

    let doc_id = "doc_poc_12345";

    let final_html = base_html
        .replace("{{TITLE}}", &frontmatter.title)
        .replace("{{SUBTITLE}}", &frontmatter.subtitle)
        .replace("{{CUSTOMER}}", &frontmatter.customer)
        .replace("{{EMPLOYEE}}", &frontmatter.employee)
        .replace("{{TECHNICIAN}}", &frontmatter.technician)
        .replace("{{DATE}}", &frontmatter.date)
        .replace("{{CSS}}", style_css)
        .replace("{{JS}}", script_js)
        .replace("{{CONTENT}}", &html_content)
        .replace("{{DOC_ID}}", doc_id);

    fs::write(&output_path, final_html)
        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

    println!("Successfully generated {}", output_path.display());
    Ok(())
}
