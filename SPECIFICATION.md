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

* **Filesystem & I/O Isolation (`src/io.rs`):** Exclusive module for filesystem interactions, file reading/writing, path resolution (`Path`, `PathBuf`), and asset retrieval. Direct `std::fs`/`std::io` calls prohibited in processing modules.
* **Pure In-Memory Processing Core:** Core modules (`src/converter.rs`, `src/template.rs`, `src/components.rs`, `src/locales.rs`, `src/hasher.rs`, `src/id.rs`) perform pure in-memory string/AST data transformations decoupled from disk I/O.
* **Strict Modular Feature Isolation (HTML, CSS, TS/JS):**
  * Extension features (`code`, `header`, `image`, `table`, `tasks`) are fully decoupled and zero-knowledge of each other.
  * Each feature maintains dedicated HTML components, TypeScript and CSS modules within its vertical slice directory (`src/features/<name>/`).
  * If a feature is omitted/disabled (`DocumentFeatures`), zero HTML elements, zero CSS rules, and zero JS/TS code for that feature are emitted in the rendered document.
* **HTML UI Components & Templating (`src/components.rs` & `src/template.rs`):**
  * `src/components.rs`: Core-universal zero-allocation HTML UI building blocks (`out: &mut impl Write`). Feature-specific HTML components reside in their respective feature modules.
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
│   └── config.toml           # Cargo Aliases and Cross-Compile configuration
├── resources/                # Embedded static resources
│   ├── images/               # Built-in vector icons and logos
│   │   └── logo.svg          # Default document header logo
│   ├── locales/              # Internationalization JSON translations
│   │   ├── de.json           # German static UI translations
│   │   └── en.json           # English static UI translations
│   └── templates/            # HTML layout and starter Markdown templates
│       ├── base.html         # Base layout template
│       └── template.md       # Starter Markdown template for init command
├── web/                      # Client-side TypeScript toolchain
│   ├── package.json          # Node and esbuild bundler configuration
│   ├── tsconfig.json         # TypeScript compiler configuration
│   └── dist/                 # Bundled JS modules generated by esbuild
│       ├── script-code.js    # Compiled code block feature JavaScript bundle
│       ├── script-core.js    # Compiled core JavaScript bundle
│       ├── script-images.js  # Compiled images feature JavaScript bundle
│       ├── script-table.js   # Compiled section table feature JavaScript bundle
│       └── script-tasks.js   # Compiled tasks feature JavaScript bundle
├── src/                      # Rust CLI backend
│   ├── main.rs               # CLI entry point and argument parsing
│   ├── lib.rs                # Module declarations and library interface
│   ├── core/                 # Core architecture, engine, stylesheets and TS runtime
│   │   ├── mod.rs            # Core module exports
│   │   ├── components.rs     # Core-universal HTML UI component generators
│   │   ├── converter.rs      # Markdown AST parser and feature detector interface
│   │   ├── error.rs          # Diagnostic error types and reporting
│   │   ├── feature.rs        # Feature trait and DocumentContext detection
│   │   ├── generator.rs      # HTML Assembler and template engine
│   │   ├── hasher.rs         # SHA-256 hash generator
│   │   ├── id.rs             # Document identifier generation
│   │   ├── image.rs          # Image optimization, WebP scaling and Base64 embedding
│   │   ├── io.rs             # Central filesystem and asset IO
│   │   ├── locales.rs        # Locale loader and translation engine
│   │   ├── parsing/          # CLI argument parsing and grammar
│   │   │   └── arguments.rs  # Zero-dependency CLI argument parsing and validation
│   │   ├── template.rs       # HTML page orchestrator
│   │   ├── utils.rs          # Base64 encoding, MIME type guessing, and Data-URI conversion
│   │   └── web/              # Core web frontend runtime and stylesheets
│   │       ├── core.css      # Base layout and styles
│   │       ├── comments.ts   # Inline check-item comment boxes and persistence
│   │       ├── core.ts       # Central core module, bundle entry point, and reset handler registry
│   │       ├── export.ts     # Document export operations (PDF export and HTML state download)
│   │       ├── fields.ts     # Persistent inputs, date shortcuts and field synchronization
│   │       ├── items.ts      # Interactive document text and list item click handlers and persistence
│   │       ├── lang.ts       # Dynamic localization dictionary
│   │       ├── search.ts     # Search toolbar and text filtering
│   │       ├── sections.ts   # Collapsible section toggling and state handlers
│   │       ├── storage.ts    # localStorage persistence manager for save and load handlers
│   │       ├── types.ts      # Shared TypeScript type definitions
│   │       └── utils.ts      # Utility functions for debouncing
│   └── features/             # Vertical slice feature modules
│       ├── mod.rs            # Central feature registry (get_all_features)
│       ├── code/             # Unified code block and copy feature vertical slice
│       │   ├── module.rs     # Rust Feature struct, trait implementation, and HTML components
│       │   ├── code.ts       # TypeScript client script for variables and copy button
│       │   └── code.css      # Isolated CSS for code blocks, variables table and copy button
│       ├── header/           # Unified document header and flexible banner vertical slice
│       │   ├── module.rs     # Rust Feature struct, trait implementation, and HTML components
│       │   └── header.css    # Isolated CSS for flexible header card and print styles
│       ├── image/            # Unified image container and lightbox vertical slice
│       │   ├── module.rs     # Rust Feature struct, trait implementation, and HTML components
│       │   ├── image.ts      # TypeScript client script for lightbox modal
│       │   └── image.css     # Isolated CSS for image container, lightbox and print
│       ├── table/            # Unified section table and tabular layout vertical slice
│       │   ├── module.rs     # Rust Feature struct and trait implementation
│       │   ├── table.ts      # TypeScript client script for section table hover and formatting
│       │   └── table.css     # Isolated CSS for section tables and print styles
│       └── tasks/            # Unified task list and checklist progress vertical slice
│           ├── module.rs     # Rust Feature struct, trait implementation, and HTML components
│           ├── tasks.ts      # TypeScript client script for checklist progress and finish box
│           └── tasks.css     # Isolated CSS for checklist items, progress bar and finish box
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
