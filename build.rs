use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=locales");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let dest_path = Path::new(&out_dir).join("locales_gen.rs");

    let locales_dir = Path::new("locales");
    let mut match_arms = String::new();

    let entries = fs::read_dir(locales_dir).ok().into_iter().flatten();
    for entry in entries.flatten() {
        let path = entry.path();
        let stem = path.file_stem().and_then(|s| s.to_str());
        let ext = path.extension();
        let abs = path.canonicalize().ok();

        if let (true, Some("json"), Some(stem_str), Some(abs_path)) =
            (path.is_file(), ext.and_then(|e| e.to_str()), stem, abs)
        {
            let abs_str = abs_path.to_str().unwrap_or("").replace('\\', "/");
            let code_stem = stem_str.to_lowercase();
            match_arms.push_str(&format!(
                "        {:?} => Some(include_str!(\"{}\")),\n",
                code_stem, abs_str
            ));
        }
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
