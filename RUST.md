# Pragmatic Rust Guidelines for Doc2Flow (d2f)

*Source: Derived and distilled from the [Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/agents/all.txt).*

This document outlines the core Rust guidelines for the Doc2Flow project. Given the nature of this project—a standalone CLI application with no APIs, no microservices, no databases, and developed by a single AI agent—these guidelines focus purely on best-practice Rust code handling, correctness, and maintainability.

## 1. Core Development Principles

*   **Idiomatic Rust (M-RUST-SHAPED):** Solve problems using Rust's native paradigms. Leverage the strong type system, ownership model, and idiomatic error handling. Avoid translating concepts from other languages 1-on-1.
*   **Strong Typing (C-NEWTYPE):** Avoid primitive obsession. Use strong types (e.g., newtype pattern) with strict, well-documented semantics to represent domain concepts clearly.
*   **Meaningful Tests (M-TAUTOLOGICAL-TESTS):** Unit tests must verify meaningful behavior and edge cases, not just assert foundational definitions or mirror the implementation's branches. 
*   **Single-Item Path (M-SINGLE-ITEM-PATH):** Ensure any public item is reachable through exactly one path. Do not clutter the namespace with redundant re-exports.

## 2. Error Handling & Correctness

*   **Application-Level Errors (M-APP-ERROR):** As a standalone application, use application-level error handling crates like `anyhow` or `eyre` for simple and effective error propagation instead of implementing exhaustive custom error enums, unless specifically required.
*   **Panics mean "Stop" (M-PANIC-IS-STOP):** A panic indicates an immediate program termination. Do not attempt to catch and continue (`catch_unwind`).
*   **Bugs vs. Errors (M-PANIC-ON-BUG):** 
    *   Use `Result` for expected, recoverable errors (e.g., missing files, invalid user input, I/O errors).
    *   Use panics (`panic!`, `expect`, `unreachable!`) for detecting contract violations and unrecoverable programming bugs.
*   **Helpful Panic Messages (M-PANIC-MESSAGE):** When intentionally panicking (e.g., via `expect` or `assert!`), provide clear, detailed messages including relevant runtime values to aid debugging.
*   **Zero Tolerance for Unsoundness (M-UNSOUND):** Unsound code is strictly forbidden. Avoid `unsafe` entirely. Since this project involves standard CLI and markdown processing tasks, there is no justifiable need for `unsafe`.

## 3. Documentation & Code Style

*   **Design for AI and Humans (M-DESIGN-FOR-AI):** Write predictable, idiomatic code. Clear signatures, strong types, and excellent documentation benefit everyone, including AI agents.
*   **No Meta-Design Docs (M-NO-META-DESIGN-DOCUMENTATION):** Document the *current behavior* and end-state of the code. Do not include process journals, "why we chose X over Y" essays, or tables of applied rules in the source files.
*   **Canonical Documentation Sections (M-CANONICAL-DOCS):** 
    *   Start item documentation with a concise summary sentence (ideally under 15 words) (M-FIRST-DOC-SENTENCE).
    *   Follow up with extended free-form documentation.
    *   Use canonical markdown headers (`# Examples`, `# Errors`, `# Panics`) when applicable.
    *   Do not list parameter tables; explain parameters naturally in the plain text.

## 4. Performance & Memory Efficiency

*   **Allocation Minimization (P-MIN-ALLOC):** Avoid redundant heap allocations (`Vec`, `String`, `Box`). Prefer borrowing (`&str`, `&[T]`) over cloning or `.to_string()` when reading or inspecting data.
*   **Pre-allocation via Capacity (P-WITH-CAPACITY):** When building collections or strings with known or estimable bounds, initialize them using `with_capacity()` to eliminate dynamic reallocations during append operations.
*   **Single-Pass Scanning (P-SINGLE-PASS):** Design string transformation and parsing algorithms (e.g., Markdown AST conversion, template placeholders) to process inputs in a single streaming pass ($O(N)$) without recursive or chained string mutations.
*   **Zero-Copy Slicing (P-ZERO-COPY):** Prefer standard library slicing, `.strip_prefix()`, `.strip_suffix()`, and `.split_once()` over creating new `String` allocations or temporary `Vec<&str>` collections via `.split().collect()`.
*   **Smart Cow Usage (P-COW-STR):** Use `std::borrow::Cow<'a, str>` for struct fields or return values that usually reference static/borrowed strings but occasionally require owned modifications.

## 5. Idiomatic Pattern Matching & Flow Control

*   **Match over Chained If-Else (I-MATCH-TABLES):** Avoid multi-branch if-else cascades for string matching or prefixes. Use concise match expressions, lookup slices, or iterator chains instead.
*   **Functional Iteration (I-ITER-CHAIN):** Prefer declarative iterator chains (`.filter()`, `.map()`, `.count()`, `.find()`) over mutable loop state tracking (`mut count += 1`).
*   **No Unnecessary Formatting Overhead (I-NO-FMT-OVERHEAD):** Avoid creating temporary intermediate `String` instances using `format!()` inside tight loops or stream renders. Write directly into pre-allocated buffer streams (`write!`, `writeln!`) or format-specifiers.

## 6. Binary Size & Compile-Time Optimization

*   **Compile-Time Asset Embedding (B-EMBED-ASSETS):** Embed static UI templates, CSS, JS, and default locales at compile time using `include_str!` or `build.rs` code generation. Do not rely on runtime filesystem paths for embedded assets.
*   **Zero-Dependency Ecosystem (B-ZERO-DEPS):** Favor idiomatic standard library features (`std::collections::HashMap`, `std::path::PathBuf`, slice parsing) over third-party crates unless strictly necessary (e.g., `pulldown-cmark`, `clap`, `serde_json`).
*   **Release Artifact Shrinking (B-STRIP-BINARY):** Release builds must target minimal executable size (enabling `lto = true`, `opt-level = "z"` / `"s"`, `codegen-units = 1`, and binary symbol stripping).

*(For project-specific architecture, tech stack, and test-driven policies, always refer to `AGENTS.md` and `SPECIFICATION.md`.)*
