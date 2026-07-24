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

*(For project-specific architecture, tech stack, and test-driven policies, always refer to `AGENTS.md` and `SPECIFICATION.md`.)*
