# Doc2Flow Agent Directives

> **[CRITICAL RULE]: YOU MUST COMMUNICATE AND WORK EXCLUSIVELY IN ENGLISH UNDER ALL CIRCUMSTANCES. DO NOT USE ANY OTHER LANGUAGE FOR RESPONSES, INLINE DOCS, OR COMMENTS (unless dealing with specific i18n content topics). 1-line concise AI responses.**

## 1. Arch & Stack
- **Spec:** `SPECIFICATION.md` (sync structure on central file changes)
- **Layering:** `src/utils/` (library), `src/core/` (engine), `src/features/` (slices).
- **CLI:** `std::env::args_os().skip(1)`, `OsStr`/`OsString` paths.
- **MD/Assets/UI:** Custom zero-alloc parser + GFM. Base64/WebP/compress. Zero-alloc HTML buffers.
- **i18n:** embedded JSON (`build.rs`).
- **Flow:** CLI > MD > Img > UI

## 2. Core Constraints & Guidelines
To save tokens, detailed rules have been migrated to agent Skills. **You must apply the rules from these skills when relevant:**

- **Rust Code:** Use `rust-file-analyzer-and-optimizer` for memory efficiency, zero-alloc, and strict idiomatic patterns (P-rules). Use `rust-code-structurer` for layout.
- **Frontend (HTML/JS/CSS):** Use `frontend-development-standards` and `javascript-module-architect`.
- **Testing & Ops:** Use `testing-and-ops-guidelines` for edge cases (M-rules) and OS/Git ops.
- **Build Workflows:** Use `run-build-workflows`.
- **Features & i18n:** Use `feature-module-builder` and `i18n-language-management`.
- **Documentation:** Use `readme-updater` for README structure, tone, and technical sync.
- **Changelog:** Use `changelog-updater` for `CHANGELOG.md` updates.

## 3. Meta
- **Text Edits:** Token-optimize for human readers. Reserve aggressive compression ONLY for `AGENTS.md` and Skill files (`SKILL.md`).