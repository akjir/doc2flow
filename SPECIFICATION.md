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

# Legacy pipeline execution
d2f.exe input.md --legacy

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
| `LEGACY` | `--legacy` | Runs using legacy processing pipeline | No | `false` |


---

## 3. Input Specification (Markdown & Extensions)

* **Base Standard:** CommonMark with GitHub Flavored Markdown (GFM) extensions (`tasklists`, `strikethrough`, `tables`).
* **YAML Frontmatter & Metadata:** Optional header metadata block delimited by `---`:
  ```yaml
  ---
  title: "Server Maintenance Guide"
  subtitle: "Standard Operating Procedure"
  date: "2026-07-25"
  version: "1.0.0"
  language: "de"
  logo: "images/custom_logo.svg"
  numbered_sections: true
  ---
  ```
  * `title`: Document title.
  * `subtitle`: Subtitle or secondary description.
  * `date`: Document date.
  * `version`: Document version string.
  * `language` / `lang`: Locale code (`en`, `de`) for static UI translations.
  * `logo`: Path to custom logo image (overridden by CLI `-l` / `--logo`).
  * `header`: Header layout (`"flex"`, `"none"`). Default: `"none"`. When `"flex"`, renders section-style header card containing logo, title, and subtitle before Section 1 and variable table.
  * `numbered_sections`: Enables section numbering (`1. `, `1.1 `). Default: `true`.
* **Callout / Note Box Annotations:** Blockquotes converted to alert panels via prefixes:
  * `>` / `> Note`: Standard Note box (`.note`, neutral styling).
  * `>?` / `>? Tip`: Tip box (`.note-tip`, green accent).
  * `>!` / `>! Important`: Important note panel (`.note-important`, purple accent).
  * `>!!` / `>!! Warning`: Warning box (`.note-warning`, yellow accent).
  * `>!!!` / `>!!! Caution`: Caution box (`.note-caution`, red accent).
* **Document Structure & Structural Mapping:**
  * **Level 1 Headings (`#`):** Non-collapsible section blocks (`.section`, `.sh.sh-h1`, `.sb`) with primary header styling.
  * **Level 2 Headings (`##`):** Collapsible section blocks (`.section`, `.sh`, `.sb`) with completion badges (`.sbadge`) and toggle indicators (`.stog`).
  * **Level 3–6 Headings (`###`–`######`):** Styled subheadings inside section bodies (`.subh`).
* **Checklists & List Items:**
  * **Task Items (`- [ ]`, `- [x]`):** Interactive checkboxes (`.doc-item.check-item`) with dynamic completion tracking.
  * **Bullet & Ordered Items (`-`, `1.`):** Formatted list entries (`.doc-item.simple-item`) with nested list support.
  * **Text Paragraph Items:** Standalone text paragraph blocks (`.doc-item.text-item`).
