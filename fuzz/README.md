# Fuzzing RE#

Coverage-guided fuzzing for the resharp engine, built on
[`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer +
AddressSanitizer).

## Prerequisites

- A nightly toolchain (`cargo-fuzz` injects `-Cllvm-args=-sanitizer-coverage-*`
  RUSTFLAGS that only compile on nightly): `rustup toolchain install nightly`.
- `cargo install cargo-fuzz`.
- A C/C++ compiler (clang or gcc) for the libFuzzer runtime.

This `fuzz/` crate is its own Cargo workspace, so the stable build at the repo
root is unaffected.

## Targets

- **`compile`** -- robustness of `Regex::with_options`. An arbitrary pattern is
  compiled under a sweep of option configurations (every `UnicodeMode`,
  `hardened`, and the inline-flag bundle). Parse / capacity / size errors are
  expected `Err`s; only a panic, stack overflow, OOM, or ASAN report is a
  finding. This is the primary target.
- **`match_invariants`** -- self-consistency of matching with no oracle. For any
  pattern that compiles and any haystack, asserts that matches are in-bounds and
  non-overlapping, that `find_all` non-empty iff `is_match`, and that
  `find_anchored` matches only at offset 0. Checks both the default and hardened
  engines.
- **`diff_regex`** -- differential `is_match` against the
  [`regex`](https://crates.io/crates/regex) crate, restricted to the syntax
  subset where the two engines share semantics (ascii literals, `.`, classes,
  alternation, concatenation, grouping, greedy quantifiers). Excludes anchors,
  `\b`, the perl `\w`/`\d`/`\s` classes, and all resharp-only operators, so
  leftmost-longest vs leftmost-greedy only changes match length, never
  existence. resharp runs in `UnicodeMode::Ascii` and regex runs with
  `.unicode(false)`.

## Running

```sh
# list targets
cargo +nightly fuzz list

# run a target (Ctrl-C to stop); the dictionary steers mutation toward
# resharp syntax
cargo +nightly fuzz run compile -- -dict=fuzz/dictionaries/resharp.dict

# time-boxed campaign
cargo +nightly fuzz run match_invariants -- \
  -dict=fuzz/dictionaries/resharp.dict -max_total_time=300

cargo +nightly fuzz run diff_regex -- -max_total_time=300
```

On Linux, `cargo-fuzz` may default to the musl target, whose static libc
conflicts with AddressSanitizer; if the build complains, force the gnu target:

```sh
cargo +nightly fuzz run compile --target x86_64-unknown-linux-gnu -- \
  -dict=fuzz/dictionaries/resharp.dict
```

## Reproducing a crash

```sh
# replay the exact input
cargo +nightly fuzz run <target> fuzz/artifacts/<target>/crash-<hash>

# minimize it
cargo +nightly fuzz tmin <target> fuzz/artifacts/<target>/crash-<hash>
```

Each crash message embeds a self-contained reproducer: the pattern source and,
for the match / diff targets, the haystack as a hex string.

## Corpus and artifacts

- `corpus/compile/seed-*` -- curated seeds exercising resharp features
  (intersection, complement, lookarounds, wildcards). Committed.
- `corpus/<target>/*` (other) -- libFuzzer corpus growth. Ignored.
- `artifacts/` -- crash reproducers. Ignored (raw fuzzer input).
- `Cargo.lock` -- committed so the fuzz toolchain stays reproducible.
