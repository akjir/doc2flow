---
name: feature-module-builder
description: Scaffolds, implements, and registers vertical slice feature modules adhering to Doc2Flow architecture.
---

# Feature Module Builder

**Goal:** Scaffold, implement, and register vertical slice feature modules (`src/features/<name>/`) with zero-alloc Rust traits, isolated BEM CSS, vanilla JS client logic, and tests.

## USE WHEN
- Creating/scaffolding a new vertical slice feature (e.g., search, zoom, tabs, outline).
- Adding `module.rs`, `<name>.js`, and `<name>.css` under `src/features/<name>/`.
- Registering features in `src/features/mod.rs` (`get_all_features`) and syncing `SPECIFICATION.md`.

## EXECUTION WORKFLOW
Follow these 5 steps sequentially:

1. **Scaffold Slice:** Create `src/features/<name>/` directory:
   - `module.rs`: Feature struct `<Name>Feature` implementing `Feature` trait (`name`, `is_enabled`, `javascript`, `css`), `new()`, local feature constants (CSS classes, selectors, keys, defaults), and unit tests. Prohibit central constant dumpster files.
   - `<name>.js` *(if interactive)*: Vanilla JS logic attached to `window.d2f` namespace.
   - `<name>.css` *(if styled)*: Scoped CSS using BEM classes and `:root` variables.
2. **Implement `Feature` Trait:**
   - `name(&self) -> &'static str`: Return unique feature ID (e.g., `"code"`).
   - `is_enabled(&self, ctx: &DocumentContext) -> bool`: Fast 1-pass detection on `ctx.frontmatter` or `ctx.raw_markdown`.
   - `javascript(&self) -> Option<&'static str>`: `Some(include_str!("<name>.js"))` or `None`.
   - `css(&self) -> Option<&'static str>`: `Some(include_str!("<name>.css"))` or `None`.
3. **Register in Engine:**
   - In `src/features/mod.rs`: Add `#[path = "<name>/module.rs"] pub mod <name>;`, export `pub use <name>::<Name>Feature;`.
   - Add `Box::new(<Name>Feature::new())` to `get_all_features()`.
   - Update `DocumentFeatures::is_feature_active` and `DocumentFeatures::to_features_string` if mapped to AST parser flags.
   - Update `tests::test_feature_registry_*` with updated count and feature name.
4. **Enforce Directives (`AGENTS.md`):**
   - **Rust:** Zero `unsafe`, zero-alloc hot path, canonical doc headers, Stdlib+`Doc2FlowError` (`src/utils/error.rs`), local constants in `module.rs`. DRY template contexts (`build_template_vars`), assembly pipeline parity (conditional components identical across pathways), NO `#[inline]` on heap allocs/IO.
   - **JS:** Vanilla JS, `window.d2f` namespace (`window.d2f.<module>`). NO build step. BANNED: `export`/`import`.
   - **CSS:** BEM classes, `:root` vars, ZERO external fonts/assets, print styles (`display:block!important`, natural page breaks, exact colors).
   - **Spec:** Sync `SPECIFICATION.md` tree and module description.
5. **Verify:**
   - `./MAKE.sh --tests` (cargo tests).
   - `./MAKE.sh --examples` (validate generated HTML showcases).

> [!NOTE]
> **[BRANCH EXPERIMENT: feature/modular-building - REVERT ON MERGE]**
> Do NOT create/modify `src/features/` slices on this branch. Experimental modules reside strictly under `src/exp/`. Production slices remain frozen. Duplicate logic into `src/exp/` when needed.
