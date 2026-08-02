# Doc2Flow (`d2f`)

**Doc2Flow (`d2f`)** is a fast, lightweight command-line tool built in Rust that converts Markdown documents into standalone, interactive HTML guides, manuals, protocols, and checklists.

The generated HTML files are completely self-contained—embedding all CSS styling, JavaScript interactivity, embedded icons, and Base64-encoded local images—making them ideal for offline distribution, customer handovers, and field service workflows without external web servers or assets.

---

## Key Features

- 🚀 **Single Binary Executable:** Distributed as a lightweight, zero-dependency executable (`d2f` / `d2f.exe`).
- 📦 **Zero-External Dependencies HTML:** All styles, client scripts, icons, and local images are embedded directly into a single `.html` file.
- ✅ **Interactive Task Lists & State Persistence:**
  - Dynamic task list checkboxes (`- [ ]` / `- [x]`).
  - Section completion badges and overall document progress tracking.
  - Automatic `localStorage` persistence scoped deterministically via SHA-256 (`d2f_id`).
  - Global state reset option with a modal confirmation.
- 🔀 **Dynamic Variable Substitution (`[Variables]`):**
  - Extract parameters from Markdown tables annotated with `[Variables]`.
  - Replace `{{VARIABLE_NAME}}` placeholders inside code blocks dynamically on copy or print.
  - Interactive table UI rendered with persistent input fields.
- 🔍 **Search Toolbar & Table of Contents (TOC):**
  - Built-in live search bar for filtering text and sections.
  - Optional Table of Contents (`toc: true`) with dynamic scroll tracking.
- 🖼️ **Image Lightbox & Auto-Scaling:**
  - Converts local images to embedded Base64 data URIs.
  - Automatic WebP compression (`-s` / `--auto-scale`) for local images exceeding 250 KB.
  - Interactive image modal/lightbox for full-resolution image viewing.
- 🌐 **Multi-Language / i18n Support:**
  - Built-in English (`en`) and German (`de`) static UI translations.
  - Selectable per document via YAML frontmatter `language: "de"` setting.
- 🎨 **Custom Header Logo Support:**
  - Embed custom header logos (SVG, PNG, JPG, WebP) via CLI option (`-l` / `--logo`) or YAML frontmatter (`logo: "..."`).
- 🛠️ **Starter Template Generator:**
  - Instantly create a starter Markdown guide using `--init` / `-i`.
- 📣 **Rich Callout & Alert Panels:**
  - Color-coded alert boxes using simple blockquote prefix notation (`Note`, `Tip`, `Important`, `Warning`, `Caution`).
- 💻 **Enhanced Code Blocks:**
  - Syntax-aware containers with language tags, 1-click **Copy Code** functionality, and dynamic variable substitution.
- 📝 **Protocol Sign-Off & Signature Footer:**
  - Built-in persistent fields for agent name, completion date, signature lines, and protocol approval status.
- 🖨️ **Print & PDF Optimized:**
  - Dedicated `@media print` stylesheet that auto-expands collapsed sections and hides interactive controls for clean printouts and PDF exports.

---

## Usage

### Command Line Syntax

```bash
# Standard conversion (generates input.html)
d2f input.md

# Specify custom output path
d2f input.md -o /path/to/output.html

# Specify custom header logo image
d2f input.md -l logo.png

# Enable automatic image scaling to WebP for local images > 250 KB
d2f input.md -s

# Generate starter Markdown template (defaults to template.md)
d2f --init
d2f -i custom_template.md

# View CLI help & version
d2f --help
d2f --version
```

### CLI Parameters & Arguments

| Argument / Flag | Short | Description | Required | Default |
| --- | --- | --- | --- | --- |
| `INPUT` | — | Path to source Markdown file | Conditional (unless `--init` used) | — |
| `OUTPUT` | `-o`, `--output` | Target path for generated HTML file | No | `<INPUT_NAME>.html` |
| `LOGO` | `-l`, `--logo` | Path to custom logo image (SVG, PNG, JPG, WebP) | No | Default embedded SVG logo |
| `INIT` | `-i`, `--init` | Generates starter template Markdown file | No | `template.md` |
| `AUTO_SCALE` | `-s`, `--auto-scale` | Auto-resizes local images > 250 KB to WebP | No | `false` |

---

## Markdown Syntax & Authoring Guide

Doc2Flow uses CommonMark with GitHub Flavored Markdown (GFM) extensions alongside custom metadata and annotation syntax:

### 1. YAML Frontmatter (Metadata & Localization)

Place YAML metadata at the very top of your `.md` file to populate header metadata and configure document options:

```yaml
---
title: "Server Deployment Guide"
subtitle: "Standard Operating Procedure"
company: "Acme Corporation"  # Required field
contact: "Jane Doe"
agent: "John Smith"
date: "2026-07-25"
version: "1.0.0"
language: "de"
logo: "images/company_logo.svg"
number_sections: true
toc: false
---
```

> [!IMPORTANT]
> The `company` field is **required**. If omitted, `d2f` will raise a compiler diagnostic error.

### 2. Collapsible Sections & Headings

