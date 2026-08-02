#!/usr/bin/env bash
set -euo pipefail

# Navigate to the repository root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BUILD_EXAMPLES=false
EXAMPLES_ONLY=false
RUN_TESTS=false
CARGO_ARGS=()

for arg in "$@"; do
    if [ "$arg" = "--examples-only" ]; then
        BUILD_EXAMPLES=true
        EXAMPLES_ONLY=true
    elif [ "$arg" = "--examples" ]; then
        BUILD_EXAMPLES=true
    elif [ "$arg" = "--tests" ]; then
        RUN_TESTS=true
    elif [ "$arg" = "--release-windows" ]; then
        CARGO_ARGS+=("--release" "--target" "x86_64-pc-windows-gnu")
    elif [ "$arg" = "--release-linux" ]; then
        CARGO_ARGS+=("--release" "--target" "x86_64-unknown-linux-gnu")
    else
        CARGO_ARGS+=("$arg")
    fi
done

if [ "$EXAMPLES_ONLY" = false ]; then
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
fi

if [ "$BUILD_EXAMPLES" = true ]; then
    echo "==> Building examples..."
    D2F_BIN="./target/debug/d2f"
    for ((i=0; i<${#CARGO_ARGS[@]}; i++)); do
        if [ "${CARGO_ARGS[$i]}" = "--target" ] && [ $((i+1)) -lt ${#CARGO_ARGS[@]} ]; then
            TARGET_NAME="${CARGO_ARGS[$((i+1))]}"
            if [[ "$TARGET_NAME" == *"windows"* ]]; then
                D2F_BIN="./target/${TARGET_NAME}/release/d2f.exe"
            else
                D2F_BIN="./target/${TARGET_NAME}/release/d2f"
            fi
        elif [ "${CARGO_ARGS[$i]}" = "--release" ]; then
            D2F_BIN="./target/release/d2f"
        fi
    done

    if [ ! -f "$D2F_BIN" ]; then
        echo "Error: Binary $D2F_BIN not found. Build the project first." >&2
        exit 1
    fi

    for file in examples/*.md; do
        if [ -f "$file" ]; then
            echo "Building $file..."
            "$D2F_BIN" "$file"
        fi
    done
fi
