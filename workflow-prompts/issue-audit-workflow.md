# Issue Audit Workflow

Reusable workflow for auditing resolved issues. Run this after closing
a batch of issues to verify each was truly and completely fixed.

## 1. Extract deleted issue contents

From the project root directory:

```sh
mkdir -p tmp/deleted-issues

# Extract the content of every issue file that was deleted in git history.
# Each file is recovered from the commit just before its deletion.
git log --all --diff-filter=D --name-only --format="COMMIT:%H" -- issues/ \
  | awk '/^COMMIT:/{commit=substr($0,8)} /^issues\//{print commit, $0}' \
  | while read commit file; do
      base="$(basename "$file")"
      if [ ! -f "tmp/deleted-issues/$base" ]; then
        git show "$commit^:$file" > "tmp/deleted-issues/$base" 2>/dev/null
      fi
    done

echo "Extracted $(ls tmp/deleted-issues/ | wc -l) issue files"
```

To limit the audit to issues deleted since a specific commit (e.g.
only issues closed since the last audit):

```sh
git log <since-commit>..HEAD --diff-filter=D --name-only --format="COMMIT:%H" -- issues/ \
  | awk '/^COMMIT:/{commit=substr($0,8)} /^issues\//{print commit, $0}' \
  | while read commit file; do
      base="$(basename "$file")"
      git show "$commit^:$file" > "tmp/deleted-issues/$base" 2>/dev/null
    done
```

After running an audit, record the current HEAD commit hash somewhere
(e.g. in `SESSION.md`) so the next audit can use it as
`<since-commit>`.

## 2. Batch issues by code area

Group issues into batches of 8-12 by the subsystem they affect.
Suggested groupings (adapt to the actual issue set):

| Batch | Theme                           | Key source files                         |
|-------|---------------------------------|------------------------------------------|
| 1     | Name validation & namespaces    | `src/resolve.rs`, `src/reader.rs`        |
| 2     | Schema-level validation         | `src/reader.rs`                          |
| 3     | Import resolution               | `src/import.rs`, `src/main.rs`           |
| 4     | Literal parsing & strings       | `src/reader.rs`, `src/model/json.rs`     |
| 5     | Schema mode & idl2schemata      | `src/reader.rs`, `src/main.rs`           |
| 6     | Doc comments, warnings, errors  | `src/doc_comments.rs`, `src/error.rs`    |
| 7     | JSON serialization & model      | `src/model/json.rs`, `src/model/schema.rs` |
| 8     | Logical types & properties      | `src/reader.rs`, `src/import.rs`         |
| 9     | CLI, testing, tooling           | `src/main.rs`, `scripts/`, `tests/`      |

Not every audit will have issues in all 9 batches. Assign issues to
whichever batch best matches, and skip empty batches.

## 3. Agent prompt template

For each batch, launch a `general-purpose` sub-agent. Run up to 3
agents in parallel per wave. Each agent's prompt should include:

> You are auditing deleted issue files from an Avro IDL compiler
> project to determine if each was truly fixed. The project is at
> `/home/jon/dev/stream/avdl/main/`.
>
> Read each issue from `tmp/deleted-issues/`, then search the
> codebase for evidence of the fix. For each issue, classify it as:
>
> - **FIXED**: Fix exists AND tested (unit or integration)
> - **FIXED-NO-TEST**: Fix exists but no dedicated regression test
> - **PARTIALLY-FIXED**: Some aspects addressed, others remain
> - **NOT-FIXED**: No evidence the issue was addressed
> - **NON-ISSUE**: Closed as not-a-bug, intentional, or non-goal
> - **UPSTREAM**: Issue in the upstream Avro project, not our code
>
> Key files to search: `<list files for this batch>`
>
> Issues to investigate:
> 1. `tmp/deleted-issues/<filename>.md` -- short description
> 2. ...
>
> For each issue:
> 1. Read the full issue content
> 2. Search the specific code area for evidence of the fix
> 3. Check for tests covering the fix
> 4. Classify the issue
>
> For any issue classified as PARTIALLY-FIXED or NOT-FIXED, create a
> new issue file in `issues/` with a `$(uuidgen)` prefix using the
> re-filed issue format from section 5.
>
> For any issue that turns out to be an upstream bug (in the Apache
> Avro project itself), file it in `upstream-issues/` instead.
>
> Record informational observations (not bugs, not accomplishments)
> in `SESSION.md`.
>
> Output a summary table at the end.

## 4. Classification criteria

An issue is **FIXED** if ALL of the following hold:

- The specific edge case described in the issue now produces correct
  behavior (not just that code exists in the general area).
- There is a unit test or integration test that would catch a
  regression, OR the fix is trivially obvious from code inspection.
- The fix matches Java behavior described in the issue (unless the
  issue was explicitly closed as a non-goal or intentional
  divergence).

An issue is **FIXED-NO-TEST** if the code fix is present and correct
by inspection, but no dedicated test would catch a regression. This
does not require a new issue to be filed -- it is an informational
classification. Common for shell script fixes, CLI behavior changes,
and edge cases not covered by golden files.

An issue is **PARTIALLY-FIXED** if the core logic exists but some
aspect described in the issue remains unaddressed (e.g. the
validation exists but lacks a dedicated test, or only some of the
described cases are handled).

An issue is **NOT-FIXED** if there is no evidence the issue was
addressed. A TODO comment acknowledging the issue does not count as
a fix.

An issue is **NON-ISSUE** if it was closed as not-a-bug, intentional
behavior, or a non-goal (e.g. byte-identical output formatting).

An issue is **UPSTREAM** if it describes a bug in the Apache Avro
project itself (Java tool, grammar, golden files) rather than in
our Rust implementation.

## 5. Re-filed issue format

```markdown
# <Title>

## Symptom
<What's wrong or missing>

## Status
Re-opened during audit of deleted issues. Original issue: `<original filename>`.

## Evidence of partial fix (if any)
<What was already fixed and where>

## Remaining work
<What still needs to be done>

## Affected files
<Current file paths>

## Reproduction
<Steps or commands to reproduce the problem>
```

## 6. Verification

After all agents complete:

```sh
cargo test                # confirm no regressions
ls issues/                # list newly created issue files
ls upstream-issues/       # check for newly filed upstream issues
```

Review each newly filed issue for accuracy and completeness.

## 7. Lessons from the first audit (83 issues, Feb 2026)

- **FIXED-NO-TEST was the most common non-FIXED classification**
  (~15 of 83 issues). Most were shell script fixes or CLI behavior
  changes that can't be tested by the Rust test suite.
- **Agents sometimes filed issues proactively** beyond their audit
  mandate (e.g. an `expect()` audit issue). Review agent output for
  unexpected new files.
- **Duplicate original issues exist.** Two or more deleted issues
  sometimes describe the same underlying bug from different angles.
  Agents should recognize these and consolidate into a single
  re-filed issue.
- **The `NON-ISSUE` classification is important.** Several issues
  were intentionally closed as non-goals (e.g. byte-identical JSON
  formatting) or as upstream bugs. Agents should check `CLAUDE.md`
  for documented non-goals.
- **3 agents per wave works well.** Context window usage per agent
  was 50-100k tokens. Larger batches (12+ issues) risk hitting
  limits.
