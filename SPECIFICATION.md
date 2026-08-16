# Doc2Flow (`d2f`) — Technical Specification

## 1. Metadata & Status

| Attribute | Specification Value |
| :--- | :--- |
| **Project Name** | Doc2Flow (`d2f`) |
| **Target Binary** | `d2f` (Unix) / `d2f.exe` (Windows) |
| **Target Version** | `v0.9.4` (SemVer 2.0.0 with dynamic build metadata) |
| **Document Status** | Final / Active Implementation |
| **License** | GPL-3.0-or-later |
| **Repository** | `https://github.com/akjir/doc2flow` |

---

## 2. Vision & Scope

### 2.1 Elevator Pitch
Doc2Flow is a high-performance, single-binary CLI tool that compiles Markdown documents into standalone, zero-dependency, interactive offline HTML workflows and checklists with persistent client-side state.

### 2.2 Non-Negotiable Principles
- **Single Static Executable:** MUST compile into a single standalone binary (`d2f`/`d2f.exe`) requiring zero external runtime dependencies.
- **Zero-Dependency Self-Contained Output:** Generated HTML MUST embed all CSS stylesheets, JavaScript logic, and Base64/SVG assets inline. External CDN or network requests are STRICTLY PROHIBITED.
- **100% Safe Rust:** The codebase MUST NOT contain any `unsafe` code blocks (`#![forbid(unsafe_code)]`).
- **Offline Persistence:** Checkbox states and variable inputs MUST persist locally in browser `localStorage` scoped by a deterministic document hash (`d2f_id`).
- **Pure In-Memory Processing:** Document transformations, AST generation, and HTML rendering MUST operate purely in memory with minimal heap allocations.

### 2.3 Out of Scope
- Dynamic web server or daemon runtime modes.
- Multi-file HTML output directories or external asset bundles.
- Direct binary PDF/DOCX compilation (PDF generation is delegated to browser print rendering).
- Remote API integrations or cloud synchronization.
- Heavy JavaScript frontend frameworks (React, Vue, etc.).

---

## 3. Architecture & Technical Constraints

### 3.1 Tech Stack
- **Language & Edition:** Rust (Edition 2024).
- **Core Dependencies & Custom Engines:**
  - `image` (0.25): In-memory image processing and WebP conversion.
  - **Zero-Dependency Parsers:** Markdown token parsing and AST construction are natively implemented in `src/core/markdown.rs`; JSON deserialization and locale mapping are natively implemented via custom zero-allocation parser (`json.rs` / `language.rs`) without external crates (`pulldown-cmark`, `serde`, `serde_json`).
- **Client Runtime:** Vanilla JavaScript (ES6+), decoupled across the `window.d2f` namespace. Zero JS build step in the pipeline.
- **Release Profile:** `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`.

### 3.2 Target Platforms & Compatibility
- **Host / Build Targets:** Linux (`x86_64-unknown-linux-gnu`), Windows (`x86_64-pc-windows-msvc`), macOS (`x86_64-apple-darwin`, `aarch64-apple-darwin`).
- **Path Handling:** All filesystem paths MUST use `std::path::Path` / `PathBuf` for cross-platform safety.
- **Browser Compatibility:** Modern evergreen browsers (Chrome 90+, Firefox 90+, Safari 14+, Edge 90+) supporting CSS custom properties, CSS Flexbox/Grid, and `localStorage`.

### 3.3 System Layering & Modularity
```
┌────────────────────────────────────────────────────────┐
│                        CLI Entry                       │
│             src/main.rs | src/core/arguments.rs        │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                      Core Engine                       │
│   src/core/markdown.rs (AST) | src/core/builder.rs     │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                Vertical Feature Slices                 │
│                 src/features/<feature>/                │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                  Utility Subsystem                     │
│               src/core/utils/ (mod.rs)                 │
│         Base64 | MIME | Hasher | IO | URI | Time       │
└────────────────────────────────────────────────────────┘
```
- **Utility Subsystem (`src/core/utils/`):** Pure, domain-agnostic library modules. Direct `std::fs` calls outside `io.rs` are PROHIBITED.
- **Core Engine (`src/core/`):** Houses the AST data model, zero-alloc Markdown token parser, builder assembler, and compiler-style error diagnostics.
- **Vertical Feature Slices (`src/features/<feature>/`):** Isolated modules (`bullet`, `code`, `core`, `image`, `ordered`, `table`, `task`, `unknown`). Each slice encapsulates its HTML rendering, CSS (`<name>.css`), JS (`<name>.js`), and local constants.
- **Conditional Asset Assembly:** If a feature is absent from a document, zero CSS rules and zero JS code for that feature SHALL be emitted into the output HTML.

