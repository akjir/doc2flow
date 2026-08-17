---
name: rust-file-analyzer-and-optimizer
description: Analyzes/optimizes Rust code for memory efficiency, zero-allocation, idiomatic patterns, and performance (Doc2Flow context).
---

# Rust Analyzer & Optimizer

**Goal:** Enforce high-performance, zero-copy parsing, minimal heap allocations, and idiomatic Rust for Doc2Flow.

## USE WHEN
- Auditing `src/*.rs` for bottlenecks/code smells.
- Refactoring to eliminate heap allocations (`.to_string()`, `.clone()`, `format!`).
- Optimizing buffer writes (`out.write_str` vs `write!`).
- Enforcing idioms, zero `unsafe`, and strict `anyhow` error contexts.

## EXECUTION WORKFLOW
Follow these 4 steps sequentially, applying the 5 Pillars below:

1. **Scan:** Audit code against the 5 Pillars (allocations, macros, loops, error contexts).
2. **Trade-Off:** Weigh Performance vs. Readability. Reject readability-destroying micro-optimizations.
3. **Output:** Provide FULL refactored module (NO placeholders). Retain `#[inline]` on hot-paths and all `#[cfg(test)]` modules.
4. **Summary:** Provide a concise bulleted rationale mapping changes to specific benefits.

## THE 5 PILLARS

### 1: Memory & Allocations
- **Borrowing:** Prefer `&str`, `&[u8]`, `Cow<'a, str>`. Avoid `String` parameters/statics.
- **No Waste:** Eliminate unnecessary `.to_string()`, `.to_owned()`, `PathBuf::from()`.
- **Zero-Copy:** Use `.split_once()`, `.strip_prefix()`. AVOID intermediate collections (`.collect::<Vec<_>>()`).
- **Safe Slicing:** Use safe subslice manipulation (`.split_once()`, `.strip_prefix()`, cursor offsets). Prohibit raw pointer arithmetic (`as_ptr` diffs) for string bounds.
- **Pre-allocate:** ALWAYS use `.with_capacity()` for dynamic collections in loops.
- **Bounded Allocations:** Always clamp dynamic `usize` inputs used for memory allocation (e.g. `String::with_capacity`, padding lengths, gutter widths) against hardcoded limits or validated logical bounds to prevent OOM panics (P-BOUND-ALLOC).
- **Idiomatic Duplication:** Prefer `str::repeat()` over manual `Iterator::extend`/`repeat_n` loops for standard character/string repetition (P-IDIOM-REPEAT).
- **State Condensation:** When tracking multiple boolean configuration flags, evaluate `bitflags` or array-backed state to minimize memory footprint and avoid brittle `&&`/`||` chains.
- **Bitmask Safety:** When using integer bitmasks (`u32`, `u64`), assert `len <= bit_width` at initialization to prevent overflow (P-MASK-SAFE).
- **No Magic Capacities:** Avoid hardcoding array capacities (`[T; 8]`). Define named `const MAX_CAPACITY: usize` with `debug_assert!` checks (P-NO-MAGIC-CAP).

### 2: Parsing & Loops
- **No Chained Regex/Replace:** Replace `.replace().replace()` cascades with single-pass state machines/scanners.
- **Declarative Iterators:** Prefer `.filter()`, `.map()`, `.fold()` over imperative loops with mutable state.
- **Unified Forward Parsers:** Never duplicate forward-scanning loops across token handlers. Encapsulate index advancement and token/code skipping into generic higher-order functions or reusable iterators (P-UNIFIED-PARSER).
- **Linear Parsing & No O(N²):** Ensure token parsers and recursive descent routines parse strictly linearly (O(N)). Avoid repetitive scanning of identical byte slices on unclosed/nested tokens (P-NO-QUADRATIC).
- **Zero-Dep JSON Compliance:** Custom JSON parsers MUST explicitly support UTF-16 surrogate pair decoding (`\uD800..\uDBFF` + `\uDC00..\uDFFF`). Relying solely on `char::from_u32` for 4-digit hex escapes is strictly prohibited (fails outside BMP/emojis) (P-JSON-SURROGATE).
- **Strict Primitive Validation:** Unquoted JSON values MUST strictly validate against RFC 8259 (`true`, `false`, `null`, numbers). Permissive "read until delimiter" accepting arbitrary bare words or malformed floats is strictly prohibited; emit explicit error (P-JSON-STRICT-PRIMITIVES).
- **Stdlib Only:** ZERO external dependencies (`serde_json`, `nom`) in zero-dependency core engine components. String manipulation and validation must utilize standard library functionality exclusively (`str::from_utf8`) (P-STDLIB-ONLY).
- **Zero-Alloc Case Insensitivity:** When case-insensitive string matching is required without heap allocations, strictly use `eq_ignore_ascii_case` inside match guards instead of allocating via `.to_ascii_lowercase()` (P-ZERO-ALLOC-CASE).

### 3: Formatting & Buffer Directives
- **Static:** `out.write_str("...")` STRICTLY for static literals (no variables).
- **Dynamic:** `write!(out, "...", vars)` for HTML fragments with variables.
- **Anti-Pattern:** NEVER fragment single HTML strings into multiple `write_str` calls solely to avoid `write!`. Maintain readability.
- **Error Paths:** Do NOT micro-optimize error paths with manual buffer allocs/`write!`. Use `format!` or static strings for clarity.

