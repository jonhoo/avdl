**`compare-golden.sh idl2schemata` only tests 6 of 18 files.**

The `IDL2SCHEMATA_FILES` list in `scripts/compare-golden.sh` is
hardcoded to 6 files. All 18 `.avdl` inputs pass `idl2schemata`
comparison against Java — the remaining 12 should be added to the
list.

- **Symptom**: `scripts/compare-golden.sh idl2schemata` only covers a
  fraction of available test inputs
- **Root cause**: hardcoded file list in the script
- **Affected files**: `scripts/compare-golden.sh`
- **Reproduction**: `scripts/compare-golden.sh idl2schemata` — only
  reports 6 files
- **Suggested fix**: Expand `IDL2SCHEMATA_FILES` to include all 18
  `.avdl` files, or switch to dynamic file discovery
