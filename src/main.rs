use anyhow::{Context, Result};
use clap::Parser;
use doc2flow::converter;
use doc2flow::i18n::Locale;
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

    let base_html = include_str!("../templates/base.html");
    let style_css = include_str!("../templates/style.css");
    let script_js = include_str!("../templates/script.js");

    let doc_id = "doc_poc_12345";
    let i18n_json = serde_json::to_string(&locale)?;

    let final_html = base_html
        .replace("{{LANG_CODE}}", &locale.lang_code)
        .replace("{{TITLE}}", &frontmatter.title)
        .replace("{{SUBTITLE}}", &frontmatter.subtitle)
        .replace("{{CUSTOMER}}", &frontmatter.customer)
        .replace("{{EMPLOYEE}}", &frontmatter.employee)
        .replace("{{TECHNICIAN}}", &frontmatter.technician)
        .replace("{{DATE}}", &frontmatter.date)
        .replace("{{L_CUSTOMER}}", &locale.customer)
        .replace("{{L_EMPLOYEE}}", &locale.employee)
        .replace("{{L_TECHNICIAN}}", &locale.technician)
        .replace("{{L_DATE}}", &locale.date)
        .replace("{{L_SETUP_COMPLETED}}", &locale.setup_completed)
        .replace("{{L_NAME_PLACEHOLDER}}", &locale.name_placeholder)
        .replace("{{L_SIGNATURE_TECHNICIAN}}", &locale.signature_technician)
        .replace("{{L_DATE_PLACEHOLDER}}", &locale.date_placeholder)
        .replace("{{L_SIGNATURE_DATE}}", &locale.signature_date)
        .replace("{{L_EXPORT_PDF}}", &locale.export_pdf)
        .replace("{{L_RESET_ALL}}", &locale.reset_all)
        .replace("{{L_LOADING}}", &locale.loading)
        .replace("{{I18N_JSON}}", &i18n_json)
        .replace("{{CSS}}", style_css)
        .replace("{{JS}}", script_js)
        .replace("{{CONTENT}}", &html_content)
        .replace("{{DOC_ID}}", doc_id);

    fs::write(&output_path, final_html)
        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

    println!("Successfully generated {}", output_path.display());
    Ok(())
}
