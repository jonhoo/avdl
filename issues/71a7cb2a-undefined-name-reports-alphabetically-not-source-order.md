# Undefined name error reports alphabetically-first type, not source-order-first

## Symptom

When multiple undefined type names appear in a file, the error
diagnostic points to the alphabetically-first undefined name rather
than the one that appears first in the source file. This is confusing
because the user expects the first error to correspond to the first
problem in their file.

For example, given:

```avdl
@namespace("org.test")
protocol Test {
  record Foo {
    stringg name;   // line 4
    intt age;       // line 5
    boool flag;     // line 6
  }
}
```

The tool reports `Undefined name: org.test.boool` (line 6) instead
of `Undefined name: org.test.stringg` (line 4). `boool` sorts first
alphabetically.

Additionally, only one undefined name is reported per run, so the
user must fix each one and re-run to discover the next, always seeing
the alphabetically-first remaining error rather than the one nearest
the top of their file.

## Root cause

In `compiler.rs` around line 852, `report_unresolved_references`
sorts the unresolved list alphabetically by name before picking the
first entry with a source span:

```rust
unresolved.sort_by(|a, b| a.0.cmp(&b.0));
unresolved.dedup_by(|a, b| a.0 == b.0);
```

The alphabetical sort is used for deduplication but has the side
effect of reordering errors away from source-file order.

## Affected files

- `src/compiler.rs` -- `report_unresolved_references` function

## Reproduction

```sh
cat > tmp/multi-undef.avdl <<'EOF'
@namespace("org.test")
protocol Test {
  record Foo {
    stringg name;
    intt age;
    boool flag;
  }
}
EOF
cargo run -- idl tmp/multi-undef.avdl 2>&1
# Reports: Undefined name: org.test.boool (line 6)
# Expected: Undefined name: org.test.stringg (line 4)
```

## Suggested fix

Sort by source span offset (when available) instead of by name, so
the first error in the file is reported first. Deduplication can still
work by collecting into an `IndexSet` keyed on name, preserving
insertion (span) order. This would also make the fix-one-rerun cycle
more natural for the user.
