# Doc2Flow Agent Directives

## 1. Arch & Stack
- **Spec:** `SPECIFICATION.md` (sync structure on central file changes)
- **CLI:** `std::env::args()`
- **MD:** `pulldown-cmark`+GFM
- **Assets:** Custom Base64/MIME (`src/utils.rs`), WebP/compress (`src/image.rs`)
- **i18n:** `HashMap` via embedded JSON (`build.rs`)
- **UI/HTML:** Zero-alloc buffers (`src/components.rs`, `src/template.rs`), compile-time embeds (`include_str!`)
- **Flow:** CLI > MD > Img > UI

## 2. Rust
- **Core:** Idiomatic, newtypes, 1-path exports, NO `unsafe`
- **Errors:** Stdlib+`Doc2FlowError` (NO `anyhow`/`eyre`). `Result`=expected. `panic!`=bugs/stop (detailed msgs). NO `catch_unwind`.
- **Docs:** 15-word max start, canonical headers (Examples/Errors/Panics), NO meta/journals
- **Perf:** Min-alloc (borrow>owned), `with_capacity`, O(N) 1-pass, zero-copy (`split_once`,`strip_prefix`), `Cow`
- **Flow:** `match`/tables > `if-else`. Iterators > loops. `write_str`(static)/`write!`(dynamic) > `format!`
- **Build:** `lto=true`, `opt=z|s`, `codegen-units=1`, strip. Favor stdlib over deps.

## 3. Ops & Tests
- **Comm:** English ONLY. 1-line concise AI responses.
- **OS:** Linux dev, Win64 target. `std::path::Path/Buf` ONLY.
- **Git:** Commit ONLY if requested AND tests pass (or user overrides).
- **Test:** Priority 1. Negative/edge cases. Regen `showcase_*.html` on UI changes.

## 4. Frontend (HTML/TS/CSS)
- **HTML (Generic):**
  - `##` -> `.section.sh.sb` (collapsible)
  - `[ ]` List -> `.check-item`
  - Quotes (`>`,`>?`,`>!`) -> `.note` variants
  - Local Img -> Base64; Remote -> `<img>`; Non-img asset -> `<a>.check-item.text-item`
  - Vars replaced via frontmatter
- **TS:**
  - Config: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `verbatimModuleSyntax`
  - Scope: NO function `export`/`import`; decouple via `window.d2f` namespace
  - State: Discriminated unions, `readonly`
  - Format: `interface` (expandable), `type` (unions), `satisfies` operator
  - BANNED: `any`, `as`, `!`, `enum`, `{}`/`Object`
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