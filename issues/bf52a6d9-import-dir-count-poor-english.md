# "0 import dir(s)" is poor English in error messages

## Symptom

The "import not found" error message uses a raw count with a
parenthesized plural suffix:

```
import not found: foo.avdl (searched relative to ./bar and 0 import dir(s))
```

"0 import dir(s)" reads awkwardly. Other counts like "1 import
dir(s)" are equally clunky.

## Affected files

- `src/import.rs` (~line 99): the `import not found` error message

## Suggested fix

Use proper English pluralization:

- 0 → "no additional import directories"
- 1 → "1 import directory"
- n → "{n} import directories"

Alternatively, list the searched directories by name rather than just
reporting a count, which would be more actionable for the user (see
also `df6af1a1-import-not-found-should-list-searched-dirs.md`, which
proposes exactly that — fixing both issues together would be natural).
