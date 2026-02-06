# Nested named schema declarations inside records not handled

## Status: Not a bug — grammar limitation

## Original symptom

The Java IDL parser was believed to support declaring named schemas
(records, enums, fixed) inside a record body. Our parser ignores the
`_registry` parameter in `walk_record` and does not walk nested named
schema declarations.

## Investigation

The ANTLR grammar (`Idl.g4`) defines:

    recordBody : LBrace fields+=fieldDeclaration* RBrace;

The `recordBody` rule only permits `fieldDeclaration` children. It
does **not** include `namedSchemaDeclaration`, unlike
`protocolDeclarationBody` which does:

    protocolDeclarationBody : LBrace
        (imports+=importStatement
        |namedSchemas+=namedSchemaDeclaration
        |messages+=messageDeclaration)* RBrace ;

The Java reference implementation (`IdlReader.java`) confirms this:
`enterRecordBody` only sets up the record schema and namespace; it
does not handle nested `namedSchemaDeclaration` children, because the
grammar does not produce any.

No test files in the Avro test suite use nested type declarations
inside record bodies, further confirming this syntax is not supported.

## Conclusion

This is a grammar limitation, not a bug in our code. The `_registry`
parameter in `walk_record` is unused because the grammar does not
permit nested named schema declarations in record bodies.

The `_registry` parameter has been removed from `walk_record` since
it serves no purpose given the grammar constraint.

## Difficulty

N/A — no code fix needed.
