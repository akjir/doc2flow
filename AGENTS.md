# AI Agent Instructions for Doc2Flow (d2f)

As an AI agent working on Doc2Flow, adhere strictly to the following guidelines:

## 1. Project Specification Reference
All project specifications are located in `SPECIFICATION.md`. Do not modify `SPECIFICATION.md` unless explicitly requested or approved by the user.

## 2. Tech Stack Recommendations
Use `clap` (with the `derive` feature) for CLI argument parsing, `pulldown-cmark` for robust Markdown to HTML conversion, and `base64` along with `mime_guess` for image embedding.

## 3. Incremental Development
Implement features iteratively. Start with the CLI scaffolding and simple Markdown conversion before adding local image embedding and checklist interactivity.

## 4. Code Quality
Ensure all Rust code passes `cargo clippy` without warnings and is formatted using `cargo fmt`. Prioritize idiomatic Rust and robust error handling.

## 5. Language Interaction
The user will write in German, but you MUST always answer and write in English. Never use German for anything, neither in the code, nor in any files or artifacts you create.

## 6. Cross-Platform Guidelines
* **Primary OS:** Development and testing must be done primarily on Linux.
* **Target OS:** The target OS for the release binary is Windows 64-bit (`x86_64-pc-windows-gnu` / `msvc`).
* **Path Handling:** Always use `std::path::Path` and `std::path::PathBuf` for file system paths to guarantee cross-platform compatibility. Do not hardcode `/` or `\` separators.
* **Testing:** Ensure all unit and integration tests pass via standard `cargo test` on Unix systems.

## 7. Git Commit Policy
Only commit changes to Git when explicitly requested by the user. Do not make automatic git commits.

