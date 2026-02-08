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
- Auditing code quality, error handling, and test coverage
- Scanning for `TODO` comments in `src/` that flag deferred work now
  worth addressing

Each agent should pursue its own line of investigation autonomously. If it finds a discrepancy, it does first-level triage (identify root cause, affected files) and files an issue under `issues/`.

**Agent rules:**
- Use `scripts/compare-golden.sh` for output comparisons instead of
  writing ad-hoc comparison scripts (see CLAUDE.md for usage)
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
  (`scripts/compare-golden.sh`, `scripts/compare-adhoc.sh`, etc.)
  is insufficient or could be improved, file an issue describing the
  shortcoming rather than writing ad-hoc workarounds.

**Comparison commands:** See the "Comparing against the Java tool"
section in CLAUDE.md, or use `scripts/compare-golden.sh`.

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
     distinct parts of the same files, apply them directly on main —
     this is significantly faster than the multi-agent approach.

3. **For each wave:**
   a. **Prepare worktrees** (not in sub-agents):
      The parent agent must prepare each worktree before launching the
      sub-agent:
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
      - Uses `scripts/compare-golden.sh` to verify output correctness
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

4. Repeat for each wave until all grouped issues are resolved.

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
6. Append a brief summary of the loop to `agent-logs/LOOPS.md` for
   the user to peruse later.
7. **Return to Phase 1** for the next iteration.

---

## Verification

After each wave merge:
- `cargo test` — all tests pass
- `scripts/compare-golden.sh idl` — all 18 files report results
- `scripts/compare-golden.sh idl2schemata` — per-schema output compared
- For test suite changes: verify new tests pass and cover the intended behavior

See the "Non-goal: byte-identical output" section in CLAUDE.md.

---

## Cross-Cutting Tips from Previous Iterations

- **`compare-golden.sh` has limitations**: It can't find the Java JAR
  from worktrees (issue `2931799a`). Run comparisons from `main/`
  after merging, not from worktrees.
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
