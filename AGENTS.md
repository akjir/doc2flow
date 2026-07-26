# AI Agent Instructions for Doc2Flow (d2f)

Adhere strictly to these guidelines:

## 1. Project Specification Reference
* Specifications reside in `SPECIFICATION.md`.

## 2. Tech Stack Recommendations
* CLI parsing: `clap` (with `derive`).
* Markdown to HTML: `pulldown-cmark`.
* Asset processing: `base64`, `mime_guess`.

## 3. Incremental Development
* Implement iteratively: CLI scaffolding $\rightarrow$ simple Markdown conversion $\rightarrow$ local image embedding $\rightarrow$ interactive checklists.

## 4. Pragmatic Rust Guidelines

### Core Principles
* **Idiomatic Rust (M-RUST-SHAPED):** Use native paradigms, ownership, and strong typing. Do not translate from other languages 1-to-1.
* **Strong Typing (C-NEWTYPE):** Avoid primitive obsession; leverage the newtype pattern with documented semantics.
* **Meaningful Tests (M-TAUTOLOGICAL-TESTS):** Verify meaningful behavior and edge cases, not foundational definitions or implementation mirrors.
* **Single-Item Path (M-SINGLE-ITEM-PATH):** Ensure public items have exactly one reachability path. Avoid redundant re-exports.

### Error Handling & Correctness
* **Application-Level Errors (M-APP-ERROR):** Use `anyhow` or `eyre` for error propagation. Avoid exhaustive custom error enums unless strictly required.
* **Panics mean "Stop" (M-PANIC-IS-STOP):** Panics indicate immediate termination. Never catch panics (`catch_unwind`).
* **Bugs vs. Errors (M-PANIC-ON-BUG):**
  * Use `Result` for expected, recoverable errors (I/O, invalid input, missing files).
  * Use panics (`panic!`, `expect`, `unreachable!`) solely for contract violations and unrecoverable bugs.
* **Panic Messages (M-PANIC-MESSAGE):** Provide detailed runtime values in `expect` or `assert!` messages.
* **Zero Tolerance for Unsoundness (M-UNSOUND):** `unsafe` is strictly forbidden.

### Documentation & Code Style
* **Design for AI and Humans (M-DESIGN-FOR-AI):** Write predictable, idiomatic code with explicit signatures and types.
* **No Meta-Design Docs (M-NO-META-DESIGN-DOCUMENTATION):** Document end-state behavior only. Omit journals, design rationales, or rule tables in source files.
* **Canonical Documentation Sections (M-CANONICAL-DOCS):**
  * Start with a summary sentence under 15 words (M-FIRST-DOC-SENTENCE).
  * Follow with free-form extended docs.
  * Use canonical headers (`# Examples`, `# Errors`, `# Panics`).
  * Explain parameters naturally within text (no parameter tables).

### Performance & Memory Efficiency
* **Allocation Minimization (P-MIN-ALLOC):** Avoid redundant heap allocations (`Vec`, `String`, `Box`). Prefer borrowing (`&str`, `&[T]`) over cloning or `.to_string()`.
* **Pre-allocation via Capacity (P-WITH-CAPACITY):** Initialize collections/strings using `with_capacity()` when bounds are estimable.
* **Single-Pass Scanning (P-SINGLE-PASS):** Design string transformation and AST algorithms to process inputs in a single $O(N)$ streaming pass.
* **Zero-Copy Slicing (P-ZERO-COPY):** Use `.strip_prefix()`, `.strip_suffix()`, `.split_once()`, and std slices instead of `.split().collect()`.
* **Smart Cow Usage (P-COW-STR):** Use `std::borrow::Cow<'a, str>` for struct fields/returns that conditionally require owned modifications.

### Pattern Matching & Flow Control
* **Match over Chained If-Else (I-MATCH-TABLES):** Use `match` expressions, lookup slices, or iterator chains instead of multi-branch `if-else` cascades.
* **Functional Iteration (I-ITER-CHAIN):** Prefer declarative iterators (`.filter()`, `.map()`, `.count()`, `.find()`) over mutable loop state tracking.
* **No Unnecessary Formatting Overhead (I-NO-FMT-OVERHEAD):** Avoid temporary `format!()` strings in loops/stream renders. Write directly to streams via `write!` or `writeln!`.

