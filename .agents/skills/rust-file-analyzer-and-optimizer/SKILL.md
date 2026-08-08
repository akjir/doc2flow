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

### 2: Parsing & Loops
- **No Chained Regex/Replace:** Replace `.replace().replace()` cascades with single-pass state machines/scanners.
- **Declarative Iterators:** Prefer `.filter()`, `.map()`, `.fold()` over imperative loops with mutable state.

### 3: Formatting & Buffer Directives
- **Static:** `out.write_str("...")` STRICTLY for static literals (no variables).
- **Dynamic:** `write!(out, "...", vars)` for HTML fragments with variables.
- **Anti-Pattern:** NEVER fragment single HTML strings into multiple `write_str` calls solely to avoid `write!`. Maintain readability.
- **Error Paths:** Do NOT micro-optimize error paths with manual buffer allocs/`write!`. Use `format!` or static strings for clarity.

### 4: Idioms & Architecture
- **Errors:** Stdlib + `Doc2FlowError`. Avoid complex custom `Enum`s for basic app errors.
- **Panics:** `unwrap()`/`expect()` ONLY for true invariants with descriptive msgs. NEVER for runtime/user I/O.
- **Safety:** ZERO `unsafe` blocks.
- **Consts:** Feature constants local in `src/features/<name>/module.rs` (NO central dumpster). App metadata/limits ONLY in `src/core/constants.rs`.
- **Logic:** Prefer `match` or lookup tables over `if-else` chains.
- **CLI Parsing:** Enforce identical validation for space-separated vs equals-separated flags; reject empty values uniformly (`val.as_ref().is_empty()`).
- **Attributes:** Reserve `#[inline]` strictly for trivial getters/wrappers and hot-path inner loops/rendering. NEVER apply `#[inline]` to single-call setup, init, parser helpers, or CLI parsing logic.

### 5: HTML, XML & Asset Processing
- **Scanners:** Zero-alloc single-pass tokenizers (O(N) forward cursor). Avoid redundant scanning passes over attribute names/values.
- **Quote-Aware:** Robustly handle single quotes (`'`), double quotes (`"`), multiline values, and escaped quotes (`\"`/`\'`).
- **Sub-parsers:** Decompose complex parsers into single-responsibility sub-parsers (processing instructions `<?`, DOCTYPE, comments `<!--`, CDATA `<![CDATA[`, tags).
- **Base64 Data URIs:** Standardize with unified `to_base64_data_uri`/`to_base64_data_uri_into` with exact pre-allocation.
- **No println!:** Never use `println!` in core processing routines; reserve `stdout` for CLI output and route progress/warnings to `stderr`/`eprintln!`.