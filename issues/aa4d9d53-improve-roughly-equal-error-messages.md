# Improve error messages that are only "roughly equal" to Java's

## Symptom

Fuzz testing of 229 real-world `.avdl` files from GitHub found 52 files
where both Rust and Java fail. In 43 cases, Rust already produces a
clearly better error message. In the remaining 9 cases, the Rust and
Java error messages are roughly equal -- both report essentially the
same ANTLR parse error text or similarly unhelpful messages. The Rust
tool should do strictly better than Java in all cases.

The roughly-equal cases fall into two patterns:

### Pattern 1: Bare ANTLR "no viable alternative" / "extraneous input" errors (4 cases)

Both tools pass through the raw ANTLR error string with no additional
context. The Rust tool wraps it in `parse error: <input>: ...` while
Java wraps it in `SchemaParseException: ...`, but neither adds
diagnostic value beyond what ANTLR itself reports.

**Example A** -- unknown annotation `@beta` before `record` (indices #32, #21, #61):

```
// execution.avdl line 6-7:
@beta
record RetryParams { ... }
```

```
Rust: parse error: <input>: line 7:4 no viable alternative at input '@betarecord'
Java: SchemaParseException: line 7:4 no viable alternative at input '@betarecord'
```

The user wrote `@beta` expecting it to be a valid annotation, but the
grammar does not recognize it. ANTLR's lexer merges `@beta` and
`record` into a single token because `@` is not a standalone token in
the grammar -- the combined `@betarecord` is meaningless. Neither tool
explains this.

**Example B** -- unknown annotation `@beta` before `protocol` (index #61):

```
@namespace("org.spf4j.base.avro.jmx")
@beta
protocol Jmx { ... }
```

```
Rust: parse error: <input>: line 3:0 no viable alternative at input
      '@namespace("org.spf4j.base.avro.jmx")@betaprotocol'
Java: SchemaParseException: line 3:0 no viable alternative at input
      '@namespace("org.spf4j.base.avro.jmx")@betaprotocol'
```

**Example C** -- `?` in schema-mode file (index #228):

```
// entity.avdl line 14:
float? confidence;
```

```
Rust: parse error: <input>: line 16:15 extraneous input '?' expecting
      {DocComment, 'protocol', 'namespace', 'import', ...}
Java: SchemaParseException: Cannot read field "word" because "ctx" is null
```

Here the Rust error is already slightly better (at least mentioning `?`)
but "extraneous input `?` expecting {DocComment, 'protocol', ...}" is a
wall of grammar symbols that does not help the user understand the
problem.

### Pattern 2: Default value validation errors vs. Java's opaque `NoSuchElementException` (5 cases)

The Rust tool reports a specific field and explains the type mismatch
(e.g., "expected array, got null"). Java reports an opaque
`NoSuchElementException` with no field name or explanation. In these
cases Rust is arguably already slightly better, but the error message
could still be improved because it does not include a source location
or suggest how to fix the issue.

**Example D** -- `bytes comment = null;` without nullable type (index #115):

```
// fastq.avdl line 9:
bytes comment = null;
```

```
Rust: walk IDL parse tree for `<input>`: Invalid default for field `comment`:
      expected bytes, got null
Java: SchemaParseException: java.util.NoSuchElementException
```

**Example E** -- `array<FL_Property> properties = null;` (indices #17, #159):

```
Rust: Invalid default for field `properties`: expected array, got null
Java: SchemaParseException: java.util.NoSuchElementException
```

**Example F** -- `array<FL_SearchResult> results = null;` (index #154):

```
Rust: Invalid default for field `terms`: expected array, got null
Java: SchemaParseException: java.util.NoSuchElementException
```

**Example G** -- `array<Variant> variants = { ... }` (index #106):

```
Rust: Invalid default for field `variants`: expected array, got object
Java: SchemaParseException: java.util.NoSuchElementException
```

## Root cause

**Pattern 1:** ANTLR syntax errors now include source spans (added
in `07d8d21`), which makes the Rust output strictly better than
Java's for all Pattern 1 cases. The remaining improvement
opportunity is **semantic enrichment**: detecting common error
patterns (like `@beta` being an unknown annotation) and adding
human-readable suggestions instead of just the raw ANTLR grammar
error. The source span display is excellent, but the error *text*
is still the raw ANTLR message.

**Pattern 2:** Default-value validation errors in the tree walker
(e.g., `walk_field_default`) produce a message with the field name
and expected/actual types, but without a source file location
(line/column). The error propagates via `miette::bail!` without a
`ParseDiagnostic` span. (See also
`issues/90b85d2d-number-parsing-errors-missing-source-span.md`.)

## Affected files

- `src/reader.rs` -- `CollectingErrorListener::syntax_error()` and
  the post-parse error formatting at lines 234-252
- `src/error.rs` -- `ParseDiagnostic` struct (already supports
  source spans but is not used for ANTLR errors or default validation
  errors)

## Reproduction

Run any of the following fuzz-test inputs:

```sh
# Pattern 1: @beta annotation (index #32)
cargo run -- idl tmp/fuzz-inputs/zolyfarkas--core-schema/src/main/avro/execution.avdl

# Pattern 1: @beta before protocol (index #61)
cargo run -- idl tmp/fuzz-inputs/zolyfarkas--core-schema/src/main/avro/jmx.avdl

# Pattern 1: extraneous ? in schema mode (index #228)
cargo run -- idl tmp/fuzz-inputs/Adam-Horse--DontBetOn/dataops/avro-schemas/src/entities/entity.avdl

# Pattern 2: bytes default null (index #115)
cargo run -- idl tmp/fuzz-inputs/ytchen0323--bwa-spark-fpga/src/main/avro/fastq.avdl
```

Or use a minimal reproducer:

```avdl
// pattern1-beta.avdl -- unknown @beta annotation
@namespace("test")
protocol Test {
    @beta
    record Foo { string name; }
}
```

```avdl
// pattern2-default.avdl -- null default on non-nullable bytes
protocol Test {
    record Foo { bytes data = null; }
}
```

## Suggested fix

### Pattern 1: Add semantic suggestions to ANTLR parse errors

Source spans are already implemented (`07d8d21`). The remaining work
is semantic enrichment of the error message text:

1. **Detect common patterns and add suggestions.** After collecting
   the raw ANTLR error, pattern-match on known error shapes:
   - If the error text contains `@<word>` where `<word>` is not a
     recognized annotation name (`namespace`, `aliases`, `order`,
     `logicalType`, etc.), suggest: "unknown annotation `@<word>` --
     custom annotations are not supported in Avro IDL; use a known
     annotation or remove it."
   - If the error mentions `'?'` as extraneous input, suggest: "the
     nullable `?` suffix may not be supported in this context" or
     explain that `?` requires protocol mode.

### Pattern 2: Add source locations to default-validation errors

1. **Thread token positions through `walk_field_default`.** The field
   node's ANTLR context has start/stop tokens with line and column
   info. Use these to construct a `ParseDiagnostic` with a source span
   pointing at the default value expression.

2. **Suggest the fix.** When the default is `null` but the field type
   is non-nullable, suggest: "use `<type>? fieldName = null;` to make
   the field nullable, or provide a non-null default value."

### Expected outcome

After these improvements, the Rust error messages would not only have
better presentation (source spans, already done) but also better
semantic content (suggestions, remaining work). The Pattern 1 cases
are already strictly better than Java after `07d8d21`; these
suggestions would make them even more helpful.

## Source

Fuzz testing of 229 real-world `.avdl` files collected from GitHub.
52 files caused both tools to fail. 43 already had better Rust errors,
9 were roughly equal (this issue). 0 cases had better Java errors.
