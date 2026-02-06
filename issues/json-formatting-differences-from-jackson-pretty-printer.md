# JSON formatting differences from Jackson's DefaultPrettyPrinter

## Symptom

All 18 `.avdl` test files pass semantic comparison but have byte-level
formatting differences in the JSON output. The Rust tool uses
`serde_json`'s `PrettyFormatter` (via the custom `JavaPrettyFormatter`
wrapper), which produces consistent formatting. Java's golden files
use Jackson's `DefaultPrettyPrinter`, which has a distinctly different
-- and internally inconsistent -- formatting style.

## Detailed formatting differences

### 1. Space before colon in object keys

**Rust:** `"key": value` (no space before colon)
**Java:** `"key" : value` (space before colon)

This affects ALL protocol-level keys and most nested keys. It is the
single largest source of byte-level differences, affecting every line
of every protocol golden file.

**Exception:** Java's output is itself inconsistent. When properties
are serialized via `writeObjectField(key, jsonNode)` or
`writeTree(jsonNode)`, the JsonNode value is serialized WITHOUT the
pretty printer, so the inner content uses compact format (no space
before colon). This creates mixed formatting within the same file.
See items 4 and 5 below.

### 2. Array/object element placement ("compact" vs "expanded")

**Rust:** Each element on its own line with consistent indentation.
```json
"types": [
  {
    "type": "record",
    ...
  },
  {
    ...
  }
]
```

**Java:** Opening bracket and first element on same line; closing
bracket of previous element and comma + opening bracket of next
element on same line.
```json
"types" : [ {
  "type" : "record",
  ...
}, {
  ...
} ]
```

This affects arrays of objects (types array, fields array, request
parameters, errors array, symbols array, aliases array).

### 3. Empty containers

**Rust:** `{}` and `[]`
**Java:** `{ }` and `[ ]` (spaces inside)

This is visible in `"messages" : { }` and `"default" : [ ]` in
protocol golden files.

### 4. Property values serialized without pretty printing (mixed mode)

In Java's golden files, property values that come from `JsonNode`
trees (stored via `JsonProperties.writeProps()`) bypass the pretty
printer. This creates a mixed formatting mode within the same file.

For example, in `simple.avpr`, fields with annotated types (logical
types, custom properties) have their type objects serialized in
compact mode:
```json
"name": "d",
"type": {"type": "int", "logicalType": "date"},
"default": 0
```

While fields without custom type properties use full pretty printing:
```json
"name" : "kind",
"type" : "Kind",
"doc" : "The kind of record.",
```

This inconsistency comes from Jackson's `writeTree()` method, which
serializes a `JsonNode` without applying the generator's pretty
printer context.

### 5. Field default values sometimes compact

When field default values are stored as `JsonNode` trees in Java,
they are also serialized via `writeTree()`, producing compact format.
However, for simple scalar defaults (integers, strings, booleans),
this makes no visible difference. It only matters for complex defaults
like `{"name": "bar", "kind": "BAR"}` -- but in practice, Java's
`writeTree()` for these complex defaults DOES use the pretty printer
because `JsonGenerator.writeTree()` respects the pretty printer for
structured values (unlike `writeObjectField` which does not for the
value part).

### 6. Schema mode uses 4-space indent

The `schema_syntax.avsc` golden file uses 4-space indentation and no
spaces before colons. This appears to come from a different Jackson
`ObjectMapper` configuration used for `Schema.toString(true)` vs
`Protocol.toString(true)`. The Rust tool uses 2-space indentation
uniformly.

## Root cause

The Rust tool uses `serde_json::ser::PrettyFormatter` which produces
consistent, uniform formatting. Java's Jackson `DefaultPrettyPrinter`
produces different formatting, and additionally Jackson's `writeTree`
method creates inconsistencies within the same output.

Matching Java's formatting byte-for-byte would require implementing
a custom `serde_json::ser::Formatter` that:

1. Adds a space before colons in object keys
2. Uses Jackson's compact array/object element placement style
3. Inserts spaces inside empty containers
4. Detects when serializing property values and switches to compact
   mode (no pretty printing) to match `writeTree` behavior
5. Uses 4-space indent for schema-mode output

Items 1-3 are straightforward. Item 4 is the most challenging because
it requires tracking serialization context (are we inside a property
value or a first-class schema structure?) and switching formatting
modes accordingly.

## Affected files

- `src/model/json.rs` -- `JavaPrettyFormatter` and
  `to_string_pretty_java`
- ALL 18 golden file comparisons

## Reproduction

```sh
cargo run -- idl avro/lang/java/idl/src/test/idl/input/echo.avdl \
  tmp/echo.avpr
diff tmp/echo.avpr avro/lang/java/idl/src/test/idl/output/echo.avpr
```

## Suggested fix approach

Implement a custom `serde_json::ser::Formatter` that mimics Jackson's
`DefaultPrettyPrinter`:

1. Override `begin_object_value` to write ` : ` instead of `": "`.
2. Override `begin_array_value` and `begin_object_key` to implement
   Jackson's compact array-of-objects layout (first element on same
   line as `[`, subsequent elements after `}, {`).
3. Override `begin_array` / `end_array` and `begin_object` /
   `end_object` to produce `[ ]` and `{ }` for empty containers.

For items 4-5 (mixed compact mode for property values), the cleanest
approach may be to pre-serialize property `serde_json::Value` nodes
to compact JSON strings and embed them as raw JSON within the pretty-
printed output, or to use a two-pass approach where the first pass
identifies which subtrees should be compact.

An alternative approach is to NOT attempt to match the inconsistent
mixed-mode formatting from Java, since that inconsistency is arguably
a bug in Java's output rather than intentional behavior. In that case,
the focus would be on items 1-3 only, which would get most files
significantly closer to byte-perfect.

## Priority

Medium. The formatting differences are purely cosmetic and do not
affect correctness. However, achieving byte-perfect output would
enable the test suite to use simple `diff` instead of `jq -S`
semantic comparison, which would be a simpler and faster test.

## Relationship to existing issues

This supersedes issue `39c7d498` (float scientific notation), which
is one specific instance of formatting mismatch. The scientific
notation issue has already been fixed (the Rust tool now outputs
`-1.0E12` matching Java), but the broader formatting differences
documented here remain.
