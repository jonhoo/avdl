# No test for rejecting one-way messages with `throws` clause

## Symptom

The Avro specification states that one-way messages must have a `null`
response and no errors. The grammar allows both `oneway` and `throws`
as alternatives in the same production:

    (oneway=Oneway | Throws errors+=identifier ...)?

So they are mutually exclusive at the grammar level -- a message cannot
syntactically have both `oneway` and `throws`. However, no test
verifies this grammar-level constraint. If the grammar were ever
relaxed (e.g., to allow both and check semantically), the semantic
check in `walk_message_declaration` does not currently reject `oneway`
messages with error declarations.

There is a test (`oneway_nonvoid_return_is_rejected`) that verifies
one-way messages must return `void`, but no test for the `throws`
constraint.

## Root cause

The grammar's alternation (`oneway | throws ...`) enforces mutual
exclusion syntactically, but this is an implicit guarantee with no
explicit test. The semantic check (reader.rs line 2284) only verifies
the return type, not the absence of errors.

## Affected files

- `src/reader.rs` (`walk_message_declaration`)
- `tests/integration.rs` (missing test)

## Reproduction

The grammar rejects this syntactically, so a parse-level test would
suffice:

```avdl
protocol P {
    error E { string msg; }
    void fire(string s) oneway throws E;
}
```

This should produce a parse error from ANTLR.

## Suggested fix

Add a unit test confirming that a message with both `oneway` and
`throws` is rejected. Since the grammar enforces this, the test can
simply verify the parse fails. This protects against future grammar
changes that might relax the alternation.
