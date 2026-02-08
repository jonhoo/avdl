# Workflow: Update post-stream-summary.md

## Purpose

Update `post-stream-summary.md` to cover development work since its
last revision.
The audience is viewers who watched the livestream and want to catch
up on what happened next.
They care about **agentic coding techniques and non-obvious decisions**,
not an exhaustive changelog.

## Step 1: Determine the update window

Find the most recent commit that touched `post-stream-summary.md`:

```sh
git log -1 --format="%H|%aI" -- post-stream-summary.md
```

This commit's timestamp is the **start** of the update window.
The **end** is now.

## Step 2: Gather data

Three sources, all scoped to the update window.

### Git log

```sh
git log --since="WINDOW_START" --format="%H|%aI|%s" --reverse
```

This is the primary source.
Group commits into thematic arcs
(features, refactors, techniques, investigations).

### User prompts (JSONL)

Session files live in:

```
~/.claude/projects/-home-jon-dev-stream-avdl/*.jsonl
~/.claude/projects/-home-jon-dev-stream-avdl-main/*.jsonl
```

Extract prompts after the window start:

```sh
jq -r 'select(.type == "user")
  | select(.message.content | type == "string")
  | select(.message.content | length >= 5)
  | select(.timestamp > "WINDOW_START")
  | select(.message.content | startswith("<task-notification>") | not)
  | select(.message.content | startswith("[Request interrupted") | not)
  | select(.message.content | startswith("This session is being continued") | not)
  | "\(.timestamp) \(.message.content[:200])"' FILE.jsonl
```

Use parallel agents (one per batch of ~5–8 files) to avoid
eating the main context window.
These prompts reveal *why* things were done—steering decisions,
design questions, technique choices—which the git log alone won't show.

### Per-session costs (ccusage)

```sh
ccusage session --json > tmp/ccusage-sessions.json
```

Key fields:

- `sessionId`: project directory name for direct sessions,
  or `"subagents"` for sub-agent sessions.
- `projectPath`: contains `project-dir/session-uuid`—this
  is how to get per-conversation costs.
- `totalCost`: dollar cost.

To get the full cost of a conversation,
sum the direct session's cost with all sub-agent sessions whose
`projectPath` shares the same UUID.

Filter to avdl sessions and split direct vs sub-agent:

```sh
jq '[.[] | select(.projectPath | contains("avdl"))
  | select(.sessionId == "subagents" | not)
  | .totalCost] | add' tmp/ccusage-sessions.json

jq '[.[] | select(.projectPath | contains("avdl"))
  | select(.sessionId == "subagents")
  | .totalCost] | add' tmp/ccusage-sessions.json
```

Correlate session timestamps with the git log to attribute
approximate costs to activities.
Precision isn't important—the goal is "the loop consumed ~$X"
rather than exact per-commit accounting.

Note: the Claude Code sandbox escapes `!=` in jq.
Use `select(.field == "value" | not)` instead of
`select(.field != "value")`.

## Step 3: Group and write

1. **Read the existing summary** to understand its current structure
   and voice.
2. **Group new commits** into thematic arcs.
   Organize by technique or insight, not chronologically.
   Roll up mundane fixes; highlight non-obvious approaches.
3. **Draft new sections or extend existing ones.**
   Each section gets a takeaway heading
   (e.g., "A self-correcting loop beats a linear plan").
   Include 2–3 representative commits per section.
   Fold cost figures inline where the activity is described.
4. **Update the intro** with revised totals
   (token count, cost, time span).

### Style

Follow the `jon-style` skill:
semantic line breaks,
takeaway headings,
active voice,
no redundancy between sections.
Target conciseness—the current version is ~1150 words
and shouldn't grow much unless there's genuinely new material.

## Step 4: Verify

1. `wc -w post-stream-summary.md` — keep it concise.
2. Verify commit hashes against `git log`.
3. Check for redundancy across sections.
4. Confirm `README.md` still links to the file.
