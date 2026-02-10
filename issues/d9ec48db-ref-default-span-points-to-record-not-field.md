# Default validation error for Reference-typed field points to record keyword, not field

## Symptom

When a Reference-typed field has an invalid default value (validated
after type registration in `compiler.rs`), the source span in the error
output points to the `record` keyword rather than the specific field
with the invalid default:

```
Error:   x Invalid default for field `nested` in `Outer`: expected record Inner,
  |  got string
   +--[tmp/err-ref-default.avdl:3:3]
 2 |   record Inner { string x; }
 3 |   record Outer {
   .   ---+--
   .      `-- Invalid default for field `nested` in `Outer`: ...
 4 |     Inner nested = "bad";
   `----
```

The underline is on `record` (line 3) but the actual problem is on
line 4 (`Inner nested = "bad"`). A user or tool must read the error
message text to find the field name, rather than being directed to it
by the source highlight.

## Root cause

`DeclItem::Type` stores a single `SourceSpan` for the entire type
declaration, derived from the `record` keyword's start token (via
`span_from_context` in `reader.rs`). When `compiler.rs`
`process_decl_items` detects invalid field defaults, it uses this
type-level span (line 640-648) because no per-field span information
is available in the `DeclItem`.

The per-field span IS available during the reader.rs tree walk (where
primitive-typed defaults are validated with accurate spans), but
Reference-typed defaults can only be validated after type registration,
which happens in compiler.rs where only the type-level span is
available.

## Affected files

- `src/compiler.rs`: `process_decl_items()`, around lines 635-651
- `src/reader.rs`: `DeclItem::Type` variant (line 766)
- `src/model/schema.rs`: `Field` struct (no span field)

## Reproduction

```sh
cat > tmp/err-ref-default.avdl <<'EOF'
protocol Test {
  record Inner { string x; }
  record Outer {
    Inner nested = "bad";
  }
}
EOF
cargo run -- idl tmp/err-ref-default.avdl 2>&1
```

## Suggested fix

Add an optional `SourceSpan` to the `Field` struct in
`src/model/schema.rs` so that each field retains its source location.
When `compiler.rs` validates defaults and finds an error, it can use
the field's span instead of the type's span.

Alternatively, store a `HashMap<String, SourceSpan>` mapping field
names to their spans in the `DeclItem::Type` variant, avoiding changes
to the `Field` struct (which is part of the domain model and ideally
source-location-free).
