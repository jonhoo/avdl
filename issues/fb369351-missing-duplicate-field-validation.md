# Missing validation: duplicate field names in records

## Symptom

Records with duplicate field names are accepted without error:

```avdl
@namespace("test")
protocol P {
    record R {
        string name;
        int name;
    }
}
```

This produces valid-looking JSON output with two fields both named
`"name"`, which is semantically invalid according to the Avro
specification and would be rejected by downstream Avro consumers.

## Root cause

The `walk_record` function in `src/reader.rs` collects fields from
`walk_field_declaration` into a `Vec<Field>` by appending. There is no
check for duplicate field names before adding a field to the vector.

The Java `Schema.setFields()` method (in `Schema.java` line 978-981)
explicitly checks for duplicates using a `HashMap<String, Field>` and
throws `AvroRuntimeException("Duplicate field X in record Y: ...")`.

## Affected files

- `src/reader.rs` -- `walk_record` function (around line 572)

## Reproduction

```sh
echo '@namespace("test") protocol P { record R { string name; int name; } }' \
  | cargo run -- idl
```

Expected: error about duplicate field name `name` in record `R`.
Actual: JSON output with two `"name"` fields.

## Suggested fix

After collecting all fields in `walk_record`, check for duplicate names
using a `HashSet<String>` and return an error with a source span if a
duplicate is found:

```rust
let mut seen_names = HashSet::new();
for field in &fields {
    if !seen_names.insert(&field.name) {
        return Err(make_diagnostic(
            src, ctx,
            format!("duplicate field '{}' in record '{}'", field.name, record_name),
        ));
    }
}
```

Alternatively, the check could be done at field insertion time (inside
the `for field_ctx in body.fieldDeclaration_all()` loop) to provide a
more precise source span pointing at the duplicate field declaration.

Note: The same check should also be applied to message parameter lists
in `walk_message`, since message request parameters are fields and
should also be unique.

## Priority

Medium. This is a semantic correctness gap -- the tool accepts invalid
schemas that Java rejects, and the resulting JSON output would cause
errors in downstream Avro tooling.
