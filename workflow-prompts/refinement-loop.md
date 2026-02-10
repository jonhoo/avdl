# Self-Correcting Development Loop for avdl

## Context

The `avdl` project is a Rust port of Apache Avro's IDL compiler (`idl` and `idl2schemata` subcommands). The goal is an iterative discover-fix-verify loop that converges toward full compatibility with the Java reference implementation.

**Key paths:**
- Main repo: `/home/jon/dev/stream/avdl/main/`
- Worktrees: `/home/jon/dev/stream/avdl/avdl-worktrees/wt-{a..k}`
- Java tool: `/home/jon/dev/stream/avdl/avro-tools-1.12.1.jar`

See CLAUDE.md for test input/output/classpath directories and all
reference file paths.

---

## The General Loop

### Phase 1: Issue Discovery (Open-Ended Exploration)

Launch many sub-agents (blocking, in `main/`) for **open-ended exploration** of issues with the `idl` and `idl2schemata` subcommands. The goal is broad, creative discovery — agents should actively look for discrepancies, edge cases, and missing behaviors by:

- Investigating the Java test suites (see CLAUDE.md for paths) — reading unit tests, running .avdl files, studying what behaviors are tested
- Running `.avdl` files through both the Rust `idl` subcommand and the Java `idl` tool, comparing protocol/schema JSON outputs
- Running `idl2schemata` on representative inputs and comparing per-schema `.avsc` output against Java tool
- Exploring edge cases, error paths, unusual inputs, and uncommon IDL features for both subcommands
- Reading the Java source (`IdlReader.java`, `IdlToSchemataTool.java`) to spot behaviors not yet ported
- Auditing code quality, error handling, and test coverage of both happy and error paths
- Scanning for `TODO` comments in `src/` that flag deferred work now worth addressing
- Thinking of ways in which we could improve the test suite more broadly to cover more of the application's surface area and spec coverage
- Looking for opportunities to make errors and warnings more helpful and actionable to human or agentic users that observe them
- Finding areas of duplication across the code that should be unified (remember that duplication can be warranted if unification is overly challenging)
- Identifying unnecessary/overzealous helper functions
- Making slight mutations to valid golden `.avdl` files (replace `;` with `,`, omit separators, misspell keywords) and verifying the error output is helpful and includes source location

Each agent should pursue its own line of investigation autonomously. If it finds a discrepancy, it does first-level triage (identify root cause, affected files) and files an issue under `issues/`.

**Agent rules:**
- Use `cargo test` for output verification. Use
  `scripts/compare-adhoc.sh` for manual Java-vs-Rust comparison of
  arbitrary `.avdl` files beyond the golden test suite
- Follow the conventions in CLAUDE.md for temp files (`tmp/`),
  issue filing (`issues/`), debug examples, and JSON comparison
- Do first-level triage: symptom, root cause, affected files,
  reproduction, suggested fix
- Avoid modifying `src/` to prevent agents stepping on each others'
  toes — prefer filing issues and using `tmp/` + `examples/`
- Do not update `SESSION.md`, that will be done by the orchestrating
  agent. Instead, anything an agent _would_ have put in `SESSION.md`
  should be reported to the orchestrating agent.
- **Check existing issues before filing** — read the contents of each
  file in `issues/` to avoid filing duplicates. Also check SESSION.md
  for previously investigated items.
- **Recommend tooling improvements**: If a helper script
  (`scripts/compare-adhoc.sh`, etc.) is insufficient or could be
  improved, file an issue describing the shortcoming rather than
  writing ad-hoc workarounds.

**Comparison commands:** `cargo test` is the canonical correctness
check. For ad-hoc Java comparison, see `scripts/compare-adhoc.sh`
in CLAUDE.md.

**After agents complete:**
1. Review new files in `issues/` — deduplicate against existing issues.
   Discovery agents often rediscover the same problems, so budget time
   for deduplication even when agents check existing issues before filing.
2. Review SESSION.md for observations that warrant new issues
3. Clear SESSION.md of anything that's now covered by issues
4. Commit all new issue entries using `commit-writer` skill
5. **If no new issues were filed, STOP the loop entirely**

### Phase 2: Issue Resolution

1. **Analyze** all open issues (in `issues/`) for:
   - Semantic relationships (which are related?)
   - File overlap (which touch the same code?)
   - Dependencies (which must be fixed first because they impact others?)

