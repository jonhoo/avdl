# Unnecessary `unsafe` block in `to_string_pretty_java`

## Symptom

The `to_string_pretty_java` function in `src/model/json.rs` uses
`unsafe { String::from_utf8_unchecked(writer) }` to convert the
serializer output. This is the only `unsafe` block in the entire
codebase. While the invariant (serde_json only produces valid UTF-8)
is correct, the safe alternative has negligible cost and eliminates
the unsafe block entirely.

## Root cause

The code was written for maximum performance, but the performance
difference between `String::from_utf8_unchecked` and
`String::from_utf8(...).expect(...)` is negligible for typical schema
output sizes (a few KB).

## Affected files

- `src/model/json.rs:804` -- `to_string_pretty_java`

## Suggested fix

Replace:

```rust
Ok(unsafe { String::from_utf8_unchecked(writer) })
```

With:

```rust
Ok(String::from_utf8(writer).expect("serde_json produces valid UTF-8"))
```

## Priority

Low. The current code is correct, but removing the only `unsafe` block
in the codebase is a trivial improvement with no downside.
