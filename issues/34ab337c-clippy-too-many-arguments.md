# clippy: `too_many_arguments` warnings in `src/main.rs`

## Symptom

`cargo clippy` emits two `too_many_arguments` warnings for functions
that take 8 parameters (threshold is 7):

- `process_decl_items` (line ~483)
- `resolve_single_import` (line ~531)

Both functions thread the same set of context through:
`registry`, `import_ctx`, `protocol`, `messages`, `decl_items`,
`doc_comments`, and `source_name`.

## Root cause

These functions were extracted from a larger flow and each needs
access to the same mutable state. The parameter lists grew
organically as features (imports, doc comments, diagnostics) were
added.

## Suggested fix

Group the repeatedly-passed parameters into a context struct, e.g.:

```rust
struct ProtocolBuildContext<'a> {
    registry: &'a mut SchemaRegistry,
    import_ctx: &'a mut ImportContext,
    protocol: &'a mut Protocol,
    messages: &'a mut Vec<Message>,
    decl_items: &'a mut Vec<DeclItem>,
    doc_comments: &'a DocCommentMap,
    source_name: &'a str,
}
```

Then pass `&mut ProtocolBuildContext` as a single parameter. This
eliminates the warning and makes it easier to add future context
fields without growing every signature.

Alternatively, if the struct feels heavyweight, selectively
`#[allow(clippy::too_many_arguments)]` on the two functions with a
brief comment explaining why.

## Affected files

- `src/main.rs`
