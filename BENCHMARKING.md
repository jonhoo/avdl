# Benchmarking avdl

This document describes how to benchmark the Rust `avdl` tool,
including generating synthetic inputs for scaling tests and profiling
CPU hot spots.

## Prerequisites

- **Rust** (release build): `cargo build --release`
- **Java** (for comparison): `java -jar avro-tools-1.12.1.jar`
  (the JAR lives in the parent directory of the main checkout)
- **[hyperfine][]** for wall-clock benchmarks
- **perf** and **[inferno][]** for CPU profiling and flamegraphs

[hyperfine]: https://github.com/sharkdp/hyperfine
[inferno]: https://github.com/jonhoo/inferno

### Build configuration for profiling

Two build settings make profiling output more useful:

1. **Debug info in release builds** (`Cargo.toml`):

   ```toml
   [profile.release]
   debug = true
   ```

   This embeds DWARF debug info so `perf` can resolve function names
   and source locations. It does not affect optimization level.

2. **Frame pointers** (`.cargo/config.toml`):

   ```toml
   [target.'cfg(all())']
   rustflags = ["-C", "force-frame-pointers=yes"]
   ```

   This enables frame-pointer-based stack unwinding, which is faster
   and more reliable than DWARF unwinding for `perf record`. The
   overhead on runtime performance is negligible.

Both are already configured in this repository.

## Generating synthetic inputs

Real-world `.avdl` files rarely exceed ~50 KB. To stress-test
scaling behaviour, we generate larger inputs by duplicating the named
types from a base file with suffixed names (so the protocol still
parses cleanly).

### Base file

Use one of the larger real-world `.avdl` files as a base. CDM20 is a
good choice — it has ~40 named type definitions in ~1,480 lines
(~52 KB).

### How to multiply

The approach is:

1. Parse out the protocol header (`protocol Foo {`) and body.
2. Extract all named type names (`record`, `enum`, `fixed`
   definitions) using a regex like
   `\b(record|enum|fixed)\s+(\w+)`.
3. For copies 2 through N, apply a word-boundary regex substitution
   to append a suffix (`_2`, `_3`, ...) to every type name in the
   body. Process longer names first to avoid substring collisions
   (e.g., rename `EventType` before `Event`).
4. Concatenate: header + original body + suffixed copies + `}`.

The standard suite uses multipliers of 1, 5, 10, and 20, producing
files from 52 KB to ~1 MB. Write the generated files to
`tmp/benchmark/`:

```
tmp/benchmark/benchmark-1x.avdl    #  52 KB,  ~1,480 lines
tmp/benchmark/benchmark-5x.avdl    # 251 KB,  ~7,100 lines
tmp/benchmark/benchmark-10x.avdl   # 502 KB, ~14,100 lines
tmp/benchmark/benchmark-20x.avdl   #   1 MB, ~28,200 lines
```

Verify the generated files parse correctly:

```sh
cargo run --release -- idl tmp/benchmark/benchmark-1x.avdl /dev/null
cargo run --release -- idl tmp/benchmark/benchmark-20x.avdl /dev/null
```

## Running benchmarks manually

### Rust vs Java comparison

```sh
cargo build --release

hyperfine --warmup 3 --min-runs 10 \
  'target/release/avdl idl tmp/benchmark/benchmark-1x.avdl /dev/null' \
  'java -jar ../avro-tools-1.12.1.jar idl tmp/benchmark/benchmark-1x.avdl /dev/null'
```

Repeat for the 5x, 10x, and 20x inputs to see how the speedup ratio
changes with input size.

### Measuring optimization impact

When evaluating a code change, compare the release binary before and
after:

```sh
cargo build --release
cp target/release/avdl tmp/avdl-before

# ... make changes ...
cargo build --release

hyperfine --warmup 5 \
  'tmp/avdl-before idl tmp/benchmark/benchmark-20x.avdl /dev/null' \
  'target/release/avdl idl tmp/benchmark/benchmark-20x.avdl /dev/null'
```

Use the 20x input for optimization work — it gives enough runtime
(~70 ms) for hyperfine to measure meaningful differences.

## Profiling with perf

### Recording a profile

Use the largest synthetic input (20x / ~1 MB) to get enough samples:

```sh
cargo build --release

# Frame-pointer unwinding (fast, recommended — already configured):
perf record -g --call-graph fp \
  target/release/avdl idl tmp/benchmark/benchmark-20x.avdl /dev/null

# DWARF unwinding (slower, more detailed, no config needed):
perf record -g --call-graph dwarf \
  target/release/avdl idl tmp/benchmark/benchmark-20x.avdl /dev/null
```

### Generating flamegraphs

Use [inferno][] to convert `perf script` output to an interactive SVG:

```sh
perf script > tmp/perf-script.txt
inferno-collapse-perf < tmp/perf-script.txt > tmp/collapsed-stacks.txt
inferno-flamegraph < tmp/collapsed-stacks.txt > tmp/flamegraph.svg
```

Open `tmp/flamegraph.svg` in a browser.

### Analyzing hot functions

After generating collapsed stacks, this one-liner shows the top
functions by self-time (leaf cost):

```sh
awk '{split($0,a," "); n=a[2]; split(a[1],frames,";");
      leaf=frames[length(frames)]; self[leaf]+=n; total+=n}
     END{for(f in self) printf "%5.1f%% %4d  %s\n",
         self[f]/total*100, self[f], f}' tmp/collapsed-stacks.txt \
  | sort -rn | head -20
```

For a more detailed breakdown including inclusive time, see the
`analyze-stacks.py` script that was used for the initial profiling
(archived in the benchmark worktree's `tmp/` directory).

## Baseline results (2026-02-08)

Measured on `rustc 1.94.0-beta.2`, `openjdk 21.0.10`, Linux, using
CDM20.avdl as the base input:

| Input        | Rust    | Java   | Speedup |
|--------------|---------|--------|---------|
| 1× (52 KB)   | 5.6 ms  | 267 ms | 48×     |
| 5× (251 KB)  | 19.9 ms | 320 ms | 16×     |
| 10× (502 KB) | 37.2 ms | 351 ms | 9.4×    |
| 20× (1 MB)   | 72.1 ms | 407 ms | 5.7×    |

Rust scales linearly with input size. Java's cost is dominated by
JVM startup (~260 ms), so its marginal per-type cost is comparable to
Rust's — but Rust avoids the startup entirely.

### Where time is spent (20× input, 355 perf samples)

~97% of time is in the ANTLR parser/runtime and libc allocation.
Our code (`reader.rs`, `json.rs`, `resolve.rs`) accounts for ~3% of
self-time. The main optimization targets within our code are tracked
in `issues/`:

- `SchemaRegistry::register` reallocation (~4.2%)
- `AvroSchema` cloning in `process_decl_items` and
  `collect_named_types`
- `full_name()` allocating a `String` on every call
- `field_to_json` per-field `Map` allocation (~0.84%)
