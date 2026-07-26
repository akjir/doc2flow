use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=locales");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let dest_path = Path::new(&out_dir).join("locales_gen.rs");

    let locales_dir = Path::new("locales");
    let mut locale_entries = Vec::new();

    if let Ok(entries) = fs::read_dir(locales_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            println!("cargo:rerun-if-changed={}", path.display());

            let stem = path.file_stem().and_then(|s| s.to_str());
            let ext = path.extension().and_then(|e| e.to_str());
            let abs = path.canonicalize().ok();

            if let (true, Some("json"), Some(stem_str), Some(abs_path)) =
                (path.is_file(), ext, stem, abs)
            {
                let mut abs_str = abs_path.to_str().unwrap_or("").replace('\\', "/");
                if let Some(stripped) = abs_str.strip_prefix("//?/") {
                    abs_str = stripped.to_string();
                } else if let Some(stripped) = abs_str.strip_prefix("\\\\?\\") {
                    abs_str = stripped.to_string();
                }
                locale_entries.push((stem_str.to_lowercase(), abs_str));
            }
        }
    }

    locale_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut match_arms = String::with_capacity(locale_entries.len() * 128);
    for (code_stem, abs_path_str) in &locale_entries {
        let _ = writeln!(
            match_arms,
            "        {:?} => Some(include_str!(\"{}\")),",
            code_stem, abs_path_str
        );
    }

    let generated_code = format!(
        "/// Returns embedded locale JSON content matching the given lowercased language code.\n\
         pub fn get_embedded_locale(code: &str) -> Option<&'static str> {{\n\
         \x20   match code {{\n\
         {}\
         \x20       _ => None,\n\
         \x20   }}\n\
         }}\n",
        match_arms
    );

    fs::write(dest_path, generated_code).expect("Failed to write generated locales file");
}
