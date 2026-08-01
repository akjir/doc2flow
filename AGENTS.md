# AI Agent Instructions for Doc2Flow (d2f)

Adhere strictly to these guidelines:

## 1. Project Specification Reference
* Specifications reside in `SPECIFICATION.md`.

## 2. Tech Stack Recommendations & Architecture
* CLI parsing: zero-dependency parser in `src/utils.rs` (`std::env::args()`) using idiomatic pattern matching.
* Markdown to HTML: `pulldown-cmark` with GFM extensions.
* Asset processing & encoding: custom zero-allocation RFC 4648 Base64 encoder and MIME-type detection in `src/utils.rs` (`base64_encode`, `guess_mime_type`, `file_to_data_uri`).
* Localization (i18n): dynamic `HashMap<String, String>` dictionary populated via compile-time embedded locale JSON files (`locales/*.json`) generated in `build.rs`.
* Image Optimization: automatic image compression and WebP conversion for oversized local images (`src/image.rs`).
* HTML Templating & Components: zero-allocation buffer-pattern UI components in `src/components.rs`, orchestrated centrally by `src/template.rs`.
* Inline Asset Embeds: static HTML base skeleton (`templates/index.html`), CSS stylesheet (`templates/style.css`), TS client-side logic (`web/src/`), and locales embedded directly into binary at compile time via `include_str!` or code generation.



## 3. Incremental Development
* Implement iteratively: CLI scaffolding → simple Markdown conversion → local image embedding → interactive checklists.

## 4. Pragmatic Rust Guidelines

### Core Principles
* **Idiomatic Rust (M-RUST-SHAPED):** Use native paradigms, ownership, and strong typing. Do not translate from other languages 1-to-1.
* **Strong Typing (C-NEWTYPE):** Avoid primitive obsession; leverage the newtype pattern with documented semantics.
* **Meaningful Tests (M-TAUTOLOGICAL-TESTS):** Verify meaningful behavior and edge cases, not foundational definitions or implementation mirrors.
* **Single-Item Path (M-SINGLE-ITEM-PATH):** Ensure public items have exactly one reachability path. Avoid redundant re-exports.

### Error Handling & Correctness
* **Application-Level Errors (M-APP-ERROR):** Use standard library error propagation (`std::error::Error`) and dedicated domain error types (`Doc2FlowError`). Avoid third-party error handling dependencies (`anyhow`, `eyre`).
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
* **Match over Chained If-Else (I-MATCH-TABLES):** Strict prohibition of deep, unreadable `if ... else if` decision trees and cascades. Replace complex branches with idiomatic Rust pattern matching (`match`), lookup tables/maps, guard clauses, combinators (`Option::and_then`, `strip_prefix`, `map`), or iterators.
  ```rust
  // ❌ ANTI-PATTERN:
  if let Some(s) = text.strip_prefix("note ") {
      ("note", s)
  } else if let Some(s) = text.strip_prefix("tip ") {
      ("note-tip", s)
  } else if ...

  // ✅ BEST PRACTICE (Idiomatisches Pattern Matching / Match Table):
  match text.split_once(' ') {
      Some(("note", rest)) => ("note", rest),
      Some(("tip", rest)) => ("note-tip", rest),
      _ => ("note", text),
  }
  ```
* **Functional Iteration (I-ITER-CHAIN):** Prefer declarative iterators (`.filter()`, `.map()`, `.count()`, `.find()`) over mutable loop state tracking.
* **Formatting & Buffer Directives (I-NO-FMT-OVERHEAD):** Avoid temporary `format!()` strings in loops/stream renders.
  * `out.write_str("...")`: Use exclusively for purely static HTML strings and constants without variable interpolation.
  * `write!(out, "...", vars)`: Use for cohesive HTML fragments containing dynamic variables. Avoid fragmenting HTML structures into multiple cascading `write_str` calls solely to avoid `write!`. Priority is zero-allocation combined with maximum readability of the HTML skeleton.



