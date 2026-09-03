---
name: i18n-language-management
description: Manage compile-time i18n injection (HTML/CSS/JS). NO runtime dictionaries.
---

# i18n

**Goal:** Enforce single-target compile-time i18n injection.

## WHEN TO USE
- Update locales (`resources/locales/*.json`)
- Resolve labels (Rust: `language.rs`, `builder.rs`, `module.rs`)
- Inject strings (Templates: `{{L_*}}`, DOM: `data-*`, CSS: `attr()`)
- Read UI messages (JS: DOM only, NO dicts)

## RULES
- **Single-Target:** 1 lang per doc. NO runtime switching.
- **NO JS Dicts:** BANNED: `window.d2f.lang.*`. Zero client dict overhead.
- **Rust Engine:** `include_str!` JSONs. `init(lang)`: thread-safe dict. `localize(key)`/`t(key)` -> string or `{{key}}`. BANNED: `translate()` (P-I18N-NAMING).
- **Injection:** Substitute `{{L_KEY}}` in `builder.rs` via `localize("key")`. Features invoke `localize` during AST render.
- **CSS/JS Data:** Render localized text as HTML `data-*` attributes. CSS uses `attr(data-*)`. JS uses `dataset.*`. Fallbacks required in JS.

## WORKFLOW
1. **Add:** Key to `de.json` & `en.json`.
2. **Inject:** `{{L_KEY}}` in `template.html` -> `.replace` in `builder.rs` OR `localize("key")` in `module.rs`.
3. **Consume:** Read from DOM attrs in JS/CSS.
4. **Test:** Unit tests in `language.rs`, `builder.rs`, `module.rs`.
