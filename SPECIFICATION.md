# Project Specification: Doc2Flow (d2f)

> [!IMPORTANT]
> This file contains the authoritative project specifications. AI agents are strictly forbidden from modifying this file without explicit approval or instruction from the user.

## 1. Overview & Objectives
Doc2Flow (`d2f`) is a command-line interface (CLI) tool built for Windows that converts Markdown files into fully self-contained HTML files. The generated HTML files serve as interactive guides, manuals, and checklists for end users.

### Core Principles & Non-Negotiables
- **Single Binary Output:** The build must result in a single executable file (`d2f.exe`) with no external runtime dependencies.
- **Zero-Dependency HTML:** The generated HTML file must contain all necessary assets (CSS, JS, images via Base64) embedded directly within it. There must be absolutely no references to external servers or local directories.
- **Integrated Templates:** All required HTML/CSS/JS templates must be embedded into the binary at compile time (e.g., using `include_str!`).

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

# Help text & Version
d2f.exe --help
d2f.exe --version
```

### Parameters & Arguments

| Argument / Flag | Short | Description | Required? | Default |
| --- | --- | --- | --- | --- |
| `INPUT` | — | Path to the source Markdown file | **Yes** | — |
| `OUTPUT` | `-o`, `--output` | Target path for the generated HTML file | No | `<INPUT_NAME>.html` |

---

## 3. Input Specification (Markdown Requirements)

* **Standard:** GitHub Flavored Markdown (GFM).
* **Checklists:** Support for interactive checkbox elements (`- [ ]` and `- [x]`).
* **Local Images:** Relatively linked images (e.g., `![Alt-Text](./images/graphic.png)`) must be resolved locally by `d2f` during processing, converted into **Base64**, and embedded directly as a `data:image/...;base64,...` URI in the `src` attribute of the HTML `<img>` tag.

---

## 4. Output Specification (HTML & UX)

* **Self-Contained Document:** Generates a valid HTML5 document with fully embedded styling (`<style>`) and scripts (`<script>`).
* **Interactivity (Checklists):**
  * Checkboxes in the HTML view can be toggled (checked/unchecked) by the user.
  * The state of these checkboxes must be stored in the browser's `localStorage` so that user progress is maintained upon reloading the page.
* **Layout & Design:** Clean, modern, and responsive (optimized for both desktop and mobile views). Must include optimized print CSS for physical printing or PDF generation.

---

## 5. Roadmap & Architecture Preparation (Multi-File & Directories)

To enable future expansions without requiring significant refactoring, the internal processing pipeline should be designed with abstraction in mind:

* **Pipeline Design:** The engine should expect the Markdown input internally as a modular content stream with already resolved image paths, rather than a rigid single-file reference.
* **Phase 1 (MVP):** Processing of exactly one Markdown file.
* **Phase 2 (Future):** Support for merging multiple files (`d2f m1.md m2.md -o out.html`) and processing entire directories (`d2f ./docs/`).

---

## 6. Technical Framework & Quality Standards

* **Programming Language:** Rust (Edition 2021).
* **Target Platform:** Windows 64-Bit (`x86_64-pc-windows-msvc`).
* **File Size:** Target size of `d2f.exe` should be `< 10 MB` (utilizing binary stripping, LTO, and release optimizations).
* **Error Handling:**
  * No panics or crashes when encountering invalid paths or missing permissions.
  * Human-readable, color-coded error messages output to `stderr` (consider using crates like `anyhow` or `thiserror`).
* **Testing:**
  * Unit tests for Markdown parsing and Base64 conversion of images.
  * Integration tests for CLI input parameters and end-to-end HTML generation.
