# Doc2Flow (`d2f`)

[![Version](https://img.shields.io/badge/version-0.9.4-blue.svg)](Cargo.toml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL_v3-blue.svg)](LICENSE)

Doc2Flow is a high-performance Rust CLI tool that compiles Markdown documents into standalone, zero-dependency interactive HTML workflows and checklists. It produces self-contained files with persistent client-side state for offline execution, standard operating procedures, and technical documentation.

## Features

- **Zero-Dependency Output**: Inlines all CSS, JavaScript, and Base64-encoded assets into a single portable HTML file.
- **Client-Side State Persistence**: Retains checkbox progress and variable inputs across page reloads via SHA-256 scoped `localStorage`.
- **Interactive Workflows**: Provides collapsible sections with dynamic progress badges, interactive checklists (`- [ ]`), and 1-click code copying.
- **Dynamic Variable Interpolation**: Replaces `{{VAR_NAME}}` placeholders across code blocks dynamically using interactive table inputs.
- **Asset Processing & Optimization**: Embeds local images as Base64 data URIs with automatic WebP compression for files exceeding 250 KB.
- **Print & Export Ready**: Auto-expands collapsed sections and hides interactive controls under `@media print` for clean PDF export.

## Quick Start

Build the release executable using Cargo:

```bash
cargo build --release
```

The binary will be located at `target/release/d2f` (`target/release/d2f.exe` on Windows).

## Usage

```bash
# Convert Markdown to self-contained HTML
d2f guide.md

# Convert with custom output, logo, and image compression
d2f guide.md -o output.html -l logo.svg -s
```

## Configuration

### CLI Flags

| Flag | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `<INPUT>` | — | Path to source Markdown file | Required (unless `-i`) |
| `--output` | `-o` | Target HTML output filepath | `<INPUT_STEM>.html` |
| `--logo` | `-l` | Custom header logo path (SVG, PNG, JPG, WebP) | Embedded default SVG |
| `--init` | `-i` | Generate starter Markdown template | `template.md` |
| `--auto-scale`| `-s` | Compress local images > 250 KB to WebP | `false` |
| `--help` | `-h` | Print CLI help information | — |
| `--version` | `-V` | Print version information | — |

### Frontmatter Options

Configure document metadata via YAML frontmatter at the top of the Markdown source:

| Key | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `title` | `string` | Primary document title | `""` |
| `subtitle` | `string` | Secondary subtitle text | `""` |
| `date` | `string` | Protocol or revision date | `""` |
| `version` | `string` | Document revision string | `""` |
| `language` | `string` | UI locale (`en`, `de`) | `"en"` |
| `logo` | `string` | Path or URI to custom header logo | `""` |
| `header` | `string` | Header layout mode (`"none"`, `"flex"`) | `"none"` |
| `numbered_sections` | `bool` | Auto-number section headings (`1.`, `1.1`) | `false` |

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).