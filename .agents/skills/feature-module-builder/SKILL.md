---
name: feature-module-builder
description: Scaffolds, implements, and registers vertical slice feature modules adhering to Doc2Flow architecture.
---

# Feature Module Builder

**Goal:** Scaffold, implement, and register vertical slice feature modules (`src/features/<name>/`) with zero-alloc Rust traits, isolated BEM CSS, strict TS client logic, and tests.

## USE WHEN
- Creating/scaffolding a new vertical slice feature (e.g., search, zoom, tabs, outline).
- Adding `module.rs`, `<name>.ts`, and `<name>.css` under `src/features/<name>/`.
- Registering features in `src/features/mod.rs` (`get_all_features`) and syncing `SPECIFICATION.md`.

## EXECUTION WORKFLOW
Follow these 5 steps sequentially:

1. **Scaffold Slice:** Create `src/features/<name>/` directory:
   - `module.rs`: Feature struct `<Name>Feature` implementing `Feature` trait (`name`, `is_enabled`, `javascript`, `css`), `new()`, local feature constants (CSS classes, selectors, keys, defaults), and unit tests. Prohibit central constant dumpster files.
   - `<name>.ts` *(if interactive)*: TypeScript logic attached to `window.d2f` namespace.
   - `<name>.css` *(if styled)*: Scoped CSS using BEM classes and `:root` variables.
2. **Implement `Feature` Trait:**
   - `name(&self) -> &'static str`: Return unique feature ID (e.g., `"code"`).
   - `is_enabled(&self, ctx: &DocumentContext) -> bool`: Fast 1-pass detection on `ctx.frontmatter` or `ctx.raw_markdown`.
   - `javascript(&self) -> Option<&'static str>`: `Some(include_str!("<name>.ts"))` or `None`.
   - `css(&self) -> Option<&'static str>`: `Some(include_str!("<name>.css"))` or `None`.
3. **Register in Engine:**
   - In `src/features/mod.rs`: Add `#[path = "<name>/module.rs"] pub mod <name>;`, export `pub use <name>::<Name>Feature;`.
   - Add `Box::new(<Name>Feature::new())` to `get_all_features()`.
   - Update `tests::test_feature_registry_*` with updated count and feature name.
4. **Enforce Directives (`AGENTS.md`):**
   - **Rust:** Zero `unsafe`, zero-alloc hot path, `#[inline]`, canonical doc headers (Examples/Errors/Panics), Stdlib+`Doc2FlowError` (no `anyhow`/`eyre`), local feature constants in `module.rs` (NO central dumpster).
   - **TS:** Strict config, `window.d2f` namespace, `readonly`, discriminated unions, `satisfies`. BANNED: `export`/`import`, `any`, `as`, `!`, `enum`, `{}`/`Object`.
   - **CSS:** BEM classes, `:root` vars, ZERO external fonts/assets, print styles (`display:block!important`, natural page breaks, exact colors).
   - **Spec:** Sync `SPECIFICATION.md` tree and module description.
5. **Verify:**
   - `./MAKE.sh --tests` (cargo tests + TypeScript compilation).
   - `./MAKE.sh --examples` (validate generated HTML showcases).
