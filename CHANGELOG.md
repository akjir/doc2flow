# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Updated starter template (`templates/template.md`) with GFM table support, variables table documentation, and sample code variable usage.

### Fixed
- Fixed print styles to preserve table background colors and alternating row colors.
- Fixed header margin in print mode to match spacing between sections.
- Fixed table header background clipping and top corner rounding in print mode.

## [0.9.4] - 2026-08-02

### Added
- Added section table feature support with dedicated table.ts and table.css bundles.
- Added dynamic code variable substitution from pre-section `[Variables]` tables with editable values, state persistence, code usage filtering, and print substitution.
- Added dynamic `<meta name="features">` tag generation in base HTML reflecting enabled document features.

### Changed
- Live-update code block variable substitution on initial page load and on input change in web preview.

### Fixed
- Fixed base template CSS/JS variable substitution tags, script initialization order, and namespace bindings for window.d2f.
- Fixed state persistence and document export by registering storage handlers on script load and synchronizing HTML attributes (`value`, `checked`) for form fields and checkboxes.
- Fixed clicking the item comment icon or delete button toggling task list check items.
- Hide item comment button when an item comment box is displayed on a document item.

## [0.9.3] - 2026-08-01

### Fixed
- Decoupled generic document items (`.doc-item`) from task checklist items (`.check-item`) and moved core item, list, text, and comment box styles to `core.css`.
- Fixed search filter state and highlights not being reset when triggering resetAll.
- Fixed ReferenceError when clicking PDF export, save state, reset, or copy code buttons by binding action handlers to the global window object.

### Changed
- Render progress bar display and bottom finish box conditionally only when tasks feature is active.
- Decoupled code block styles (`code.css`) and script bundle (`script-code.js`) into conditional feature modules included only when code blocks exist.
- Expanded document reset functionality to unfold all collapsed sections, clear text fields and comments, and update i18n confirmation text.

### Added
- Added modular TypeScript architecture and build pipeline for web frontend scripts (#20).
- Added optional automatic section numbering (`number_sections: true`) for H1 and H2 headings.
- Added live-in-browser search bar and quick-filter toolbar with keyboard shortcut (Ctrl+K), toggle button next to progress bar, clear button (✖), term highlighting, and printable CSS compatibility.
- Section collapse/expand state is now persisted in `localStorage` and included in the exported HTML, so the layout is preserved across page reloads and when sharing the saved file.

## [0.9.2] - 2026-07-28

### Added
- Added dynamic SemVer 2.0.0 version and build metadata generation in build.rs (`v<VERSION>+<COUNT>.<HASH>[.dev]`) propagated to CLI (`--version`), HTML meta generator tags, document headers, and template files.
- Added automatic SVG minification and metadata stripping for imported SVG images and custom logos, removing Inkscape/Sodipodi editor clutter and XML comments.
- Enhanced accessibility (A11y) and keyboard navigation for collapsible section headers (role="button", tabindex, Enter/Space toggling), progress bar ARIA attributes, and form field screen reader labels.

### Changed
- Optimized HTML output size by embedding the item comment SVG icon once as a symbol (`#icon-comment`) and referencing it via `<use>` elements.
- Refactored HTML structure to Semantic HTML5, replacing generic container `<div>` elements with `<header>`, `<main>`, `<section>`, `<h2>`, and `<h3>` landmark and heading tags for enhanced accessibility.

### Fixed
- Fixed Markdown task list conversion to render interactive checkboxes (`.check-item`) when empty lines are present between task items (loose list format).

## [0.9.1] - 2026-07-28

### Added
- Custom logo feature (`-l` / `--logo <PATH>` CLI option and `logo: "<PATH>"` frontmatter tag) supporting SVG and raster images (PNG, JPG, WebP) with automatic Base64 embedding and graceful default fallback.
- Embed Doc2Flow version, license, and repository URL as metadata in generated HTML header comments, head meta tags, and CLI initialization templates.

### Changed
- Updated starter template (`template.md`) to showcase Level 1 headings (#), bold and strikethrough text formatting, ordered lists, and nested task lists.
- Unified H1 and H2 section collapsing behavior and suppressed expand/collapse toggle icons for empty headings.
- Replaced single-line comment inputs with auto-expanding multiline textareas for improved text wrapping, persistence, and print/PDF readability.
- Added a subtle 1px border (`var(--border-color)`) to embedded document body images (`.doc-body img`) for improved visual distinction against light backgrounds.

## [0.9.0] - 2026-07-26 (Beta 1)

### Added
- Initial public release (Beta 1) of Doc2Flow (`d2f`).
