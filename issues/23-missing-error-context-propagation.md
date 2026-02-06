# Missing `.context()` on `?` operators throughout the codebase

## Summary

Many `?` operators propagate errors without attaching additional
context that would help users diagnose failures. When an error bubbles
up through multiple layers (e.g., import resolution -> file I/O ->
JSON parsing -> schema conversion), the user sees only the innermost
error message with no indication of which file was being processed,
which import triggered the failure, or what stage of compilation
failed.

This was identified as a known TODO in `TODOs.md`:

> For every `?`, consider whether we should be propagating additional
> context (e.g., through `miette::Context::context`) that would be
> useful for users when the error eventually bubbles up to them.

The codebase already uses `.context()` / `.wrap_err()` in several
places in `src/main.rs` (the CLI layer), but the library code in
`src/import.rs` and `src/reader.rs` has numerous bare `?` operators
on operations that would benefit from context.

## Audit results by file

### Priority 1: `src/import.rs` -- Import resolution and JSON parsing

These are the highest-impact gaps because import errors are the most
common user-facing failures and currently produce confusing messages
with no import chain context.

| Line(s) | Expression | Missing context |
|---------|-----------|-----------------|
| 68-70 | `relative.canonicalize().map_err(\|e\| IdlError::Io { source: e })?` | Should say which file was being canonicalized and why (e.g., "canonicalize import path `{import_file}` relative to `{current_dir}`") |
| 77-79 | `candidate.canonicalize().map_err(\|e\| IdlError::Io { source: e })?` | Should say which candidate import path was being canonicalized (e.g., "canonicalize import path `{import_file}` in import dir `{dir}`") |
| 114-115 | `std::fs::read_to_string(path).map_err(\|e\| IdlError::Io { source: e })?` | Should say "read protocol file `{path}`". The bare `Io` variant loses the filename entirely -- the user only sees the OS error (e.g., "No such file or directory"). |
| 116-117 | `serde_json::from_str(&content).map_err(...)? ` | Already includes the path in the error message string, but not as structured miette context. Acceptable for now but could be improved. |
| 125 | `json_to_schema(type_json, default_namespace)?` | Should say "parse type at index `{i}` in protocol `{path}`". When a nested schema is malformed, the user has no idea which type in the protocol failed. |
| 137 | `json_to_message(msg_json, default_namespace)?` | Should say "parse message `{name}` in protocol `{path}`". |
| 154-155 | `std::fs::read_to_string(path).map_err(\|e\| IdlError::Io { source: e })?` | Should say "read schema file `{path}`". Same issue as line 114-115. |
| 156-157 | `serde_json::from_str(&content).map_err(...)?` | Already embeds the path. Acceptable for now. |
| 159 | `json_to_schema(&json, None)?` | Should say "parse schema from `{path}`". |
| 296 | `.collect::<Result<Vec<_>>>()?` in `parse_record` fields | Should say "parse field at index `{i}` of record `{name}`". Currently a field parse error gives no indication which record it belongs to. |
| 419 | `json_to_schema(items, default_namespace)?` in `parse_array` | Should say "parse array items schema". |
| 434 | `json_to_schema(values, default_namespace)?` in `parse_map` | Should say "parse map values schema". |
| 559 | `json_to_schema(type_json, default_namespace)?` in `json_to_field` | Should say "parse type for field `{name}`". |
| 611-612 | `.collect::<Result<Vec<_>>>()?` in `json_to_message` request params | Should say "parse request parameter at index `{i}` of message". |
| 618 | `json_to_schema(resp, default_namespace)?` in `json_to_message` | Should say "parse response type for message". |
| 626-627 | `.collect::<Result<Vec<_>>>()?` in `json_to_message` errors | Should say "parse error type at index `{i}` for message". |

### Priority 2: `src/reader.rs` -- IDL tree walking

The reader already uses `make_diagnostic` to produce rich errors with
source spans, so most `?` operators here propagate errors that already
carry location information. However, several call sites lose the
"what were we doing" context.

