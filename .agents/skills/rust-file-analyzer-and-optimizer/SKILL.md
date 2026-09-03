---
name: rust-file-analyzer-and-optimizer
description: Analyzes/optimizes Rust code for memory efficiency, zero-allocation, idiomatic patterns, and performance (Doc2Flow context).
---
# Rust Analyzer & Optimizer
**Goal:** Enforce high-perf, 0-copy, min-alloc, idiomatic Rust.

## USE WHEN
- Auditing `src/*.rs` for bottlenecks.
- Eliminating heap allocs (`.to_string()`, `format!`).
- Optimizing I/O buffers.
- Enforcing 0 `unsafe` & strict errors.

## WORKFLOW
1. Scan (5 Pillars). 2. Trade-Off (Perf > Readability, but no micro-obfuscation). 3. Output Full Module. 4. Summary rationale.

## 5 PILLARS

### 1: Mem & Alloc
- **Borrow:** `&str`, `&[u8]`, `Cow<'a, str>`. NO `String` params.
- **0 Waste:** Drop `.to_string()`, `.to_owned()`, `PathBuf::from()`.
- **0-Copy:** `.split_once()`, `.strip_prefix()`. NO intermediate `Vec`.
- **Safe Slice:** Cursor offsets. NO `as_ptr` diffs.
- **Cap:** ALWAYS `.with_capacity()`. NO magic size `[T;8]`, use named `const MAX_CAPACITY`. Clamp dynamic lengths to prevent OOM (P-BOUND-ALLOC).
- **Idioms:** `str::repeat()` over iter loops (P-IDIOM-REPEAT).
- **State:** `bitflags`/arrays over `&&`/`||` bools. Assert mask limits (P-MASK-SAFE).
- **Lookup:** `HashMap.get()` with borrowed keys. 0 alloc on query (P-ZERO-COST-LOOKUP).
- **Hash:** Stream `Hasher::update`. NO buffer concat (P-STREAM-HASH).

### 2: Parse & Loops
- **Regex:** NO chained `.replace()`. Use state machine scanners.
- **Iterators:** `.filter()`, `.map()`, `.fold()` > imperative loops.
- **Unified Scanners:** 0 dup forward loops. Reusable generic iterators (P-UNIFIED-PARSER).
- **Linear:** O(N) strict. 0 redundant scan (P-NO-QUADRATIC).
- **Modular:** Loop >40 lines -> extract stateless helpers (P-LOOP-MODULARITY).
- **Compression:** Math-based resize, 1-2 passes max (P-EFFICIENT-IO-LOOPS).
- **JSON:** Decode UTF-16 surrogates (P-JSON-SURROGATE), strict RFC 8259 primitives (P-JSON-STRICT-PRIMITIVES), 0 deps (P-STDLIB-ONLY).
- **Case:** `eq_ignore_ascii_case` > alloc `.to_ascii_lowercase()` (P-ZERO-ALLOC-CASE).

### 3: Formatting & Buffers
- **Static:** `out.write_str("...")`.
- **Dynamic:** `write!(out, "...", vars)`. NO fragmentation to avoid `write!`.
- **Errors:** `format!` > manual alloc micro-opts.

### 4: Idioms & Arch
- **Traits:** standard (`Display`). NO custom dup fns.
- **Constructors:** `Default` for ZSTs. `new()` MUST delegate to `Default`. Enforce `#[must_use]`.
- **Regs:** `[&'static dyn Trait; N]` static arrays. O(1) routing via masks/indices. NO `ptr::eq` or loops (P-O1-DISPATCH).
- **Tripwire Tests:** Explicit lengths on static arrays.
- **Dispatchers:** Flat (<50 lines). Delegate `try_parse_*` (P-FLATTEN-DISPATCH).
- **UTF-8:** Encapsulate lookarounds (`is_alphanumeric_at`) (P-UTF8-LOOKAROUND).
- **Layer:** `src/utils/` -> `src/core/`/`src/features/`.
- **Err:** `Doc2FlowError`, typed enums, `#[source]` chaining (P-ERR-PRESERVE). `unwrap()` for invariants only. NO `unsafe`.
- **Consts:** Local in `module.rs`.
- **CLI:** Pure iter/`OsString`, handle `=`, reject empty (P-CLI-PURE).
- **Attrs:** `#[inline]` on hot-path only (P-ATTR-USAGE).
- **Docs:** Inline domain quirks. Truthy bool matrix (`true,yes,1`).
- **Tests:** Semantic asserts. Fallback `_ =>` (M-FALLBACK-TESTS), extreme bounds (M-EXTREME-BOUND-TESTS), path edges (M-PATH-EDGE-TESTS), delimiter inject (M-DELIM-INJECT-TESTS), temp dir I/O (M-IO-TESTS).
- **Crypto:** Delimit with length-prefix or `\x00` (P-CRYPTO-KEY-SEP).
- **Combinators:** `.filter()`. NO if/else in closures (P-FUNCTIONAL-COMBINATORS).

### 5: HTML, XML, Assets
- **Scanners:** O(N) 0-alloc single-pass. Quote-aware (`'`,`"`).
- **Sub-parsers:** Isolate tag/comment/cdata parsers.
- **XML:** Robust detect `<?xml`, `<!--`, `<!DOCTYPE` before `<svg` (P-ROBUST-XML-DETECT).
- **HTML:** Tolerant whitespace/quotes (P-DEFENSIVE-HTML), flexible unwrapping (P-DECOUPLED-DOM).
- **Base64:** Exact pre-alloc `to_base64_data_uri`.
- **Print:** NO `println!` in core logic. Use stderr.

> [!NOTE]
> **[BRANCH EXP: feature/modular-building - REVERT ON MERGE]**
> Exp `src/exp/` ONLY. Prod frozen. Apply pillars to duplicated code.