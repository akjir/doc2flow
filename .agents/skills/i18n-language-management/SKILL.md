---
name: i18n-language-management
description: Manages Doc2Flow internationalization and compile-time direct language injection into HTML, CSS, and JS without runtime dictionary objects.
---

# I18n & Language Management

**Goal:** Enforce Doc2Flow single-target compile-time i18n injection architecture across Rust engine, HTML templates, CSS attributes, and Vanilla JS client modules.

## USE WHEN
- Adding/updating localized UI terms in `resources/locales/` (`de.json`, `en.json`).
- Resolving localized labels/messages in Rust engine (`src/core/language.rs`, `src/core/builder.rs`, `src/features/*/module.rs`).
- Injecting localized strings into HTML templates (`{{L_*}}`), DOM data attributes (`data-label`, `data-confirm`), or CSS pseudo-elements (`content: attr(data-label)`).
- Interacting with localized UI messages in client-side JavaScript without runtime dictionary objects.

## ARCHITECTURE DIRECTIVES

1. **Single-Target Document Compilation:**
   - Every compiled Doc2Flow document targets exactly ONE language (`document.parameters.language`, e.g. `"de"`, `"en"`).
   - Zero runtime language switching in browser.
   - BANNED: Runtime dictionary objects (`window.d2f.lang.dictionary`, `window.d2f.i18n`). Zero client dictionary overhead.

2. **Rust Compile-Time Localization Engine (`src/core/language.rs`):**
   - Embedded locale JSONs: `DE_JSON`, `EN_JSON` via `include_str!`.
   - `init(lang)`: Initializes thread-safe active dictionary for document target language. Case-insensitive (`"de"`, `"en"`), fallback to empty map.
   - `localize(key)` / `t(key)`: Static dictionary lookup. Returns translated string or `"{{<key>}}"` fallback.
   - Naming: Standardize on `localize()` (alias `t()`). BANNED: `translate()` (P-I18N-NAMING).

3. **Direct HTML & Template Injection:**
   - Template placeholders (`{{L_<KEY>}}`, e.g., `{{L_EXPORT_PDF}}`, `{{L_SAVE_STATE}}`, `{{L_RESET_ALL}}`, `{{L_CONFIRM_RESET}}`) are substituted during `builder::build()` using `localize("<key>")`.
   - Element rendering in feature modules (`DocumentElementRenderer`) invokes `localize("<key>")` directly during AST traversal.

4. **Direct CSS / Data-Attribute Injection:**
   - Dynamic localized badges/labels (e.g., shoutout `data-label`, callouts) are rendered as HTML attributes: `data-label="Hinweis"`.
   - CSS consumes attributes via `content: attr(data-label);`.

5. **Client JS Interaction:**
   - Client scripts access localized text directly from DOM attributes: `btn.dataset.confirm`, `el.getAttribute('data-confirm')`, `el.title`, or pre-rendered textContent.
   - Provide safe fallback defaults in JS when attributes are absent.

## WORKFLOW

1. **Add Key:** Add dictionary pair to both `resources/locales/de.json` and `resources/locales/en.json`.
2. **Inject in Rust:**
   - In template: Add `{{L_<KEY>}}` placeholder in `resources/templates/template.html`.
   - In builder: Add `.replace("{{L_<KEY>}}", &crate::core::language::localize("<key>"))` in `src/core/builder.rs`.
   - Or in feature: Call `crate::core::language::localize("<key>")` in `module.rs` `render_element`.
3. **Consume in DOM/JS:** Read attribute in JS (`btn.dataset.confirm`) or render directly into HTML.
4. **Test:** Add unit tests in `src/core/language.rs`, `src/core/builder.rs`, and feature `module.rs`.
