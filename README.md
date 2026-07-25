# Doc2Flow (`d2f`)

**Doc2Flow (`d2f`)** is a fast, lightweight command-line tool built in Rust that converts Markdown documents into standalone, interactive HTML guides, protocols, and checklists.

The generated HTML files are completely self-contained—embedding all CSS styling, JavaScript interactivity, and base64-encoded local images—making them ideal for offline distribution, customer handovers, and field service workflows without external web servers or assets.

---

## Key Features

- 🚀 **Single Binary Executable:** Distributed as a lightweight, zero-dependency executable (`d2f.exe`).
- 📦 **Zero-External Dependencies HTML:** All styles, scripts, icons, and local images are embedded directly into a single `.html` file via base64 encoding.
- ✅ **Interactive Task Lists & State Persistence:**
  - Dynamic task list checkboxes (`- [ ]` / `- [x]`).
  - Section completion counters and global progress tracking.
  - Automatically saves completion state in the browser's `localStorage`.
  - Includes a global state reset option with a modal confirmation.
- 🌐 **Multi-Language / i18n Support:**
  - Built-in English (`en`) and German (`de`) static UI translations.
  - Selectable per document via the YAML frontmatter `language: "de"` setting.
- 📣 **Rich Callout & Alert Panels:**
  - Styled alert boxes using simple blockquote prefix notation (`Note`, `Tip`, `Important`, `Warning`, `Caution`).
- 💻 **Enhanced Code Blocks:**
  - Syntax-aware code block containers with language tag headers and 1-click **Copy Code** buttons.
- 📝 **Protocol Sign-Off & Signature Footer:**
  - Built-in form fields for technician names, completion dates, signature lines, and a final protocol approval checkbox.
- 🖨️ **Print & PDF Optimized:**
  - Dedicated print stylesheet (`@media print`) that auto-expands all collapsible sections and hides interactive buttons for clean physical printouts or PDF exports.

---

## Usage

### Command Line Syntax

```bash
# Standard conversion (generates input.html)
d2f input.md

# Specify custom output path
d2f input.md -o /path/to/output.html

# View CLI help
d2f --help

# View CLI version
d2f --version
```

---

## Markdown Syntax & Authoring Guide

Doc2Flow uses CommonMark with GitHub Flavored Markdown (GFM) extensions alongside extended syntax conventions:

### 1. YAML Frontmatter (Metadata & Localization)

Place YAML metadata at the very top of your `.md` file to populate the header table and set document options:

```yaml
---
title: "Server Deployment Guide"
subtitle: "Standard Operating Procedure"
customer: "Acme Corporation"
employee: "Jane Doe"
technician: "John Smith"
date: "2026-07-25"
language: "de"
---
```

### 2. Collapsible Sections & Headings

- **`## Section Title` (Level 2 Heading):** Creates a collapsible section container with a completion badge (`0/3 completed`) and toggle indicator.
- **`### Subheading` (Level 3 Heading):** Creates a sub-section label within a section.

```markdown
## 1. Initial Inspection

### System Check
- [ ] Inspect hardware for physical damage
- [ ] Verify power supply connections
```

### 3. Interactive Checklists vs. Bullet Lists

- **`- [ ]` / `- [x]`:** Interactive task item tracked by progress counters and saved in `localStorage`.
- **`- Item`:** Standard bulleted item for non-interactive information.

```markdown
## 2. Configuration Tasks

- [ ] Configure IP settings
- Standard reference parameter: Subnet 255.255.255.0
- [ ] Apply security patch
```

### 4. Callout / Alert Panels

Format blockquotes with specific prefixes to render color-coded callout boxes:

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

### 5. Code Blocks

Use standard fenced code blocks with language identifiers to enable language header tags and copy functionality:

```ini
```ini
[Network]
IPAddress = 192.168.1.100
SubnetMask = 255.255.255.0
Gateway = 192.168.1.1
```
```

### 6. Local Image Embedding

Link local images with standard Markdown image syntax. Doc2Flow reads local image files, converts them to Base64 data URIs, and embeds them directly into the HTML document:

```markdown
![System Architecture](./images/architecture.png)
```

---

## Building & Development

### Prerequisites

- [Rust Toolchain](https://www.rust-lang.org/) (2024 Edition)

### Build Executable

```bash
# Build debug binary
cargo build

# Build optimized release binary
cargo build --release
```

The release binary will be placed at `target/release/d2f.exe` (Windows) or `target/release/d2f` (Linux/macOS).

### Running Tests

```bash
# Run unit and integration tests
cargo test
```

---

## Architecture & Project Structure

```
doc2flow/
├── src/
│   ├── main.rs            # CLI entrypoint (clap parser & workflow)
│   ├── lib.rs             # Module declarations & exports
│   ├── converter.rs       # Markdown parsing & HTML generation engine
│   ├── image_embedder.rs  # Local image path resolution & Base64 encoder
│   ├── i18n.rs            # Multi-language locale loader & dictionary
│   └── utils.rs           # Core helper utilities
├── templates/
│   ├── base.html          # Embedded HTML output skeleton
│   ├── style.css          # Embedded stylesheet & print rules
│   └── script.js          # Embedded interactivity & localStorage logic
├── locales/
│   ├── en.json            # English translations
│   └── de.json            # German translations
├── tests/
│   └── integration_test.rs # Integration test suite
├── SPECIFICATION.md       # Technical specification document
├── AGENTS.md              # AI agent guidelines
└── README.md              # Project documentation
```

---

## License

This project is licensed under the **GNU General Public License v3.0** (GPL-3.0). See the [LICENSE](file:///home/stefan/Development/doc2flow/LICENSE) file for full details.