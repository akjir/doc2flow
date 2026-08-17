# Doc2Flow Agent Directives

## 1. Arch & Stack
- **Spec:** `SPECIFICATION.md` (sync structure on central file changes)
- **Layering:** `src/utils/` (generic library), `src/core/` (domain engine), `src/features/` (vertical slices). Core/features access `utils` through `src/utils/mod.rs` API.
- **CLI:** `std::env::args_os().skip(1)` (pure parser: callers strip binary; `OsStr`/`OsString` paths)
- **MD:** Custom zero-alloc parser + GFM
- **Assets:** Custom Base64/MIME (`src/utils/`), WebP/compress (`src/core/image.rs`)
- **i18n:** `HashMap` via embedded JSON (`build.rs`)
- **UI/HTML:** Zero-alloc buffers (`src/core/components.rs`, `src/core/builder.rs`), compile-time embeds (`include_str!`)
- **Pipelines:** DRY template contexts (`build_template_vars`), parity across entry points (identical conditional components; NO hardcoded blanks). Single predicate feature dispatch (`is_feature_active`).
- **Flow:** CLI > MD > Img > UI

## 2. Rust
- **Traits:** Rely on standard traits (`std::fmt::Display` -> `.to_string()`). NEVER create custom duplicate methods (e.g. `to_string_custom()`) or standalone functions.
- **Default over New:** Strictly implement `std::default::Default` for all parameter-less structs and Zero-Sized Types (ZSTs).
- **Delegated Constructors:** BANNED: Implementing a standalone `new()` method that duplicates field initialization. IF `new()` is required for API ergonomics, it MUST strictly delegate to `Self::default()`.
- **Zero-Allocation Registries:** Enforce static array definitions (`[&'static dyn Trait; N]`) for central registries (e.g., feature modules) to guarantee O(1) startup and zero runtime heap allocation.
- **Tripwire Testing:** Use explicit lengths and hardcoded arrays in registry tests to force manual verification when expanding system features (e.g., adding a new module).
- **State:** Multiple boolean flags -> evaluate `bitflags` or array-backed state (min memory footprint, avoid brittle `&&`/`||` chains).
- **Dispatch:** O(1) routing ONLY (precompute array indices/masks at init). NO loops/strings/`ptr::eq` on hot-path AST handling (P-O1-DISPATCH).
- **Bitmask:** Integer bitmasks (`u32`) MUST assert `len <= bit_width` at init (P-MASK-SAFE).
- **Buffers:** NO magic capacities (`[T; 8]`). Define named `const MAX_CAPACITY: usize` with `debug_assert!` against truncations (P-NO-MAGIC-CAP).
- **Alloc Safety:** Clamp dynamic alloc lengths (`with_capacity`, padding, gutters) against validated bounds to prevent OOM panics (P-BOUND-ALLOC). Prefer `str::repeat()` over manual iterator/`extend` loops for repetition (P-IDIOM-REPEAT).
- **Core:** Idiomatic, newtypes, 1-path exports, NO `unsafe`
- **Clean:** Zero legacy debt/compat shims. Remove dead/obsolete code when adding new code.
- **Consts:** Feature constants local in `src/features/<name>/module.rs` (NO central dumpster). App metadata/limits ONLY in `src/core/constants.rs`.
- **CLI:** Pure parser inputs (callers strip binary with `args_os().skip(1)`). OS-agnostic paths via `std::ffi::OsStr`/`OsString` (no UTF-8 assumption). Identical validation for space (`-o ""`) vs equals (`-o=`) syntax; reject empty values uniformly (`val.as_ref().is_empty()`). Avoid fragile flag peeking: require explicit `=` or strict bounds for optional values/hyphenated args (P-CLI-PURE).
- **Errors:** Stdlib+`Doc2FlowError` (NO `anyhow`/`eyre`). Strongly typed error enums for modules/parsing (NO `Result<T, String>`). `Result`=expected. `panic!`=bugs/stop (detailed msgs). NO `catch_unwind`. Safe bounds/slicing on diagnostic buffers. `From` conversions (NO `.to_<domain>()`). NO manual buffer micro-allocs on error paths; use `format!` or static strings. Retain error context via `#[source]` chaining (wrap external library errors in enum variants, never flatten into `Error::Message(format!(...))`) (P-ERR-PRESERVE).
- **Attributes:** Enforce `#[must_use]` on all constructors, factories, and pure builders (`new`, `with_capacity`). Reserve `#[inline]` exclusively for trivial getters/wrappers and hot-path trait implementations/loops. BANNED: `#[inline]` on internal utilities, large/branching functions, large `match` blocks, complex string operations, heap allocs (`String::with_capacity`), I/O, multi-branch, init, setup, CLI parsing, or parser helpers without cross-crate profiling (P-ATTR-USAGE). Rely on LTO and compiler heuristics.
- **Zero-Alloc Case Insensitivity:** For case-insensitive string matching without heap allocs, strictly use `eq_ignore_ascii_case` inside match guards; NEVER allocate via `.to_ascii_lowercase()` (P-ZERO-ALLOC-CASE).
- **Docs:** English ONLY (all inline docs & comments). 15-word max start, canonical headers (Examples/Errors/Panics), NO meta/journals. Explicitly document intentional domain quirks inline (e.g. strict H1/H2->H3 AST nesting for UI layout) to prevent regressions.
- **Perf:** Min-alloc (borrow>owned), `with_capacity`, O(N) 1-pass, zero-copy (`split_once`,`strip_prefix`), `Cow`. Safe subslice indexing ONLY; NO raw pointer arithmetic (`as_ptr` diffs) for string bound searches.
- **Compression Loops:** Never use iterative scaling loops for compression targets (e.g. shrinking image by 10% repeatedly). Calculate target dimensions mathematically (area-to-byte ratio) to limit heavy encoding operations to 1-2 passes maximum (P-EFFICIENT-IO-LOOPS).
- **Parsing:** Flexible boolean deserialization from maps/frontmatter/headers; account for case-insensitive truthy matrix (`true`, `yes`, `y`, `1`).
- **Parser Loops:** Unified forward cursors (NO duplicated scanning loops). Encapsulate index advancement & code/token skipping in generic higher-order functions/iterators (P-UNIFIED-PARSER).
- **Dispatchers:** Flatten monolithic dispatchers (max 50 lines). Dispatch to discrete, strongly-typed `try_parse_* -> Option<usize>` functions (P-FLATTEN-DISPATCH).
- **UTF-8 Lookaround:** Abstract UTF-8 lookahead/lookbehind into semantically named helper functions (e.g. `is_alphanumeric_at`). NO inline char-boundary math in core logic loops (P-UTF8-LOOKAROUND).
- **Linear Parsing:** O(N) linear parsing ONLY. Validate deeply nested/unclosed tokens prevent quadratic O(N²) scanning (cache boundaries or linear scan) (P-NO-QUADRATIC).
- **Loop Modularity:** String-processing loops (>40 lines) MUST extract core logic into stateless, isolated processor functions; orchestrators remain purely structural (P-LOOP-MODULARITY).
- **Zero-Cost Lookups:** NEVER unconditionally clone owned keys (`PathBuf`, `String`) querying `HashMap`/`BTreeMap` in loops; query `.get()` with borrowed keys, allocating ONLY on insertion (P-ZERO-COST-LOOKUP).
- **String/Buffer:** Exact `with_capacity` pre-alloc. Direct buffer streaming (`write_str`/`push_str`). NO intermediate `Vec`/strings on hot paths.
- **HTML/XML/SVG:** Zero-alloc tokenizers (O(N) 1-pass forward cursor). Quote-aware (single `'`, double `"`, multiline, escaped `\"`/`\'`). Sub-parsers for declarations (`<?`), DOCTYPE, comments (`<!--`), CDATA (`<![CDATA[`), tags. NO redundant scanning passes over attribute names/values. NO `println!` in core processing routines.
- **Robust XML/SVG Detection:** BANNED: naive `.starts_with("<svg")` prefix-only matching for raw XML/SVG payloads. Payload detection MUST account for `<?xml ... ?>`, `<!DOCTYPE ... >`, and comment headers (`<!--`) before `<svg` (P-ROBUST-XML-DETECT).
- **Functional Combinators:** BANNED: imperative `if/else` inside `Option`/`Result` closures (`.or_else(|| ...)`). Mandate declarative chaining (`.filter()`, `.map()`, `.and_then()`). Standardize optional string emptiness filtering on `.as_deref().filter(|s| !s.trim().is_empty())` (P-FUNCTIONAL-COMBINATORS).
- **Defensive HTML Parsing:** Manual string parsing of HTML MUST tolerate arbitrary whitespace, case-insensitivity, and single/double quote boundaries (`'`,`"`) (P-DEFENSIVE-HTML).
- **Decoupled DOM Assumptions:** Structural HTML modifications (unwrapping tags) MUST NOT rely on exact byte-for-byte matches; use flexible attribute & tag parsing with whitespace trimming (P-DECOUPLED-DOM).
- **Flow:** `match`/tables > `if-else`. Iterators > loops. `write_str`(static)/`write!`(dynamic) > `format!` (hot-path buffers).
- **Async State:** BANNED: `thread_local!` for app/session state in async/multi-thread runtimes (state loss on thread hop). Pass context or use thread-safe global sync (`RwLock`, `Mutex`, `arc-swap`) (P-ASYNC-STATE).
- **Zero-Copy JSON:** NO zero-copy `&str` JSON deserialization on unvetted data/escapes (`\n`, `\uXXXX`). Default to owned `String` in map values to prevent silent parse errors (P-OWNED-JSON).
- **Zero-Dep JSON:** Custom JSON parsers MUST explicitly support UTF-16 surrogate pair decoding (`\uD800..\uDBFF` + `\uDC00..\uDFFF`). Relying solely on `char::from_u32` for 4-digit hex escapes is strictly prohibited (fails outside BMP/emojis) (P-JSON-SURROGATE).
- **Strict Primitives:** Unquoted JSON values MUST strictly validate against RFC 8259 (`true`, `false`, `null`, numbers). Permissive "read until delimiter" accepting arbitrary bare words or malformed floats is strictly prohibited; emit explicit error (P-JSON-STRICT-PRIMITIVES).
- **Stdlib Only:** ZERO external dependencies (`serde_json`, `nom`) in zero-dependency core engine components. String manipulation and validation must utilize standard library functionality exclusively (`str::from_utf8`) (P-STDLIB-ONLY).
- **Static Assets:** NEVER swallow deserialization errors (`.unwrap_or_default()`, `.ok()`) on embedded static assets. Use `.expect()` to fail fast at startup (P-FAILFAST-STATIC).
- **Honest Returns:** NEVER return `Cow` when all code paths return `Cow::Owned` (e.g. data behind short-lived locks). Use explicit `String` or `Arc<str>` (P-HONEST-RETURNS).
- **I18n Naming:** Standardize on `localize()` (alias `t()`) for static dictionary lookup. BANNED: `translate()` (P-I18N-NAMING).
- **Crypto Delimiters:** NEVER use raw printable delimiters (`:`, `|`, `,`) without escaping inputs for hashes/composite keys. Mandate length-prefixing or strict null-byte (`\x00`) delimiters + sanitization (P-CRYPTO-KEY-SEP).
- **Streaming Hasher:** NO intermediate `String`/`Vec<u8>` heap allocs solely to concatenate data for hashing. Stream sequentially via `Hasher::update` / `Sha256::update`; alloc ONLY final output (P-STREAM-HASH).
- **Build:** `lto=true`, `opt=z|s`, `codegen-units=1`, strip. Favor stdlib over deps.

