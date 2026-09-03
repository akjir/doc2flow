---
name: testing-and-ops-guidelines
description: Core operations, testing rules, and Git conventions for Doc2Flow.
---
# Testing & Ops Guidelines
**Goal:** Enforce strict testing standards, OS compatibility, and Git workflows.

## USE WHEN
- Writing unit or integration tests.
- Committing code.
- Managing OS-level paths and I/O.

## Ops & Build Execution
- **OS:** Linux dev, Win64 target. `std::path::Path/Buf` ONLY.
- **Git:** Commit ONLY if requested AND tests pass (or user overrides).
- **Build/Test/Examples:** ALL building, testing, and example generation MUST use `./MAKE.sh` (e.g. `./MAKE.sh --tests`, `./MAKE.sh --examples`). Do NOT use `cargo build` or `cargo test` directly unless specifically troubleshooting.

## Testing Rules (Priority 1)
- **Negative/Edge Cases:** Must test failures, bounds, and edge cases.
- **Path Edge Tests (M-PATH-EDGE-TESTS):** Mandatory filesystem edge-case tests (hidden files `.env`, trailing dots `file.`, compound extensions `.tar.gz`, trailing slashes `/`) for all `std::path::Path`/`PathBuf` inspection logic.
- **I/O Tests (M-IO-TESTS):** Mandatory temporary filesystem tests (`std::env::temp_dir()`) or memory cursors for I/O-bound functions, file resolvers, and image encoders.
- **Fallback Tests (M-FALLBACK-TESTS):** Explicit tests for fallback/default `_ => {}` arms.
- **Extreme Bound Tests (M-EXTREME-BOUND-TESTS):** Mandate extreme edge-case unit tests (`usize::MAX`, `0`, bounds) for string length math & buffer sizing.
- **Delimiter Injection Tests (M-DELIM-INJECT-TESTS):** Mandate boundary bleeding / delimiter injection unit tests (e.g. `A:`+`B` vs `A`+`:B`) on all composite ID and hash generators.
- **Semantic Assertions:** Use semantic token assertions (e.g. `.contains("bullet")`) over exact full strings on formatted output (`Display`).
- **Global Locks (M-GLOBAL-TEST-LOCK):** Shared global mutable state (`LazyLock`/`RwLock`) MUST synchronize in `#[cfg(test)]` via a dedicated test `Mutex<()>` .
- **UI Changes:** Regen `showcase_*.html` on UI changes.
