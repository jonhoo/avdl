# Reduce AvroSchema cloning via shared ownership

## Symptom

Profiling shows `AvroSchema` trees being cloned at least twice during
compilation: once in `process_decl_items` (when handing a schema to
`SchemaRegistry::register`) and again in `collect_named_types` (when
building the `SchemaLookup` table for JSON serialization). For deeply
nested records with many fields, each clone copies the entire tree.

## Root cause

The schema ownership model passes `AvroSchema` by value into the
`SchemaRegistry`, then later `collect_named_types` calls
`schema.clone()` to insert into a `HashMap<String, AvroSchema>` lookup
table (`json.rs:130`, `json.rs:149`). The protocol's `types: Vec<AvroSchema>`
also holds its own copy. This means each named type exists as at least
three independent heap allocations:

1. The protocol's `types` vec
2. The `SchemaRegistry`'s `IndexMap`
3. The `SchemaLookup` `HashMap`

## Affected files

- `src/model/json.rs`: `collect_named_types` (lines 113–164) — clones
  schemas into the lookup table
- `src/resolve.rs`: `SchemaRegistry` — owns one copy
- `src/main.rs`: protocol construction — owns another copy in `types`

## Reproduction

No functional bug — this is a performance issue observable under
profiling. Compare `perf record` flamegraphs for a large input and
look for `<avdl::model::schema::AvroSchema as core::clone::Clone>::clone`
stacks.

## Suggested fix

Use `Rc<AvroSchema>` (or `Arc<AvroSchema>` if thread safety is needed
later) for shared ownership. The schema tree is built once during
parsing and then read during JSON serialization — it is never mutated
after construction, making `Rc` a natural fit.

Concretely:

- Change `SchemaRegistry` to store `IndexMap<String, Rc<AvroSchema>>`.
- Change `SchemaLookup` to `HashMap<String, Rc<AvroSchema>>`.
- Change `Protocol.types` to `Vec<Rc<AvroSchema>>`.
- `collect_named_types` can then `Rc::clone()` (a pointer bump) instead
  of deep-cloning the tree.

This eliminates all deep clones of `AvroSchema` and reduces
allocation pressure proportional to the number of named types.

The `AvroSchema: Clone` derive can remain for test convenience, but
production code paths would use `Rc` sharing instead.