| Line(s) | Expression | Missing context |
|---------|-----------|-----------------|
| 90 | `parser.idlFile().map_err(\|e\| IdlError::Parse(format!("{e:?}")))?` | Should say "parse IDL source `{source_name}`". The ANTLR error message is often cryptic; wrapping it with the filename would help. |
| 113 | `walk_idl_file(...)?` | This is the top-level call inside `parse_idl_named`. Adding context like "walk IDL parse tree for `{source_name}`" would help when inner errors lack source spans. |
| 245 | `walk_json_value(&value_ctx, token_stream, src)?` | Inside `walk_schema_properties`. Should say "parse value for schema property `{name}`". |
| 401 | `walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src)?` | In `walk_protocol`. Could say "parse protocol schema properties". Low priority since the inner error carries a source span. |
| 462 | `walk_fixed(&fixed_ctx, ...)?` | In `walk_named_schema`. The inner error carries a span; adding "parse fixed declaration" would be modest improvement. |
| 464 | `walk_enum(&enum_ctx, ...)?` | Same as above -- "parse enum declaration". |
| 466 | `walk_record(&record_ctx, ...)?` | Same -- "parse record declaration". |
| 476 | `registry.register(schema.clone()).map_err(IdlError::Other)?` | Should say "register schema `{full_name}` in `{source_name}`". When a duplicate registration error occurs, the user doesn't know which file caused it. |
| 496 | `walk_schema_properties(...)?` in `walk_record` | Low priority -- inner error has span. |
| 529 | `walk_field_declaration(...)?` in `walk_record` | Low priority -- inner error has span. |
| 567 | `walk_full_type(...)?` in `walk_field_declaration` | Low priority -- inner error has span. |
| 573 | `walk_variable(...)?` in `walk_field_declaration` | Low priority -- inner error has span. |
| 604 | `walk_json_value(&json_ctx, token_stream, src)?` in `walk_variable` | Should say "parse default value for field `{field_name}`". Users often have malformed defaults and the error currently doesn't identify which field. |
| 635 | `walk_schema_properties(...)?` in `walk_enum` | Low priority. |
| 685 | `walk_schema_properties(...)?` in `walk_fixed` | Low priority. |
| 701 | `parse_integer_as_u32(size_tok.get_text())?` in `walk_fixed` | Should say "parse fixed size for `{fixed_name}`". |
| 725 | `walk_schema_properties(...)?` in `walk_full_type` | Low priority. |
| 731 | `walk_plain_type(...)?` in `walk_full_type` | Low priority. |
| 773 | `walk_primitive_type(&prim_ctx, src)?` in `walk_nullable_type` | Low priority. |
| 852 | `parse_integer_as_u32(precision_tok.get_text())?` in `walk_primitive_type` | Should say "parse decimal precision". |
| 855 | `parse_integer_as_u32(scale_tok.get_text())?` in `walk_primitive_type` | Should say "parse decimal scale". |
| 887 | `walk_full_type(...)?` in `walk_array_type` | Low priority. |
| 904 | `walk_full_type(...)?` in `walk_map_type` | Low priority. |
| 920 | `walk_full_type(...)?` in `walk_union_type` | Low priority. |
| 939 | `walk_schema_properties(...)?` in `walk_message` | Low priority. |
| 945 | `walk_result_type(...)?` in `walk_message` | Low priority. |
| 962 | `walk_full_type(...)?` in `walk_message` | Low priority. |
| 968 | `walk_variable(...)?` in `walk_message` | Low priority. |
| 1049 | `walk_json_object(&obj_ctx, ...)?` in `walk_json_value` | Low priority. |
| 1053 | `walk_json_array(&arr_ctx, ...)?` in `walk_json_value` | Low priority. |
| 1055 | `walk_json_literal(&lit_ctx, src)?` in `walk_json_value` | Low priority. |
| 1103 | `walk_json_value(&value_ctx, ...)?` in `walk_json_object` | Low priority. |
| 1117 | `walk_json_value(&val_ctx, ...)?` in `walk_json_array` | Low priority. |
| 1255 | `i64::from_str_radix(hex, 16).map_err(...)?` | Currently uses `IdlError::Other` with a text message. Can be upgraded to `make_diagnostic_from_token` (see note below). |
| 1259 | `i64::from_str_radix(hex, 16).map_err(...)?` | Same. |
| 1264 | `i64::from_str_radix(&number, 8).map_err(...)?` | Same. |
| 1268 | `i64::from_str_radix(oct, 8).map_err(...)?` | Same. |
| 1272 | `number.parse::<i64>().map_err(...)?` | Same. |
| 1280 | `serde_json::to_value(long_value).map_err(...)?` | Same. |
| 1283 | `serde_json::to_value(int_value).map_err(...)?` | Same. |
| 1291 | `text.parse().map_err(...)?` | Same. |
| 1314 | `u32::from_str_radix(&number[2..], 16).map_err(...)?` | Same. |
| 1317 | `u32::from_str_radix(&number, 8).map_err(...)?` | Same. |
| 1321 | `number.parse().map_err(...)?` | Same. |

