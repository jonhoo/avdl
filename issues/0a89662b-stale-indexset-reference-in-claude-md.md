# Stale `IndexSet` reference in CLAUDE.md

## Symptom

The "Named type serialization" section of `CLAUDE.md` (line 325)
states:

> The `schema_to_json` function tracks
> `known_names: &mut IndexSet<String>` to decide which form to use.

The actual code in `src/model/json.rs` uses `HashSet<String>`, not
`IndexSet<String>`. The `IndexSet` type is not used anywhere in
`model/json.rs`; it is only used in `resolve.rs` for the
`SchemaRegistry`'s backing `IndexMap`.

## Root cause

The documentation was written when `known_names` used `IndexSet`
(for insertion-order tracking), and was not updated when the type
was changed to `HashSet` (since insertion order of "already seen"
names is irrelevant for the contains check).

## Affected files

- `CLAUDE.md` (line 325, "Named type serialization" section)

## Suggested fix

Change `IndexSet<String>` to `HashSet<String>` in the CLAUDE.md
documentation.
