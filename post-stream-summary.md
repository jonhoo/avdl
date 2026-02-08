# Post-Stream Development

The [YouTube livestream](https://youtube.com/live/NqV_KhDsMIs) ended at
commit `fac28dd`.
At that point the Rust `avdl` tool could parse most `.avdl` files and emit protocol JSON,
but had known issues with imports, schema mode, namespace handling, and edge cases.
What follows is a summary of ~30 hours of offline work that brought the tool to
full compatibility with the Java reference implementation—and
the agentic coding techniques that made it possible.

The project consumed roughly **1 billion tokens** (~$650 at per-token rates,
done on the $200/month Claude Code Max plan).
~98% of input tokens were cache hits,
typical for agentic sessions that repeatedly read the same project files.
All work used Claude Code with Opus 4.6 and occasional Haiku 4.5 for sub-agents.

## 1. A self-correcting loop beats a linear plan

The single most impactful technique was designing a **self-correcting development loop**—a
reusable prompt that Claude executes repeatedly,
converging toward zero issues.

The loop has three phases that repeat until Phase 1 finds nothing new:

1. **Discover**: launch parallel agents (one per `.avdl` test file) to
   compare Rust output against Java golden files.
   Each agent reports discrepancies but is *explicitly forbidden from fixing anything*.
   This separation is critical—discovery agents that also fix things
   tend to paper over root causes.

2. **File**: triage discoveries into `issues/`, one file per issue,
   with symptom, root cause, reproduction, and suggested fix.

3. **Fix**: create git worktree branches, dispatch agents to fix
   batched issues in parallel, merge back, and run the full test suite.

The loop ran 9 iterations over two days.
Early iterations found 8–12 issues each;
the final iteration found zero.
A 21-minute autonomous burst landed 12 commits—the loop running
with "continue without prompting me" at peak throughput.

**Parallel agents via git worktrees.**
Each fix phase dispatches multiple agents in separate git worktrees (`avdl-worktrees/wt-{a..k}`),
avoiding merge conflicts between concurrent agents editing the same files.
The orchestrator merges branches sequentially and runs the test suite after each merge.
A learned refinement: batch related fixes into single branches
to reduce merge conflicts.

**Context conservation.**
Sessions routinely exceeded 100k tokens,
so protecting the main context window was essential.
Discovery, code review, and multi-file fixes all ran as sub-agents—the
main context only sees summaries.
Issue files (`issues/<uuid>.md`) served as persistent memory across sessions.
Periodically triaging SESSION.md into issue files and clearing it
prevented stale context from misleading the model.

**Cost.**
The loop consumed the bulk of the $570 post-stream budget.
Sub-agents accounted for two-thirds of total spend—each
discovery wave launched parallel agents across all 18 test files.
The single most expensive session ($118 in sub-agent costs alone)
ran iterations 6–8 plus the full codebase review.

**Commits**: `889a643` (loop prompt draft),
`376e2ab`+`0bd8c91`+`cf2851c`+`a7a895e` (single parallel wave),
`0a33b70` (iteration 8).

## 2. Steer the process, not just the code

The loop prompt was refined between iterations based on observed failure modes.
Examples of real-time steering:

- *"Don't launch phase 3 yet, do my two new prompts first"*—injecting
  manual tasks between automated phases.
- *"At the end of this iteration, see if you recommend any changes to loop-prompt.md"*—asking
  Claude to meta-improve the loop itself.

**Progressive CLAUDE.md refinement.**
Conventions emerged in the loop prompt during iteration
(e.g., "use `tmp/` for intermediate files," "file issues with specific format").
The user periodically asked Claude to review which conventions were durable enough
for CLAUDE.md,
promoted them,
then deduplicated to avoid drift between the two files.

**Check in your reusable prompts.**
Prompts that live only in conversation history are lost at session boundaries
and can't be improved by future sessions.
This project checks three reusable prompts into `workflow-prompts/`:
the refinement loop, the closed-issue audit, and the upstream-bug-report workflow.
Treating prompts as versioned artifacts—like code—lets them accumulate improvements
across sessions.

**Commits**: `c23216a` (adapt CLAUDE to loop-prompt),
`ac3536f` (iteration 1 lessons),
`140539d` (sandbox discipline).

## 3. Look beyond the immediate task

### Use brutal-review on the whole tree, not just the diff

Rather than only reviewing the latest commit,
the `brutal-review` skill was run against the entire codebase,
producing a thorough, pedantic review.
Claude then triaged the review output for actionable items aligned with project goals.
This surfaced cross-cutting issues incremental reviews miss:
inconsistent error handling, `.unwrap()` calls that should be `.expect()`,
missing `.context()` on `?` operators, and stale comments from earlier refactors.

**Commits**: `d20966c` (.unwrap→.expect in tests),
`8fdef0e` (eliminate .expect via structural refactoring),
`5e9384a` (error context on bare `?`).

### Validate upstream before reporting

After reaching full compatibility,
attention shifted to bugs in the Java reference implementation itself.
The user taught Claude Apache contribution conventions
(JIRA format, minimal repros, high confidence requirements).
One "upstream issue" (`OnTheClasspath` files being different)
was correctly identified as *not a bug*—the
three files are intentionally different fixtures testing three import mechanisms.
The confirmed bug got a minimal reproduction and grammar evidence.

**Commits**: `c7dae31` (file upstream issue),
`339560e` (convert to JIRA format),
`0219834` (clarify OnTheClasspath fixtures).

### Fuzz test against real-world inputs

To validate beyond the official test suite,
229 `.avdl` files were collected from public GitHub repositories
and run through both the Rust and Java tools.
This caught edge cases the test suite didn't cover:
surrogate pairs in string literals, hex float literal suffixes,
dashed identifiers, duplicate types in unions.
Cost: ~$15 (search + execution).

**Commits**: `c4ea026` (regression tests from fuzz testing),
`4023040` (surrogate pairs),
`4c272db` (dashed identifiers).

### Ask "should we?" before "how do we?"

Several sessions involved investigating design choices rather than just implementing:

- *"Consider if we should remove the `preserve_order` feature"*—led
  to spec research and quoting the Avro specification before deciding.
- *"Does the binary-only nature of this tool change what error types we should use?"*—led
  to replacing `miette::Report` error chains with simpler `miette::miette!()` calls.

**Commits**: `d694ca8` (intentional divergences section),
`569471f` (replace `IdlError` with `miette::Result`).

## 4. Go beyond the port

### Rich error diagnostics

The Java tool's error messages are minimal.
The Rust version uses [miette](https://docs.rs/miette)-powered diagnostics
with source spans, file names, and highlighted regions:

```
Error: × duplicate field name `name`
   ╭─[input.avdl:5:12]
 4 │   string name;
 5 │   int name;
   ·        ────
   ╰─
```

This required threading source spans through ANTLR parse errors,
registry errors, and import resolution errors—work
that had no Java equivalent to port from.

**Commits**: `69eb914` (source spans via .wrap_err),
`ea034b1` (source spans for duplicate types),
`c74408` (actual filename in diagnostics).

### Dependency reduction

Late in the project,
`clap`, `thiserror`, and `miette-derive` were replaced with lighter alternatives
(`lexopt`, manual `Display`/`Error`, manual `Diagnostic`)—eliminating
proc-macro compile time for a CLI with only two subcommands.

**Commit**: `26021d7` (replace clap/thiserror/miette-derive).

### Benchmarking

Profiled with `perf` + `hyperfine` against Java `avro-tools`
on synthetic inputs scaled from 52 KB to 1 MB.
Results: ~50× faster on typical files (5.6 ms vs 267 ms),
narrowing to ~6× on 1 MB inputs where JVM startup is less dominant.
~97% of time is in the ANTLR parser/runtime; our code accounts for ~3%.
Optimization targets filed as issues.
Cost: ~$2—benchmarking is compute-heavy, not token-heavy.

**Commits**: `37d48cb` (benchmarking guide),
`cef6504` (optimization issues from profiling).