**Note on numeric literal functions:** `parse_integer_literal`,
`parse_floating_point_literal`, and `parse_integer_as_u32` currently
accept only `text: &str` and return `IdlError::Other` for parse
failures. However, all call sites already have both the token and
`SourceInfo` available:

- `walk_json_literal` (line ~1135): has `tok` and `src`
- `walk_fixed` (line ~729): has `size_tok` and `src`
- `walk_primitive_type` (lines ~897, ~900): has `precision_tok` /
  `scale_tok` and `src`

These functions should be refactored to accept `src: &SourceInfo` and
`tok: &dyn Token` parameters so they can use
`make_diagnostic_from_token` instead of `IdlError::Other`. This is a
straightforward signature change, not an architectural constraint.

### Priority 3: `src/main.rs` -- CLI entry point

The CLI layer already has the best context coverage in the codebase.
A few remaining gaps:

| Line(s) | Expression | Missing context |
|---------|-----------|-----------------|
| 66 | `miette::set_hook(...)?` | Infallible in practice; no context needed. |
| 93 | `read_input(&input)?` | Already returns a contextual error via `wrap_err`. Good. |
| 94 | `parse_and_resolve(&source, &input_dir, import_dirs)?` | Could say "compile IDL from `{input}`" to wrap the entire pipeline. |
| 236 | `parse_idl(source).map_err(miette::Report::new)?` | Should say "parse IDL source". This is the entry point where raw `IdlError` values from the reader cross into `miette::Result`; adding context here would annotate all parse errors with "while parsing the IDL source". |
| 247 | `resolve_imports(...)?` inside `parse_and_resolve` | Could say "resolve imports". |
| 278 | `import_ctx.resolve_import(&import.path, current_dir).map_err(miette::Report::new)?` | Should say "resolve import `{import.path}`". Currently the error only says the file wasn't found but doesn't indicate which import statement triggered the search. |
| 335 | `resolve_imports(&nested_imports, ...)?` in IDL import branch | Should say "resolve nested imports from `{resolved_path}`". This is critical for import chains -- without it, a deeply-nested import failure gives no indication of the chain that led to it. |

## Recommended approach

### Phase 1: High-impact file I/O in `src/import.rs`

Add `.context()` to `read_to_string` and `canonicalize` calls in
`import_protocol`, `import_schema`, and `resolve_import`. These are
the errors users encounter most often (missing files, permission
errors) and currently lose the filename entirely through the bare
`IdlError::Io` variant.

### Phase 2: JSON parse chain in `src/import.rs`

Add context to `json_to_schema`, `json_to_field`, and
`json_to_message` calls that identifies the enclosing type name and
source file. This helps users who have a malformed `.avsc` or `.avpr`
file understand exactly which type definition is broken.

### Phase 3: Import chain context in `src/main.rs`

Add context to `resolve_imports` recursive calls so that nested
import failures produce a chain like:

    Error: resolve nested imports from player.avdl
      Caused by: import schema position.avsc
        Caused by: parse schema from position.avsc
          Caused by: record missing 'name'

### Phase 4: Reader tree walk in `src/reader.rs`

Add context to `parse_idl_named` entry point and to specific pain
points like `walk_variable` default value parsing, `walk_fixed` size
parsing, and `registry.register()`. Most inner errors already carry
miette source spans, so the additional context is less critical but
still improves the error narrative.

## Note on error type compatibility

The library functions in `import.rs` and `reader.rs` return
`crate::error::Result<T>` which uses `IdlError`. To use
`miette::Context::context()` on these, either:

1. Convert `IdlError` results to `miette::Result` before adding
   context (as `main.rs` already does in several places), or
2. Implement `miette::Diagnostic` more fully on `IdlError` and use
   `miette::Context` directly, or
3. Use a pattern like `.map_err(|e| IdlError::Other(format!("...: {e}")))`
   to embed context in the error message string (less structured but
   works with the existing error type).

Option 3 is the simplest approach for the library code; option 1 is
already the pattern used at the `main.rs` boundary.

## Action item: document error propagation practice in CLAUDE.md

Once the `.context()` work is done, the project's `CLAUDE.md` should
document the error propagation practice so that future code follows the
same guidance. Specifically:

- Use `.context()` / `.wrap_err()` on `?` operators where the error
  would otherwise lose information about what was being attempted
- Prefer `make_diagnostic` / `make_diagnostic_from_token` in
  `reader.rs` over `IdlError::Other` for parse-related errors
- Include the file path or type name being processed in error context
