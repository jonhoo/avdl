# Upstream Bug Report Workflow

Convert internal `upstream-issues/*.md` notes into polished, JIRA-ready
bug reports (`.jira.md`) suitable for filing against the Apache Avro
project. Run this for each upstream issue that isn't already a `.jira.md`.

## Prerequisites

- **Java tool JAR:** `avro-tools-1.12.1.jar` (see CLAUDE.md for path)
- **Java source:** `avro/lang/java/idl/src/main/java/org/apache/avro/idl/`
- **ANTLR grammar:** `avro/share/idl_grammar/org/apache/avro/idl/Idl.g4`
- **Avro specification (local):**
  `avro/doc/content/en/docs/1.12.0/Specification/_index.md`
- **Avro IDL language (local):**
  `avro/doc/content/en/docs/1.12.0/IDL Language/_index.md`
- **Spec URLs (for citation):**
  https://avro.apache.org/docs/1.12.0/specification/ and
  https://avro.apache.org/docs/1.12.0/idl-language/
- **Existing `.jira.md` files** in `upstream-issues/` serve as format
  examples

---

## Phase 1: Triage

1. Read the source `upstream-issues/<uuid>-<desc>.md` file.
2. Identify the affected Java source files from the issue or by
   searching `avro/lang/java/`.
3. Confirm the issue is about the Java tool (not our Rust port).

## Phase 2: Investigate the Java source

1. Read the relevant Java source files identified in Phase 1.
   `IdlReader.java` is the primary listener; the ANTLR grammar
   (`Idl.g4`) defines the syntax rules.
2. Trace the exact code path that causes the bug. Record:
   - File paths and line numbers
   - The mechanism (stack operations, missing validation, scoping
     errors, etc.)
   - The grammar rule chain if the bug involves parsing
3. Check whether any existing Java tests cover this case:
   - `avro/lang/java/idl/src/test/idl/input/` (golden `.avdl` files)
   - `avro/lang/java/idl/src/test/java/` (unit tests)
4. Check for source comments that acknowledge the issue.

## Phase 3: Confirm this is actually an upstream bug

Before investing in a polished report, build confidence that the
observed behavior is genuinely a bug — not a misunderstanding of the
IDL semantics, an intentional design choice, or a known limitation.

1. **Read the specification.** Check both the local Avro specification
   and IDL language docs (see Prerequisites for paths). Look for:
   - Explicit prohibition of the behavior ("Unions may not...")
   - Implicit constraints (e.g., a grammar production that the spec
     says is only valid in certain contexts)
   - Ambiguity — if the spec doesn't clearly say the behavior is
     wrong, the case is weaker

2. **Read Java source comments.** Search `IdlReader.java` and related
   files for comments near the affected code path. Look for:
   - `// TODO` or `// FIXME` that acknowledge the issue
   - Comments explaining *why* a particular behavior was chosen
   - Javadoc on public methods that describes intended semantics
   - References to JIRA tickets (e.g., `AVRO-1234`)

3. **Check the Java test suite for intent signals.** Look at
   `avro/lang/java/idl/src/test/` for tests that exercise related
   behavior. A test that explicitly asserts the behavior you think is
   buggy is strong evidence it's intentional. An absence of tests is
   neutral — it may simply be untested.

4. **Consider alternative explanations:**
   - Could this be undefined behavior that the spec intentionally
     leaves to implementations?
   - Could the spec have changed between versions, making the Java
     tool correct for an older spec?
   - Is this a case where the Java tool's behavior, while surprising,
     produces output that is still valid Avro?

5. **Classify the result:**
   - **Confirmed bug:** The spec explicitly prohibits the behavior,
     or the tool produces output that is invalid per the spec.
     Proceed to Phase 4.
   - **Likely bug:** The spec is ambiguous, but the tool produces
     output that no Avro consumer can parse, or silently loses data.
     Proceed, but note the ambiguity in the report description.
   - **Not a bug:** The behavior is intentional, documented, or
     within the spec's latitude. Do not write a `.jira.md`. Instead,
     add a note to the original `.md` explaining why it's not
     upstream-actionable, and move the file to `issues/` if the
     behavior still affects our Rust port.

## Phase 4: Write a minimal reproduction

1. Write a standalone `.avdl` file in `tmp/` that triggers the bug.
   Strip everything not necessary — no extra records, no imports, no
   unrelated annotations. The goal is the smallest input that
   demonstrates the problem.
2. Run it through the Java tool:
   ```sh
   java -jar avro-tools-1.12.1.jar idl tmp/repro.avdl tmp/repro.avpr
   ```
3. Capture the exact output (JSON, error message, or stack trace).
4. If the bug is silent corruption (valid exit code, invalid output),
   show the full JSON output and highlight the invalid portion.

## Phase 5: Verify the spec quote for citation

Phase 3 already established that the behavior violates the spec.
This phase captures the exact text for the report.

1. Find the relevant section of the Avro specification or IDL
   language reference (see Prerequisites for local file paths).
2. Quote the exact text that the Java behavior violates.
3. Record the published URL for citation in the report (e.g.,
   `https://avro.apache.org/docs/1.12.0/specification/#unions`).
4. If the spec is ambiguous, explain why the behavior is still wrong
   (e.g., produces invalid schema JSON that no consumer can parse).

## Phase 6: Write the `.jira.md`

Create `upstream-issues/<uuid>-<desc>.jira.md` using the template
below. Then **delete the original `.md` file** — it is superseded by
the `.jira.md` and should not remain alongside it.

