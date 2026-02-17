# Add doc comment content assertion tests matching Java's `testDocCommentsAndWarnings`

## Symptom

The Java `TestIdlReader.testDocCommentsAndWarnings()` test (and the
compiler's `TestIdl.docCommentsAndWarnings()`) do more than just count
warnings. They assert on the **content** of doc comments attached to
specific types, fields, and messages in `comments.avdl`:

- `protocol.getType("testing.DocumentedEnum").getDoc()` == `"Documented Enum"`
- `protocol.getType("testing.DocumentedFixed").getDoc()` == `"Documented Fixed Type"`
- `documentedError.getDoc()` == `"Documented Error"`
- `documentedError.getField("reason").doc()` == `"Documented Reason Field"`
- `documentedError.getField("explanation").doc()` == `"Default Doc Explanation Field"`
- `documentedMethod.getDoc()` == `"Documented Method"`
- `documentedMethod.getRequest().getField("message").doc()` == `"Documented Parameter"`
- `documentedMethod.getRequest().getField("defMsg").doc()` == `"Default Documented Parameter"`
- Undocumented types/methods have `null` doc

Our `test_comments` integration test compares the full JSON output
against the golden `.avpr` file, which implicitly verifies all doc
strings. The `test_comments_warnings` test verifies the warning
positions.

However, we lack **explicit doc-comment content assertions** that
would survive a golden file update that accidentally drops or
corrupts a doc string. The golden file comparison would silently
accept the new golden file on regeneration.

## Root cause

The doc comment tests were written as golden-file comparisons rather
than specific assertions, because the golden files were considered
sufficient. The Java tests take a belt-and-suspenders approach by
asserting both golden file equality AND specific field values.

## Affected files

- `tests/integration.rs` (the `test_comments` test)
- The golden file `avro/lang/java/idl/src/test/idl/output/comments.avpr`

## Suggested fix

Add a test (e.g., `test_comments_doc_content`) that parses
`comments.avdl` through the `Idl` builder and makes explicit
assertions on the doc strings in the JSON output:

```rust
let json = parse_and_serialize(&input_path("comments.avdl"), &[]);
let types = json["types"].as_array().unwrap();
// Find DocumentedEnum and assert doc == "Documented Enum"
// Find DocumentedFixed and assert doc == "Documented Fixed Type"
// Find DocumentedError and assert doc, field docs
// Find UndocumentedEnum and assert doc is absent
// Assert message doc strings
```

This mirrors the Java test's approach and catches regressions even
if the golden file is regenerated.
