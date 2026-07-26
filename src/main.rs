use clap::Parser;
use doc2flow::converter;
use doc2flow::error::{Doc2FlowError, Result};
use doc2flow::i18n::Locale;
use doc2flow::template;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Path to the input Markdown file")]
    input: Option<PathBuf>,

    #[arg(short, long, help = "Path to the output HTML file (optional)")]
    output: Option<PathBuf>,

    #[arg(
        short = 'i',
        long = "init",
        num_args = 0..=1,
        default_missing_value = "template.md",
        help = "Generate a starter template Markdown file (optional target path, default: template.md)"
    )]
    init: Option<PathBuf>,
    #[arg(
        short = 's',
        long = "auto-scale",
        help = "Automatically resize local images exceeding 250 KB to WebP"
    )]
    auto_scale: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(init_path) = args.init {
        let template_content = template::generate_template_markdown();
        fs::write(&init_path, template_content).map_err(|e| Doc2FlowError::Io {
            path: Some(init_path.clone()),
            source: e,
        })?;
        println!("Successfully generated template {}", init_path.display());
        return Ok(());
    }

    let input_path = match args.input {
        Some(path) => path,
        None => {
            return Err(Doc2FlowError::Message(
                "Missing input file. Specify input path or use --init to generate a template."
                    .to_string(),
            ));
        }
    };

    let output_path = args.output.unwrap_or_else(|| {
        let mut p = input_path.clone();
        p.set_extension("html");
        p
    });

    let md_content = fs::read_to_string(&input_path).map_err(|e| Doc2FlowError::Io {
        path: Some(input_path.clone()),
        source: e,
    })?;

    let file_name = input_path.to_str();
    let (frontmatter, markdown_body) =
        converter::parse_and_validate_frontmatter(&md_content, file_name)?;
    let locale = Locale::from_lang_code(&frontmatter.language);

    let html_content = converter::convert_markdown_to_html_with_locale(markdown_body, &locale)?;

    let d2f_id = doc2flow::id::generate_d2f_id(&frontmatter)?;
    let rendered_html = template::render(&frontmatter, &locale, &html_content, &d2f_id)?;

    let base_dir = input_path.parent();
    let final_html = doc2flow::image::embed_images_as_base64_with_source(
        &rendered_html,
        Some(&md_content),
        file_name,
        base_dir,
        args.auto_scale,
    )?;

    fs::write(&output_path, final_html).map_err(|e| Doc2FlowError::Io {
        path: Some(output_path.clone()),
        source: e,
    })?;

    println!("Successfully generated {}", output_path.display());
    Ok(())
}