2. **Group into waves** of non-conflicting fixes that can run in parallel. Issues touching the same files go in the same wave or sequential waves. Foundation issues (those that block validation of others) come first.

   **Wave grouping guidance:**
   - Prioritize functional fixes (correctness bugs, missing features)
     over non-functional improvements (error context, code quality,
     style). Complete all functional fixes before starting quality
     waves.
   - When grouping, note which files each issue touches. If two issues
     in the same wave will modify the same file (e.g., both add tests
     to `json.rs`), the parent agent must resolve merge conflicts when
     merging. To minimize conflicts, order merges so that additive
     changes (new tests, new functions) merge before transformative
     changes (refactors, renames).
   - **Batch small fixes**: Small, self-contained fixes to the same
     file (e.g., `main.rs` one-liners) can often be batched into a
     single wave and applied sequentially by the parent agent without
     sub-agents. If all issues in a wave are 5-15 line changes to
     distinct parts of the same files, the parent can apply them
     directly instead of spawning sub-agents — this is significantly
     faster than the multi-agent approach. **Even so, do this work in
     a worktree, not in `main/`** (see below).

3. **All implementation work happens in worktrees, never in `main/`.**
   Whether a wave is handled by sub-agents or applied directly by the
   parent agent, the code changes must be made in a worktree and merged
   back into `main` afterward. Working directly in `main/` risks
   collisions with parallel agents and makes it hard to revert a wave
   cleanly if something goes wrong. The only operations that belong in
   `main/` are read-only exploration (Phase 1), merges, and
   verification.

4. **For each wave:**
   a. **Prepare worktrees** (not in sub-agents):
      The parent agent must prepare each worktree before launching the
      sub-agent (or before starting work itself for parent-applied batches):
      ```bash
      cd /home/jon/dev/stream/avdl/avdl-worktrees/wt-X
      git stash 2>/dev/null  # save any leftover state; consider committing to main
      git checkout -B fix/issue-UUID-description main
      ```
      Each worktree must have a unique branch name — git worktrees
      cannot share branch names. Use the first 8 characters of the
      issue UUID: `fix/issue-<uuid8>-description`
      (e.g., `fix/issue-39c7d498-float-formatting`).
   b. Launch one **blocking** sub-agent per worktree. Each agent:
      - Reads the issue file for full context
      - Implements the fix
      - Runs `cargo test` to verify output correctness
      - Creates debug example in `examples/` to verify (`cargo run --example`)
      - Runs `cargo test` to check for regressions
      - Cleans up debug examples
      - Stages changes with `git add <specific-files>`
      - Commits using the `commit-writer` skill
      - If modifying library code (e.g., `reader.rs`), use
        `touch src/reader.rs` before `cargo test` if test results
        seem stale — build caches sometimes miss recompilation.
      - When fixing a bug in `main.rs`, check whether test helpers
        (`parse_idl2schemata`, `process_decl_items_test`) have the
        same bug. Past iterations had divergence from updating
        `main.rs` without updating the test helper.

      > **Sub-agent permissions:** Sub-agents launched with
      > `run_in_background` cannot perform interactive operations —
      > `git commit`, writing to `/tmp`, and using skills (like
      > `commit-writer`) are all auto-denied because permission
      > prompts cannot reach the user. Use blocking (non-background)
      > sub-agents, which forward permission prompts to the user.
      > The parent agent must still prepare the worktree (git
      > checkout) before launching either kind.

   c. After each sub-agent completes, the **parent agent**:
      - Checks the worktree's `SESSION.md` for observations the
        sub-agent recorded. Incorporate relevant findings into `main`'s
        `SESSION.md`, then clear the worktree's `SESSION.md`.
      - Merges into `main`:
        If the change rebases cleanly onto `main` then prefer that,
        otherwise merge the change:
        ```bash
        cd /home/jon/dev/stream/avdl/main
        git merge fix/issue-UUID-description
        ```
   d. Verify: `cargo test` in main
   e. If merge conflicts occur (common when multiple agents in a wave
      modify the same file — especially test files like
      `integration.rs`), resolve them before proceeding. Prefer
      merging additive changes first to create a clean base for
      subsequent merges. For files heavily modified by prior merges
      (e.g., `reader.rs` test section), consider `git checkout --ours`
      and manually applying the branch's additions — this is often
      faster and safer than resolving inline conflict markers.

5. Repeat for each wave until all grouped issues are resolved.

