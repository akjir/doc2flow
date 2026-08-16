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
- **State Condensation:** When tracking multiple boolean configuration flags, evaluate `bitflags` or array-backed state to minimize memory footprint and avoid brittle `&&`/`||` chains.
- **Bitmask Safety:** When using integer bitmasks (`u32`, `u64`), assert `len <= bit_width` at initialization to prevent overflow (P-MASK-SAFE).
- **No Magic Capacities:** Avoid hardcoding array capacities (`[T; 8]`). Define named `const MAX_CAPACITY: usize` with `debug_assert!` checks (P-NO-MAGIC-CAP).

### 2: Parsing & Loops
- **No Chained Regex/Replace:** Replace `.replace().replace()` cascades with single-pass state machines/scanners.
- **Declarative Iterators:** Prefer `.filter()`, `.map()`, `.fold()` over imperative loops with mutable state.
- **Unified Forward Parsers:** Never duplicate forward-scanning loops across token handlers. Encapsulate index advancement and token/code skipping into generic higher-order functions or reusable iterators (P-UNIFIED-PARSER).
- **Linear Parsing & No O(N²):** Ensure token parsers and recursive descent routines parse strictly linearly (O(N)). Avoid repetitive scanning of identical byte slices on unclosed/nested tokens (P-NO-QUADRATIC).

### 3: Formatting & Buffer Directives
- **Static:** `out.write_str("...")` STRICTLY for static literals (no variables).
- **Dynamic:** `write!(out, "...", vars)` for HTML fragments with variables.
- **Anti-Pattern:** NEVER fragment single HTML strings into multiple `write_str` calls solely to avoid `write!`. Maintain readability.
- **Error Paths:** Do NOT micro-optimize error paths with manual buffer allocs/`write!`. Use `format!` or static strings for clarity.

### 4: Idioms & Architecture
- **Standard Traits:** Rely on standard traits (`std::fmt::Display` -> `.to_string()`). NEVER create custom methods (e.g., `to_string_custom()`) or standalone functions duplicating std traits.
- **Constructor Delegation:** If a struct derives `Default`, any implementation of `new()` must delegate to `Self::default()` rather than repeating field initialization.
- **O(1) Hot-Path Routing:** Precompute array indices, state masks, and routing lookups during initialization. Never use for-loops, string matching, or pointer comparisons (`std::ptr::eq`) to resolve modules/AST handlers during active parsing/rendering loops (P-O1-DISPATCH).
- **Flatten Monolithic Dispatchers:** Avoid massive `if/else` or loop dispatchers (>50 lines). Dispatch token parsing to discrete, strongly-typed functions (e.g., `try_parse_* -> Option<usize>`) (P-FLATTEN-DISPATCH).
- **Encapsulate UTF-8 Lookarounds:** Abstract UTF-8 boundary checks and char lookahead/lookbehind (`slice[idx..].chars().next()`) into semantic helper functions (`is_alphanumeric_at`, `is_alphanumeric_before`, `is_alphanumeric_after`) (P-UTF8-LOOKAROUND).
- **Layering:** `src/utils/` = generic project-agnostic library (NO domain logic). `src/core/` & `src/features/` consume it via `src/utils/mod.rs` API.
- **Errors:** Stdlib + `Doc2FlowError` (`src/utils/error.rs`). Avoid complex custom `Enum`s for basic app errors.
- **Panics:** `unwrap()`/`expect()` ONLY for true invariants with descriptive msgs. NEVER for runtime/user I/O.
- **Safety:** ZERO `unsafe` blocks.
- **Consts:** Feature constants local in `src/features/<name>/module.rs` (NO central dumpster). App metadata/limits ONLY in `src/core/constants.rs`.
- **Logic:** Prefer `match` or lookup tables over `if-else` chains.
- **CLI Parsing:** Enforce identical validation for space-separated vs equals-separated flags; reject empty values uniformly (`val.as_ref().is_empty()`).
- **Attributes:** Enforce `#[must_use]` on all constructors, factories, and pure builder methods (`new`, `with_capacity`). Reserve `#[inline]` strictly for trivial getters/wrappers and hot-path inner loops. NEVER apply `#[inline]` to functions performing heap allocs (`String::with_capacity`), I/O, multi-branch logic, setup, init, parser helpers, or CLI parsing logic.
- **Domain Quirks:** Explicitly document intentional domain deviations inline (e.g. strict H1/H2->H3 AST nesting for UI layout) to protect against accidental refactoring.
- **Boolean Parsing:** Account for multiple case-insensitive truthy variants (`true`, `yes`, `y`, `1`) when deserializing boolean parameters from maps/frontmatter/headers.
- **Pipelines & Parity:** DRY template contexts (`build_template_vars`), render conditional components identically across entry points, and single-predicate feature dispatch (`is_feature_active`).
- **Resilient Test Assertions:** Assert specific semantic tokens (e.g., `.contains("bullet")`) on formatted string outputs (like `Display`) rather than brittle exact full-string matches.
- **Fallback Testing:** Always write explicit `#[test]` cases for fallback or default `_ => {}` match arms (M-FALLBACK-TESTS).

### 5: HTML, XML & Asset Processing
- **Scanners:** Zero-alloc single-pass tokenizers (O(N) forward cursor). Avoid redundant scanning passes over attribute names/values.
- **Quote-Aware:** Robustly handle single quotes (`'`), double quotes (`"`), multiline values, and escaped quotes (`\"`/`\'`).
- **Sub-parsers:** Decompose complex parsers into single-responsibility sub-parsers (processing instructions `<?`, DOCTYPE, comments `<!--`, CDATA `<![CDATA[`, tags).
- **Base64 Data URIs:** Standardize with unified `to_base64_data_uri`/`to_base64_data_uri_into` with exact pre-allocation.
- **No println!:** Never use `println!` in core processing routines; reserve `stdout` for CLI output and route progress/warnings to `stderr`/`eprintln!`.

> [!NOTE]
> **[BRANCH EXPERIMENT: feature/modular-building - REVERT ON MERGE]**
> Optimization work on this branch targets `src/exp/`. Production code (`src/core/`, `src/features/`, `src/utils/`) is frozen. When duplicating functions into `src/exp/`, apply all 5 Pillars immediately.