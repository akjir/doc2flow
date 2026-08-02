---
name: doc2flow
description: Guides usage, build processes, CLI commands, and Markdown conversion workflows for doc2flow (d2f). Activate this skill when building the project with MAKE.sh, generating HTML flowcharts from Markdown files, or developing doc2flow features.
---

# Doc2Flow (`d2f`) Skill Guide

**Doc2Flow (`d2f`)** is a fast, lightweight command-line tool written in Rust that converts Markdown documents into standalone, interactive HTML guides, protocols, and manuals with zero external dependencies.

---

## 1. Build Workflows (`MAKE.sh`)
- **Build/Test/Make:** `./MAKE.sh [--release] [--tests] [--examples]` (LTO/Size, Cargo Tests, md->HTML)

## 2. CLI `d2f` (Path: `./target/{debug,release}/d2f`)
- `d2f in.md`: Convert (auto-names in.html)
- `d2f in.md -o out.html`: Custom output
- `d2f in.md -l logo.png`: Custom logo (SVG/PNG/JPG/WebP)
- `d2f in.md -s`: Compress local images >250KB to WebP
- `d2f -i out.md`: Generate starter template

## 3. MD Features
- **Sections:** `## Heading` = collapsible `.section`
- **Tasks:** `- [ ]`/`- [x]` = dynamic progress + localStorage
- **Vars:** `[Variables]` tables replace `{{VAR}}` in code blocks (copy/print)
- **Callouts:** `> [!NOTE|TIP|IMPORTANT|WARNING|CAUTION]`
- **Assets:** Local images -> Base64 URIs (zero-dep HTML)
- **i18n:** YAML frontmatter `language: "de"|"en"`

## 4. Arch Rules
- **Rust:** No `unsafe`.
- **Alloc:** `Cow<'a, str>`, zero-copy slicing, pre-alloc buffers.
- **Assets:** Embedded via `include_str!` (i18n, HTML/CSS/JS).
- **Paths:** Strict `std::path::PathBuf` (Linux dev -> Win64 target).