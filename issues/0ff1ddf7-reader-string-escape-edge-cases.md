# String escape handling edge cases untested in reader.rs

- **Symptom**: Several branches in the string escape processing code in
  `reader.rs` are at 0% coverage:
  1. **Invalid Unicode code point** (lines 2658-2660): When a `\uXXXX`
     escape decodes to an invalid `char::from_u32` result.
  2. **Malformed \u escape** (lines 2662-2664): When `\u` is not
     followed by enough hex digits.
  3. **Invalid octal escape** (lines 2692-2698): When an octal escape
     sequence is malformed or produces an invalid code point.
  4. **Trailing backslash** (lines 2705-2708): A string ending with a
     lone `\`.
  5. **Surrogate pair parsing failures** (lines 2740-2741, 2750-2751):
     High surrogate not followed by valid `\u` low surrogate.

- **Root cause**: The test suite's `.avdl` files don't exercise these
  edge cases in string literals. The string escape code faithfully ports
  Java's handling (including surrogate pairs and octal escapes), but
  tests only cover the common ASCII and simple Unicode cases.

- **Affected files**: `src/reader.rs` (lines 2640-2710, 2724-2751)

- **Reproduction**: Run `cargo llvm-cov --text` and observe hit count 0
  on the lines listed above.

- **Suggested fix**: Add unit tests for `interpret_string_literal` (or
  the enclosing function) with edge-case inputs:
  - `"\u{invalid}"` (too few hex digits)
  - `"\uD800"` (lone high surrogate)
  - `"\uD800\u0041"` (high surrogate followed by non-low-surrogate)
  - `"\777"` (octal escape)
  - `"trailing backslash\"`