---

## 4. Functional Requirements

### 4.1 CLI Interface & Parameters
The CLI executable MUST support the following grammar: `d2f [OPTIONS] [INPUT]`

| Flag / Option | Short | Argument | Description | Default |
| :--- | :--- | :--- | :--- | :--- |
| `<INPUT>` | — | Path | Path to input Markdown file. | Required (unless `--init`) |
| `--output` | `-o` | `<PATH>` | Target HTML output filepath. | `<INPUT_STEM>.html` |
| `--logo` | `-l` | `<PATH>` | Custom header logo (SVG, PNG, JPG, WebP). | Embedded default SVG |
| `--init` | `-i` | `[PATH]` | Generates starter Markdown template. | `template.md` |
| `--auto-scale` | `-s` | — | Auto-resizes local images > 250 KB to WebP. | `false` |
| `--help` | `-h` | — | Prints CLI help text. | — |
| `--version` | `-V` | — | Prints dynamic SemVer version string. | — |

*Validation Rule:* Argument parsing MUST uniformly reject empty values for both space-separated (`-o ""`) and assignment (`-o=`) syntax.

### 4.2 Markdown Input Specifications
- **YAML Frontmatter:** Optional header enclosed by `---`:
  - `title` (string): Document title.
  - `subtitle` (string): Document subtitle.
  - `date` (string): Protocol or revision date.
  - `version` (string): Document version string.
  - `language` (string): Locale code (`en`, `de`) for UI localization (default: `"en"`).
  - `logo` (string): Relative path or URI to header logo (overridden by CLI `-l`).
  - `header` (string): Header layout mode (`"flex"`, `"none"`, default: `"none"`).
  - `numbered_sections` (bool): Automatic heading numbering (`1.`, `1.1`) (default: `true`).
  - Custom keys: Preserved in `DocumentParameters.variables` map.
- **Heading Hierarchy:**
  - `# Heading 1`: Primary section container (`.section`, `.sh.sh-h1`, `.sb`).
  - `## Heading 2`: Collapsible section container (`.section`, `.sh`, `.sb`) with completion badges (`.sbadge`) and fold indicators (`.stog`).
  - `###` to `######`: Subheadings inside section bodies (`.subh`).
- **Interactive Checklists & Tasks:**
  - `- [ ]` / `- [x]`: Rendered as interactive checkboxes (`.check-item`) with dynamic section progress tracking.
- **Hierarchical Lists:**
  - `- `, `* `: Unordered bullet lists (`.bullet-item`) with recursive `--indent` levels.
  - `1. `: Ordered numerical lists (`.order-item`) with automatic sequential position numbering.
- **Block Directives:**
  - `:::<name>` container blocks (e.g. `:::variables`) containing arbitrary child elements.
- **Callout & Shoutout Panels:**
  - `>` / `> Note`: Informational callout (`.note`, neutral styling).
  - `>?` / `>? Tip`: Proactive tip callout (`.note-tip`, green accent).
  - `>!` / `>! Important`: High-priority callout (`.note-important`, purple accent).
  - `>!!` / `>!! Warning`: Warning callout (`.note-warning`, yellow accent).
  - `>!!!` / `>!!! Caution`: Critical caution callout (`.note-caution`, red accent).
