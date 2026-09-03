---
name: run-build-workflows
description: Executes MAKE.sh for builds, cross-compilation, tests, and HTML example generation.
---
# Build Workflows (`MAKE.sh`)

## Use
Triggers build, cross-compile, tests, HTML examples.

## How
Root: `./MAKE.sh [FLAGS]`

**Flags:**
- `-h`, `--help`: Help
- `--release`: Opt build
- `--release-windows`: Win exe
- `--release-linux`: Lin exe
- `--tests`: Cargo tests
- `--examples`: Build + HTML examples
- `--examples-only`: HTML examples ONLY