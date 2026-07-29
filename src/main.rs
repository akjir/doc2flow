use doc2flow::converter;
use doc2flow::error::{Doc2FlowError, Result};
use doc2flow::i18n::Locale;
use doc2flow::io;
use doc2flow::template;
use doc2flow::utils::{help_message, parse_args};
use std::env;

fn main() -> Result<()> {
    let args = parse_args(env::args()).map_err(Doc2FlowError::Message)?;

    if args.show_help {
        println!("{}", help_message());
        return Ok(());
    }

    if args.show_version {
        println!("d2f {}", env!("D2F_FULL_VERSION"));
        return Ok(());
    }

    if let Some(init_path) = args.init {
        let template_content = template::generate_template_markdown();
        io::write_file(&init_path, template_content)?;
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

    let md_content = io::read_file_to_string(&input_path)?;

    let file_name = input_path.to_str();
    let (frontmatter, markdown_body) =
        converter::parse_and_validate_frontmatter(&md_content, file_name)?;
    let language_code = frontmatter.language.as_deref().unwrap_or("en");
    let locale = Locale::from_lang_code(language_code);

    let html_content = converter::convert_markdown_to_html_with_options(
        markdown_body,
        &locale,
        frontmatter.number_sections,
    )?;

    let base_dir = input_path.parent();

    let logo_path = args
        .logo
        .as_deref()
        .or_else(|| frontmatter.logo.as_deref().map(std::path::Path::new));
    let logo_html = doc2flow::image::load_logo(logo_path, base_dir);

    let d2f_id = doc2flow::id::generate_d2f_id(&frontmatter)?;
    let rendered_html =
        template::render(&frontmatter, &locale, &html_content, &d2f_id, Some(&logo_html))?;

    let final_html = doc2flow::image::embed_images_as_base64_with_source(
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
