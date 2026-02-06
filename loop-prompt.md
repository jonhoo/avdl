We're aiming to set up a self-correcting loop of development. Do the
following phases in order. Then, when the last phase is completed,
return to the first phase again. Only exit this loop when the first
phase finds no new issues.

Note that _all_ agents must be run in blocking mode so that they can ask
for permissions. Also, agents should use `tmp/`, not `/tmp` as a
temporary folder.

Agents should debug issues using Rust example files in examples/ that
are run with `cargo run --example`.

For the first iteration of phase 1, use the issues in issues/, plus
theses two:

- how can we improve the Rust test suite for the crate to have a more robust test suite natively?
- the one outlined in TODOs.md

For subsequent issues, we should use sub-agents to do more open-ended
exploration/discovery of new issues.

Phase 1:

Start many sub-agents in `main/`, each of which should attempt to
identify issues with the `idl` and `idl2schemata` subcommands. They can
do so by investigating the tests that exist for the Java implementation
in `avro/lang/java/idl/src/test/`, including running on known .avdl
files, the Java unit tests, etc., and compare the output to the expected
output when running the java tool. if discrepancies are found, the agent
shoudl do first-level triage of the observed bug and then file an issue
under issues/ (filename: `uuidgen` + short description). the agents
should _not_ attempt to fix the issues. they should avoid changing the
source files in src/ as much as possible to avoid stepping on each
others' toes. If agents need to compare json outputs, use `jq -S .` to
get sorted, formatted JSON that can then be compared for equality.

Commit all the new entries in issues/ using the commit-writer skill.

If there are no new entries, **do not** go to phase 2.

Phase 2:

Analyze issues/ and check which are semantically related and which
likely require changing the same files. Also, triage them to understand
which likely need to be fixed _first_, because they will impact many of
the others. Based on this analysis, come up with the order and grouping
of issues to have sub-agents address.

For each such grouping, run a sub-agent to debug and fix that issue in
one of the pre-existing worktrees in ./avdl-worktrees/ (check out a new
branch that branches from `main`). each agent should commit its work at
the end of fixing the issue using the commit-writer skill. you should
then merge all the fixes back into `main` after each wave of agents has
finished.

Phase 3:

Go to phase 1.
