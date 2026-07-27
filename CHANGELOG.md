# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Custom logo support via CLI argument (`-l`, `--logo <PATH>`) and Markdown YAML frontmatter (`logo: "<PATH>"`).
- Automatic asset processing for logos: clean inline SVG injection for `.svg` files and Base64 Data URI conversion for raster images (PNG, JPG, WebP, GIF, etc.).
- Graceful error handling with `stderr` warning diagnostics and default SVG logo fallback if a custom logo path is missing or unreadable.
- CLI options precedence over Frontmatter settings for custom logo resolution.

## [0.9.0] - 2026-07-26 (Beta 1)

### Added
- Initial public release (Beta 1) of Doc2Flow (`d2f`).
