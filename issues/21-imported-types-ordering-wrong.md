# Imported types appear after local types instead of in declaration order

## Symptom

In the output of `import.avdl`, locally-defined types (`Bar`) appear
before imported types (`Position`, `Player`, `ImportBar`,
`NestedType`, etc.), even though the `import` statements precede the
local type definitions in the source file. The Java tools emit types
in declaration/import order: imported types first (in the order their
imports appear), then local types.

Golden type order:
  Position, Player, ImportBar, NestedType, FromAfar, VeryFar,
  FarAway, Baz, Foo, Bar

Actual type order:
  Bar, ImportBar, FromAfar, NestedType, VeryFar, FarAway, Baz, Foo

## Root cause

The two-phase approach in `src/main.rs` separates parsing from import
resolution:

1. `parse_idl()` parses the source and registers locally-defined
   types into the `SchemaRegistry` (via `walk_named_schema`). At this
   point, `Bar` is registered.
2. `resolve_imports()` then processes import statements and merges
   imported types into the same registry via `registry.merge()`.
3. `rebuild_protocol_types()` replaces the protocol's `types` with
   all schemas from the registry in registration order.

Because local types are registered in step 1 before imports are
resolved in step 2, local types appear first in the registry. The
Java tools process imports and local declarations in a single
interleaved pass (the ANTLR listener encounters imports and type
definitions in source order), so imported types appear in the correct
position.

## Affected files

- `src/main.rs:220-243` -- `parse_and_resolve` function
- `src/main.rs:319-331` -- `rebuild_protocol_types`

## Reproduction

```sh
cargo run -- idl \
  --import-dir avro/lang/java/idl/src/test/idl/input/ \
  --import-dir avro/lang/java/idl/src/test/idl/putOnClassPath/ \
  avro/lang/java/idl/src/test/idl/input/import.avdl /dev/stdout \
  | python3 -c "
import json, sys
for t in json.load(sys.stdin)['types']:
    print(t['name'])
"
```

## Suggested fix

Instead of a two-phase parse-then-resolve approach, process imports
during the tree walk itself. One way: rather than collecting
`ImportEntry` values for later resolution, resolve each import as it
is encountered during `walk_protocol` (pass the `ImportContext` and
`SchemaRegistry` into the walker). This interleaves import resolution
with local type registration, matching Java's declaration-order
semantics.

Alternatively, keep the two-phase approach but defer local type
registration: collect local schemas into a separate list during
parsing, resolve imports first (which fills the registry), then
register local schemas. This would require changing `walk_protocol`
to not call `registry.register()` for local types during the walk.
