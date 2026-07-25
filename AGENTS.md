# AI Agent Instructions for Doc2Flow (d2f)

As an AI agent working on Doc2Flow, adhere strictly to the following guidelines:

## 1. Project Specification Reference
All project specifications are located in `SPECIFICATION.md`. Do not modify `SPECIFICATION.md` unless explicitly requested or approved by the user.

## 2. Tech Stack Recommendations
Use `clap` (with the `derive` feature) for CLI argument parsing, `pulldown-cmark` for robust Markdown to HTML conversion, and `base64` along with `mime_guess` for image embedding.

## 3. Incremental Development
Implement features iteratively. Start with the CLI scaffolding and simple Markdown conversion before adding local image embedding and checklist interactivity.

## 4. Code Quality
* **Linting & Formatting:** Ensure all Rust code passes `cargo clippy` without warnings and is formatted using `cargo fmt`. Prioritize idiomatic Rust and robust error handling.
* **Rust Guidelines (`RUST.md`):** Whenever writing or modifying Rust code, `RUST.md` MUST be read and strictly adhered to. For tasks that do not involve writing or editing Rust code (e.g., HTML/CSS/JS template edits, documentation, configuration), reading `RUST.md` is not required.

## 5. Language Interaction
The user will write in German, but you MUST always answer and write in English. Never use German for anything, neither in the code, nor in any files or artifacts you create.

## 6. Cross-Platform Guidelines
* **Primary OS:** Development and testing must be done primarily on Linux.
* **Target OS:** The target OS for the release binary is Windows 64-bit (`x86_64-pc-windows-gnu` / `msvc`).
* **Path Handling:** Always use `std::path::Path` and `std::path::PathBuf` for file system paths to guarantee cross-platform compatibility. Do not hardcode `/` or `\` separators.
* **Testing:** Ensure all unit and integration tests pass via standard `cargo test` on Unix systems.

## 7. Git Commit Policy
Only commit changes to Git when explicitly requested by the user. Do not make automatic git commits. Even when requested, commits may ONLY be executed if all tests pass, or if the user explicitly confirms the commit after being informed of a failing test.

## 8. Test-Driven & Quality First Policy
* **Highest Priority:** Testing is the highest priority. Always write comprehensive unit and integration tests wherever possible and appropriate.
* **Negative & Edge-Case Testing:** Proactively brainstorm potential failure modes, invalid inputs, and edge cases for functions, and write tests to verify proper handling.
* **Mandatory Execution:** Always run the full test suite (`cargo test`) to verify changes before concluding a task.
* **Showcase Regeneration:** Whenever significant changes are made to the renderer, templates, CSS, JS, or converter logic, always regenerate the showcase HTML files (`tests/showcase_de.html` and `tests/showcase_en.html`) using `cargo run`.

## 9. HTML Template & UI Guidelines
* **Generic Templates:** `templates/base.html`, `templates/style.css`, and `templates/script.js` form the foundation of the output. They must remain completely generic and devoid of customer-specific text or hardcoded logic.
* **Markdown Mapping:** When parsing the markdown into HTML using `pulldown-cmark`, ensure Level 2 headings (`##`) translate into collapsible sections with `.section` and `.sh`/`.sb` classes. Markdown unordered lists with checkboxes must be wrapped in `.check-item` classes. Callout annotations must map blockquote prefixes (`>`/`> Note`, `>?`/`>? Tip`, `>!`/`>! Important`, `>!!`/`>!! Warning`, `>!!!`/`>!!! Caution`) to visual alert panels (`.note`, `.note-tip`, `.note-important`, `.note-warning`, `.note-caution`).
* **Placeholders:** Replace predefined generic placeholders (e.g., `{{TITLE}}`, `{{CUSTOMER}}`) in the `base.html` using the frontmatter or metadata parsed from the Markdown file.
