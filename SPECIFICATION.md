# Project Specification: Doc2Flow (d2f)

## 1. Overview & Objectives
Doc2Flow (`d2f`) is a command-line interface (CLI) tool built for Windows that converts Markdown files into fully self-contained HTML files. The generated HTML files serve as interactive guides, manuals, protocols, and checklists for end users.

### Core Principles & Non-Negotiables
- **Single Binary Output:** The build must result in a single executable file (`d2f.exe`) with no external runtime dependencies.
- **Zero-Dependency HTML:** The generated HTML file must contain all necessary assets (CSS, JS, images via Base64) embedded directly within it. There must be absolutely no references to external servers or local directories.
- **Integrated Templates & Localization:** All required HTML/CSS/JS templates and i18n locale definitions must be embedded into the binary at compile time (e.g., using `include_str!`).

---

## 2. CLI Interface & Usage

### Executable
The name of the executable file is **`d2f.exe`**.

### Command Line Syntax
```bash
# Standard execution (generates input.html)
d2f.exe input.md

# Explicit output path
d2f.exe input.md -o custom_output.html

# Enable automatic image compression and WebP conversion for local images > 250 KB
d2f.exe input.md -s

# Generate a starter Markdown template (defaults to template.md)
d2f.exe --init
d2f.exe -i custom_template.md

# Help text & Version
d2f.exe --help
d2f.exe --version
```

### Parameters & Arguments

| Argument / Flag | Short | Description | Required? | Default |
| --- | --- | --- | --- | --- |
| `INPUT` | — | Path to the source Markdown file | Conditional (unless `--init` is used) | — |
| `OUTPUT` | `-o`, `--output` | Target path for the generated HTML file | No | `<INPUT_NAME>.html` |
| `INIT` | `-i`, `--init` | Generates a starter template Markdown file | No | `template.md` |
| `AUTO_SCALE` | `-s`, `--auto-scale` | Automatically resizes local images exceeding 250 KB to WebP | No | `false` |

---

## 3. Input Specification (Markdown & Extensions)

* **Base Standard:** CommonMark with GitHub Flavored Markdown (GFM) extensions (`ENABLE_TASKLISTS`, `ENABLE_STRIKETHROUGH`, `ENABLE_TABLES`).
* **YAML Frontmatter & Metadata:** The Markdown file can contain YAML-style frontmatter delimited by `---` at the beginning of the document:
  ```yaml
  ---
  title: "Server Maintenance Guide"
  subtitle: "Standard Operating Procedure"
  company: "Acme Corp"
  contact: "John Doe"
  agent: "Jane Smith"
  date: "2026-07-25"
  version: "1.0.0"
  language: "de"
  ---
  ```
  * `title`: Document main header title.
  * `subtitle`: Document subtitle or description.
  * `company`: Company organization name (**required**; throws diagnostic error if missing).
  * `contact`: Contact person name.
  * `agent`: Operator/Agent responsible.
  * `date`: Document creation date (used alongside metadata for `d2f_id` document identity generation).
  * `version`: Document version string (included in document identity hash).
  * `language` / `lang`: Specifies locale code (`en`, `de`) for static UI translations.
  * **Upper Metadata Table:** Header table renders Company (`{{COMPANY}}`), Contact (`{{CONTACT}}`), Agent (`{{AGENT}}`), and an interactive persistent Date input field.
* **Callout / Note Box Annotations:** Blockquotes are transformed into styled visual alert panels using prefix conventions:
  * `>` or `> Note`: Standard Note box (`.note`, neutral styling).
  * `>?` or `>? Tip`: Tip box (`.note-tip`, green accent).
  * `>!` or `>! Important`: Important box (`.note-important`, purple accent).
  * `>!!` or `>!! Warning`: Warning box (`.note-warning`, yellow accent).
  * `>!!!` or `>!!! Caution`: Caution / Danger box (`.note-caution`, red accent).
* **Document Structure & Structural Mapping:**
  * **Level 1 Headings (`#`):** Define non-collapsible section blocks (`.section`, `.sh.sh-h1`, `.sb`) with primary header styling (`--bd`), omitting section completion badges while including their task items in overall document progress calculations.
  * **Level 2 Headings (`##`):** Define collapsible section blocks (`.section`, `.sh`, `.sb`) with section completion badges (`.sbadge`) and toggle indicators (`.stog`).
  * **Level 3–6 Headings (`###` to `######`):** Define styled subheadings inside section bodies (`.subh`).
