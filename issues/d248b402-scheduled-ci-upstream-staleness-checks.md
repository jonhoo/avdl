# Add scheduled CI jobs to detect upstream staleness

## Symptom

The `scheduled.yml` workflow catches Rust toolchain and dependency
breakage (nightly build, `cargo update`), but there is no automation
to detect when the two non-Cargo upstream dependencies — the `avro`
submodule and the `antlr4rust` ANTLR tool JAR — have new releases.
Falling behind silently means we miss specification changes, new test
cases, parser fixes, and grammar updates until someone notices
manually.

## Root cause

Neither upstream publishes a Cargo crate we depend on, so `cargo
update` and Dependabot don't cover them. The avro submodule is pinned
to a specific commit (currently `c499eefb4`, roughly
`apache-avro-derive@0.17.1-489-gc499eefb4`), and the ANTLR JAR is
pinned to `v0.5.0-beta` in `scripts/regenerate-antlr.sh`.

## Affected files

- `.github/workflows/scheduled.yml`
- `scripts/regenerate-antlr.sh` (JAR_URL variable)
- `avro` submodule pointer

## Reproduction

Not a bug — this is a missing feature. To see the gap, compare the
latest tag on `apache/avro` with the submodule commit, or check
`AmatanHead/antlr4` releases against `v0.5.0-beta`.

## Suggested fix

Add two new jobs to `scheduled.yml` (or a new workflow), both running
on `schedule` alongside the existing nightly cron.

### Job 1: Avro submodule tag check

```yaml
avro-staleness:
  runs-on: ubuntu-latest
  name: Check for new apache/avro releases
  steps:
    - uses: actions/checkout@v4
      with:
        submodules: true

    # Fetch all tags from the upstream avro repo so we can compare.
    - name: Fetch upstream tags
      run: |
        cd avro
        git fetch --tags https://github.com/apache/avro.git

    # Compare the pinned submodule commit against the latest
    # release-* tag. If we're behind, collect the list of missed
    # releases with links to their GitHub release pages.
    - name: Check for newer releases
      id: check
      run: |
        cd avro
        CURRENT=$(git rev-parse HEAD)
        # Tags follow the pattern release-1.12.0, release-1.13.0, etc.
        LATEST_TAG=$(git tag --list 'release-*' --sort=-v:refname \
                     | head -1)
        LATEST_COMMIT=$(git rev-parse "$LATEST_TAG")
        if [ "$CURRENT" = "$LATEST_COMMIT" ]; then
          echo "up_to_date=true" >> "$GITHUB_OUTPUT"
        else
          # List every release tag that is NOT an ancestor of HEAD
          # (i.e., tags we haven't incorporated yet).
          MISSED=""
          for tag in $(git tag --list 'release-*' \
                       --sort=-v:refname); do
            if ! git merge-base --is-ancestor "$tag" HEAD 2>/dev/null
            then
              VER="${tag#release-}"
              URL="https://github.com/apache/avro/releases/tag/$tag"
              MISSED="${MISSED}- [\`$tag\`]($URL)\n"
            fi
          done
          echo "up_to_date=false" >> "$GITHUB_OUTPUT"
          # Write multi-line body to a file for the issue step.
          {
            echo "The \`avro\` submodule is behind upstream."
            echo "Current commit: \`$CURRENT\`"
            echo ""
            echo "Missed releases:"
            echo -e "$MISSED"
          } > /tmp/issue-body.md
        fi

    # File (or update) a GitHub issue when we're behind.
    - name: Create or update issue
      if: steps.check.outputs.up_to_date == 'false'
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        TITLE="avro submodule is behind upstream"
        EXISTING=$(gh issue list --label staleness \
                   --search "$TITLE" --state open --json number \
                   --jq '.[0].number // empty')
        BODY=$(cat /tmp/issue-body.md)
        if [ -n "$EXISTING" ]; then
          gh issue edit "$EXISTING" --body "$BODY"
        else
          gh issue create --title "$TITLE" \
            --label staleness --body "$BODY"
        fi
```

Key details:
- Uses a `staleness` label to find/update existing issues rather than
  creating duplicates.
- Lists every missed release with links, not just "you're behind".
- No-ops cleanly when the submodule is already at or ahead of the
  latest tag.

### Job 2: ANTLR regeneration drift check