- **`# Section Title` (Level 1 Heading):** Creates a primary, non-collapsible section header container.
- **`## Section Title` (Level 2 Heading):** Creates a collapsible section container with a live completion badge (`0/3 completed`) and toggle indicator.
- **`### Subheading` (Level 3–6 Headings):** Renders styled subheadings inside section bodies.

```markdown
# 1. Overview

## 1.1 Initial Inspection

### Hardware Verification
- [ ] Inspect hardware for physical damage
- [ ] Verify power supply connections
```

### 3. Checklists & List Items

- **`- [ ]` / `- [x]`:** Interactive task item tracked by progress counters and saved in `localStorage`.
- **`- Item` / `1. Item`:** Standard bulleted or numbered items for non-interactive information.

```markdown
## 2. Configuration Tasks

- [ ] Configure network IP parameters
- Standard reference parameter: Subnet 255.255.255.0
- [ ] Apply latest security patch
```

### 4. Code Blocks & Dynamic Variable Substitution (`[Variables]`)

Annotate a Markdown table with `[Variables]` to extract key-value variables. Place `{{VARIABLE_NAME}}` placeholders inside code blocks to substitute values dynamically on copying and printing.

```markdown
| Parameter | Default |
| --- | --- |
| [Variables] | |
| IP_ADDRESS | 192.168.1.100 |
| GATEWAY | 192.168.1.1 |

```bash
ping {{IP_ADDRESS}} -g {{GATEWAY}}
```
```

In the rendered HTML, variables are presented in an interactive table with editable text inputs. Updates automatically propagate to code block copy actions and persist in `localStorage`.

### 5. Callout / Alert Panels

Format blockquotes with specific prefix symbols to render color-coded callout panels:

```markdown
> Standard note message box.

>? Pro Tip: Use keyboard shortcuts for faster navigation.

>! Important: Back up all data before proceeding.

>!! Warning: Disconnecting power during update will corrupt firmware.

>!!! Caution: High voltage area. Exercise extreme care!
```

| Syntax | Alert Type | Badge Label (EN / DE) | Visual Theme |
| --- | --- | --- | --- |
| `>` | Note | Note / Hinweis | Neutral Grey |
| `>?` | Tip | Tip / Tipp | Green |
| `>!` | Important | Important / Wichtig | Blue |
| `>!!` | Warning | Warning / Warnung | Orange |
| `>!!!` | Caution | Caution / Achtung | Red |

### 6. Image Embedding & Lightbox

Standard Markdown images are automatically read, converted to Base64 data URIs, and embedded:

```markdown
![System Architecture](./images/architecture.png)
```

Clicking an image in the rendered HTML opens an interactive lightbox modal for detailed inspection.

---

## Building & Development

### Prerequisites

- [Rust Toolchain](https://www.rust-lang.org/) (2024 Edition)
- [Node.js](https://nodejs.org/) & [TypeScript](https://www.typescriptlang.org/) (Client script toolchain)

### Build Executable

```bash
# Build debug binary
cargo build

# Build optimized release binary
cargo build --release
```

`build.rs` automatically compiles client TypeScript modules using `esbuild` and embeds them alongside CSS and locale JSON files into the Rust binary.

The release binary will be created at `target/release/d2f` (Linux/macOS) or `target/release/d2f.exe` (Windows).

### Running Tests

```bash
# Run unit, integration, and doc tests
cargo test
```

---

## Architecture & Project Structure

```text
doc2flow/
├── .cargo/               # Cargo cross-compile configuration & aliases
├── locales/              # Static UI translations (de.json, en.json)
├── styles/               # Modular CSS (code, core, images, tasks, toc)
├── templates/            # HTML base layout and starter Markdown templates
├── web/                  # TypeScript client toolchain
│   ├── package.json      # Bundler & Node scripts
│   ├── tsconfig.json     # TypeScript configuration
│   └── src/
│       ├── core/         # Storage, items, sections, fields, export, search
│       └── features/     # Code copy/variables, images/lightbox, tasks, TOC
├── src/                  # Rust CLI backend engine
│   ├── main.rs           # CLI entrypoint & argument parsing
│   ├── lib.rs            # Module declarations & exports
│   ├── components.rs     # Zero-allocation HTML UI generators
│   ├── converter.rs      # Markdown AST parser & feature detector
│   ├── error.rs          # Compiler-style diagnostic reporting
│   ├── hasher.rs         # SHA-256 hash generator
│   ├── i18n.rs           # Locale loader & translation engine
│   ├── id.rs             # Document identifier (d2f_id) generator
│   ├── image.rs          # Base64 embedding & WebP auto-scaling
│   ├── io.rs             # Centralized filesystem I/O operations
│   ├── template.rs       # HTML page orchestrator & feature assembler
│   └── utils.rs          # MIME type detection & CLI utilities
├── tests/                # Integration test suite & showcase fixtures
├── build.rs              # TypeScript build integration & version metadata
├── CHANGELOG.md          # Keep a Changelog documentation
├── SPECIFICATION.md      # Technical specification document
├── AGENTS.md             # AI agent directives
└── README.md             # Project documentation
```

---

## License

This project is licensed under the **GNU General Public License v3.0** (GPL-3.0). See the [LICENSE](file:///home/stefan/Development/doc2flow/LICENSE) file for full details.