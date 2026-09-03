# Doc2Flow (d2f)

[![Version](https://img.shields.io/badge/version-0.9.4-blue.svg)](Cargo.toml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL_v3-blue.svg)](LICENSE)
[![Rust: 2024](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)]()

Doc2Flow is a high-performance command-line tool written in Rust that converts Markdown documents into standalone, interactive HTML workflows and checklists.

The generated HTML files are completely self-contained. All styling, JavaScript logic, and images are embedded directly into a single file. They require no web server, no internet connection, and no external runtime dependencies. State such as checkbox progress, user comments, and custom input values are preserved locally in the browser across sessions.

---

## Key Capabilities

- **Self-Contained Single-File Output**: Embeds all stylesheets, scripts, and local images (as Base64 data URIs) into a single portable HTML file.
- **Client-Side State Persistence**: Checkbox states, user comments, and dynamic variable inputs are saved to browser `localStorage` scoped by a unique document identifier (`d2f_id`).
- **Interactive Checklists and Workflows**: Features collapsible sections, dynamic section completion badges, interactive checklists (`- [ ]`), and clickable text elements.
- **Dynamic Variable Replacement**: Define variables once in a table to dynamically update placeholders (`{{VAR_NAME}}`) inside code blocks with one-click clipboard copying.
- **In-Browser Search and Filter**: Instant live search (`Ctrl+K`) with term highlighting and section filtering.
- **Item-Level Comments**: Add notes and comments to any checklist item, list entry, or paragraph.
- **Asset Optimization**: Automatically compresses local images larger than 250 KB to WebP when the auto-scale option is enabled.
- **Print and PDF Export Ready**: Dedicated print stylesheet expands collapsed sections and hides interface toolbars for clean PDF export via the browser print dialog.

---

## Installation

### Prerequisites

- Rust toolchain (Edition 2024, Rust 1.85 or newer)
- Cargo package manager

### Building from Source

Clone the repository and compile the release binary:

```bash
git clone https://github.com/akjir/doc2flow.git
cd doc2flow
cargo build --release
```

The compiled executable is located at:
- **Linux / macOS**: `target/release/d2f`
- **Windows**: `target/release/d2f.exe`

---

## Quick Start

### 1. Generate a Starter Template

Create an initial Markdown template containing example syntax and configuration:

```bash
d2f --init
```

This creates a file named `template.md` in the current directory. You can also specify a custom path:

```bash
d2f --init my_checklist.md
```

### 2. Compile Markdown to HTML

Convert a Markdown file into a standalone HTML file:

```bash
d2f guide.md
```

This generates `guide.html` in the same directory.

### 3. Advanced Compilation

Specify a custom output path, a company logo, and enable automatic image compression:

```bash
d2f guide.md --output /var/www/procedure.html --logo assets/company_logo.svg --auto-scale
```

---

## Command-Line Interface

```text
Usage: d2f [OPTIONS] [INPUT]
```

### Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<INPUT>` | Path to the source Markdown file | Yes (unless using `--init`) |

### Options

| Option | Short | Parameter | Description | Default |
| :--- | :--- | :--- | :--- | :--- |
| `--output` | `-o` | `<PATH>` | Destination path for the generated HTML file | `<INPUT_STEM>.html` |
| `--logo` | `-l` | `<PATH>` | Path to a custom logo image (SVG, PNG, JPG, WebP) | Default embedded logo |
| `--init` | `-i` | `[PATH]` | Generate a starter Markdown template file | `template.md` |
| `--auto-scale` | `-s` | None | Compress local images larger than 250 KB to WebP | `false` |
| `--help` | `-h` | None | Print command-line help information | — |
| `--version` | `-V` | None | Print version information | — |

---

## Document Configuration (YAML Frontmatter)

Each Markdown file can include an optional YAML frontmatter block at the top of the file enclosed by triple dashes (`---`).

```yaml
---
title: "Server Deployment Checklist"
subtitle: "Standard Operating Procedure"
date: "2026-09-03"
version: "1.0.0"
language: "en"
header: true
comments: true
numbered_sections: true
---
```

### Frontmatter Keys

| Key | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `title` | `string` | Primary document title displayed in the header banner and browser tab | `""` |
| `subtitle` | `string` | Secondary descriptive text displayed beneath the title | `""` |
| `date` | `string` | Protocol, revision, or document date | `""` |
| `version` | `string` | Document revision or version string | `""` |
| `language` | `string` | UI localization code (`en` for English, `de` for German) | `"en"` |
| `logo` | `string` | Relative path or URI to a custom header logo image | `""` |
| `header` | `bool` | Enable or disable the top header banner card | `true` |
| `comments` | `bool` | Enable or disable interactive item comment boxes and icons | `true` |
| `numbered_sections` | `bool` | Automatically number section headings (`1.`, `1.1`) | `false` |

---

## Markdown Syntax Guide

Doc2Flow supports standard GitHub Flavored Markdown (GFM) along with specialized syntax designed for operational runbooks and workflows.

### Section Headings

- `# Heading 1`: Defines a top-level section container.
- `## Heading 2`: Defines a collapsible section container. Includes an interactive completion badge and fold indicator.
- `### Heading 3` to `###### Heading 6`: Subheadings within section content.

### Interactive Checklists and Lists

- **Task Items**: `- [ ]` creates an unchecked interactive checkbox; `- [x]` creates a pre-checked checkbox.
- **Nested Tasks**: Indent by two spaces (for example, `  - [ ]`) to create sub-tasks.
- **Interactive Selection**: Clicking anywhere on a task item, list entry, or text paragraph toggles a green highlight and records completion in browser storage.
- **Bullet Lists**: Unordered lists using `- ` or `* `.
- **Ordered Lists**: Sequential numbered lists using `1. `, `2. `, etc.

### Notice and Callout Boxes

Create distinct callout panels using prefix indicators:

| Syntax | Box Type | Styling |
| :--- | :--- | :--- |
| `> Text` | Note | Neutral informational panel |
| `>? Text` | Tip | Green accent panel for best practices and hints |
| `>! Text` | Important | Purple accent panel for critical instructions |
| `>!! Text` | Warning | Yellow accent panel for caution notices |
| `>!!! Text` | Caution | Red accent panel for danger and error warnings |

### Code Blocks and Dynamic Variables

#### 1. Defining Variables

Define key-value pairs using a `:::variables` block directive or a `[Variables]` table before the first main heading:

```markdown
:::variables
| Variable | Value |
| --- | --- |
| TARGET_HOST | 192.168.1.10 |
| SERVICE_PORT | 8080 |
:::
```

#### 2. Using Variables in Code

Placeholders matching `{{VARIABLE_NAME}}` inside fenced code blocks are dynamically substituted:

````markdown
```bash
curl -X GET http://{{TARGET_HOST}}:{{SERVICE_PORT}}/api/health
```
````

In the generated HTML:
- An interactive table allows users to edit variable values in real time.
- Clicking the **Copy Code** button copies the command with current variable values substituted.
- Modified variable values are automatically persisted in `localStorage`.

### Tables

Standard GFM tables render as responsive data tables with aligned columns and hover highlights:

```markdown
| Service | Port | Protocol | Status |
| :--- | :---: | :--- | :--- |
| Web Gateway | 443 | HTTPS | Active |
| Database | 5432 | TCP | Standby |
```

### Images and Attachments

- **Local Images**: `![Architecture](./images/architecture.png)` is embedded directly into the HTML as Base64 data. Missing local images display a placeholder fallback.
- **Remote Images**: `![Status](https://example.com/status.png)` are referenced as external images.
- **Image Lightbox**: Clicking any rendered image opens a modal lightbox viewer.
- **Non-Image Files**: File references such as `![Manual](./files/manual.pdf)` are automatically converted into downloadable external links.

---

## Interactive Browser Features

When opening the generated HTML document in any modern browser:

- **Keyboard Navigation**: Press `Enter` or `Space` on focused section headers to collapse or expand them.
- **Live Search (`Ctrl+K`)**: Use the search toolbar to filter document content in real time with keyword highlighting.
- **Item Comments**: Click the comment icon next to checklist items or paragraphs to record observations or notes.
- **Footer Action Bar**:
  - **Export PDF**: Automatically expands all sections and triggers the browser print dialog.
  - **Save State**: Serializes current progress and inputs directly into the document.
  - **Reset**: Restores the document to its default state after confirmation.
- **Offline Storage**: All user interactions are scoped to the specific document using a SHA-256 identifier, ensuring independent state persistence across different files.

---

## Technical Principles

- **Zero External Dependencies**: Standalone output requires no network calls, CDN links, or external assets.
- **Safe Implementation**: Built with 100% safe Rust (`#![forbid(unsafe_code)]`).
- **Memory Efficient**: Zero-copy parsing and stream rendering minimize memory allocations during compilation.
- **Standards Compliant**: Produces semantic HTML5 elements with accessibility (WCAG / WAI-ARIA) support.

---

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).