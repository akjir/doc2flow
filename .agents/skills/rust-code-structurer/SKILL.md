---
name: rust-code-structurer
description: Reorganizes Rust source code to adhere to strict formatting and vertical layout rules without altering any business logic, runtime behavior, or test semantics. Use when formatting or restructuring Rust files.
---

# Rust Code Structurer

**Goal:** Establish deterministic, standardized code layout across Rust (`.rs`) files with zero logic or behavior changes.

## USE WHEN
- Reorganizing Rust source files for clean code hygiene.
- Standardizing imports, struct/impl groupings, and function ordering.
- Formatting code before committing or code reviews.

## NON-NEGOTIABLE DIRECTIVE
**ZERO LOGIC CHANGE:** NEVER modify, add, or remove functional logic, algorithms, variable bindings, signatures, error handling, or test assertions. ONLY reorder, group, and format existing code items.

## 1. FILE LAYOUT & VERTICAL ORDER
Every `.rs` file must strictly follow this top-to-bottom sequence:

1. **Inner Module Docs:** Module rustdocs (`//! ...`).
2. **File Attributes:** Outer attributes (`#![allow(...)]`, `#![deny(...)]`).
3. **Grouped & Sorted Imports (`use`):**
   - 3 blank-line-separated groups:
     1. `std::...` (Standard library)
     2. Third-party crates (`image`)
     3. Internal crate items (`crate::...`, `super::...`)
   - Alphabetize lines within each group.
   - Alphabetize merged curly-brace imports (e.g., `use std::collections::{BTreeMap, HashMap};`).
4. **Global Constants & Statics:** Module `const` and `static` items (alphabetized).
5. **Type Definitions & Implementations:**
   - Structs, Enums, Type Aliases (sorted alphabetically by type name).
   - **Impl Placement:** Every `impl` block MUST immediately follow its associated Struct/Enum definition.
6. **Standalone / Free Functions:** Alphabetized by function name.
7. **Test Module:** `#[cfg(test)] mod tests { ... }` MUST always be the last item at the file bottom.

## 2. STRUCTURAL ORDERING RULES

### A. Structs & Enums
- **Field Visibility:** `pub` fields first, then private fields.
- **Field Sorting:** Alphabetical order within the same visibility tier.
- **Derives:** Alphabetize macros inside `#[derive(...)]` (e.g., `#[derive(Clone, Debug, PartialEq)]`).
- **Default over New:** Strictly derive/implement `Default` for all parameter-less structs and Zero-Sized Types (ZSTs).
- **Strongly Typed Errors:** Define descriptive Error enums (implementing `Display` and `std::error::Error`) for modules and parsing logic; never use `Result<T, String>`.
- **State Condensation:** Evaluate `bitflags` or array-backed state representation for multiple boolean configuration flags.

### B. Implementation (`impl`) Blocks
For type `MyType`:
1. **Primary/Inherent `impl MyType`:**
   - **Constructors First:** `new()`, `default()`, `with_capacity()`, or custom constructors at the top (enforce `#[must_use]`). If struct implements `Default`, `new()` MUST strictly delegate to `Self::default()`. Standalone `new()` duplicating field inits is BANNED.
   - **Inherent Methods:** Sorted alphabetically after constructors.
2. **Trait Implementations (`impl Trait for MyType`):**
   - Implement standard library traits (e.g., `impl Display for MyType` with `.to_string()`); never create custom duplicate methods (e.g., `to_string_custom()`) or standalone functions.
   - Directly below primary `impl MyType`.
   - Sorted alphabetically by Trait name (e.g., `impl Display` before `impl From<T>`).

### C. Control Flow & Match Sorting
- **Match Arms:** Order `enum` variants in `match` expressions matching declaration order in `enum` definition.
- **Catch-all Arm:** Fallback (`_ => ...`) MUST always be the last arm.
- **Flattened Dispatchers:** Keep token dispatch loops flat (<50 lines) by delegating parsing to discrete, strongly-typed helper functions (`try_parse_*`).
- **Encapsulated Lookarounds:** Abstract UTF-8 boundary checks and char inspection into semantic helper functions rather than inline pointer/char math.
- **Pure CLI & OS-Agnostic Paths:** Parser functions must accept pure argument iterators (caller strips binary via `args_os().skip(1)`), use `std::ffi::OsStr`/`OsString` for filesystem paths without assuming UTF-8, and avoid fragile flag peeking.
- **Zero-Dep JSON Compliance:** Custom JSON parsers MUST decode UTF-16 surrogate pairs (`\uD800..\uDBFF` + `\uDC00..\uDFFF`) into scalar chars; never rely solely on `char::from_u32` for 4-digit hex escapes.
- **Strict Primitive Validation:** Unquoted JSON values must strictly validate against RFC 8259 (`true`, `false`, `null`, numbers). Permissive "read until delimiter" logic is strictly prohibited.
- **Stdlib Only:** Core engine modules must use standard library functionality exclusively without third-party crates (`serde_json`, `nom`).

### D. Tests & Assertions
- **Resilient Assertions:** Assert specific semantic tokens (e.g., `.contains("bullet")`) on formatted string outputs (like `Display`) instead of brittle exact full-string matches.

## 3. DOCUMENTATION & VISIBILITY STANDARDS
- **Rustdoc Comments:** Retain `/// ...` comments directly above items/attributes with no blank lines.
- **Domain Quirks:** Retain and enforce `// ...` inline documentation explaining intentional structural deviations (e.g. AST nesting constraints).
- **Export Hygiene:** Prefer explicit named imports/exports (`pub use module::{A, B};`) over wildcards (`pub use module::*`).

## EXECUTION WORKFLOW
1. **Parse:** Scan target file and inventory items (imports, types, impls, free functions, tests).
2. **Imports:** Partition into `std`, external, and internal groups; alphabetize groups and inner braces.
3. **Types & Impls:** Group each struct/enum with its inherent `impl` and trait `impl`s (alphabetized by trait).
4. **Methods:** Sort constructors to top of inherent `impl`, alphabetize remaining methods.
5. **Functions:** Alphabetize standalone free functions.
6. **Tests:** Anchor `#[cfg(test)] mod tests` at bottom of file.
7. **Verify:** Ensure zero expressions, logic, comments, or tests were dropped or mutated.
