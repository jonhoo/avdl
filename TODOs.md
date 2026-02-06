In `extract_doc_comment`, when stripping the `/**` and `*/`, let's add a
debug assert to make sure that what we stripped was actually those
characters. Maybe use `strip_prefix` and friends?

In `strip_indents`, let's prefer doing the stripping with the `regex`
crate to match what the Java version of this code does. Also, let's
expand the set of test cases for `strip_indents` to cover more odd
combinations of characters people might use in their avdl code.

Handle the TODO around unknown logical types in `src/import.rs`. We
should make sure to always preserve custom logical types in conversions.

Handle the TODO around additional properties on primitives in
`src/import.rs`. We should make sure to always preserve additional
properties.

`primitive_from_str` should probably return an `Err` rather than panic
