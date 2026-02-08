# Named type key ordering: aliases emitted before properties (should be after)

## Symptom

For named types (record, enum, fixed) that have both `@aliases` and
custom `@properties`, the Rust tool serializes `"aliases"` before the
custom properties in the JSON output. Java serializes custom properties
before `"aliases"`.

Rust output:
```json
{
  "type": "record",
  "name": "Foo",
  "fields": [...],
  "aliases": ["OldFoo"],
  "custom": "value"
}
```

Java output:
```json
{
  "type": "record",
  "name": "Foo",
  "fields": [...],
  "custom": "value",
  "aliases": ["OldFoo"]
}
```

This is semantically equivalent (JSON objects are unordered per
RFC 8259), so `jq -S` comparison does not detect it. But it produces
byte-level differences that make the output harder to compare against
Java.

## Root cause

In `src/model/json.rs`, the serialization for `Record`, `Enum`, and
`Fixed` inserts aliases before properties:

```rust
// Record (line ~247):
if !aliases.is_empty() { obj.insert("aliases", ...); }
for (k, v) in properties { obj.insert(k, v); }
```

In Java's `Schema.java`, the order is reversed — `writeProps(gen)` is
called before `aliasesToJson(gen)`:

```java
// RecordSchema.toJson (line ~1048-1049):
writeProps(gen);
aliasesToJson(gen);
```

The same pattern applies to `EnumSchema` (lines 1162-1163) and
`FixedSchema` (lines 1370-1371) in Java.

Note: Field-level serialization is NOT affected — both Rust and Java
emit aliases before properties for fields.

## Affected files

- `src/model/json.rs` — `schema_to_json` function, the `Record`,
  `Enum`, and `Fixed` match arms

## Why the golden tests don't catch it

None of the 18 golden test `.avdl` files define a named type that has
BOTH `@aliases` and custom `@properties` on the same type. The
`simple.avdl` file has `@aliases` on `Kind` (an enum) and
`@my-property` on `TestRecord` (a record), but not both on the same
type.

## Reproduction

```sh
cat > tmp/aliases-props.avdl <<'EOF'
@namespace("test")
protocol P {
  @custom("value")
  @aliases(["OldRecord"])
  record Foo { string name; }
}
EOF
cargo run -- idl tmp/aliases-props.avdl tmp/aliases-props-rust.avpr
java -jar ../avro-tools-1.12.1.jar idl tmp/aliases-props.avdl tmp/aliases-props-java.avpr
python3 tmp/compare_keys.py tmp/aliases-props-rust.avpr tmp/aliases-props-java.avpr
```

## Suggested fix

In `src/model/json.rs`, for the `Record`, `Enum`, and `Fixed` arms,
swap the order of the aliases and properties insertions:

```rust
// Before (current):
if !aliases.is_empty() { ... insert aliases ... }
for (k, v) in properties { obj.insert(k, v); }

// After (matches Java):
for (k, v) in properties { obj.insert(k, v); }
if !aliases.is_empty() { ... insert aliases ... }
```

This is a simple three-line swap in each of the three match arms.
