---
name: javascript-module-architect
description: Generates, refactors, and validates isolated Vanilla JavaScript modules adhering to Doc2Flow namespace and Airbnb coding standards.
---
# JS Architect
**Goal:** Generate/refactor Vanilla JS into performant, isolated Doc2Flow (`d2f`) modules.

## USE WHEN
- New client JS (`src/features/<name>/<name>.js`, `src/exp/`).
- Refactor/modernize JS.
- Enforce strict IIFE, zero global scope, Airbnb rules.

## 1. ARCH & NAMESPACE
- **Encapsulate:** 100% IIFE: `(() => { ... })();`.
- **0 Pollution:** NO global var/fn.
- **Public API:** `window.d2f.[name]`. Guard: `window.d2f = window.d2f || {};`.
- **Interop:** Call via `window.d2f.[other].[fn]()`.
- **0 Deps:** Vanilla JS. NO jQuery/bundler/export/import.
- **Events:** `init()` on `DOMContentLoaded`/`document.readyState`.

## 2. RULES
- **Vars:** `const` (default), `let` (reassign). NO `var`.
- **Eq:** `===`, `!==`. NO `==`.
- **Fn:** Arrow `()=>{}`, NO param mutate. Implicit return.
- **Obj/Arr:** Spread `...`, destructure `{a}`, shorthand `{a,b(){}}`.
- **Iter:** `.map()`, `.filter()`, `.reduce()`, `.forEach()`. NO `for...in/of`.
- **Str:** `'...'` (static), `${...}` (interp).

## 3. TEMPLATE
```javascript
(() => {
  const state = {};
  const priv = () => {};
  window.d2f = window.d2f || {};
  window.d2f.core = { pub: () => {} };
})();
```

## 4. WORKFLOW
1. **Design:** IIFE scope/helpers.
2. **Impl:** Immutable, arrows, functional iterators.
3. **Export:** Guard `window.d2f`, attach API.
4. **Audit:** 0 leaks, 0 `var`/`==`, pure Vanilla.
