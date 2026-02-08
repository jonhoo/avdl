# Doc comment warning positions not verified in tests

## Symptom

The `test_comments_warnings_count` test in `integration.rs` verifies
that parsing `comments.avdl` produces exactly 24 out-of-place doc
comment warnings, and that each warning contains the string "Ignoring
out-of-place documentation comment". However, the test does not verify
the exact (line, char) positions of each warning.

Java's `TestIdlReader.testDocCommentsAndWarnings` (line 147-156) asserts
the exact list of 24 warnings with specific line/column numbers:

```
(21,8), (21,45), (22,5), (23,5), (24,5), (25,5),
(26,7), (27,7), (28,7), (33,7), (34,7), (35,5),
(36,5), (37,7), (42,7), (43,7), (46,9), (47,5),
(54,7), (55,7), (58,9), (59,7), (60,11), (61,11)
```

Our CLI output currently matches these positions exactly (verified
manually during this audit), but a regression in the position
calculation would not be caught by the existing test.

## Root cause

The test was written to verify warning count and presence, not
positional accuracy. The `Warning` struct contains the full message
with embedded line/column info, but the test only checks
`warning.message.contains(...)`, not the exact message content.

## Affected files

- `tests/integration.rs` -- `test_comments_warnings_count`

## Reproduction

The test passes even if warning positions are wrong, as long as:
- There are exactly 24 warnings.
- Each warning message contains "Ignoring out-of-place documentation
  comment".

For example, if all 24 warnings reported line 1, char 1 due to a bug,
the test would still pass.

## Suggested fix

Extend `test_comments_warnings_count` to assert the exact (line, char)
positions of each warning, matching the expected values from Java's
`TestIdlReader.testDocCommentsAndWarnings`. This can be done by
asserting the full warning message string (which embeds the line/column)
or by extracting positions from the `Warning` structs.

The Java test also verifies the actual doc comment content ("Documented
Enum", "Documented Fixed Type", etc.) and checks that undocumented
elements have `null` docs. Our golden-file comparison for
`comments.avdl` already covers doc comment content indirectly, but
adding explicit assertions would provide a stronger regression guard.

Priority: low. The CLI output currently matches Java, and the golden
file comparison tests catch most doc comment content issues. The risk
is that a warning position regression would go undetected until someone
manually checks.