### 4: Idioms & Architecture
- **Standard Traits:** Rely on standard traits (`std::fmt::Display` -> `.to_string()`). NEVER create custom methods (e.g., `to_string_custom()`) or standalone functions duplicating std traits.
- **Default over New:** Strictly implement `std::default::Default` for all parameter-less structs and Zero-Sized Types (ZSTs).
- **Constructor Delegation:** BANNED: Implementing a standalone `new()` method that duplicates field initialization. IF `new()` is required for API ergonomics, it MUST strictly delegate to `Self::default()`.
- **Zero-Allocation Registries:** Enforce static array definitions (`[&'static dyn Trait; N]`) for central registries (e.g., feature modules) to guarantee O(1) startup and zero runtime heap allocation.
- **Tripwire Testing:** Use explicit lengths and hardcoded arrays in registry tests to force manual verification when expanding system features (e.g., adding a new module).
- **O(1) Hot-Path Routing:** Precompute array indices, state masks, and routing lookups during initialization. Never use for-loops, string matching, or pointer comparisons (`std::ptr::eq`) to resolve modules/AST handlers during active parsing/rendering loops (P-O1-DISPATCH).
- **Flatten Monolithic Dispatchers:** Avoid massive `if/else` or loop dispatchers (>50 lines). Dispatch token parsing to discrete, strongly-typed functions (e.g., `try_parse_* -> Option<usize>`) (P-FLATTEN-DISPATCH).
- **Encapsulate UTF-8 Lookarounds:** Abstract UTF-8 boundary checks and char lookahead/lookbehind (`slice[idx..].chars().next()`) into semantic helper functions (`is_alphanumeric_at`, `is_alphanumeric_before`, `is_alphanumeric_after`) (P-UTF8-LOOKAROUND).
- **Layering:** `src/utils/` = generic project-agnostic library (NO domain logic). `src/core/` & `src/features/` consume it via `src/utils/mod.rs` API.
- **Errors:** Stdlib + `Doc2FlowError` (`src/utils/error.rs`). Strongly typed error enums for modules/parsing (NO `Result<T, String>`).
- **Panics:** `unwrap()`/`expect()` ONLY for true invariants with descriptive msgs. NEVER for runtime/user I/O.
- **Safety:** ZERO `unsafe` blocks.
- **Consts:** Feature constants local in `src/features/<name>/module.rs` (NO central dumpster). App metadata/limits ONLY in `src/core/constants.rs`.
- **Logic:** Prefer `match` or lookup tables over `if-else` chains.
- **CLI Parsing:** Pure parser inputs (callers strip binary with `args_os().skip(1)`). OS-agnostic paths via `std::ffi::OsStr`/`OsString` (no UTF-8 assumption). Identical validation for space (`-o ""`) vs equals (`-o=`) syntax; reject empty values uniformly (`val.as_ref().is_empty()`). Avoid fragile flag peeking: require explicit `=` or strict bounds for optional values/hyphenated args (P-CLI-PURE).
- **Attributes:** Enforce `#[must_use]` on all constructors, factories, and pure builder methods (`new`, `with_capacity`). Reserve `#[inline]` strictly for trivial getters/wrappers and hot-path trait implementations/loops. BANNED: `#[inline]` on large/branching functions, large `match` blocks, complex string operations, heap allocs (`String::with_capacity`), I/O, multi-branch logic, setup, init, parser helpers, or CLI parsing without profiling. Rely on LTO and compiler heuristics (P-ATTR-USAGE).
- **Domain Quirks:** Explicitly document intentional domain deviations inline (e.g. strict H1/H2->H3 AST nesting for UI layout) to protect against accidental refactoring.
- **Boolean Parsing:** Account for multiple case-insensitive truthy variants (`true`, `yes`, `y`, `1`) when deserializing boolean parameters from maps/frontmatter/headers.
- **Pipelines & Parity:** DRY template contexts (`build_template_vars`), render conditional components identically across entry points, and single-predicate feature dispatch (`is_feature_active`).
- **Resilient Test Assertions:** Assert specific semantic tokens (e.g., `.contains("bullet")`) on formatted string outputs (like `Display`) rather than brittle exact full-string matches.
- **Fallback Testing:** Always write explicit `#[test]` cases for fallback or default `_ => {}` match arms (M-FALLBACK-TESTS).
- **Extreme Boundary Testing:** Mandate extreme edge-case unit tests (`usize::MAX`, `0`, overflow bounds) for functions performing length/padding math (M-EXTREME-BOUND-TESTS).
- **Path Edge-Case Testing:** All functions analyzing `std::path::Path`/`PathBuf` components (extensions, filenames) MUST include unit tests for filesystem edge cases: hidden files (`.env`), missing filenames/trailing slashes (`/`), empty extensions/trailing dots (`file.`), and compound extensions (`.tar.gz`) (M-PATH-EDGE-TESTS).

### 5: HTML, XML & Asset Processing
- **Scanners:** Zero-alloc single-pass tokenizers (O(N) forward cursor). Avoid redundant scanning passes over attribute names/values.
- **Quote-Aware:** Robustly handle single quotes (`'`), double quotes (`"`), multiline values, and escaped quotes (`\"`/`\'`).
- **Sub-parsers:** Decompose complex parsers into single-responsibility sub-parsers (processing instructions `<?`, DOCTYPE, comments `<!--`, CDATA `<![CDATA[`, tags).
- **Base64 Data URIs:** Standardize with unified `to_base64_data_uri`/`to_base64_data_uri_into` with exact pre-allocation.
- **No println!:** Never use `println!` in core processing routines; reserve `stdout` for CLI output and route progress/warnings to `stderr`/`eprintln!`.

> [!NOTE]
> **[BRANCH EXPERIMENT: feature/modular-building - REVERT ON MERGE]**
> Optimization work on this branch targets `src/exp/`. Production code (`src/core/`, `src/features/`, `src/utils/`) is frozen. When duplicating functions into `src/exp/`, apply all 5 Pillars immediately.