### Binary Size & Compile-Time Optimization
* **Compile-Time Asset Embedding (B-EMBED-ASSETS):** Embed static templates, CSS, JS, and locales via `include_str!` or `build.rs`. Never rely on runtime paths.
* **Zero-Dependency Ecosystem (B-ZERO-DEPS):** Prefer standard library solutions (`HashMap`, `PathBuf`, slice parsing) over third-party crates unless strictly required.
* **Release Artifact Shrinking (B-STRIP-BINARY):** Release profiles must set `lto = true`, `opt-level = "z"` / `"s"`, `codegen-units = 1`, and enable binary stripping.

## 5. Language Interaction
* Respond and write **EXCLUSIVELY in English**.
* Never use German in code, comments, files, or artifacts, even if queried in German.

## 6. Cross-Platform Guidelines
* Primary OS: Linux (development and testing).
* Target OS: Windows 64-bit (`x86_64-pc-windows-gnu` / `msvc`).
* Path Handling: Always use `std::path::Path` and `std::path::PathBuf`. Never hardcode `/` or `\` separators.
* Testing: Guarantee all tests pass via `cargo test` on Unix systems.

## 7. Git Commit Policy
* Commit ONLY when explicitly instructed by the user.
* Execute commits ONLY if `cargo test` passes 100%, or if explicit user confirmation is given for failing tests.

## 8. Test-Driven & Quality Policy
* Priority: Testing is the highest priority.
* Proactively write negative and edge-case tests.
* Execute full test suite (`cargo test`) before completing any task.
* Regenerate showcase HTML files (`tests/showcase_de.html`, `tests/showcase_en.html`) via `cargo run` whenever renderer, template, CSS, JS, or conversion logic changes.

## 9. HTML Template & UI Guidelines
* Generic Templates: `templates/base.html`, `templates/style.css`, and `templates/script.js` must remain completely generic and devoid of customer-specific text.
* Markdown Mapping:
  * Level 2 headings (`##`) $\rightarrow$ collapsible `.section` with `.sh`/`.sb` classes.
  * Checkbox unordered lists $\rightarrow$ wrapped in `.check-item`.
  * Blockquote callouts $\rightarrow$ map prefixes (`>`, `>?`, `>!`, `>!!`, `>!!!`) to alert panels (`.note`, `.note-tip`, `.note-important`, `.note-warning`, `.note-caution`).
  * Image & Link Handling: Local image files are converted to Base64 `data:` URIs. Remote image URLs (`http://`, `https://`) are preserved as `<img>` tags. Non-image resources (e.g., `.pdf`, `.zip`) specified in image tags are converted to external link elements (`<a>`).
* Placeholders: Replace generic placeholders (e.g., `{{TITLE}}`, `{{CUSTOMER}}`) in `base.html` using frontmatter/metadata.

## 10. Client-Side JavaScript Guidelines
* **Zero External Frameworks (JS-NO-FRAMEWORKS):** Pure Vanilla JS (ES6+) only. No jQuery, runtime bundles, or external libraries.
* **Event Delegation & Lifetime (JS-EVENT-LIFECYCLE):**
  * Prefer event delegation over global event loops.
  * Clean up temporary window/document event listeners (e.g., modal/lightbox `keydown` handlers) upon state changes.
* **DOM Query Caching (JS-DOM-EFFICIENT):** Avoid repetitive global queries (`document.querySelectorAll`) in tight loops or update functions. Cache local references.
* **Non-Blocking Execution (JS-PERF-ASYNC):** Keep main-thread execution short. Eliminate DOM read/write interleaving (layout thrashing).
* **State Persistence Sync (JS-STATE-DOM-SYNC):** Synchronize in-memory JS state with physical DOM attributes (`checked`, `value`, `textContent`, `selected`) prior to document export/download for 100% offline fidelity.
* **Safe Clipboard API (JS-CLIPBOARD-SAFE):** Use `navigator.clipboard.writeText()` with async `.catch()`. Fall back gracefully in non-secure contexts (`http:` / restricted IFrames).