# Project Specification: Doc2Flow (d2f)

## 1. Overview & Objectives
Doc2Flow (`d2f`) is a command-line interface (CLI) tool built for Windows that converts Markdown files into fully self-contained HTML files. The generated HTML files serve as interactive guides, manuals, protocols, and checklists for end users.

### Core Principles & Non-Negotiables
- **Single Binary Output:** Builds into a single executable (`d2f.exe`) with zero external runtime dependencies.
- **Zero-Dependency HTML:** Output HTML embeds all CSS, JS, and Base64-encoded images directly—no external server or local path references.
- **Integrated Templates & Localization:** HTML/CSS/JS templates and i18n JSON files are embedded into the binary at compile time via `include_str!`.
- **100% Safe Rust:** Strict prohibition of `unsafe` code blocks across the entire codebase.

---

## 2. CLI Interface & Usage

### Executable
The target binary is **`d2f.exe`**.

### Command Line Syntax
```bash
# Standard execution (generates input.html)
d2f.exe input.md

# Explicit output path
d2f.exe input.md -o custom_output.html

# Custom header logo
d2f.exe input.md -l logo.png
d2f.exe input.md --logo=custom_logo.svg

# Enable automatic image compression/WebP conversion for local images > 250 KB
d2f.exe input.md -s

# Generate starter Markdown template (defaults to template.md)
d2f.exe --init
d2f.exe -i custom_template.md

# Help & Version
d2f.exe --help
d2f.exe --version
```

### Parameters & Arguments

| Argument / Flag | Short | Description | Required? | Default |
| --- | --- | --- | --- | --- |
| `INPUT` | — | Path to source Markdown file | Conditional (unless `--init` used) | — |
| `OUTPUT` | `-o`, `--output` | Target path for generated HTML file | No | `<INPUT_NAME>.html` |
| `LOGO` | `-l`, `--logo` | Path to custom logo (SVG, PNG, JPG, WebP) | No | Default embedded SVG logo |
| `INIT` | `-i`, `--init` | Generates starter template Markdown file | No | `template.md` |
| `AUTO_SCALE` | `-s`, `--auto-scale` | Auto-resizes local images > 250 KB to WebP | No | `false` |

---

## 3. Input Specification (Markdown & Extensions)

* **Base Standard:** CommonMark with GitHub Flavored Markdown (GFM) extensions (`tasklists`, `strikethrough`, `tables`).
* **YAML Frontmatter & Metadata:** Optional header metadata block delimited by `---`:
  ```yaml
  ---
  title: "Server Maintenance Guide"
  subtitle: "Standard Operating Procedure"
  company: "Acme Corp" # Required
  contact: "John Doe"
  agent: "Jane Smith"
  date: "2026-07-25"
  version: "1.0.0"
  language: "de"
  logo: "images/custom_logo.svg"
  number_sections: true
  ---
  ```
  * `company`: Company name (**required**; throws diagnostic error if missing).
  * `language` / `lang`: Locale code (`en`, `de`) for static UI translations.
  * `logo`: Path to custom logo image (overridden by CLI `-l` / `--logo`).
  * `number_sections`: Enables section numbering (`1. `, `1.1 `). Default: `true`.
  * **Header Metadata Table:** Renders Company (`{{COMPANY}}`), Contact (`{{CONTACT}}`), Agent (`{{AGENT}}`), and an interactive persistent Date field.
* **Callout / Note Box Annotations:** Blockquotes converted to alert panels via prefixes:
  * `>` / `> Note`: Standard Note box (`.note`, neutral styling).
  * `>?` / `>? Tip`: Tip box (`.note-tip`, green accent).
  * `>!` / `>! Important`: Important box (`.note-important`, purple accent).
  * `>!!` / `>!! Warning`: Warning box (`.note-warning`, yellow accent).
  * `>!!!` / `>!!! Caution`: Caution box (`.note-caution`, red accent).
* **Document Structure & Structural Mapping:**
  * **Level 1 Headings (`#`):** Non-collapsible section blocks (`.section`, `.sh.sh-h1`, `.sb`) with primary header styling.
  * **Level 2 Headings (`##`):** Collapsible section blocks (`.section`, `.sh`, `.sb`) with completion badges (`.sbadge`) and toggle indicators (`.stog`).
  * **Level 3–6 Headings (`###`–`######`):** Styled subheadings inside section bodies (`.subh`).
* **Checklists & List Items:**
  * **Task Items (`- [ ]`, `- [x]`):** Interactive checkboxes (`.check-item`) with dynamic completion tracking.
  * **Bullet & Ordered Items (`-`, `1.`):** Formatted list entries (`.simple-item`) with nested list support.