### Phase 3: Cleanup and Reflection

After all waves complete:

1. **Update resolved issues**: Remove issue files for fixes that were
   merged. Also close issues that are non-goals (see CLAUDE.md):
   design-choice differences from Java should be documented in the
   "Intentional divergences from Java" section of `README.md` and then
   closed; low-impact domain model gaps with zero effect on JSON output
   can be closed with a TODO comment in the code. Re-verify
   remaining open issues against the current code — discovery-filed
   issues go stale quickly as fixes land.
   For a deeper retrospective audit of closed issues (run periodically
   at the user's request, not every iteration), see
   `workflow-prompts/closed-issue-audit.md`.
2. **Update existing issue files**: Enrich remaining issues with any
   new information learned during the iteration (e.g., partially
   addressed gaps, updated priorities).
3. **Review SESSION.md**: File issues for any observations that
   warrant them. Clean up entries that have been addressed.
4. **Review permission friction**: Look at which bash commands agents
   commonly needed manual approval for during this iteration. Consider
   whether wrapper scripts, allowlist entries, or other tooling could
   reduce the permission prompts for future iterations.
5. **Attempt to improve this prompt**: Review what went well and what
   was friction during this iteration. Update this file with any
   improvements — better instructions, new tips, corrected
   assumptions. Commit the change.
6. Append a brief summary of the loop to a new file
   `agent-logs/loop-NN.md` (zero-padded iteration number) for
   the user to peruse later.
7. **Return to Phase 1** for the next iteration.

---

## Verification

After each wave merge:
- `cargo test` covers both `idl` (18 `.avpr` golden comparisons) and `idl2schemata` (61 `.avsc` golden comparisons)
- For test suite changes: verify new tests pass and cover the intended behavior

See the "Non-goal: byte-identical output" section in CLAUDE.md.

---

## Cross-Cutting Tips from Previous Iterations

- **Reserved property sets differ between Java source and JAR**: The
  git submodule Java source may be a different version than the
  avro-tools JAR. Always validate behavior against the JAR, not
  just the source code.
- **Agents may re-scope when investigation reveals the real bug is
  elsewhere**: Agent B (Wave 1, iteration 2) was assigned
  cross-namespace resolution but found it was already correct; it
  instead fixed a different issue (unresolved refs as warnings instead
  of errors). The parent should recognize and credit this correctly
  when closing issues.
- **Sub-agent issue file deletion**: If a sub-agent deletes issue files
  as part of its commit, the parent doesn't need a separate close
  commit.
- **Merge conflict resolution for multi-branch refactors**: When Wave 4
  (iteration 6) merged both an IdlError-to-miette refactor and an
  expect-audit that touched the same files (`main.rs`, `reader.rs`,
  `import.rs`), merge conflicts arose. The fix: merge the larger
  refactor first (more structural changes), then resolve the smaller
  change against the new code. For `main.rs`, the conflict was between
  two patterns for the same function — take the structural improvement
  (match instead of if/else) and the error handling improvement
  (`.into_diagnostic()` instead of `IdlError::Io`).
- **Discovery agent deduplication is essential**: 5 discovery agents
  filed 7 issues but also generated ~30 SESSION.md observations, many
  overlapping. Budget time after Phase 1 to clean SESSION.md of items
  now tracked as issues, and verify that newly filed issues don't
  overlap with each other or with existing issues.
- **Focused discovery agents outperform broad ones in later
  iterations**: In iteration 7, 3 agents with narrow, deep mandates
  (spec compliance audit, import edge cases, property handling) were
  more effective than 5 broad agents. They produced thorough
  SESSION.md audit trails with minimal overlap and one genuine bug.
  As the codebase matures, prefer fewer (but still >3), more specialized
  agents. But consider the exploration-exploitation trade-off: you
  should still run _some_ broad agents _some_ of the time.
- **Non-issue investigations still have value**: Even when a TODO is
  determined to be a non-issue, documenting the analysis in
  SESSION.md prevents the question from being re-asked. Consider
  briefly adding a code comment explaining the design decision.
- **Error UX audits are productive even for mature codebases**: In
  iteration 15, an error-focused discovery agent found 4 genuine
  diagnostic quality issues (duplicate error text, lost source spans,
  unhelpful messages) despite 14 prior iterations. Error messages are
  a rich source of improvements because they interact with many
  subsystems (ANTLR, miette, import resolution) that evolve
  independently.