### Binary Size & Compile-Time Optimization
* **Compile-Time Asset Embedding (B-EMBED-ASSETS):** Embed static templates, CSS, TS client bundles (`web/dist/`), and locales via `include_str!` or `build.rs`. Never rely on runtime paths.
* **Zero-Dependency Ecosystem (B-ZERO-DEPS):** Prefer standard library solutions (`HashMap`, `PathBuf`, slice parsing) over third-party crates unless strictly required.
* **Release Artifact Shrinking (B-STRIP-BINARY):** Release profiles must set `lto = true`, `opt-level = "z"` / `"s"`, `codegen-units = 1`, and enable binary stripping.

## 5. Language & Response Style
* Respond and write **EXCLUSIVELY in English**.
* Never use German in code, comments, files, or artifacts, even if queried in German.
* **Concise Communication:** Keep AI response output short, simple, direct, and restricted to a single line.

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
* Regenerate showcase HTML files (`tests/showcase_de.html`, `tests/showcase_en.html`) via `cargo run` whenever renderer, template, CSS, TS, or conversion logic changes.

## 9. HTML Template & UI Guidelines
* Generic Templates: `templates/index.html`, `templates/style.css`, and client scripts in `web/src/` must remain completely generic and devoid of customer-specific text.
* Modular HTML Components: Reusable UI fragments (headers, callouts, code blocks, task/list items) are rendered via zero-allocation buffer functions in `src/components.rs` and orchestrated exclusively through `src/template.rs`.
* Markdown Mapping:

  * Level 2 headings (`##`) → collapsible `.section` with `.sh`/`.sb` classes.
  * Checkbox unordered lists → wrapped in `.check-item`.
  * Blockquote callouts → map prefixes (`>`, `>?`, `>!`, `>!!`, `>!!!`) to alert panels (`.note`, `.note-tip`, `.note-important`, `.note-warning`, `.note-caution`).
  * Image & Link Handling: Local image files are converted to Base64 `data:` URIs. Remote image URLs (`http://`, `https://`) are preserved as `<img>` tags. Non-image resources (e.g., `.pdf`, `.zip`) specified in image tags are converted to external link elements (`<a>`) wrapped in standard `.check-item.text-item` containers for CSS parity.
* Placeholders: Replace generic placeholders (e.g., `{{TITLE}}`, `{{CUSTOMER}}`) in `base.html` using frontmatter/metadata.

## 10. Client-Side TypeScript Guidelines
* **Strict Compiler Configuration (TS-STRICT-CONFIG):**
  * Enforce `"strict": true` across all TypeScript projects (`strictNullChecks`, `noImplicitAny`, `strictFunctionTypes`).
  * Enforce `"noUncheckedIndexedAccess": true` to require `T | undefined` handling for indexed array and object map accesses.
  * Enforce `"exactOptionalPropertyTypes": true` to strictly distinguish optional properties (`foo?: string`) from explicit `undefined` (`foo: string | undefined`).
  * Enforce `"verbatimModuleSyntax": true` to require explicit type-only imports (`import type { User } from './types'`), optimizing bundler tree-shaking (esbuild).
* **Type Inference & Immutability (TS-INFERENCE-IMMUTABLE):**
  * Rely on automatic type inference where context is unambiguous; omit redundant type annotations on obvious local variable declarations.
  * Protect data structures from unintentional mutation using `readonly` property and array modifiers (`readonly apiEndpoint: string`, `readonly permissions: readonly string[]`).
* **State Modeling with Discriminated Unions (TS-DISCRIMINATED-UNIONS):**
  * Model application states (API responses, UI state transitions) using discriminated unions to make impossible states unrepresentable (e.g. `{ status: 'idle' } | { status: 'loading' } | { status: 'success'; data: T } | { status: 'error'; error: Error }`).
* **Type vs. Interface Usage (TS-INTERFACE-VS-TYPE):**
  * Use `interface` for expandable object structures, component props, and public contracts that benefit from declaration merging.
  * Use `type` for union types, tuples, primitive aliases, and mapped types.
