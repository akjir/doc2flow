---
name: javascript-module-architect
description: Generates, refactors, and validates isolated Vanilla JavaScript modules adhering to Doc2Flow namespace and Airbnb coding standards.
---

# JavaScript Module Architect

**Goal:** Generate/Refactor Vanilla JavaScript into highly performant, isolated modules for Doc2Flow (`d2f`).

## USE WHEN
- Creating or scaffolding new client-side JavaScript modules (`src/features/<name>/<name>.js`, `src/exp/`).
- Refactoring, modernizing, or debugging existing JavaScript files.
- Ensuring strict IIFE encapsulation, zero global pollution, and adherence to Airbnb golden rules.

## 1. ARCHITECTURE & NAMESPACE (STRICT)
- **Encapsulation:** Every file MUST be encapsulated in an IIFE: `(() => { ... })();`.
- **Zero Pollution:** NEVER declare variables or functions in the global scope.
- **Public API Export:** Assign public methods strictly to `window.d2f.[module_name]`. Ensure namespace guard (`window.d2f = window.d2f || {};`).
- **Module Interop:** Access external functions ONLY via `window.d2f.[other_module].[method]()`.
- **i18n / Localization:** Language-specific text MUST use `window.d2f.utils.translate(key)` (returns dictionary value or `{{KEY}}`; zero fallbacks).
- **Zero Dependencies:** 100% Vanilla JS. No external libraries (e.g., jQuery). BANNED: `export`/`import` (no bundler).
- **DOM Events:** Attach listeners inside an initialization function (`init()`) called on `DOMContentLoaded` or guarded by `document.readyState`.

## 2. GOLDEN RULES
- **Variables:** BANNED: `var`. USE: `const` (default) and `let` (only for reassignment).
- **Equality:** BANNED: `==`, `!=`. USE: `===`, `!==`.
- **Functions:** USE arrow functions `() => {}` for callbacks/anonymous functions. DO NOT mutate parameters. USE implicit returns for single expressions.
- **Objects & Arrays:** USE spread operator `...` for shallow cloning. USE object destructuring `const {a} = obj;` and property shorthands `{a, b() {}}`.
- **Iteration:** BANNED: `for...in`, `for...of`. USE higher-order functions: `.map()`, `.filter()`, `.reduce()`, `.forEach()`.
- **Strings:** USE single quotes `'...'` for static text. USE template literals `${...}` for programmatic interpolation.

## 3. REQUIRED BOILERPLATE TEMPLATE
```javascript
(() => {
  // 1. Private Scope
  const internalState = {};
  const privateHelper = () => {};

  // 2. Cross-Module Access & i18n
  // const label = window.d2f.utils?.translate('key');
  // window.d2f.otherModule.doSomething();

  // 3. Public API Export
  window.d2f = window.d2f || {};
  window.d2f.core = {
    publicMethod: () => {},
  };
})();
```

## 4. EXECUTION WORKFLOW
1. **Design Scope:** Define module scope and private helpers within the IIFE.
2. **Implement Logic:** Use immutable patterns (`const`, `...`), arrow callbacks, and functional iterators (`map`/`filter`/`forEach`).
3. **Export API:** Safeguard `window.d2f = window.d2f || {};` and attach public interface to `window.d2f.<module_name>`.
4. **Audit Compliance:** Validate zero global leaks, zero `var`/loose equality, single quotes for literals, and pure Vanilla JS execution.
