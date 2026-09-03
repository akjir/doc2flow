---
name: feature-module-builder
description: Scaffolds, implements, and registers vertical slice feature modules adhering to Doc2Flow architecture.
---
# Feature Builder
**Goal:** Scaffold/impl/reg vertical slices (`src/features/<name>/`): zero-alloc Rust, BEM CSS, vanilla JS, tests.

## USE WHEN
- New feature (search, tabs).
- Add `module.rs`,`<name>.js`,`<name>.css` in `src/features/<name>/`.
- Reg in `src/features/mod.rs` & `SPECIFICATION.md`.

## WORKFLOW
1. **Scaffold:** `src/features/<name>/`:
   - `module.rs`: `struct <Name>Feature` (ZST), impl `Default`, `FeatureModule`, `DocumentElementRenderer`. `#[must_use] new()` -> `Self::default()`. Local consts/CSS classes/tests. NO global const dump.
   - `<name>.js` (opt): Vanilla JS on `window.d2f`.
   - `<name>.css` (opt): BEM + `:root` vars.
2. **Impl `FeatureModule`:**
   - `name()->&'static str`: (e.g. `"code"`).
   - `javascript()->&[&'static str]`: `&[include_str!("<name>.js")]` or `&[]`.
   - `css()->Option<&'static str>`: `Some(include_str!("<name>.css"))` or `None`.
3. **Register:**
   - `src/features/mod.rs`: `#[path="<name>/module.rs"] pub mod <name>; pub use <name>::<Name>Feature; pub static <NAME>_FEATURE: <Name>Feature = <Name>Feature;`.
   - Add to `pub static ALL_FEATURE_MODULES: [&'static dyn FeatureModule; N]`.
   - Update `test_all_feature_modules_count_and_registration`.
4. **Enforce `AGENTS.md`:**
   - **Rust:** 0 `unsafe`/alloc hot paths, Stdlib+`Doc2FlowError` (NO `Result<T,String>`). Local consts. Default/new delegation. Static arrays `[_; N]`. No custom dup traits. `bitflags` arrays. Inline domain docs. Truthy bools (`true,yes,y,1`). DRY templates. O(1) case-insensitive `eq_ignore_ascii_case`. Borrowed map lookups. Loop split >40 lines. Tripwire/path-edge tests. Resilient semantic asserts.
   - **JS:** Vanilla, `window.d2f.<module>`. NO build/export/import.
   - **CSS:** BEM, `:root`, 0 deps, print block/colors.
   - **Spec:** Sync `SPECIFICATION.md`.
5. **Verify:** `./MAKE.sh --tests` & `--examples`.
