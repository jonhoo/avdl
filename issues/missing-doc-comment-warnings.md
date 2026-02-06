# Missing warnings for out-of-place documentation comments

## Symptom

The Rust tool silently discards out-of-place documentation comments
without generating any warnings. The Java implementation generates
detailed warnings with line and column numbers for each misplaced
doc comment.

For example, given `comments.avdl` which contains multiple misplaced
doc comments (e.g., doc comments before enum values, between fields,
on unused positions), the Java tool produces 24 specific warnings
like:

    Line 21, char 8: Ignoring out-of-place documentation comment.
    Did you mean to use a multiline comment ( /* ... */ ) instead?

The Rust tool produces no warnings at all. The `comments.avdl`
golden file comparison passes (the doc comments that ARE correctly
placed produce the right output), but the user gets no feedback about
misplaced ones.

## Java behavior tested

### New IdlReader (ANTLR-based)

`TestIdlReader.testDocCommentsAndWarnings()` (in
`avro/lang/java/idl/src/test/java/org/apache/avro/idl/TestIdlReader.java`)
verifies 24 warnings with exact line/column positions, using the
pattern:

    "Line %d, char %d: Ignoring out-of-place documentation comment.
     Did you mean to use a multiline comment ( /* ... */ ) instead?"

### Old JavaCC-based compiler

`TestIdl.docCommentsAndWarnings()` (in
`avro/lang/java/compiler/src/test/java/org/apache/avro/compiler/idl/TestIdl.java`)
verifies warnings with two patterns:

    "Found documentation comment at line %d, column %d. Ignoring
     previous one at line %d, column %d: \"%s\""

    "Ignoring out-of-place documentation comment at line %d,
     column %d: \"%s\""

### TestIdlTool / TestIdlToSchemataTool

Both `TestIdlTool.java` and `TestIdlToSchemataTool.java` capture
stderr and assert that a specific warning appears when the license
header is parsed as a doc comment (line 1, char 1).

## Root cause

The Rust `src/doc_comments.rs` implements doc comment extraction by
scanning backward from a parse node's start token. When a doc comment
is found, it's returned. When it's not associated with any named
declaration, it's simply ignored -- there's no mechanism to track
"orphaned" doc comments and report them as warnings.

The Java `IdlReader` uses a `propertiesStack` pattern where doc
comments are pushed onto a stack when encountered and popped when
consumed by a declaration. If a doc comment remains on the stack when
the next doc comment arrives (or when the parser reaches a position
where no doc comment should exist), a warning is generated.

## Affected files

- `src/doc_comments.rs` -- No warning generation infrastructure
- `src/reader.rs` -- No warning collection or propagation
- `src/main.rs` -- No stderr warning output (except for unresolved
  type references)

## Reproduction

```sh
cargo run -- idl avro/lang/java/idl/src/test/idl/input/comments.avdl \
    tmp/comments.avpr 2>tmp/comments-stderr.txt
cat tmp/comments-stderr.txt
# Empty -- no warnings produced

# Compare with Java:
java -jar ../avro-tools-1.12.1.jar idl \
    avro/lang/java/idl/src/test/idl/input/comments.avdl \
    tmp/comments-java.avpr 2>tmp/comments-java-stderr.txt
cat tmp/comments-java-stderr.txt
# 24 warning lines
```

## Suggested fix

### Phase 1: Warning infrastructure

Add a warning collection mechanism. Options:

1. Return warnings alongside the parse result from `parse_idl()`:
   `fn parse_idl(source: &str) -> Result<(IdlFile, Vec<DeclItem>, Vec<Warning>)>`

2. Use a thread-local or passed-in warning collector (similar to
   Java's `DocCommentHelper.getAndClearWarnings()`).

### Phase 2: Detect orphaned doc comments

During the tree walk in `reader.rs`, track which doc comments were
consumed by declarations. After walking a scope (enum body, record
body, message list), check for unconsumed doc comments and add
warnings.

### Phase 3: Emit to stderr

In `main.rs`, after parsing, iterate the collected warnings and
write them to stderr, prefixed with "Warning: ".

## Priority

Medium. The JSON output is correct; this only affects diagnostic
feedback to users who misplace doc comments. However, both Java test
suites (old and new) explicitly test this behavior, making it a
notable compatibility gap.
