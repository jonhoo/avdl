# Add doc comment `stripIndents` test cases from Java `DocCommentHelperTest`

## Symptom

The Java `DocCommentHelperTest` class has two specific test cases for
the `stripIndents` method that our `doc_comments.rs` unit tests do
not replicate verbatim:

1. `stripIndentsFromDocCommentWithStars`:
   - Input:  `"* First line\n\t  * Second Line\n\t * * Third Line\n\t  *\n\t  * Fifth Line"`
   - Expected: `"First line\nSecond Line\n* Third Line\n\nFifth Line"`
   - Key behaviors: `\t * * Third Line` keeps second `*`; blank
     star-only line `\t  *` becomes empty line.

2. `stripIndentsFromDocCommentWithoutStars`:
   - Input: `"First line\n\t Second Line\n\t  * Third Line\n\t  \n\t  Fifth Line"`
   - Expected: `"First line\nSecond Line\n * Third Line\n \n Fifth Line"`
   - Key behavior: Non-star-prefixed mode strips common tab indent,
     preserving internal whitespace.

Both cases were verified to produce correct output from our tool (via
ad-hoc testing), but there are no unit tests that assert these exact
Java test vectors. Our `doc_comments.rs` tests cover the same
behaviors through different input patterns.

## Root cause

Our strip_indents tests were written independently rather than being
ported from the Java test suite.

## Affected files

- `src/doc_comments.rs` (test module)

## Suggested fix

Add two unit tests to the `doc_comments.rs` test module that use the
exact same input/output strings as the Java `DocCommentHelperTest`:

```rust
#[test]
fn test_strip_indents_java_parity_with_stars() {
    assert_eq!(
        strip_indents("* First line\n\t  * Second Line\n\t * * Third Line\n\t  *\n\t  * Fifth Line"),
        "First line\nSecond Line\n* Third Line\n\nFifth Line"
    );
}

#[test]
fn test_strip_indents_java_parity_without_stars() {
    assert_eq!(
        strip_indents("First line\n\t Second Line\n\t  * Third Line\n\t  \n\t  Fifth Line"),
        "First line\nSecond Line\n * Third Line\n \n Fifth Line"
    );
}
```

Priority: Low. Our existing tests cover the same edge cases through
different vectors, so this is purely for parity documentation.
