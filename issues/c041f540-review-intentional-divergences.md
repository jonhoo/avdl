# Review intentional divergences from Java and populate README section

## Symptom

The README has a new "Intentional divergences from Java" section with
only the byte-identical formatting entry. There are likely other
intentional differences scattered across the codebase (closed issues,
TODO comments, CLAUDE.md notes) that should be documented there.

## Suggested approach

1. Search closed issues (via `git log --diff-filter=D -- issues/`) for
   issues that were closed as design choices or non-goals.
2. Search `src/` for TODO comments that mention intentional divergence
   or Java compatibility decisions.
3. Check `CLAUDE.md` for documented non-goals and architecture decisions
   that imply behavioral differences.
4. For each divergence found, add a one-line bullet to the README
   section. Each item should be concise enough to scan quickly — a bold
   summary phrase followed by a single sentence of context.
5. Remove the `<!-- TODO -->` comment from the README section once the
   list is populated.

## Affected files

- `README.md`
- `issues/` (closed issues as input)
- `src/` (TODO comments as input)

## Reproduction

N/A — this is a documentation task.
