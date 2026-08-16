# Doc2Flow Agent Directives

## 1. Arch & Stack
- **Spec:** `SPECIFICATION.md` (sync structure on central file changes)
- **Layering:** `src/utils/` (generic library), `src/core/` (domain engine), `src/features/` (vertical slices). Core/features access `utils` through `src/utils/mod.rs` API.
- **CLI:** `std::env::args()`
- **MD:** `pulldown-cmark`+GFM
- **Assets:** Custom Base64/MIME (`src/utils/`), WebP/compress (`src/core/image.rs`)
- **i18n:** `HashMap` via embedded JSON (`build.rs`)
- **UI/HTML:** Zero-alloc buffers (`src/core/components.rs`, `src/core/builder.rs`), compile-time embeds (`include_str!`)
- **Pipelines:** DRY template contexts (`build_template_vars`), parity across entry points (identical conditional components; NO hardcoded blanks). Single predicate feature dispatch (`is_feature_active`).
- **Flow:** CLI > MD > Img > UI

## 2. Rust
- **Traits:** Rely on standard traits (`std::fmt::Display` -> `.to_string()`). NEVER create custom duplicate methods (e.g. `to_string_custom()`) or standalone functions.
- **Constructors:** If struct derives `Default`, `new()` MUST delegate to `Self::default()` (no repeated field inits).
- **State:** Multiple boolean flags -> evaluate `bitflags` or array-backed state (min memory footprint, avoid brittle `&&`/`||` chains).
- **Dispatch:** O(1) routing ONLY (precompute array indices/masks at init). NO loops/strings/`ptr::eq` on hot-path AST handling (P-O1-DISPATCH).
- **Bitmask:** Integer bitmasks (`u32`) MUST assert `len <= bit_width` at init (P-MASK-SAFE).
- **Buffers:** NO magic capacities (`[T; 8]`). Define named `const MAX_CAPACITY: usize` with `debug_assert!` against truncations (P-NO-MAGIC-CAP).
- **Core:** Idiomatic, newtypes, 1-path exports, NO `unsafe`
- **Clean:** Zero legacy debt/compat shims. Remove dead/obsolete code when adding new code.
- **Consts:** Feature constants local in `src/features/<name>/module.rs` (NO central dumpster). App metadata/limits ONLY in `src/core/constants.rs`.
- **CLI:** Identical validation for space (`-o ""`) vs equals (`-o=`) syntax. Reject empty values uniformly (`val.as_ref().is_empty()`).
- **Errors:** Stdlib+`Doc2FlowError` (NO `anyhow`/`eyre`). `Result`=expected. `panic!`=bugs/stop (detailed msgs). NO `catch_unwind`. Safe bounds/slicing on diagnostic buffers. `From` conversions (NO `.to_<domain>()`). NO manual buffer micro-allocs on error paths; use `format!` or static strings.
- **Attributes:** Enforce `#[must_use]` on all constructors, factories, and pure builders (`new`, `with_capacity`). Reserve `#[inline]` exclusively for trivial getters/wrappers and hot-path loops. NO `#[inline]` on heap allocs (`String::with_capacity`), I/O, multi-branch, init, setup, CLI parsing, parser helpers, or simple `const` fns.
- **Docs:** English ONLY (all inline docs & comments). 15-word max start, canonical headers (Examples/Errors/Panics), NO meta/journals. Explicitly document intentional domain quirks inline (e.g. strict H1/H2->H3 AST nesting for UI layout) to prevent regressions.
- **Perf:** Min-alloc (borrow>owned), `with_capacity`, O(N) 1-pass, zero-copy (`split_once`,`strip_prefix`), `Cow`. Safe subslice indexing ONLY; NO raw pointer arithmetic (`as_ptr` diffs) for string bound searches.
- **Parsing:** Flexible boolean deserialization from maps/frontmatter/headers; account for case-insensitive truthy matrix (`true`, `yes`, `y`, `1`).
- **String/Buffer:** Exact `with_capacity` pre-alloc. Direct buffer streaming (`write_str`/`push_str`). NO intermediate `Vec`/strings on hot paths.
- **HTML/XML/SVG:** Zero-alloc tokenizers (O(N) 1-pass forward cursor). Quote-aware (single `'`, double `"`, multiline, escaped `\"`/`\'`). Sub-parsers for declarations (`<?`), DOCTYPE, comments (`<!--`), CDATA (`<![CDATA[`), tags. NO redundant scanning passes over attribute names/values. NO `println!` in core processing routines.
- **Flow:** `match`/tables > `if-else`. Iterators > loops. `write_str`(static)/`write!`(dynamic) > `format!` (hot-path buffers).
- **Build:** `lto=true`, `opt=z|s`, `codegen-units=1`, strip. Favor stdlib over deps.

## 3. Ops & Tests
- **Comm:** English ONLY. 1-line concise AI responses.
- **OS:** Linux dev, Win64 target. `std::path::Path/Buf` ONLY.
- **Git:** Commit ONLY if requested AND tests pass (or user overrides).
- **Test:** Priority 1. Negative/edge cases. Semantic token assertions (e.g. `.contains("bullet")`) over exact full strings on formatted output (`Display`). Explicit tests for fallback/default `_ => {}` arms (M-FALLBACK-TESTS). Regen `showcase_*.html` on UI changes.

## 4. Frontend (HTML/JS/CSS)
- **HTML (Generic):**
  - `##` -> `.section.sh.sb` (collapsible)
  - `[ ]` List -> `.check-item`
  - Quotes (`>`,`>?`,`>!`) -> `.note` variants
  - Local Img -> Base64; Remote -> `<img>`; Non-img asset -> `<a>.check-item.text-item`
  - Vars replaced via frontmatter
- **JS (Current):** Vanilla JS ONLY (`.js`). NO TS/build step. NO `export`/`import`; decouple via `window.d2f` namespace (`window.d2f.<module>`).
- **TS (Legacy):** `strict`, NO `export`/`import` (`window.d2f`), `readonly`, discriminated unions. BANNED: `any`, `as`, `!`, `enum`, `{}`/`Object`, `?`.
- **CSS:**
  - `:root` vars, BEM classes, ZERO external deps (fonts)
  - Print: Hide UI/buttons, expand collapsed (`display:block!important`), natural page breaks (no forced), exact print colors, no strikethrough
  - File: 1.Base > 2.Layout > 3.Components > 4.Print > 5.Responsive

## 5. Changelog
- **Rule:** User-facing/bugfixes ONLY under `[Unreleased]`. 1-line bullets.
- **BANNED:** Internal refactors/tests. No empty `[Unreleased]`.

## 6. Meta / Docs
- **Text Edits:** Token-optimize for human readers (balanced). Reserve aggressive compression ONLY for `AGENTS.md` and Skill files (`SKILL.md`).
- **Self-Editing:** Maintain aggressive token compression when updating `AGENTS.md` or Skill files.