---
name: run-build-workflows
description: Executes MAKE.sh for builds, cross-compilation, tests, and HTML example generation.
---

# Build Workflows (`MAKE.sh`)

## When to use
Triggers: build (standard/optimized), cross-compile (Win/Linux), run cargo tests, generate HTML examples.

## How to use
Execute in project root: `./MAKE.sh [FLAGS]`

**Flags:**
- `--release`: Optimized build (LTO/size).
- `--release-windows`: Build Windows executable.
- `--release-linux`: Build Linux executable.
- `--tests`: Run cargo tests.
- `--examples`: Build project AND generate HTML examples.
- `--examples-only`: Generate HTML examples ONLY (no build).