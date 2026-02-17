# Thread `CompileContext` to reduce argument counts

## Symptom

`process_decl_items` and `resolve_single_import` in `compiler.rs`
both accept 9 parameters and require
`#[allow(clippy::too_many_arguments)]`. The `CompileContext` struct
(lines 557-567) was created to group exactly these parameters, but
the functions still receive the individual fields rather than a
`&mut CompileContext`.

## Root cause

`CompileContext` was introduced to group state for the `compile`
method, but `process_decl_items` and `resolve_single_import` were
not refactored to accept it. These functions are called from two
sites: `parse_and_resolve` (which owns a `CompileContext` and
destructures it) and from each other (recursive import resolution).

## Affected files

- `src/compiler.rs` (lines 632-642, 667-677, 769-779, 872-882)

## Suggested fix

Change `process_decl_items` and `resolve_single_import` to accept
`&mut CompileContext` plus the non-context parameters (`decl_items`,
`current_dir`, `source`, `source_name`). This reduces the argument
count from 9 to 5, eliminating the `clippy::too_many_arguments`
suppression.

The call site in `parse_and_resolve` (line 632) already has a
`CompileContext` and can pass `&mut ctx` directly instead of
destructuring it. The recursive call in `resolve_single_import`
(line 872) already receives these same parameters and can forward
them as `ctx`.
