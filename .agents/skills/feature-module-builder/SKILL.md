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
   - `module.rs`: Feature struct `<Name>Feature` deriving `Default` (ZST), implementing `FeatureModule` / `DocumentElementRenderer`, `#[must_use] new()` delegating strictly to `Self::default()`, local feature constants (CSS classes, selectors, keys, defaults), and unit tests. Prohibit central constant dumpster files.
   - `<name>.js` *(if interactive)*: Vanilla JS logic attached to `window.d2f` namespace.
   - `<name>.css` *(if styled)*: Scoped CSS using BEM classes and `:root` variables.
2. **Implement `FeatureModule` Trait:**
   - `name(&self) -> &'static str`: Return unique feature ID (e.g., `"code"`).
   - `javascript(&self) -> &[&'static str]`: `&[include_str!("<name>.js")]` or `&[]`.
   - `css(&self) -> Option<&'static str>`: `Some(include_str!("<name>.css"))` or `None`.
3. **Register in Engine:**
   - In `src/features/mod.rs`: Add `#[path = "<name>/module.rs"] pub mod <name>;`, export `pub use <name>::<Name>Feature;`, define static instance `pub static <NAME>_FEATURE: <Name>Feature = <Name>Feature;`.
   - Register in static zero-allocation array `pub static ALL_FEATURE_MODULES: [&'static dyn FeatureModule; N]`.
   - Update tripwire tests `tests::test_all_feature_modules_count_and_registration` with updated count and feature name.
4. **Enforce Directives (`AGENTS.md`):**
   - **Rust:** Zero `unsafe`, zero-alloc hot path, canonical doc headers, Stdlib+`Doc2FlowError` (`src/utils/error.rs`), strongly typed error enums (NO `Result<T, String>`), local constants in `module.rs`. Derive/implement `Default` for parameter-less structs; `#[must_use] new()` MUST strictly delegate to `Self::default()`. Static registries (`[&'static dyn Trait; N]`) for zero heap allocations. Rely on standard traits (`Display` -> `.to_string()`), NO duplicate custom methods. `bitflags`/array state for multi-boolean flags. Inline docs for intentional domain quirks. Case-insensitive truthy matrix (`true`, `yes`, `y`, `1`) for boolean flags. DRY template contexts (`build_template_vars`), assembly pipeline parity (conditional components identical across pathways), NO `#[inline]` on heap allocs/IO. Tripwire tests and resilient semantic token assertions.
   - **JS:** Vanilla JS, `window.d2f` namespace (`window.d2f.<module>`). Localized strings MUST use `window.d2f.utils.translate(key)`. NO build step. BANNED: `export`/`import`.
   - **CSS:** BEM classes, `:root` vars, ZERO external fonts/assets, print styles (`display:block!important`, natural page breaks, exact colors).
   - **Spec:** Sync `SPECIFICATION.md` tree and module description.
5. **Verify:**
   - `./MAKE.sh --tests` (cargo tests).
   - `./MAKE.sh --examples` (validate generated HTML showcases).

> [!NOTE]
> **[BRANCH EXPERIMENT: feature/modular-building - REVERT ON MERGE]**
> Do NOT create/modify `src/features/` slices on this branch. Experimental modules reside strictly under `src/exp/`. Production slices remain frozen. Duplicate logic into `src/exp/` when needed.
