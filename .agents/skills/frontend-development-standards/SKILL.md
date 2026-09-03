---
name: frontend-development-standards
description: Enforces HTML, CSS, and generic frontend standards for Doc2Flow.
---
# Frontend Development Standards
**Goal:** Maintain strict frontend standards for HTML generation and CSS styling in Doc2Flow.

## USE WHEN
- Creating or modifying HTML generators in Rust.
- Writing or updating CSS files.
- Managing static assets and responsive/print layouts.

## HTML (Generic)
- `##` -> `.section.sh.sb` (collapsible)
- `[ ]` List -> `.check-item`
- Quotes (`>`,`>?`,`>!`) -> `.note` variants
- Local Img -> Base64; Remote -> `<img>`; Non-img asset -> `<a>.check-item.text-item`
- Vars replaced via frontmatter

## CSS
- `:root` vars, BEM classes, ZERO external deps (fonts)
- Print: Hide UI/buttons, expand collapsed (`display:block!important`), natural page breaks (no forced), exact print colors, no strikethrough
- File Structure: 1.Base > 2.Layout > 3.Components > 4.Print > 5.Responsive

## Workflow Context
- Ensure zero-alloc buffers and compile-time embeds (`include_str!`) are used in Rust when generating this HTML.
