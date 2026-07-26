# AI Agent Instructions for Doc2Flow (d2f)

As an AI agent working on Doc2Flow, adhere strictly to the following guidelines:

## 1. Project Specification Reference
All project specifications are located in `SPECIFICATION.md`. Do not modify `SPECIFICATION.md` unless explicitly requested or approved by the user.

## 2. Tech Stack Recommendations
Use `clap` (with the `derive` feature) for CLI argument parsing, `pulldown-cmark` for robust Markdown to HTML conversion, and `base64` along with `mime_guess` for image embedding.

## 3. Incremental Development
Implement features iteratively. Start with the CLI scaffolding and simple Markdown conversion before adding local image embedding and checklist interactivity.

## 4. Code Quality & Pragmatic Rust Guidelines
* **Linting & Formatting:** Ensure all Rust code passes `cargo clippy` without warnings and is formatted using `cargo fmt`. Prioritize idiomatic Rust and robust error handling.
* **Core Development Principles:**
  * **Idiomatic Rust (M-RUST-SHAPED):** Solve problems using Rust's native paradigms. Leverage the strong type system, ownership model, and idiomatic error handling. Avoid translating concepts from other languages 1-on-1.
  * **Strong Typing (C-NEWTYPE):** Avoid primitive obsession. Use strong types (e.g., newtype pattern) with strict, well-documented semantics to represent domain concepts clearly.
  * **Meaningful Tests (M-TAUTOLOGICAL-TESTS):** Unit tests must verify meaningful behavior and edge cases, not just assert foundational definitions or mirror the implementation's branches.
  * **Single-Item Path (M-SINGLE-ITEM-PATH):** Ensure any public item is reachable through exactly one path. Do not clutter the namespace with redundant re-exports.
* **Error Handling & Correctness:**
  * **Application-Level Errors (M-APP-ERROR):** As a standalone application, use application-level error handling crates like `anyhow` or `eyre` for simple and effective error propagation instead of implementing exhaustive custom error enums, unless specifically required.
  * **Panics mean "Stop" (M-PANIC-IS-STOP):** A panic indicates an immediate program termination. Do not attempt to catch and continue (`catch_unwind`).
  * **Bugs vs. Errors (M-PANIC-ON-BUG):**
    * Use `Result` for expected, recoverable errors (e.g., missing files, invalid user input, I/O errors).
    * Use panics (`panic!`, `expect`, `unreachable!`) for detecting contract violations and unrecoverable programming bugs.
  * **Helpful Panic Messages (M-PANIC-MESSAGE):** When intentionally panicking (e.g., via `expect` or `assert!`), provide clear, detailed messages including relevant runtime values to aid debugging.
  * **Zero Tolerance for Unsoundness (M-UNSOUND):** Unsound code is strictly forbidden. Avoid `unsafe` entirely. Since this project involves standard CLI and markdown processing tasks, there is no justifiable need for `unsafe`.
* **Documentation & Code Style:**
  * **Design for AI and Humans (M-DESIGN-FOR-AI):** Write predictable, idiomatic code. Clear signatures, strong types, and excellent documentation benefit everyone, including AI agents.
  * **No Meta-Design Docs (M-NO-META-DESIGN-DOCUMENTATION):** Document the *current behavior* and end-state of the code. Do not include process journals, "why we chose X over Y" essays, or tables of applied rules in the source files.
  * **Canonical Documentation Sections (M-CANONICAL-DOCS):**
    * Start item documentation with a concise summary sentence (ideally under 15 words) (M-FIRST-DOC-SENTENCE).
    * Follow up with extended free-form documentation.
    * Use canonical markdown headers (`# Examples`, `# Errors`, `# Panics`) when applicable.
    * Do not list parameter tables; explain parameters naturally in the plain text.
* **Performance & Memory Efficiency:**
  * **Allocation Minimization (P-MIN-ALLOC):** Avoid redundant heap allocations (`Vec`, `String`, `Box`). Prefer borrowing (`&str`, `&[T]`) over cloning or `.to_string()` when reading or inspecting data.
  * **Pre-allocation via Capacity (P-WITH-CAPACITY):** When building collections or strings with known or estimable bounds, initialize them using `with_capacity()` to eliminate dynamic reallocations during append operations.
  * **Single-Pass Scanning (P-SINGLE-PASS):** Design string transformation and parsing algorithms (e.g., Markdown AST conversion, template placeholders) to process inputs in a single streaming pass ($O(N)$) without recursive or chained string mutations.
  * **Zero-Copy Slicing (P-ZERO-COPY):** Prefer standard library slicing, `.strip_prefix()`, `.strip_suffix()`, and `.split_once()` over creating new `String` allocations or temporary `Vec<&str>` collections via `.split().collect()`.
  * **Smart Cow Usage (P-COW-STR):** Use `std::borrow::Cow<'a, str>` for struct fields or return values that usually reference static/borrowed strings but occasionally require owned modifications.
* **Idiomatic Pattern Matching & Flow Control:**
  * **Match over Chained If-Else (I-MATCH-TABLES):** Avoid multi-branch if-else cascades for string matching or prefixes. Use concise match expressions, lookup slices, or iterator chains instead.
  * **Functional Iteration (I-ITER-CHAIN):** Prefer declarative iterator chains (`.filter()`, `.map()`, `.count()`, `.find()`) over mutable loop state tracking (`mut count += 1`).
  * **No Unnecessary Formatting Overhead (I-NO-FMT-OVERHEAD):** Avoid creating temporary intermediate `String` instances using `format!()` inside tight loops or stream renders. Write directly into pre-allocated buffer streams (`write!`, `writeln!`) or format-specifiers.
* **Binary Size & Compile-Time Optimization:**
  * **Compile-Time Asset Embedding (B-EMBED-ASSETS):** Embed static UI templates, CSS, JS, and default locales at compile time using `include_str!` or `build.rs` code generation. Do not rely on runtime filesystem paths for embedded assets.
  * **Zero-Dependency Ecosystem (B-ZERO-DEPS):** Favor idiomatic standard library features (`std::collections::HashMap`, `std::path::PathBuf`, slice parsing) over third-party crates unless strictly necessary (e.g., `pulldown-cmark`, `clap`, `serde_json`).
  * **Release Artifact Shrinking (B-STRIP-BINARY):** Release builds must target minimal executable size (enabling `lto = true`, `opt-level = "z"` / `"s"`, `codegen-units = 1`, and binary symbol stripping).

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