* **Code Blocks:** Fenced code blocks (` ```lang `) with language tags and 1-click **Copy Code** button.
* **Image & Link Handling:**
  * Relative local images converted to embedded Base64 `data:image/...;base64,...` URIs.
  * Remote image URLs (`http://`, `https://`) preserved as `<img>` tags.
  * Non-image resources (e.g. `.pdf`, `.zip`) rendered as external link elements (`<a>`).

---

## 4. Output Specification (HTML & UX)

* **Self-Contained Document:** Generates a single HTML5 file with fully embedded CSS (`<style>`) and JavaScript (`<script>`).
* **Document Identity (`d2f_id`):** Deterministic SHA-256 key derived from metadata (`company`, `title`, `subtitle`, `date`, `version`) to uniquely scope browser `localStorage`.
* **Internationalization & Localization (i18n):**
  * Supports localized UI elements via frontmatter `language` tag, mapping to embedded locale JSON files (default: `en`).
  * Placeholders formatted as `{{L_KEY}}` map to `"key"` in target locale JSON. Missing keys emit non-blocking `stderr` warnings.
* **Interactivity & State Persistence:**
  * Interactive checkboxes and input field values are persisted per document in `localStorage` via `d2f_id`.
  * Section badges dynamically track completed items (e.g. `2/5 completed`).
  * Reset button clears stored state following modal confirmation.
* **Protocol & Sign-off Footer:** Agent signature input, completion date input, signature line, and "Process Completed" sign-off box.
* **Layout & Print Optimization:** Responsive CSS layout with `@media print` rules that automatically expand collapsed sections, hide control buttons, and preserve print colors.

---

## 5. Module Architecture & Subsystem Decoupling

* **Filesystem & I/O Isolation (`src/io.rs`):** Exclusive module for filesystem interactions, file reading/writing, path resolution (`Path`, `PathBuf`), and asset retrieval. Direct `std::fs`/`std::io` calls prohibited in processing modules.
* **Pure In-Memory Processing Core:** Core modules (`src/converter.rs`, `src/template.rs`, `src/components.rs`, `src/i18n.rs`, `src/hasher.rs`, `src/id.rs`) perform pure in-memory string/AST data transformations decoupled from disk I/O.
* **Strict Modular Feature Isolation (HTML, CSS, TS/JS):**
  * Extension features (`tasks`, `images`, `toc`) are fully decoupled and zero-knowledge of each other.
  * Each feature maintains dedicated TypeScript (`web/src/features/`) and CSS (`styles/`) modules.
  * If a feature is omitted/disabled (`DocumentFeatures`), zero HTML elements, zero CSS rules, and zero JS/TS code for that feature are emitted in the rendered document.
* **HTML UI Components & Templating (`src/components.rs` & `src/template.rs`):**
  * `src/components.rs`: Reusable zero-allocation HTML UI building blocks (`out: &mut impl Write`).
  * `src/template.rs`: Central HTML page orchestrator, feature style assembler (`render_styles`), and script bundle assembler (`render_scripts`).
* **Centralized Diagnostic Error Handling (`src/error.rs`):** Runtime, I/O, and syntax errors map to domain error types (`Doc2FlowError`) with compiler-style `stderr` warnings (`print_warning`).

---

## 6. Technical Framework & Quality Standards

* **Programming Language:** Rust (Edition 2024) for CLI backend, TypeScript 7.0 for client toolchain.
* **Target Platform:** Windows 64-Bit (`x86_64-pc-windows-msvc`).
* **Version & Build Metadata:** Dynamic SemVer 2.0.0 versioning evaluated at compile time in `build.rs`:
  * Format: `v<MAJOR>.<MINOR>.<PATCH>+<COMMIT_COUNT>.<COMMIT_HASH>[.dev]`
  * Exported as `D2F_FULL_VERSION` compiler env var; embedded in `d2f --version` output, HTML `<meta name="generator">` tags, and header comments.
* **Binary Size:** Executable size target `< 10 MB` using stripping, LTO, and release optimizations.
* **Core Dependencies:** `pulldown-cmark`, `serde`, `serde_json`, `image` (custom Base64/MIME helpers in `src/utils.rs`).
* **Error Handling & Testing:** Zero panics on invalid paths/inputs; human-readable diagnostic error messages on `stderr`. Unit and integration test suite coverage.

---

## 7. Directory & File Structure

```text
doc2flow/
├── .cargo/
│   └── config.toml           # Cargo Aliases / Cross-Compile config
├── locales/                  # I18n JSON files (de.json, en.json)
├── styles/                   # Modular CSS stylesheets
│   ├── core.css              # Core layout, typography, base styles, print & responsive
│   ├── images.css            # Image & Lightbox feature styles
│   ├── tasks.css             # Tasks, checklists, progress bar & finish box styles
│   └── toc.css               # Table of Contents feature styles
├── templates/                # HTML layout & starter Markdown templates
│   ├── base.html             # Base layout template
│   └── template.md           # Starter Markdown template (--init)
├── web/                      # Client-side TypeScript toolchain
│   ├── package.json          # Node/esbuild bundler config
│   ├── tsconfig.json         # TypeScript config
│   ├── src/
│   │   ├── core/             # Base features (collapse, search, storage, main)
│   │   └── features/         # Extension components (tasks, images, toc)
│   └── dist/                 # Bundled JS modules generated by esbuild
├── src/                      # Rust CLI backend
│   ├── main.rs               # CLI entry point & argument parsing
│   ├── lib.rs                # Module declarations & exports
│   ├── io.rs                 # Central filesystem & asset I/O
│   ├── error.rs              # Diagnostic error types & reporting
│   ├── converter.rs          # Markdown AST parser & feature detector
│   ├── template.rs           # HTML page orchestrator
│   ├── components.rs         # HTML UI component generators
│   └── i18n.rs               # Locale loader & translation engine
├── tests/
│   └── integration_test.rs   # CLI & end-to-end integration tests
├── build.rs                  # TypeScript build integration & version metadata
├── Cargo.toml                # Rust dependencies & build profile
├── SPECIFICATION.md          # Functional specification
├── AGENTS.md                 # AI agent directives
├── CHANGELOG.md              # Version history
└── README.md                 # Project documentation
```