* **Checklists & List Items:**
  * **Task Items (`- [ ]`, `- [x]`):** Rendered as interactive checkboxes (`.check-item`) with dynamic completion tracking.
  * **Bullet & Ordered Items (`-`, `1.`):** Rendered as clean, formatted list entries (`.simple-item`) with nested list support.
* **Code Blocks:**
  * Fenced code blocks (` ```lang `) display language tags and include an interactive 1-click **Copy Code** button.
* **Image & Link Handling:**
  * Relatively linked local images (e.g., `![Alt-Text](./images/graphic.png)`) are resolved locally by `d2f`, converted to Base64, and embedded directly as `data:image/...;base64,...` URIs.
  * Remote image URLs (`http://`, `https://`) are preserved as `<img>` tags.
  * Non-image resources (e.g., `.pdf`, `.zip`) specified in image tags are converted to external link elements (`<a>`).
  * Standard Markdown hyperlinks (`[Link Text](url)`) are rendered as `<a>` tags.

---

## 4. Output Specification (HTML & UX)

* **Self-Contained Document:** Generates a single valid HTML5 document with fully embedded styling (`<style>`) and script logic (`<script>`).
* **Document Identity (`d2f_id`):** Generates a deterministic SHA-256 identity key derived from metadata (`company`, `title`, `subtitle`, `date`, `version`) to uniquely scope client-side state storage.
* **Internationalization & Localization (i18n):**
  * Supports localized static UI elements based on the `language` frontmatter tag (matching the lowercased code to embedded locale JSON files, defaulting to `en`).
  * **Dynamic Resource Loading:** Locale resources are loaded dynamically into a `HashMap<String, String>` from flat JSON key-value files.
  * **Template Replacement Scheme:** Placeholders in HTML templates formatted as `{{L_KEY}}` automatically map to the lowercased key `"key"` in the respective locale JSON file (e.g. `{{L_COMPANY}}` maps to `"company"`).
  * **Missing Key Handling:** Template placeholders missing from the target locale emit a non-blocking warning message on `stderr` during rendering without causing panic or termination.
  * Automatically translates controls, buttons, metadata labels, progress indicators, callout header tags, and print titles.
* **Interactivity & State Persistence:**
  * Checkboxes can be toggled by end users.
  * Checkbox states and metadata input values are persisted per document in browser `localStorage` using `d2f_id`.
  * Section badges dynamically calculate checked vs. total items (e.g., `2/5 completed`).
  * Reset button clears state after user confirmation in a modal overlay.
* **Protocol & Sign-off Footer:**
  * Provides agent signature input fields, completion date input, signature line, and a "Process Completed" / "Vorgang abgeschlossen" sign-off box.
* **Layout & Print Optimization:**
  * Clean, modern, responsive CSS layout.
  * Dedicated `@media print` rules automatically expand all sections, hide interactive buttons (copy, reset), and output clean printed pages or PDF exports.

---

## 5. Roadmap & Architecture Preparation (Multi-File & Directories)

To enable future expansions without requiring significant refactoring, the internal processing pipeline is designed with abstraction in mind:

* **Pipeline Design:** The engine expects Markdown input internally as a modular content stream with resolved image paths rather than a single static file reference.
* **Phase 1 (MVP):** Processing of exactly one Markdown file.
* **Phase 2 (Future):** Support for merging multiple files (`d2f m1.md m2.md -o out.html`) and processing entire directories (`d2f ./docs/`).

---

## 6. Technical Framework & Quality Standards

* **Programming Language:** Rust (Edition 2024).
* **Target Platform:** Windows 64-Bit (`x86_64-pc-windows-msvc`).
* **File Size:** Target size of `d2f.exe` `< 10 MB` (utilizing binary stripping, LTO, and release optimizations).
* **Core Libraries:** `pulldown-cmark`, `serde`, `serde_json`, `image` (custom zero-dependency modules in `src/utils.rs` for Base64 encoding, MIME type guessing, and CLI argument parsing).
* **Error Handling:**
  * No panics or crashes when encountering invalid paths or missing permissions.
  * Human-readable, color-coded diagnostic compiler-style error messages output to `stderr`.
* **Testing:**
  * Unit tests for Markdown parsing, frontmatter validation, callout parsing, i18n locale resolution, document hashing, and Base64 image embedding.
  * Integration tests for CLI parameters and end-to-end HTML generation with fixture validation.