* **Code Blocks & Variable Substitution (`[Variables]` & `{{VARIABLE_NAME}}`):**
  * Fenced code blocks (` ```lang `) with language tags and 1-click **Copy Code** button.
  * **Dynamic Variable Substitution:** Markdown table annotated with `[Variables]` extracts key-value pairs and replaces `{{VARIABLE_NAME}}` placeholders inside code blocks when copying or printing.
  * **Smart Variable Filtering & Validation:** Automatically scans code blocks for `{{VAR}}` placeholders. Only variables used in at least one code block are displayed in the table (unused table entries emit CLI warnings and are omitted; missing code block variables are added to the table with empty input fields and emit CLI warnings).
  * **Interactive Table & State Persistence:** Rendered before Section 1 in a dark gray container (`.item-table-var-wrap`). Column 2 (`Value`) values are rendered as editable text inputs (`.item-table-var-input.persistent-field`) that save state in `localStorage` and single-file HTML exports.

* **Image & Link Handling:**
  * Relative local images converted to embedded Base64 `data:image/...;base64,...` URIs.
  * Remote image URLs (`http://`, `https://`) preserved as `<img>` tags.
  * Broken or unreachable images gracefully display an embedded fallback placeholder SVG (`placeholder.svg`).
  * Non-image resources (e.g. `.pdf`, `.zip`) rendered as external link elements (`<a>`).

---

## 4. Output Specification (HTML & UX)

* **Self-Contained Document:** Generates a single HTML5 file with fully embedded CSS (`<style>`) and JavaScript (`<script>`).
* **Document Identity (`d2f_id`):** Deterministic SHA-256 key derived from metadata (`title`, `version`, `date`) to uniquely scope browser `localStorage`.
* **Internationalization & Localization (i18n):**
  * Supports localized UI elements via frontmatter `language` tag, mapping to embedded locale JSON files (default: `en`).
  * Placeholders formatted as `{{L_KEY}}` map to `"key"` in target locale JSON. Missing keys emit non-blocking `stderr` warnings.
* **Interactivity & State Persistence:**
  * Interactive checkboxes and input field values are persisted per document in `localStorage` via `d2f_id`.
  * Section badges dynamically track completed items (e.g. `2/5 completed`).
  * Reset button clears stored state, unfolds all collapsed sections, and resets search filters following modal confirmation.
* **Protocol & Sign-off Footer:** Agent signature input, completion date input, signature line, and "Process Completed" sign-off box.
* **Layout & Print Optimization:** Responsive CSS layout with `@media print` rules that automatically expand collapsed sections, hide control buttons, and preserve print colors.

---

## 5. Module Architecture & Subsystem Decoupling

* **Project-Agnostic Library Layer (`src/utils/`):** Dedicated generic subsystem (`base64`, `error`, `hasher`, `io`, `mime`, `uri`) completely decoupled from Doc2Flow domain logic. Reusable across arbitrary projects. `src/core/` and `src/features/` consume it through the centralized API exported by `src/utils/mod.rs`.
* **Filesystem & I/O Isolation (`src/utils/io.rs`):** Exclusive module for generic filesystem interactions, file reading/writing, path resolution (`resolve_path`), and asset retrieval. Direct `std::fs`/`std::io` calls prohibited in processing modules.
* **Pure In-Memory Processing Core:** Core modules (`src/core/converter.rs`, `src/core/builder.rs`, `src/core/components.rs`, `src/core/locales.rs`, `src/core/id.rs`) perform pure in-memory string/AST data transformations decoupled from disk I/O.
* **Domain Image & Logo Processing (`src/core/image.rs`):** Image optimization, SVG sanitization, WebP downscaling, and domain-specific logo path resolution (`resolve_logo_path`).
* **Strict Modular Feature Isolation (HTML, CSS, TS/JS):**
  * Extension features (`code`, `header`, `images`, `tables`, `tasks`) are fully decoupled and zero-knowledge of each other.
  * Each feature maintains dedicated HTML components, TypeScript and CSS modules within its vertical slice directory (`src/features/<name>/`). Compiled JS resides directly in `src/features/<name>/<name>.js`.
  * If a feature is omitted/disabled (`DocumentFeatures`), zero HTML elements, zero CSS rules, and zero JS/TS code for that feature are emitted in the rendered document.
* **HTML UI Components & Builder Engine (`src/components.rs` & `src/core/builder.rs`):**
  * `src/components.rs`: Core-universal zero-allocation HTML UI building blocks (`out: &mut impl Write`). Feature-specific HTML components reside in their respective feature modules.
  * `src/core/builder.rs`: Central HTML page orchestrator, dynamic feature style assembler (`assemble_styles`), and script bundle assembler (`assemble_scripts`).
* **Constants Architecture & Encapsulation Rules:**
  * **Feature-Specific Constants (Strict Encapsulation):** Constants used exclusively by an individual feature (e.g. CSS class names, frontmatter keys, selector strings, feature-internal default values) MUST be defined directly in the respective `src/legacy/features/<feature_name>/module.rs` (or private submodules). Distributing feature constants across central files or dumpsters is strictly prohibited to eliminate tight coupling.
  * **Global System Constants (`src/legacy/core/constants.rs`):** Reserved exclusively for application-wide, feature-independent system metadata and global core defaults (e.g. `APP_NAME`, `CLI_BANNER`, `APP_VERSION`, `REPOSITORY_URL`, `LICENSE_TERMS`, `LICENSE_URL`, global system/I/O limits).
* **Centralized Diagnostic Error Handling:** Runtime, I/O, and syntax errors map to diagnostic compiler-style error types (`Error` in `src/core/error.rs`, `Doc2FlowError` in `src/legacy/utils/error.rs`).
* **Legacy Subsystem (`src/legacy/`):**
  * Contains the self-contained legacy conversion engine (`src/legacy/legacy.rs`, `core/`, `features/`, `utils/`).
  * Zero coupling to the root `src/core/` pipeline.
  * Activated when `--legacy` is passed on the CLI.

---

## 6. Technical Framework & Quality Standards

* **Programming Language:** Rust (Edition 2024) for CLI backend, TypeScript 7.0 for client toolchain.
* **Target Platform:** Windows 64-Bit (`x86_64-pc-windows-msvc`).
* **Version & Build Metadata:** Dynamic SemVer 2.0.0 versioning evaluated at compile time in `build.rs`:
  * Format: `v<MAJOR>.<MINOR>.<PATCH>+<COMMIT_COUNT>.<COMMIT_HASH>[.dev]`
  * Exported as `D2F_FULL_VERSION` compiler env var; embedded in `d2f --version` output, HTML `<meta name="generator">` tags, and header comments.
* **Binary Size:** Executable size target `< 10 MB` using stripping, LTO, and release optimizations.
* **Core Dependencies:** `pulldown-cmark`, `serde`, `serde_json`, `image` (custom Base64/MIME helpers in `src/legacy/utils/`).
* **Error Handling & Testing:** Zero panics on invalid paths/inputs; human-readable diagnostic error messages on `stderr`. Unit and integration test suite coverage.

---

## 7. Directory & File Structure

```text
doc2flow/
├── .cargo/
│   └── config.toml           # Cargo Aliases and Cross-Compile configuration
├── resources/                # Embedded static resources
│   ├── images/               # Built-in vector icons and logos
│   │   ├── logo.svg          # Default document header logo
│   │   └── placeholder.svg   # Default fallback image placeholder
│   ├── locales/              # Internationalization JSON translations
│   │   ├── de.json           # German static UI translations
│   │   └── en.json           # English static UI translations
│   └── templates/            # HTML layout and starter Markdown templates
│       ├── base.html         # Base layout template
│       └── template.md       # Starter Markdown template for init command
├── web/                      # Client-side TypeScript toolchain
│   ├── package.json          # Node and esbuild bundler configuration
│   └── tsconfig.json         # TypeScript compiler configuration
├── src/                      # Rust CLI backend
│   ├── main.rs               # CLI entry point and argument parsing
│   ├── lib.rs                # Module declarations and library interface
│   ├── core/                 # Core modular engine and document AST pipeline
│   │   ├── mod.rs            # Core module exports
│   │   ├── build/            # Document building and output generator
│   │   │   ├── mod.rs        # Build module root
│   │   │   └── builder.rs    # Document builder engine
│   │   ├── document.rs       # Document AST data structures
│   │   ├── document_json.rs  # AST serialization and development inspection
│   │   ├── error.rs          # Compiler-style diagnostic error reporting
│   │   ├── parse/            # Document and CLI argument parsing
│   │   │   ├── mod.rs        # Parse module root
│   │   │   ├── arguments.rs  # CLI argument parser with --legacy support
│   │   │   ├── markdown.rs   # Zero-alloc Markdown parser
│   │   │   └── parser.rs     # Document parser entry point
│   │   └── utils/            # Core filesystem and encoding utilities
│   └── legacy/               # Isolated legacy conversion subsystem
│       ├── mod.rs            # Legacy subsystem root
│       ├── legacy.rs         # Legacy CLI execution runner
│       ├── utils/            # Generic legacy utility library
│       ├── core/             # Legacy core processing engine
│       └── features/         # Legacy vertical slice features

├── tests/
│   ├── example_onboarding.html # Compiled onboarding showcase HTML fixture
│   ├── example_onboarding.md # Onboarding Markdown showcase source
│   ├── integration_test.rs   # CLI and end-to-end integration tests
│   ├── showcase_de.html      # Compiled German showcase HTML fixture
│   ├── showcase_de.md        # German Markdown showcase source
│   ├── showcase_en.html      # Compiled English showcase HTML fixture
│   └── showcase_en.md        # English Markdown showcase source
├── build.rs                  # TypeScript build integration and version metadata
├── Cargo.toml                # Rust dependencies and build profile
├── SPECIFICATION.md          # Functional specification
├── AGENTS.md                 # AI agent directives
├── CHANGELOG.md              # Version history
└── README.md                 # Project documentation
```
