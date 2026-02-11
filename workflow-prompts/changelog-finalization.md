# Changelog Finalization Workflow

Audit and finalize the `[Unreleased]` section of `CHANGELOG.md` before
cutting a new release. This is a content-quality workflow — it
validates existing entries, finds missing ones, tightens wording, and
verifies commit references. It does **not** bump the version, update
compare links, modify `Cargo.toml`, or create tags. Those belong to a
separate release-cutting step.

The workflow runs 8 sequential steps (plus a prerequisite step 0).
Steps 1, 2, 7, and 8 are "fan-out" steps that launch one blocking
sub-agent per entry or section. Steps 3–6 each delegate to a single
blocking sub-agent. All sub-agents are blocking (not background) so the
orchestrator can review their output before proceeding to the next step.

**Why sub-agents everywhere:** Even for single-review steps (3–6), the
work is delegated to a sub-agent so the orchestrator's context stays
clean. The orchestrator's job is to read agent output, apply agreed
changes to `CHANGELOG.md`, and re-read the file before the next step.

**Key paths:**
- Project root: `/home/jon/dev/stream/avdl/main/`
- Changelog: `CHANGELOG.md`
- Changelog guidelines: the "Changelog" section of `CLAUDE.md`

**Key references:**
- [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/)
- The six categories: Added, Changed, Deprecated, Removed, Fixed,
  Security

---

## Step 0: Determine the commit range

The commit range defines "since last release" — the set of commits
whose changes should be reflected in `[Unreleased]`.

1. Read `CHANGELOG.md` and extract the tag from the `[Unreleased]`
   compare link at the bottom. The line looks like:
   ```
   [Unreleased]: https://github.com/jonhoo/avdl/compare/TAG...HEAD
   ```
   Extract `TAG` (everything between `compare/` and `...HEAD`).

2. Check if the tag exists locally:
   ```sh
   git tag -l 'TAG'
   ```

3. **If the tag exists**, the range is `TAG..HEAD`.

4. **If the tag does not exist**, find the boundary commit:
   - Try the most recent "Release" commit:
     ```sh
     git log --oneline --grep="^Release" --format="%H %s" -1
     ```
   - Or fall back to the latest existing tag:
     ```sh
     git describe --tags --abbrev=0
     ```
   The range is then `<boundary>..HEAD`.

5. Verify the range looks correct:
   ```sh
   git log --oneline RANGE
   ```
   Every commit hash already cited in `[Unreleased]` entries should
   appear in this output.

6. Store two values for the rest of the workflow:
   - `RANGE` — the range string (e.g., `v0.1.4+1.12.1..HEAD`)
   - `LOG` — the full `git log --oneline RANGE` output

---

## Step 1: Validate existing entries

For every entry in the `[Unreleased]` section that has content (across
all 6 categories), launch one blocking sub-agent to investigate whether
the entry accurately summarizes the change it references.

**Skip categories with no entries** — step 2 will check whether they
should have entries.

**Batching:** If there are more than ~10 entries, batch 2–3 per agent
to keep agent count manageable. Each agent should still evaluate each
entry independently and produce a separate verdict per entry.

### Agent prompt template

