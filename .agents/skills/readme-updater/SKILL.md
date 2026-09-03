---
name: readme-updater
description: Enforces professional standards, clear language, zero emojis, and accurate technical sync for README.md.
---
# README Updater
**Goal:** Maintain an accurate, serious, professional, and accessible `README.md` without emojis.

## USE WHEN
- Updating or auditing `README.md`.
- New features, CLI flags, frontmatter keys, or markdown syntax changes are introduced.

## CORE DIRECTIVES
1. **Tone & Style:**
   - **Language:** Exclusively English. Clear, simple, accessible phrasing for technical and non-technical readers.
   - **Zero Emojis:** Strictly NO emojis or decorative Unicode symbols in text, headers, badges, or tables.
   - **Formality:** Professional, serious, objective, and structured. No marketing hype or colloquialisms.
2. **Technical Truth & Sync:**
   - **Spec Alignment:** Synchronize with `SPECIFICATION.md`, `src/core/arguments.rs`, and `src/core/document.rs`.
   - **CLI Flags:** Keep all options (`-o`, `-l`, `-i`, `-s`, `-h`, `-V`), types, and defaults identical to code.
   - **Frontmatter:** Validate types and defaults (e.g. `header: bool = true`, `comments: bool = true`, `numbered_sections: bool = false`).
   - **Markdown Syntax:** Detail headers (H1/H2 collapsible/badges), checklists (`- [ ]`), callouts (`>`, `>?`, `>!`, `>!!`, `>!!!`), code variables (`:::variables`, `{{VAR}}`), tables, images/lightbox, and non-image download links.
   - **Client Features:** Document keyboard navigation (`Enter`/`Space`), live search (`Ctrl+K`), item comments, footer action bar (PDF export, save state, reset), and deterministic `localStorage` scoping (`d2f_id`).
   - **Build & Install:** Maintain accurate Rust/Cargo requirements (Edition 2024) and target paths (`d2f`/`d2f.exe`).

## STANDARD STRUCTURE
1. **Header & Badges:** Title, version, license, Rust edition, platforms (clean badges, zero emojis).
2. **Overview:** Pitch, self-contained single-file architecture, offline persistence.
3. **Key Capabilities:** Bulleted summary of major features.
4. **Installation:** Prerequisites, git clone, cargo build release, binary output paths.
5. **Quick Start:** `--init` template, standard compile, advanced compilation example.
6. **CLI Reference:** Usage grammar, argument table, options table with types/defaults.
7. **Document Configuration:** YAML frontmatter example and complete options reference table.
8. **Markdown Syntax Guide:** Headings, checklists, notice boxes, variables/code blocks, tables, images.
9. **Interactive Browser Features:** Search, comments, footer actions, local persistence.
10. **Technical Principles:** Safe Rust, zero deps, zero-alloc performance, HTML5 accessibility.
11. **License:** SPDX/GPL-3.0-or-later reference.

## WORKFLOW
1. Audit changes in `SPECIFICATION.md` and codebase (`src/core/arguments.rs`, `src/core/document.rs`).
2. Update `README.md` following standard structure and simple, professional English.
3. Verify zero emojis across the file (`grep`/script).
4. Verify markdown formatting, tables, and internal/external links.
