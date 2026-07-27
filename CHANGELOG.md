# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Custom logo feature (`-l` / `--logo <PATH>` CLI option and `logo: "<PATH>"` frontmatter tag) supporting SVG and raster images (PNG, JPG, WebP) with automatic Base64 embedding and graceful default fallback.

### Changed
- Decoupled filesystem and I/O operations into central `src/io.rs` module, establishing pure in-memory core processing.

## [0.9.0] - 2026-07-26 (Beta 1)

### Added
- Initial public release (Beta 1) of Doc2Flow (`d2f`).
