# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added automatic SVG minification and metadata stripping for imported SVG images and custom logos, removing Inkscape/Sodipodi editor clutter and XML comments.

### Changed
- Optimized HTML output size by embedding the item comment SVG icon once as a symbol (`#icon-comment`) and referencing it via `<use>` elements.

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
