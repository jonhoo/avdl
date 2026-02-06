# Imported schema references use fully-qualified names in same namespace

## Symptom

When importing `.avsc` files, unqualified type references (like
`"Position"`) are qualified to their full name
(`"avro.examples.baseball.Position"`) during import. When serializing
back to JSON, the serializer doesn't know to shorten same-namespace
references because `AvroSchema::Reference` only stores the full name
string — namespace and simple name are not tracked separately.

## Root cause

`AvroSchema::Reference(String)` stores only the fully-qualified name.
In `schema_to_json`, the Reference arm tries to split on `.` to
extract namespace, but the namespace may not always be separable this
way (e.g., multi-part names). Additionally, the `schema_ref_name`
logic depends on knowing the reference's namespace separately.

## Location

- `src/import.rs:218-229` — reference qualification during import
- `src/model/json.rs:456-486` — Reference serialization
- `src/model/schema.rs` — `Reference(String)` definition

## Expected behavior

References should use simple names when they're in the same namespace
as the enclosing context. Either:
- Store `Reference { name: String, namespace: Option<String> }`
- Or defer qualification and qualify only at serialization time

## Difficulty

Moderate — requires changing the `Reference` variant or the
qualification strategy.