> You are validating a single changelog entry for the `avdl` project
> at `/home/jon/dev/stream/avdl/main/`.
>
> **Entry under review (from the `SECTION` section):**
> ```
> ENTRY_TEXT
> ```
>
> **Commit hash referenced:** `HASH` (or "none" if no hash present)
>
> **Commit range for this release:** `RANGE`
> **Full commit log for this range:**
> ```
> LOG
> ```
>
> Read the "Changelog" section of `CLAUDE.md` for the project's
> changelog conventions.
>
> Your task:
> 1. If a hash is present, run `git show HASH` to read the full
>    commit message and diff.
> 2. Determine whether the entry text accurately summarizes the
>    user-visible effect of that commit.
> 3. Check whether the entry is in the correct section (Added,
>    Changed, Fixed, etc.) per the CLAUDE.md guidelines: new
>    capabilities (including new validations and warnings) go in
>    **Added**, changes to existing behavior in **Changed**, and
>    actual bug corrections in **Fixed**.
> 4. Check whether the entry uses imperative mood (e.g., "Validate
>    ...", not "Validates ...").
> 5. Check whether the entry describes the user-visible effect, not
>    the implementation technique.
> 6. If the commit is a multi-issue fix (e.g., touches several
>    unrelated things), verify that the changelog entry accurately
>    represents the part of the commit it claims to describe.
>
> **Output format:**
> ```
> ### Entry: "FIRST_LINE_OF_ENTRY..."
> - **Section:** SECTION
> - **Verdict:** ACCURATE | NEEDS-REWORDING | WRONG-SECTION | INACCURATE
> - **Issues found:** (list, or "none")
> - **Suggested replacement:** (full entry text if verdict is not ACCURATE)
> - **Suggested section:** (if WRONG-SECTION)
> - **Reasoning:** (brief explanation)
> ```

### After agents complete

Review all agent outputs. For each entry with a verdict other than
ACCURATE, evaluate the suggestion and apply it to `CHANGELOG.md` if
you agree. Use judgment — the agent may be wrong about section
placement or wording. Re-read `CHANGELOG.md` before proceeding.

---

## Step 2: Find missing entries

For each of the 6 changelog categories, launch one blocking sub-agent
to investigate whether there are commits in the range that represent
user-visible changes not already covered by existing entries.

Launch all 6 agents even for categories that already have entries —
existing entries may not cover all relevant commits. All 6 can be
launched in parallel.

### Agent prompt template

> You are looking for missing changelog entries in the **SECTION**
> category for the `avdl` project at
> `/home/jon/dev/stream/avdl/main/`.
>
> **Category definition (per Keep a Changelog):**
> - **Added**: new features
> - **Changed**: changes in existing functionality
> - **Deprecated**: soon-to-be removed features
> - **Removed**: now removed features
> - **Fixed**: bug fixes
> - **Security**: vulnerability fixes
>
> **Current entries in this category:**
> ```
> EXISTING_ENTRIES (or "none")
> ```
>
> **Commit range:** `RANGE`
>
> Read the "Changelog" section of `CLAUDE.md` for the project's
> changelog conventions. Key rules:
> - Only user-facing changes belong. Internal changes (test
>   infrastructure, CI, issue tracking, code formatting, dev-only
>   scripts) do not.
> - The litmus test: "would a user deciding whether to upgrade care?"
> - Entries need not map 1:1 to commits; several commits may unify
>   under one entry.
> - Each entry references a primary commit hash at the end.
>
> Your task:
> 1. Run `git log --oneline RANGE` to see all commits.
> 2. For each commit, run `git show --stat HASH` to see what files
>    changed. Skip commits that only touch `issues/`,
>    `upstream-issues/`, `agent-logs/`, `workflow-prompts/`,
>    `SESSION.md`, `.claude/`, or test-only files — unless the test
>    change reflects a user-visible behavior change.
> 3. For commits that touch `src/`, `scripts/`, or `Cargo.toml`, run
>    `git show HASH` to read the full diff and commit message.
> 4. Determine whether any of these commits represent user-visible
>    changes in the **SECTION** category that are not already covered
>    by the existing entries.
> 5. For each missing change, draft a changelog entry in imperative
>    mood with the primary commit hash.
>
> **Output format:**
> ```
> ### Category: SECTION
> - **Missing entries found:** N
> - **Entry 1:** "Draft entry text (HASH)"
>   - **Commits:** HASH1, HASH2, ...
>   - **Reasoning:** why this belongs in SECTION
> - **Entry 2:** ...
> (or "No missing entries found.")
> ```
>
> Remember: refactors, test additions, CI changes, documentation
> updates, and issue-tracking housekeeping are NOT user-facing and
> should NOT be suggested as changelog entries.

### After agents complete

Review all suggestions. Add entries you agree with to `CHANGELOG.md`.
Reject entries that are not genuinely user-facing. If an agent suggests
an entry that overlaps with an existing one, consider expanding the
existing entry rather than adding a new one. Re-read `CHANGELOG.md`
before proceeding.

---

## Step 3: Deduplicate and re-sort

Delegate to one blocking sub-agent for context isolation.

### Agent prompt template

> You are reviewing the `[Unreleased]` section of `CHANGELOG.md` for
> the `avdl` project at `/home/jon/dev/stream/avdl/main/` to
> deduplicate and re-sort entries.
>
> Read `CHANGELOG.md` and examine the `[Unreleased]` section. Check
> for:
>
> 1. **Duplicate entries**: Two entries that describe the same change,
>    possibly worded differently or referencing different commits from
>    the same logical change. Recommend merging them into one entry,
>    keeping the best wording and the most representative commit hash.
>
> 2. **Misplaced entries**: An entry in Fixed that is really a new
>    capability (belongs in Added), or an entry in Added that is
>    really a change to existing behavior (belongs in Changed).
>    Recommend moving it. See the "Changelog" section of `CLAUDE.md`
>    for the project's categorization rules.
>
> 3. **Sort order within categories**: Entries within a category
>    should be ordered by significance (most important first), not
>    chronologically. If no clear significance ordering exists, group
>    related entries together.
>
> 4. **Entry granularity**: Multiple small entries that describe
>    facets of the same user-visible change should be unified into one
>    entry. Conversely, one entry that describes multiple unrelated
>    changes should be split.
>
> **Output format:**
> For each recommendation, state:
> - What to change (merge, move, reorder, split)
> - The current entry text(s)
> - The suggested result
> - Reasoning
>
> If no changes are needed, say so explicitly.

### After the agent completes

Apply agreed changes to `CHANGELOG.md`. Re-read before proceeding.

---

## Step 4: Backward references

Delegate to one blocking sub-agent.

### Agent prompt template

> You are reviewing the `[Unreleased]` section of `CHANGELOG.md` for
> the `avdl` project at `/home/jon/dev/stream/avdl/main/` to see if
> any entries would benefit from referencing earlier changelog entries
> or versions.
>
> Read the full `CHANGELOG.md`. For each entry in `[Unreleased]`,
> check whether it refers to functionality that was introduced or
> changed in a prior release. If so, consider whether adding a brief
> backward reference (e.g., "(introduced in 0.1.3)" or "...matching
> the format already used since 0.1.1") would help the reader
> understand the context.
>
> **Guidelines:**
> - Only add backward references that genuinely help the reader. Most
>   entries will not need them.
> - Good candidates: fixes for regressions, improvements to recently
>   added features, changes that reverse or modify a prior decision.
> - Bad candidates: generic improvements, new features unrelated to
>   prior work, fixes for bugs that predate the changelog.
> - Keep the reference concise — a parenthetical version number or a
>   brief clause, not a full sentence.
>
> **Output format:**
> For each suggested reference:
> - The current entry text
> - The suggested rewrite
> - Which prior entry/version it references
> - Why the reference helps
>
> If no backward references are warranted, say so explicitly.

### After the agent completes

Apply agreed changes to `CHANGELOG.md`. Re-read before proceeding.

---

## Step 5: Earn their place

Delegate to one blocking sub-agent.

### Agent prompt template

> You are reviewing the `[Unreleased]` section of `CHANGELOG.md` for
> the `avdl` project at `/home/jon/dev/stream/avdl/main/` to check
> that every entry earns its place.
>
> Read the "Changelog" section of `CLAUDE.md` for the project's
> guidelines. The key principle:
>
> > Every entry must earn its place: the changelog is for users of
> > the library and binary, not for contributors. Internal changes
> > (test infrastructure, CI, issue tracking, code formatting,
> > dev-only scripts) do not belong. When in doubt, ask: "would a
> > user deciding whether to upgrade care about this?"
>
> Read the `[Unreleased]` section of `CHANGELOG.md`. For each entry,
> apply the litmus test. Flag any entry that:
> - Describes a refactor with no user-visible effect
> - Describes test infrastructure changes
> - Describes CI/CD changes
> - Describes documentation updates to developer-facing files
> - Describes issue tracking or project management housekeeping
> - Describes code formatting, clippy fixes, or lint cleanup
> - Is too granular (a minor wording tweak to an error message that
>   users are unlikely to encounter)
>
> Also flag entries that mix user-visible and internal changes.
> Recommend rewording to focus on the user-visible part.
>
> **Output format:**
> For each flagged entry:
> - The entry text
> - **Verdict:** REMOVE | REWORD
> - Suggested rewrite (if REWORD)
> - Reasoning
>
> If all entries earn their place, say so explicitly.

### After the agent completes

Remove entries the agent correctly flagged. Reword mixed entries.
Re-read `CHANGELOG.md` before proceeding.

---

## Step 6: Consistency review

Delegate to one blocking sub-agent.

### Agent prompt template

> You are reviewing the `[Unreleased]` section of `CHANGELOG.md` for
> the `avdl` project at `/home/jon/dev/stream/avdl/main/` for
> consistency with the rest of the changelog and with the Keep a
> Changelog standard.
>
> Read the full `CHANGELOG.md`. Also read the "Changelog" section of
> `CLAUDE.md` for the project's conventions. Fetch and review the
> guidelines at <https://keepachangelog.com/en/1.1.0/>.
>
> Compare the `[Unreleased]` section against the released versions
> for:
> 1. **Capitalization and punctuation** — do entries start with a
>    capital letter? End without a period? Consistent with prior
>    releases?
> 2. **Level of detail** — are entries roughly the same length and
>    depth as prior releases, or is this release significantly more
>    or less detailed?
> 3. **Imperative mood** — "Validate ...", not "Validates ..." or
>    "Validated ..."?
> 4. **Commit hash format** — `(abcdef0)` at end of each line,
>    7–8 hex characters, in parentheses?
> 5. **Entry length** — not wildly longer or shorter than entries in
>    prior releases?
> 6. **Section headings** — all 6 Keep a Changelog categories present
>    under `[Unreleased]`?
>
> Also note anything that looks "out of order" (e.g., a major feature
> buried below minor fixes) or stylistically inconsistent.
>
> **Output format:**
> A list of observations, each with:
> - What was observed
> - Which entry or entries it affects
> - Suggested fix (if any)
>
> If the section is consistent, say so explicitly.

### After the agent completes

Apply any style fixes. Re-read `CHANGELOG.md` before proceeding.

---

## Step 7: Find commit hashes

For every entry in `[Unreleased]` that does not end with a commit hash
in parentheses `(abcdef0)`, launch one blocking sub-agent to find the
primary commit.

**If all entries already have hashes, skip this step entirely.**

### Agent prompt template

> You are finding the primary commit hash for a changelog entry in
> the `avdl` project at `/home/jon/dev/stream/avdl/main/`.
>
> **Entry (from the `SECTION` category):**
> ```
> ENTRY_TEXT
> ```
>
> **Commit range:** `RANGE`
> **Full commit log for this range:**
> ```
> LOG
> ```
>
> Your task:
> 1. Search the commit log for the commit that best represents this
>    changelog entry.
> 2. Use `git log --all --grep="KEYWORD"` for keyword searches, and
>    `git show HASH` to inspect candidates.
> 3. The "primary commit" is the one commit that most fully represents
>    the change described. If the change spans multiple commits, pick
>    the one with the most substantive code change (not the cleanup
>    or follow-up).
>
> **Output format:**
> ```
> ### Entry: "FIRST_LINE_OF_ENTRY..."
> - **Primary commit:** HASH
> - **Commit subject:** (the commit's subject line)
> - **Confidence:** HIGH | MEDIUM | LOW
> - **Reasoning:** why this commit was chosen
> - **Alternative candidates:** HASH2 (subject), HASH3 (subject)
> ```

### After agents complete

For HIGH and MEDIUM confidence matches, append the hash to the entry
in `(hash)` format. For LOW confidence, investigate manually before
adding. Re-read `CHANGELOG.md` before proceeding.

---

## Step 8: Validate commit hashes

For every entry in `[Unreleased]` that ends with a commit hash in
parentheses, launch one blocking sub-agent to verify the hash is
appropriate for the entry.

**Batching:** Same as step 1 — if there are more than ~10 entries,
batch 2–3 per agent.

### Agent prompt template

> You are validating that a commit hash is appropriate for its
> changelog entry in the `avdl` project at
> `/home/jon/dev/stream/avdl/main/`.
>
> **Entry (from the `SECTION` category):**
> ```
> ENTRY_TEXT
> ```
>
> **Commit hash:** `HASH`
>
> **Commit range for this release:** `RANGE`
> **Full commit log for this range:**
> ```
> LOG
> ```
>
> Your task:
> 1. Run `git show HASH` to read the full commit message and diff.
> 2. Verify the hash resolves to a real commit (not truncated or
>    mistyped).
> 3. Verify the commit falls within the release range (not from a
>    prior release). Cross-check against the log above.
> 4. Verify the commit's changes match what the entry describes. The
>    commit does not need to be the *only* commit for the change, but
>    it should be the *primary* one — the most substantive code
>    change, not a follow-up or cleanup.
> 5. If the hash is wrong, search the range for a better candidate
>    and suggest it.
>
> **Output format:**
> ```
> ### Entry: "FIRST_LINE_OF_ENTRY..."
> - **Hash:** HASH
> - **Verdict:** VALID | WRONG-COMMIT | OUT-OF-RANGE | INVALID-HASH
> - **Issues found:** (list, or "none")
> - **Suggested replacement:** HASH (if verdict is not VALID)
> - **Reasoning:** (brief explanation)
> ```

### After agents complete

For any entry with a verdict other than VALID:
- **WRONG-COMMIT**: Evaluate the suggested replacement. If it fits
  better, swap the hash.
- **OUT-OF-RANGE**: The entry may belong in a prior release's section,
  or the hash may be from an ancestor commit that was cherry-picked.
  Investigate.
- **INVALID-HASH**: The hash is truncated or mistyped. Find the
  correct hash.

Re-read `CHANGELOG.md` to confirm the final state.

---

## Final verification

After all 8 steps, run this checklist:

1. **Every entry has a commit hash:**
   Visually scan each bullet under `[Unreleased]`. Each should end
   with `(HASH)` where HASH is 7–8 hex characters.

2. **Every commit hash is valid:**
   For each hash, verify it resolves:
   ```sh
   git rev-parse --short HASH
   ```

3. **Every commit hash is in the release range:**
   ```sh
   git log --oneline RANGE | grep HASH
   ```

4. **All 6 category headings are present** under `[Unreleased]`,
   even if empty (per project convention).

5. **Imperative mood spot-check:** Re-read each entry. Does it
   complete the sentence "This release will ..." naturally?

6. **User-facing spot-check:** For each entry, answer: "Would a user
   deciding whether to upgrade care about this?" If no, remove it.

7. **Compare link is correct:** The `[Unreleased]` link at the bottom
   of the file should point to `compare/TAG...HEAD` where TAG is the
   tag for the most recent released version.

8. **Read the final section as a coherent release note.** Does it
   tell a clear story of what changed? Are the most important changes
   prominent?

---

## Lessons

(This section accrues tips from running the workflow. Initially empty.)
