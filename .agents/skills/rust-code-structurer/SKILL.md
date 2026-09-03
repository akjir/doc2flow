---
name: rust-code-structurer
description: Reorganizes Rust source code to adhere to strict formatting and vertical layout rules without altering any business logic, runtime behavior, or test semantics. Use when formatting or restructuring Rust files.
---
# Rust Structurer
**Goal:** Deterministic `.rs` layout. 0 logic changes.

## USE WHEN
- Reorganizing/formatting `.rs` code.

## 0 LOGIC CHANGE
NEVER modify logic/sigs/errors/tests. ONLY reorder/format.

## 1. FILE LAYOUT
1. **Docs:** `//! ...`
2. **Attrs:** `#![...]` (mandatory `#![forbid(unsafe_code)]` in crate roots).
3. **Imports:** 3 groups separated by blank line: `std::`, 3rd-party, `crate::`/`super::`. Alphabetize lines & braced items.
4. **Consts/Statics:** Alphabetized.
5. **Types & Impls:** Struct/Enum/Alias (alphabetized). `impl` MUST immediately follow Type.
6. **Free Fns:** Alphabetized.
7. **Tests:** `#[cfg(test)] mod tests { ... }` at bottom.

## 2. STRUCTURAL RULES
**A. Structs/Enums:**
- Visibility: `pub` > private. Alphabetize within tier.
- Macros: Alphabetize `#[derive(...)]`.
- `Default`: Impl for all ZSTs/parameter-less types.
- Errors: Strongly typed `enum` (NO `Result<T,String>`).
- State: `bitflags`/array for bools.

**B. Impls (`impl Type`):**
1. Inherent: Constructors top (`new`,`default`,`with_capacity`). Enforce `#[must_use]`. `new` MUST delegate to `Default` if impl'd (or `Self` for ZSTs). Methods alphabetized below.
2. Traits: standard traits (`Display`), sorted alphabetically by trait below inherent impl. NO custom dup fns.

**C. Control Flow / Parsing:**
- Match: Order arms by enum decl. `_ =>` last.
- Dispatchers: Flat (<50 lines). Delegate to `try_parse_*`.
- UTF-8: Encapsulate boundaries to helpers.
- CLI/Paths: Pure iterators, OS-agnostic (`OsStr`/`OsString`).
- I/O: Direct `std::fs` isolated to `src/core/io.rs` (P-IO-ISOLATION).
- JSON: Decode UTF-16 surrogate pairs, RFC 8259 validation, NO deps.
- Case: `eq_ignore_ascii_case` in guards (0-alloc).
- Attrs: `#[inline]` ONLY on trivial/hot-paths. BANNED on allocs, I/O, CLI, large match (P-ATTR-USAGE).
- Err: `#[source]` chain (P-ERR-PRESERVE).

**D. Tests:**
- Asserts: Semantic tokens (`.contains()`) vs exact string.
- Edge: Path logic -> hidden, dot, multi-ext, empty tests (M-PATH-EDGE-TESTS).
- I/O: Mandatory temp dir/mem cursor tests for I/O functions (M-IO-TESTS).

## 3. DOCS/VISIBILITY
- Keep `///` above items.
- Keep `//` domain quirks docs.
- Prefer named exports (`pub use x::{A, B};`).

## WORKFLOW
1. Parse/inventory items.
2. Sort imports (3 groups).
3. Group Type+Impls, alphabetize traits.
4. Sort methods (constructors top, rest alpha).
5. Sort free fns.
6. Tests at bottom.
7. Verify 0 logic drop.
