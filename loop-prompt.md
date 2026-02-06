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
  agent

**Comparison commands:** See the "Comparing against the Java tool"
section in CLAUDE.md, or use `scripts/compare-golden.sh`.

**After agents complete:**
1. Review new files in `issues/` — deduplicate against existing issues
2. Commit all new issue entries using `commit-writer` skill
3. **If no new issues were filed → STOP the loop entirely**

### Phase 2: Issue Resolution

1. **Analyze** all open issues for:
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

      **Sub-agent permission limitations:** Sub-agents launched with
      `run_in_background` cannot perform interactive operations — `git
      commit`, writing to `/tmp`, and using skills (like
      `commit-writer`) are all auto-denied because permission prompts
      cannot reach the user. Blocking (non-background) sub-agents CAN
      do these things since permission prompts are forwarded to the
      user. However, the parent agent must still prepare the worktree
      (git checkout) before launching either kind of sub-agent.
   c. After each sub-agent completes, the **parent agent** reviews the
      staged changes, commits in the worktree, then merges into `main`:
      ```bash
      cd /home/jon/dev/stream/avdl/avdl-worktrees/wt-X
      git diff --cached --stat  # review
      git commit -m "..."       # parent commits
      cd /home/jon/dev/stream/avdl/main
      git merge fix/issue-UUID-description
      ```
   d. Verify: `cargo test` in main
   e. If merge conflicts occur (common when multiple agents in a wave
      modify the same file), resolve them before proceeding. Prefer
      merging additive changes first to create a clean base for
      subsequent merges.

4. Repeat for each wave until all grouped issues are resolved.

### Phase 3: Return to Phase 1

---

## Verification

After each wave merge:
- `cargo test` — all tests pass
- `scripts/compare-golden.sh idl` — all 18 files report results
- `scripts/compare-golden.sh idl2schemata` — per-schema output compared
- For test suite changes: verify new tests pass and cover the intended behavior
