# SchemaRegistry IndexMap reallocation and schema cloning

## Symptom

Profiling on a 1 MB `.avdl` input (355 samples) shows
`SchemaRegistry::register` → `indexmap::insert_full` → `push_entry` →
`RawVecInner::finish_grow` → `realloc` accounting for 15 samples
(~4.2% of total time). This is the single largest hot spot within our
code (excluding antlr4rust and libc).

## Root cause

`SchemaRegistry::new()` creates an empty `IndexMap` with no
pre-allocated capacity. For protocols with many named types (e.g.,
CDM20 has ~40), the map grows incrementally, triggering multiple
`realloc` calls as the backing storage doubles each time.

Additionally, `process_decl_items` at `main.rs:513` does
`registry.register(schema.clone())` — cloning the entire `AvroSchema`
tree to hand ownership to `register()`. This clone is expensive for
deeply nested records and contributes to the allocation pressure in the
same call chain.

## Affected files

- `src/resolve.rs`: `SchemaRegistry::new()` (line 90–94) — no
  pre-allocation
- `src/main.rs`: `process_decl_items` (line 513) — `schema.clone()`
  before `register()`

## Reproduction

Profile with `perf record` on a large `.avdl` file:

```sh
perf record -g --call-graph dwarf \
  target/release/avdl idl tests/testdata/cdm20-1mb.avdl /dev/null
perf script | inferno-collapse-perf | inferno-flamegraph > tmp/flame.svg
```

Look for `SchemaRegistry::register` → `indexmap` → `realloc` in the
flamegraph.

## Suggested fix

1. **Pre-size the registry.** Before walking declarations, do a cheap
   count of type-definition nodes in the parse tree and create the
   registry with `IndexMap::with_capacity(type_count)`. This
   eliminates incremental reallocation for the common case.

2. **Avoid the clone in `process_decl_items`.** `DeclItem::Type`
   currently holds a borrowed or shared reference; change it to yield
   the `AvroSchema` by value so `register()` can take ownership
   without cloning. This may require adjusting the `DeclItem` enum to
   own the schema (e.g., `DeclItem::Type(AvroSchema, Option<Span>)`
   by value rather than by clone at the call site).
