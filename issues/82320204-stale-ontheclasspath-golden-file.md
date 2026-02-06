# Stale `OnTheClasspath.avpr` golden file

## Symptom

The golden file `putOnClassPath/OnTheClasspath.avpr` contains a record
type `VeryFar` which does not appear in any source `.avdl` file. Our
Rust output for `OnTheClasspath.avdl` with `--import-dir` produces
`FromAfar` + `NestedType`, which matches what the source files actually
define.

## Root cause

The golden `.avpr` file appears to be stale — it was likely generated
from an older version of the source `.avdl` files and never updated
when `VeryFar` was removed or renamed.

## Affected files

- `avro/lang/java/idl/src/test/idl/putOnClassPath/OnTheClasspath.avpr`
  (golden file with `VeryFar`)
- `avro/lang/java/idl/src/test/idl/putOnClassPath/OnTheClasspath.avdl`
  (source, defines `FromAfar`)
- `avro/lang/java/idl/src/test/idl/putOnClassPath/nestedtypes.avdl`
  (source, defines `NestedType`)

## Reproduction

```sh
# Our output:
cargo run -- idl \
  --import-dir avro/lang/java/idl/src/test/idl/input \
  --import-dir avro/lang/java/idl/src/test/idl/putOnClassPath \
  avro/lang/java/idl/src/test/idl/putOnClassPath/OnTheClasspath.avdl

# Compare against the golden file:
diff <(jq -S . tmp/ontheclasspath.avpr) \
     <(jq -S . avro/lang/java/idl/src/test/idl/putOnClassPath/OnTheClasspath.avpr)
```

The diff will show `VeryFar` in the golden file but `FromAfar` +
`NestedType` in our output.

## Suggested fix

This is an upstream issue in the Avro test suite's golden files. No
code change needed on our side. When writing tests that compare against
`OnTheClasspath.avpr`, the test should either:

- Skip golden-file comparison for this file and instead assert the
  expected types structurally, or
- Use the Rust output as the reference (since it matches the source
  files).