### Content rules

- **Factual and technical.** No editorial commentary, no blame, no
  "should be easy to fix."
- **No references to this project** (the Rust port). The report must
  stand on its own as an Apache Avro bug.
- **One bug per report.** If investigation reveals multiple bugs,
  write separate `.jira.md` files.
- **Specific title.** Include the component prefix `[IDL]` and
  describe the observable symptom, not the root cause.
- **Minimal reproduction.** Self-contained — no external imports,
  no classpath dependencies, no references to test suite files.
- **Full output.** Include the complete error message, stack trace,
  or (for silent bugs) the full JSON showing the corruption.
- **Root cause with line references.** Reference specific files and
  line numbers in the Java source at the stated Affects Version.
- **Suggested fix.** At least one concrete approach. Two is better
  (gives the maintainer options).

---

## `.jira.md` template

```markdown
# [IDL] <Descriptive title — observable symptom>

- **Component:** java / idl
- **Affects Version:** <version, e.g. 1.12.1>

## Description

<1-2 paragraphs: what the bug is, quoting the specification if it
explicitly prohibits the behavior.>

### Minimal reproduction

\`\`\`avdl
<standalone .avdl input — smallest possible>
\`\`\`

\`\`\`
$ java -jar avro-tools-<version>.jar idl repro.avdl output.avpr
<exact command output, error message, or "no error">
$ cat output.avpr
<JSON output if relevant>
\`\`\`

### Expected behavior

<What should happen per the specification.>

### Actual behavior

<What actually happens. Include full JSON/error output.>

## Root cause

<Trace through the Java source with file names and line numbers.
Explain the mechanism — stack misalignment, missing validation,
scoping error, etc.>

## Suggested fix

<At least one concrete approach. Reference specific methods or
grammar rules to modify.>
```

---

## Verification checklist

Before considering the `.jira.md` complete:

- [ ] Phase 3 classification is "confirmed bug" or "likely bug"
- [ ] Original `.md` has been deleted
- [ ] Reproduction is self-contained (no imports, no classpath, no
      test-suite references)
- [ ] Reproduction is minimal (nothing can be removed without losing
      the bug)
- [ ] Exact commands and output are included
- [ ] Specification reference is accurate and quoted
- [ ] Root cause line numbers match the stated Affects Version
- [ ] No references to this project (the Rust port) appear anywhere
- [ ] Title is descriptive and uses `[IDL]` prefix
- [ ] "Suggested fix" section has at least one concrete approach
- [ ] Report covers exactly one bug

---

## Agent prompt template

For batch conversion, launch one sub-agent per issue:

> You are investigating an internal bug report to determine whether
> it is genuinely an upstream Apache Avro bug, and if so, converting
> it into a JIRA-ready bug report. The project is at
> `/home/jon/dev/stream/avdl/main/`.
>
> Read the workflow at `workflow-prompts/upstream-bug-report.md` and
> follow it phase by phase. **Phase 3 (confirm it's actually a bug)
> is critical** — do not skip it. If you conclude the behavior is
> intentional or within spec latitude, report that finding instead
> of writing a `.jira.md`.
>
> **Source issue:** `upstream-issues/<filename>.md`
>
> **Output (if confirmed):** `upstream-issues/<filename>.jira.md`
>
> After writing the `.jira.md`, delete the original `.md`.
> If the issue is not a bug, do not write a `.jira.md` — instead,
> add a note to the original `.md` explaining why.
>
> Key references:
> - Java source: `avro/lang/java/idl/src/main/java/org/apache/avro/idl/`
> - Grammar: `avro/share/idl_grammar/org/apache/avro/idl/Idl.g4`
> - Avro spec (local): `avro/doc/content/en/docs/1.12.0/Specification/_index.md`
> - Avro IDL (local): `avro/doc/content/en/docs/1.12.0/IDL Language/_index.md`
> - Existing `.jira.md` files in `upstream-issues/` for format examples
>
> Use `tmp/` for reproduction files. Run the reproduction through
> `java -jar` to capture exact output. Run through the verification
> checklist before finishing.

---

## Apache JIRA conventions

These are extracted from the Apache contribution guidelines for
reference. The `.jira.md` files are drafts — final JIRA tickets may
need minor formatting adjustments when filed.

- **Project key:** AVRO
- **Title:** Specific and descriptive. Bad: "Bug in parser." Good:
  "[IDL] Nested union silently produces empty type array."
- **Four-element structure:** (1) what you intended, (2) steps to
  reproduce, (3) expected result, (4) actual result.
- **Environment:** OS, Java version, Avro version.
- **One bug per ticket.**
- **No sensitive data** (credentials, API keys).
- **Search for duplicates** before filing — check the AVRO JIRA
  project for similar titles.
- **Mailing list:** `dev@avro.apache.org` for discussion if unsure
  whether something is a bug vs. intended behavior.

---

## Tips

1. **Silent corruption is worse than a crash.** When the Java tool
   produces invalid output without an error, emphasize this in the
   description — downstream consumers will get confusing failures
   with no indication that the IDL tool is the source.
2. **The grammar and the semantic layer are separate.** Many IDL bugs
   stem from the grammar allowing syntax that the specification
   prohibits, with no semantic validation to catch it. When this is
   the pattern, note both the grammar rule that permits it and the
   missing validation in `IdlReader.java`.
3. **Check whether the golden test suite covers the case.** If no
   golden test exists, mention it — it explains why the bug went
   undetected and is useful context for the maintainer.
