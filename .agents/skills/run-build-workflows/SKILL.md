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
- `-h`, `--help`: Show help.
- `--release`: Optimized build (LTO/size).
- `--release-windows`: Build Windows executable.
- `--release-linux`: Build Linux executable.
- `--tests`: Run cargo tests.
- `--examples`: Build project AND generate HTML examples.
- `--examples-only`: Generate HTML examples ONLY (no build).
- `--experimental-building`: [BRANCH EXPERIMENT: REVERT ON MERGE] Full TS/Cargo build, run tests, render `examples/template_exp.html`. Cannot combine with other flags.
- `--experimental-building-without-tests`: [BRANCH EXPERIMENT: REVERT ON MERGE] TS/Cargo build, render `examples/template_exp.html` (no tests). Cannot combine with other flags.