## 3. Ops & Tests
- **Comm:** English ONLY. 1-line concise AI responses.
- **OS:** Linux dev, Win64 target. `std::path::Path/Buf` ONLY.
- **Git:** Commit ONLY if requested AND tests pass (or user overrides).
- **Test:** Priority 1. Negative/edge cases. Mandatory filesystem edge-case tests (hidden files `.env`, trailing dots `file.`, compound extensions `.tar.gz`, trailing slashes `/`) for all `std::path::Path`/`PathBuf` inspection logic (M-PATH-EDGE-TESTS). Mandatory temporary filesystem tests (`std::env::temp_dir()`) or memory cursors for I/O-bound functions, file resolvers, and image encoders (M-IO-TESTS). Semantic token assertions (e.g. `.contains("bullet")`) over exact full strings on formatted output (`Display`). Explicit tests for fallback/default `_ => {}` arms (M-FALLBACK-TESTS). Mandate extreme edge-case unit tests (`usize::MAX`, `0`, bounds) for string length math & buffer sizing (M-EXTREME-BOUND-TESTS). Mandate boundary bleeding / delimiter injection unit tests (e.g. `A:`+`B` vs `A`+`:B`) on all composite ID and hash generators (M-DELIM-INJECT-TESTS). Shared global mutable state (`LazyLock`/`RwLock`) MUST synchronize in `#[cfg(test)]` via a dedicated test `Mutex<()>` (M-GLOBAL-TEST-LOCK). Regen `showcase_*.html` on UI changes.