- **Code Blocks & Variables:**
  - ` ```lang ` fenced code blocks with language tags, 1-click **Copy Code** button, and variable interpolation (`{{VAR_NAME}}`).
  - `[Variables]` markdown table: Extracts key-value pairs into editable persistent text fields (`.persistent-field`) that dynamically substitute code block placeholders upon copy.
- **Image & Resource Embedding:**
  - Local image paths: Read from disk, converted, and embedded as Base64 `data:<mime>;base64,<data>` URIs.
  - Missing/broken images: Gracefully replaced by embedded vector placeholder SVG (`placeholder.svg`).
  - Non-image assets (`.pdf`, `.zip`): Rendered as external download link elements (`<a>`).

### 4.3 Output Document Specifications
- Single HTML5 document with complete inline `<style>` and `<script>` blocks.
- **Document Hash (`d2f_id`):** Deterministic SHA-256 hash computed over document metadata (`title`, `version`, `date`) scoping client `localStorage`.
- **Localization (i18n):** Static UI placeholders (`{{L_KEY}}`) resolved from embedded JSON dictionaries (`de.json`, `en.json`).
- **Print Optimization (`@media print`):** Collapsed sections MUST automatically expand (`display: block !important`), interactive toolbars MUST be hidden, and background colors MUST be preserved.

---

## 5. Non-Functional Requirements

### 5.1 Performance & Resource Targets
- **Compilation Speed:** A 50 KB Markdown document MUST parse and compile to HTML in < 20 ms on modern hardware.
- **Binary Footprint:** Compiled release executable MUST NOT exceed 10 MB.
- **Memory Footprint:** In-memory transformation MUST NOT exceed 5x input file size during processing.

### 5.2 Efficiency & Zero-Allocation Rules
- Buffers MUST be pre-allocated with exact or estimated capacities (`with_capacity`).
- Slicing and string manipulation MUST prioritize zero-copy techniques (`&str`, `split_once`, `strip_prefix`, `Cow`).
- Direct buffer streaming (`write_str` / `write!`) MUST be used in place of intermediate allocations or `format!`.

### 5.3 Accessibility (a11y) & UX
- Collapsible section headers MUST implement `role="button"`, `tabindex="0"`, and `aria-expanded` state attributes.
- Keyboard navigation MUST support section toggling via `Enter` and `Space` keys.
- Checkbox elements MUST provide associated accessible labels.

---

## 6. Data Model & I/O Architecture

### 6.1 I/O Isolation Rules
- All disk reads and writes MUST be isolated within `src/core/utils/io.rs`.
- Processing engines MUST operate exclusively on UTF-8 strings or AST models in memory.

### 6.2 Core Data Structures (`src/core/document.rs`)
```rust
pub struct Document {
    pub body: Vec<DocumentElement>,
    pub header: Vec<DocumentElement>,
    pub parameters: DocumentParameters,
}

pub enum DocumentElement {
    BlockDirective { name: String, children: Vec<DocumentElement> },
    BulletListItem { content: String, children: Vec<DocumentElement> },
    CheckBoxItem { checked: bool, content: String, children: Vec<DocumentElement> },
    CodeBlock { content: String, language: Option<String> },
    HorizontalRule,
    Image { alt: String, url: String },
    OrderedListItem { position: usize, content: String, children: Vec<DocumentElement> },
    Section { level: usize, title: String, children: Vec<DocumentElement> },
    Shoutout { kind: ShoutoutElementKind, content: String },
    Table { alignments: Vec<TableAlignment>, rows: Vec<Vec<String>> },
    Text(String),
    Unknown(String),
}

pub struct DocumentParameters {
    pub title: String,
    pub subtitle: String,
    pub date: String,
    pub version: String,
    pub language: String,
    pub logo: String,
    pub header: String,
    pub numbered_sections: bool,
    pub variables: HashMap<String, String>,
}

pub struct DocumentFeature {
    pub bullet: bool,
    pub code_block: bool,
    pub image: bool,
    pub ordered: bool,
    pub shoutout: bool,
    pub table: bool,
    pub task: bool,
    pub unknown: bool,
}
```

---

## 7. Error Handling & Diagnostics

### 7.1 Diagnostic Error Reporting
- Runtime errors, invalid frontmatter, or malformed syntax MUST emit compiler-style diagnostic messages on `stderr`.
- Diagnostic output MUST follow rustc conventions:
  ```text
  error: <summary message>
   --> <filepath>:<line>:<col>
    |
  <line> | <source snippet>
    | <carets> <annotation>
    |
  = help: <actionable fix instructions>
  ```

### 7.2 Safety & Process Termination
- Application MUST NOT panic under invalid user input, missing files, or malformed syntax.
- CLI process MUST return `ExitCode::SUCCESS` (0) on successful compilation and `ExitCode::FAILURE` (1) on fatal errors.

---

## 8. Quality & Developer Guidelines

### 8.1 Rust Coding Standards
- **Zero Unsafe:** Strictly zero `unsafe` blocks across all crates.
- **Idiomatic Types:** Newtypes, strong enum variants, and `Result<T, Error>` return types.
- **Single Source of Constants:** Feature-specific constants MUST reside exclusively inside `src/features/<feature>/module.rs`. Global application metadata MUST reside in `src/core/constants.rs`.
- **Inline Attributes:** `#[inline]` MUST NOT be used on heap-allocating functions, I/O routines, CLI parsers, or multi-branch logic.
- **Documentation:** Inline docs MUST be written in concise English with standard headers (`# Examples`, `# Errors`, `# Panics`).

