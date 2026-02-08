# Upstream Issues

Likely bugs in the Apache Avro Java tool (`avro-tools`) discovered
while building the Rust port. These are tracked separately from
`issues/` because they cannot be fixed in this project — they need
to be filed upstream against the
[AVRO](https://issues.apache.org/jira/projects/AVRO) JIRA project.

"Likely" because each issue reflects our best understanding of the
Avro specification, but we may be wrong. The conversion workflow
includes a dedicated phase for verifying each issue against the spec
and Java source before writing a report.

## File types

- **`.md`** — Raw notes awaiting investigation and conversion.
- **`.jira.md`** — Investigated and written up as JIRA-ready bug
  reports, following the
  [Apache bug writing guide](https://infra.apache.org/bug-writing-guide.html).

## Workflow

To convert a `.md` into a `.jira.md`, follow the workflow in
[`workflow-prompts/upstream-bug-report.md`](../workflow-prompts/upstream-bug-report.md).

## Naming convention

`<uuid-prefix>-<short-description>.<ext>`

Use the first 8 characters of `$(uuidgen)` as the prefix.
