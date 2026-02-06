# Self-Correcting Development Loop for avdl

## Context

The `avdl` project is a Rust port of Apache Avro's IDL compiler (`idl` and `idl2schemata` subcommands). The goal is an iterative discover-fix-verify loop that converges toward full compatibility with the Java reference implementation.

**Key paths:**
- Main repo: `/home/jon/dev/stream/avdl/main/`
- Worktrees: `/home/jon/dev/stream/avdl/avdl-worktrees/wt-{a..k}`
- Java tool: `/home/jon/dev/stream/avdl/avro-tools-1.12.1.jar`
- Test inputs: `avro/lang/java/idl/src/test/idl/input/` (18 .avdl files)
- Expected outputs: `avro/lang/java/idl/src/test/idl/output/` (18 .avpr/.avsc)
- Classpath imports: `avro/lang/java/idl/src/test/idl/putOnClassPath/`
- Extra tests: `avro/lang/java/idl/src/test/idl/extra/`
- Java test classes: `avro/lang/java/idl/src/test/java/org/apache/avro/idl/`
- Java tools tests: `avro/lang/java/tools/src/test/idl/` (protocol.avdl/.avpr, schema.avdl/.avsc — additional golden-file pairs for the `idl` and `idl2schemata` CLI entry points)
- Java tools test classes: `avro/lang/java/tools/src/test/java/org/apache/avro/tool/` (e.g., `TestIdlTool.java`, `TestIdlToSchemataTool.java`)

---

## The General Loop

### Phase 1: Issue Discovery (Open-Ended Exploration)

Launch many sub-agents (blocking, in `main/`) for **open-ended exploration** of issues with the `idl` and `idl2schemata` subcommands. The goal is broad, creative discovery — agents should actively look for discrepancies, edge cases, and missing behaviors by:

- Investigating the Java test suites in `avro/lang/java/idl/src/test/` and `avro/lang/java/tools/src/test/idl/` — reading unit tests, running .avdl files, studying what behaviors are tested
- Running `.avdl` files through both the Rust `idl` subcommand and the Java `idl` tool, comparing protocol/schema JSON outputs
- Running `idl2schemata` on representative inputs and comparing per-schema `.avsc` output against Java tool
- Exploring edge cases, error paths, unusual inputs, and uncommon IDL features for both subcommands
- Reading the Java source (`IdlReader.java`, `IdlToSchemataTool.java`) to spot behaviors not yet ported
- Auditing code quality, error handling, and test coverage

Each agent should pursue its own line of investigation autonomously. If it finds a discrepancy, it does first-level triage (identify root cause, affected files) and files an issue under `issues/`.

**Agent rules:**
- Use `mktemp -d tmp/XXXXXX` for temp files (NOT `/tmp`)
- Avoid modifying `src/` to prevent agents stepping on each others' toes — prefer filing issues and using `tmp/` + `examples/`
- Use `jq -S .` for JSON comparison
- File new issues under `issues/` with filename `$(uuidgen)-short-description.md`
- Do first-level triage: symptom, root cause, affected files, reproduction, suggested fix
- Check existing `issues/` first to avoid duplicates
- Use Rust example files in `examples/` for debugging
- Do not update `SESSION.md`, that will be done by the orchestrating agent

**Comparison commands:**
```bash
INPUT_DIR=avro/lang/java/idl/src/test/idl/input
OUTPUT_DIR=avro/lang/java/idl/src/test/idl/output
CLASSPATH_DIR=avro/lang/java/idl/src/test/idl/putOnClassPath

# Rust tool:
cargo run -- idl [--import-dir $INPUT_DIR] [--import-dir $CLASSPATH_DIR] $INPUT_DIR/foo.avdl tmp/foo.avpr
# Java tool:
java -jar ../avro-tools-1.12.1.jar idl $INPUT_DIR/foo.avdl tmp/foo-java.avpr
# Compare:
diff <(jq -S . tmp/foo.avpr) <(jq -S . $OUTPUT_DIR/foo.avpr)
```

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

3. **For each wave:**
   a. **Prepare worktrees** (not in sub-agents):
      The parent agent must prepare each worktree before launching the
      sub-agent:
      ```bash
      cd /home/jon/dev/stream/avdl/avdl-worktrees/wt-X
      git stash 2>/dev/null  # save any leftover state; consider committing to main
      git checkout -B fix/issue-description main
      ```
      Each worktree must have a unique branch name (git worktrees cannot
      share branch names).
   b. Launch one **blocking** sub-agent per worktree. Each agent:
      - Reads the issue file for full context
      - Implements the fix
      - Creates debug example in `examples/` to verify (`cargo run --example`)
      - Runs `cargo test` to check for regressions
      - Cleans up debug examples
      - Stages changes with `git add <specific-files>`
      **Note:** Sub-agents cannot `git commit`, write to `/tmp`, or use
      the `commit-writer` skill because these operations require
      interactive permission approval which is auto-denied for
      background/sub-agents. The parent agent must commit after the
      sub-agent finishes.
   c. After each sub-agent completes, the **parent agent** reviews the
      staged changes, commits in the worktree, then merges into `main`:
      ```bash
      cd /home/jon/dev/stream/avdl/avdl-worktrees/wt-X
      git diff --cached --stat  # review
      git commit -m "..."       # parent commits
      cd /home/jon/dev/stream/avdl/main
      git merge fix/issue-description
      ```
   d. Verify: `cargo test` in main
   e. If merge conflicts occur, resolve before proceeding

4. Repeat for each wave until all grouped issues are resolved.

### Phase 3: Return to Phase 1

---

## Verification

After each wave merge:
- `cargo test` — all tests pass
- `cargo run -- idl <input> tmp/out && diff <(jq -S . tmp/out) <(jq -S . <expected>)`
- For `idl2schemata`: compare per-schema output against `java -jar avro-tools idl2schemata`
- For test suite changes: verify new tests pass and cover the intended behavior
