#!/usr/bin/env bash
set -euo pipefail

# Navigate to the repository root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BUILD_EXAMPLES=false
RUN_TESTS=false
CARGO_ARGS=()

for arg in "$@"; do
    if [ "$arg" = "--examples" ]; then
        BUILD_EXAMPLES=true
    elif [ "$arg" = "--tests" ]; then
        RUN_TESTS=true
    else
        CARGO_ARGS+=("$arg")
    fi
done

echo "==> Building TypeScript..."
(cd web && npm run build)

echo "==> Running Cargo build..."
if [ ${#CARGO_ARGS[@]} -gt 0 ]; then
    cargo build "${CARGO_ARGS[@]}"
else
    cargo build
fi

if [ "$RUN_TESTS" = true ]; then
    echo "==> Running tests..."
    if [ ${#CARGO_ARGS[@]} -gt 0 ]; then
        cargo test "${CARGO_ARGS[@]}"
    else
        cargo test
    fi
fi

if [ "$BUILD_EXAMPLES" = true ]; then
    echo "==> Building examples..."
    D2F_BIN="./target/debug/d2f"
    for arg in "${CARGO_ARGS[@]:-}"; do
        if [ "$arg" = "--release" ]; then
            D2F_BIN="./target/release/d2f"
            break
        fi
    done

    for file in examples/*.md; do
        if [ -f "$file" ]; then
            echo "Building $file..."
            "$D2F_BIN" "$file"
        fi
    done
fi