```yaml
antlr-staleness:
  runs-on: ubuntu-latest
  name: Check for new antlr4rust releases
  steps:
    - uses: actions/checkout@v4
      with:
        submodules: true

    - name: Install Java
      uses: actions/setup-java@v4
      with:
        distribution: temurin
        java-version: 21

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    # Extract the currently pinned version from the script so we
    # don't have to hard-code it in the workflow too.
    - name: Determine pinned version
      id: pinned
      run: |
        PINNED=$(grep -oP '(?<=download/)[^/]+' \
                 scripts/regenerate-antlr.sh)
        echo "version=$PINNED" >> "$GITHUB_OUTPUT"

    # Query the GitHub API for the latest release on the fork.
    - name: Check for newer release
      id: check
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        LATEST=$(gh api repos/AmatanHead/antlr4/releases/latest \
                 --jq '.tag_name')
        PINNED="${{ steps.pinned.outputs.version }}"
        if [ "$LATEST" = "$PINNED" ]; then
          echo "up_to_date=true" >> "$GITHUB_OUTPUT"
        else
          echo "up_to_date=false" >> "$GITHUB_OUTPUT"
          echo "latest=$LATEST" >> "$GITHUB_OUTPUT"
        fi

    # If a newer release exists, download it, regenerate, and check
    # whether the generated files actually changed.
    - name: Regenerate with new JAR
      if: steps.check.outputs.up_to_date == 'false'
      run: |
        LATEST="${{ steps.check.outputs.latest }}"
        # Replace the JAR URL in the script with the new release.
        # The exact asset filename may differ per release, so we
        # fetch the asset list and pick the *-complete.jar.
        ASSET_URL=$(gh api \
          "repos/AmatanHead/antlr4/releases/tags/$LATEST" \
          --jq '.assets[]
                | select(.name | endswith("-complete.jar"))
                | .browser_download_url')
        # Download and regenerate.
        mkdir -p tmp
        curl -fSL -o tmp/antlr4-tool.jar "$ASSET_URL"
        java -jar tmp/antlr4-tool.jar -Dlanguage=Rust \
          -o src/generated -Xexact-output-dir \
          avro/share/idl_grammar/org/apache/avro/idl/Idl.g4
        # Check for meaningful changes (ignore the path comment).
        if git diff --quiet src/generated/; then
          echo "DRIFT=false" >> "$GITHUB_ENV"
        else
          echo "DRIFT=true" >> "$GITHUB_ENV"
          git diff src/generated/ > tmp/antlr-diff.txt
        fi
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

    # File (or update) a GitHub issue.
    - name: Create or update issue
      if: steps.check.outputs.up_to_date == 'false'
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        LATEST="${{ steps.check.outputs.latest }}"
        PINNED="${{ steps.pinned.outputs.version }}"
        URL="https://github.com/AmatanHead/antlr4/releases/tag/$LATEST"
        TITLE="antlr4rust has a newer release ($LATEST)"
        {
          echo "Pinned: \`$PINNED\`"
          echo "Latest: [\`$LATEST\`]($URL)"
          echo ""
          if [ "$DRIFT" = "true" ]; then
            echo "Regenerating with the new JAR **changes** the"
            echo "generated parser. Review the diff carefully."
          else
            echo "Regenerating with the new JAR produces **no diff**"
            echo "in \`src/generated/\`. Safe to bump the version."
          fi
        } > /tmp/issue-body.md
        BODY=$(cat /tmp/issue-body.md)
        EXISTING=$(gh issue list --label staleness \
                   --search "$TITLE" --state open --json number \
                   --jq '.[0].number // empty')
        if [ -n "$EXISTING" ]; then
          gh issue edit "$EXISTING" --body "$BODY"
        else
          gh issue create --title "$TITLE" \
            --label staleness --body "$BODY"
        fi
```

Key details:
- Extracts the pinned version from `regenerate-antlr.sh` so the
  workflow doesn't duplicate the version string.
- Actually regenerates and diffs, so the issue can say whether the
  new release produces different output or is a no-op bump.
- Uses the same `staleness` label and idempotent issue
  creation/update pattern as the avro job.

### Shared concerns

- Both jobs need `issues: write` permission to create/edit issues.
  Add it to the workflow-level `permissions` block.
- Create the `staleness` label in the repo if it doesn't exist yet
  (or create it in the workflow with `gh label create --force`).
- Consider running these weekly rather than nightly — upstream
  releases are infrequent and the jobs involve network fetches.
