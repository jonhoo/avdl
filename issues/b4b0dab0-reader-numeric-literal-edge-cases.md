# Hex and octal integer/float literal parsing untested in reader.rs

- **Symptom**: The hex and octal integer literal parsing branches in
  `parse_integer_literal` (lines 2782-2800 of `reader.rs`) are at 0%
  coverage. Similarly, `parse_float_text` has untested hex float
  parsing (lines 2908-2935) and `Infinity` output (line 2831). The
  long suffix (`L`) stripping path (lines 2774-2775) is also uncovered.

- **Root cause**: The Avro test suite `.avdl` files don't use hex
  (`0xFF`), octal (`0777`), negative hex (`-0xFF`), long-suffixed
  (`42L`), or hex float (`0x1.8p10`) literals as default values.
  These are valid per the ANTLR grammar's `IntegerLiteral` and
  `FloatingPointLiteral` rules but are uncommon in practice.

- **Affected files**: `src/reader.rs` (lines 2773-2800, 2831,
  2908-2935, 2958-2959)

- **Reproduction**: Run `cargo llvm-cov --text` and observe that the
  hex/octal branches in `parse_integer_literal` and
  `parse_float_text` show hit count 0.

- **Suggested fix**: Add unit tests or inline IDL tests for:
  - `int field = 0xFF;` (hex)
  - `int field = 0777;` (octal)
  - `long field = 42L;` (long suffix)
  - `int field = -0xFF;` (negative hex)
  - `long field = -0777L;` (negative octal, long suffix)
  - `double field = 0x1.8p10;` (hex float)
  - `float field = Infinity;` (positive infinity)