* **`satisfies` Operator & Utility Types (TS-SATISFIES-UTILITIES):**
  * Use the `satisfies` operator (TS 4.9+) to validate object literals against schemas without widening or narrowing their exact inferred property types.
  * Adhere to DRY principles by reusing types via standard Utility Types (`Pick`, `Omit`, `Partial`, `ReturnType`, `Parameters`) rather than duplicating interfaces manually.
* **TypeScript Anti-Patterns & Prohibitions (TS-ANTI-PATTERNS):**
  * **Prohibit `any` (TS-NO-ANY):** Never use `any`. Use `unknown`, generics, or specific union types instead.
  * **Prohibit Assertions (TS-NO-ASSERTIONS):** Avoid type assertions (`as Type`) and non-null assertions (`!`). Use Zod / Valibot for runtime validation of external payloads or explicit type guards (`typeof input === 'string'`).
  * **Prohibit TypeScript `enum`s (TS-NO-ENUMS):** Avoid traditional TypeScript `enum` declarations due to runtime overhead and type-checking quirks. Use string union types (`type UserRole = 'ADMIN' | 'USER'`) or `as const` object declarations instead.
  * **Prohibit Ambiguous Object Types (TS-NO-EMPTY-OBJECT):** Avoid `Object`, `{}` or `object` for general key-value maps. Use `Record<string, unknown>` for dynamic string-keyed objects or `object` strictly for non-primitives.

## 11. CSS & Styling Guidelines
* **CSS Variables (CSS-VARS):** Define all primary/brand colors, status backgrounds, alert variant colors, radii, borders, and font families centrally under `:root`.
* **Zero External Dependencies (CSS-ZERO-DEPS):** Never use external webfont imports (`@import url(...)`). Use lean, cross-platform system font stacks (e.g. `Arial, sans-serif`, `'Courier New', monospace`).
* **Class Naming Conventions (CSS-BEM-NAMING):** Maintain flat, consistent BEM-style class names (`.doc-*`, `.sb`, `.sh`, `.check-item`, `.note`, `.btn-*`, `.btn-reset`).
* **Print CSS Specification (CSS-PRINT-MEDIA):**
  * Hide interactive controls and progress indicators: `.btnrow, .copy-btn, .stog, .item-comment-icon, .item-comment-del, .sbadge, .pb-wrap, .pb, .pt { display: none !important; }`.
  * Expand all sections: `.sb.collapsed { display: block !important; }`.
  * Continuous page flow: Eliminate forced page breaks (`break-inside: avoid;` is omitted so `.section`, `##`, and child elements break naturally across pages without creating large whitespace gaps).
  * Force exact background/color rendering: Use `-webkit-print-color-adjust: exact; print-color-adjust: exact;` on section headers (`.sh`), code blocks, callouts (`.note`), and finish signatures box (`.finish`).
  * Remove strikethrough on checked items: `.check-item.checked { background: transparent !important; }` and `.check-item.checked .check-label { text-decoration: none !important; color: var(--gray) !important; }`.
* **File Code Structure (CSS-STRUCTURE):** Subdivide `templates/style.css` using explicit comment header blocks: `1. Base`, `2. Layout`, `3. Components`, `4. Print`, and `5. Responsive`.

## 12. Changelog Policy
* Automatically update `CHANGELOG.md` under `[Unreleased]` ONLY when user-facing features, bug fixes, or user-facing changes are made. Keep entries short, simple, and limited to concise single-line bullet points.
* Do NOT leave an empty `## [Unreleased]` section in `CHANGELOG.md`. When releasing a new version, omit `## [Unreleased]` until new changes are added.
* **STRICT PROHIBITION:** Do NOT include internal code refactorings, module restructuring, optimizations, or test infrastructure changes in `CHANGELOG.md`. Only user-visible changes belong in the changelog.