### 8.2 Testing Requirements
- **Pass Rate:** 100% test pass rate required across all unit, doc, and integration tests.
- **Negative & Edge Testing:** Tests MUST explicitly cover invalid CLI arguments, missing delimiters, unclosed directives, broken image paths, and empty inputs.
- **Showcase Parity:** Changes to UI components MUST re-validate and match `examples/showcase_en.html` and `examples/showcase_de.html`.

---

## 9. Repository File Structure

```text
doc2flow/
├── .cargo/
│   └── config.toml           # Cargo aliases and cross-compilation config
├── examples/                 # Showcase Markdown and compiled HTML samples
│   ├── code_variables.md     # Code variables showcase
│   ├── recipe.md             # Recipe checklist showcase
│   ├── showcase_de.md        # German feature showcase
│   ├── showcase_en.md        # English feature showcase
│   └── stellar_evolution.md  # Detailed documentation showcase
├── resources/                # Embedded static resources
│   ├── images/               # Built-in vector icons and logos
│   │   ├── logo.svg          # Default document header logo
│   │   └── placeholder.svg   # Fallback broken image placeholder
│   ├── locales/              # Embedded JSON translation maps
│   │   ├── de.json           # German UI dictionary
│   │   └── en.json           # English UI dictionary
│   └── templates/            # HTML base layouts and starter templates
│       ├── template.html     # Base layout template
│       └── template.md       # Starter markdown template for --init
├── src/                      # Rust CLI & Core Engine
│   ├── main.rs               # CLI entry point and process execution
│   ├── lib.rs                # Public library exports
│   ├── core/                 # Core domain engine and AST pipeline
│   │   ├── mod.rs            # Core module declarations
│   │   ├── arguments.rs      # CLI argument parser and validator
│   │   ├── builder.rs        # HTML page assembler and asset injector
│   │   ├── constants.rs      # Global system metadata and defaults
│   │   ├── document.rs       # Document AST and element definitions
│   │   ├── error.rs          # Diagnostic compiler-style error types
│   │   ├── feature.rs        # Document AST feature scanner and traits
│   │   ├── format.rs         # Text formatting and escaping utilities
│   │   ├── language.rs       # Embedded locale loader
│   │   ├── markdown.rs       # Zero-alloc Markdown parser
│   │   └── utils/            # Domain-agnostic utilities (IO, Base64, MIME, Hasher, URI, Time)
│   └── features/             # Vertical slice feature modules
│       ├── mod.rs            # Feature registry and dispatcher
│       ├── bullet/           # Bullet list vertical slice
│       ├── code/             # Code block vertical slice
│       ├── core/             # Core base styles and client scripts
│       ├── image/            # Image display and lightbox slice
│       ├── ordered/          # Ordered list vertical slice
│       ├── table/            # Table rendering slice
│       ├── task/             # Interactive task checkbox slice
│       └── unknown/          # Unrecognized element fallback slice
├── tests/
│   └── integration_test.rs   # End-to-end and CLI integration test suite
├── build.rs                  # Build script for locales and Git version metadata
├── Cargo.toml                # Package definition and release profile
├── MAKE.sh                   # Automation workflow script
├── SPECIFICATION.md          # Project technical specification (Single Source of Truth)
├── AGENTS.md                 # AI agent directives and constraints
├── CHANGELOG.md              # Version release history
├── README.md                 # User guide and project overview
└── LICENSE                   # GPL-3.0-or-later license text
```
