use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=locales");

    generate_version_metadata();

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

fn get_git_commit_count() -> String {
    let output = std::process::Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let count = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !count.is_empty() && count.chars().all(|c| c.is_ascii_digit()) {
                count
            } else {
                "0".to_string()
            }
        }
        _ => "0".to_string(),
    }
}

fn get_git_commit_hash() -> String {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !hash.is_empty() {
                hash
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

fn is_git_dirty() -> bool {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output();
    match output {
        Ok(out) if out.status.success() => !String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        _ => false,
    }
}

fn sync_package_json_version(pkg_version: &str) {
    let package_json_path = Path::new("web/package.json");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if let Ok(content) = fs::read_to_string(package_json_path) {
        let mut lines = Vec::new();
        let mut modified = false;
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("\"version\":") {
                let indent = &line[..line.len() - trimmed.len()];
                let has_comma = line.trim_end().ends_with(',');
                let comma_str = if has_comma { "," } else { "" };
                let new_line = format!("{indent}\"version\": \"{pkg_version}\"{comma_str}");
                if line != new_line {
                    lines.push(new_line);
                    modified = true;
                    continue;
                }
            }
            lines.push(line.to_string());
        }

        if modified {
            let mut new_content = lines.join("\n");
            if content.ends_with('\n') {
                new_content.push('\n');
            }
            let _ = fs::write(package_json_path, new_content);
        }
    }
}

fn generate_version_metadata() {
    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    sync_package_json_version(&pkg_version);

    let commit_count = get_git_commit_count();
    let commit_hash = get_git_commit_hash();
    let suffix = if is_git_dirty() { ".dev" } else { "" };

    let full_version = format!("v{}+{}.{}{}", pkg_version, commit_count, commit_hash, suffix);
    println!("cargo:rustc-env=D2F_FULL_VERSION={}", full_version);

    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/refs");
}

