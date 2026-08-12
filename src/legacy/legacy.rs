//! Legacy CLI execution entry point for Doc2Flow.

use crate::legacy::core::args::{help_message, parse_args};
use crate::legacy::core::builder;
use crate::legacy::core::converter;
use crate::legacy::core::locales::Locale;
use crate::legacy::utils::error::{Doc2FlowError, Result};
use crate::legacy::utils::io;
use std::env;

/// Runs the legacy conversion pipeline using CLI arguments.
///
/// # Errors
///
/// Returns [`Doc2FlowError`] if argument parsing, file IO, or rendering fails.
pub fn run() -> Result<()> {
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
        let template_content = builder::generate_template_markdown();
        io::write_file(&init_path, template_content)?;
        println!("Successfully generated template {}", init_path.display());
        return Ok(());
    }

    let Some(input_path) = args.input else {
        return Err(Doc2FlowError::Message(
            "Missing input file. Specify input path or use --init to generate a template."
                .to_string(),
        ));
    };

    let output_path = args
        .output
        .unwrap_or_else(|| input_path.with_extension("html"));

    let md_content = io::read_file_to_string(&input_path)?;

    let file_name = input_path.to_str();
    let (frontmatter, markdown_body) =
        converter::parse_and_validate_frontmatter(&md_content, file_name)?;
    let language_code = frontmatter.language.as_deref().unwrap_or("en");
    let locale = Locale::from_lang_code(language_code);

    let frontmatter_map = frontmatter.to_hashmap();
    let ctx = crate::legacy::core::feature::DocumentContext::new(&frontmatter_map, markdown_body);
    let all_features = crate::legacy::features::get_all_features();
    let features = crate::legacy::core::converter::DocumentFeatures::resolve(&all_features, &ctx);

    let (html_content, _) = converter::convert_markdown_to_html_with_options(
        markdown_body,
        &locale,
        frontmatter.numbered_sections,
    )?;

    let base_dir = input_path.parent();

    let logo_path = args
        .logo
        .as_deref()
        .or_else(|| frontmatter.logo.as_deref().map(std::path::Path::new));
    let logo_html = crate::legacy::core::image::load_logo(logo_path, base_dir);

    let d2f_id = crate::legacy::core::id::generate_d2f_id(&frontmatter)?;
    let rendered_html = builder::render(
        &frontmatter,
        &locale,
        &html_content,
        &d2f_id,
        Some(&logo_html),
        &features,
    )?;

    let final_html = crate::legacy::core::image::embed_images_as_base64_with_source(
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
