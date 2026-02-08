# Number parsing errors lack source spans

## Symptom

Integer and float literal parsing failures (e.g., overflow when
parsing a fixed size like `fixed F(99999999999999)`) produce errors
via `miette::miette!()` without source spans. The error message
includes the value but not the source location.

## Root cause

Several sites in `reader.rs` (around lines 1847+) parse numeric
literals from token text using `.parse::<i32>()` or similar, and
wrap failures with `miette::miette!()` or `.context()`. These don't
carry source spans because the token's byte offset isn't threaded
through to the error construction.

## Affected files

- `src/reader.rs` — numeric literal parsing sites

## Suggested fix

At each numeric parsing site, use `make_diagnostic_from_token()` (or
`make_diagnostic()` with the token's context) to create a
`ParseDiagnostic` pointing at the offending numeric literal.

Low priority since these errors are rare in practice (they only
occur when someone writes a literal that overflows the expected
integer type).