## 4. Frontend (HTML/JS/CSS)
- **HTML (Generic):**
  - `##` -> `.section.sh.sb` (collapsible)
  - `[ ]` List -> `.check-item`
  - Quotes (`>`,`>?`,`>!`) -> `.note` variants
  - Local Img -> Base64; Remote -> `<img>`; Non-img asset -> `<a>.check-item.text-item`
  - Vars replaced via frontmatter
- **JS:** Vanilla JS ONLY (`.js`). ZERO TS/build steps. NO `export`/`import`; decouple via `window.d2f` namespace (`window.d2f.<module>`).
- **CSS:**
  - `:root` vars, BEM classes, ZERO external deps (fonts)
  - Print: Hide UI/buttons, expand collapsed (`display:block!important`), natural page breaks (no forced), exact print colors, no strikethrough
  - File: 1.Base > 2.Layout > 3.Components > 4.Print > 5.Responsive

## 5. Changelog
- **Rule:** User-facing/bugfixes ONLY under `[Unreleased]`. 1-line bullets.
- **BANNED:** Internal refactors/tests. No empty `[Unreleased]`.

## 6. Meta / Docs
- **Text Edits:** Token-optimize for human readers (balanced). Reserve aggressive compression ONLY for `AGENTS.md` and Skill files (`SKILL.md`).
- **Self-Editing:** Maintain aggressive token compression when updating `AGENTS.md` or Skill files.