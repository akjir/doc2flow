#!/usr/bin/env bash
set -euo pipefail

# Navigate to the repository root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

show_help() {
    cat << 'EOF'
Usage: ./MAKE.sh [FLAGS]

Build workflows for Doc2Flow.

Flags:
  -h, --help            Show this help message and exit
  --release             Optimized build (release mode)
  --release-windows     Build Windows executable (x86_64-pc-windows-gnu)
  --release-linux       Build Linux executable (x86_64-unknown-linux-gnu)
  --tests               Run cargo tests
  --examples            Build project and generate HTML examples
  --examples-only       Generate HTML examples only (skip TypeScript & Cargo builds)
EOF
}

BUILD_EXAMPLES=false
EXAMPLES_ONLY=false
RUN_TESTS=false
CARGO_ARGS=()

for arg in "$@"; do
    case "$arg" in
        -h|--help)
            show_help
            exit 0
            ;;
        --examples-only)
            BUILD_EXAMPLES=true
            EXAMPLES_ONLY=true
            ;;
        --examples)
            BUILD_EXAMPLES=true
            ;;
        --tests)
            RUN_TESTS=true
            ;;
        --release)
            CARGO_ARGS+=("--release")
            ;;
        --release-windows)
            CARGO_ARGS+=("--release" "--target" "x86_64-pc-windows-gnu")
            ;;
        --release-linux)
            CARGO_ARGS+=("--release" "--target" "x86_64-unknown-linux-gnu")
            ;;
        *)
            echo "Error: Unknown argument '$arg'" >&2
            echo "Run './MAKE.sh --help' for usage information." >&2
            exit 1
            ;;
    esac
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
