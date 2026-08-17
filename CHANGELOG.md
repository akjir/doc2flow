# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added interactive item selection for text, bullet, and ordered list items with green highlight and persistent localStorage state.
- Added modular shoutout feature slice for informational, tip, important, warning, and caution callout box rendering.
- Added collapsible section support in the core pipeline with accessible headers, keyboard navigation, fold indicator toggling, and state persistence.
- Added permissive boolean parsing support for document frontmatter configuration parameters.
- Added modular table feature slice for responsive table rendering, column alignments, and row hover states.
- Added modular image feature slice for image rendering, lightbox modal viewing, and broken image fallback handling.
- Added inline markdown link parsing and rendering (`[text](url)`) across all text, list, and task elements.
- Added modular task feature slice for task checklist rendering and state presentation.
- Added modular ordered list feature slice for numbered list element rendering.
- Added modular bullet list feature slice for unordered list element rendering and nested indentation.
- Added inline text formatting (bold, italic, strikethrough, inline code) for text paragraph elements in the core pipeline.
- Added support for markdown horizontal rule dividers (`---`, `----`, etc.) in the core pipeline.
- Added modular code feature slice for syntax and fenced code block rendering.
- Added support for optional header layout (`header: "flex"`) rendering a structured banner card with logo, title, and subtitle before sections.
- Added automatic SVG fallback placeholder display for broken or unreachable external image loads.
- Added validation enforcing mandatory level-1 headings and restricting pre-heading content to whitelisted block directives (`:::variables`).
- Added automatic variable table generation from `{{VAR}}` code block placeholders during document parsing.
- Updated starter template (`templates/template.md`) with GFM table support, variables table documentation, and sample code variable usage.

### Changed
- Refactored `Document.header` into `DocumentHeader` struct with strict single-table validation for `:::variables` block directives and automatic code feature activation.
- Updated image fallback placeholder SVG to remove the outer dashed border and optimize vector size.
- Adjusted typography sizing to reduce inline code font size and increase code block font size.
- Unified document item hover and interaction styling under `item-` prefix in core stylesheet.
- Deduplicated shared code styling tokens across core and code stylesheets.
- Aligned Markdown section hierarchy so H1 and H2 act as top-level sections that accept only H3 child sections.

### Removed
- Removed Table of Contents (TOC) feature and table_of_contents frontmatter option.

### Fixed
- Fixed image lightbox modal failing to open by constructing elements via DOM APIs and skipping script tags during HTML image post-processing.
- Fixed inline code box vertical alignment and asymmetric padding to ensure proper centering with surrounding text.
- Fixed potential panic during diagnostic error caret rendering for long source lines and zero-column offsets.
- Fixed print styles to preserve table background colors and alternating row colors.
- Fixed header margin in print mode to match spacing between sections.
- Fixed table header background clipping and top corner rounding in print mode.
- Fixed excessive state persistence calls during search filtering, task item background clicks, and reset handling.
- Fixed code block variable placeholders not restoring to initial template values upon resetting document state.

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
