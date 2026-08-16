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
     2. Third-party crates (`serde`, `pulldown_cmark`)
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
- **State Condensation:** Evaluate `bitflags` or array-backed state representation for multiple boolean configuration flags.

### B. Implementation (`impl`) Blocks
For type `MyType`:
1. **Primary/Inherent `impl MyType`:**
   - **Constructors First:** `new()`, `default()`, `with_capacity()`, or custom constructors at the top (enforce `#[must_use]`). If struct derives `Default`, `new()` must delegate to `Self::default()`.
   - **Inherent Methods:** Sorted alphabetically after constructors.
2. **Trait Implementations (`impl Trait for MyType`):**
   - Implement standard library traits (e.g., `impl Display for MyType` with `.to_string()`); never create custom duplicate methods (e.g., `to_string_custom()`) or standalone functions.
   - Directly below primary `impl MyType`.
   - Sorted alphabetically by Trait name (e.g., `impl Display` before `impl From<T>`).

### C. Control Flow & Match Sorting
- **Match Arms:** Order `enum` variants in `match` expressions matching declaration order in `enum` definition.
- **Catch-all Arm:** Fallback (`_ => ...`) MUST always be the last arm.

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
