---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-067
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-18
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - projects/
  - spikes/
  - app.just
  - .github/workflows/ci.yml
  - AGENTS.md
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-074-stage-021-follow-ups-cap-legacy-artifact-file-names-parse-co.md

tags:
  - process
  - tooling
  - file-names
  - release
  - stage-022
---

# DEC-067: legacy artifact names are cut to the generator's slug cap, and CI gates tracked name length

## Decision

1. **Every generated artifact is renamed to the name today's generator would give it.**
   A tracked `SPEC-`, `HANDOFF-`, `STAGE-`, `PATCH-` or `SPIKE-` file whose slug is
   longer than `slugify`'s 60-character cap (`SLUG_MAX`, `scripts/_lib.sh`) is moved with
   `git mv` to the ID, its cycle word if any, the slug cut to 60 characters, and its
   `-timeline` suffix if any. A spec and its timeline keep one slug. Every reference to
   an old name is rewritten, except verbatim quotes of command output that record
   history. The ID stays the identity; the slug is only a hint, as `_lib.sh` already says.
2. **A CI gate, `just name-length-check`, fails on any tracked path component longer
   than 150 bytes.** It runs in the `cost-data` job. It covers hand-named files too
   (DECs, docs), which no generator caps.
3. **The release semver check uses the crates.io baseline,** through a `just
   semver-check` recipe, not `--baseline-rev`. Tags cut before the renames keep the long
   names, so a git baseline at those tags still fails.

## Context

SPEC-073's build ran `cargo semver-checks --baseline-rev v0.2.1`, which clones the repo
into `target/semver-checks/`. It failed with `File name too long (os error 63)` and was
worked around with `--baseline-root` on a `git archive` of the tag.

SPEC-074's design probe on `28ecd5d` reproduced it against v0.2.2. The failing path is
the clone's temporary file, `.HANDOFF-064-build-cfa-sensor-…-regression-guard.md.tmp-77733-`
plus 32 hex digits: the original name plus about 45 bytes. HANDOFF-064's name is 235
bytes, so the temporary name passes macOS's 255-byte limit. Any name over about 210 bytes
fails this way.

- The generators have capped slugs at 60 characters since template v0.6.49 (PR 164,
  2026-11-10), after a long backlog summary broke `frame-stage`. Every name made since
  then is short.
- 83 older files still have slugs over 60 characters. Six names exceed 200 bytes and 13
  exceed 150. The longest hand-named file is a 130-byte DEC.
- `cargo semver-checks` with its default crates.io baseline ran clean on `28ecd5d` (202
  checks, 202 pass). The published crate packages no `projects/`, so names cannot break
  it.

## Alternatives Considered

- **Rename only the six names over 200 bytes.** Rejected. It fixes today's crash but
  leaves names that differ from the generator's rule for no reason, and the next tool
  with a longer suffix fails on the 150–200 byte ones.
- **Change `slugify` or `SLUG_MAX`.** Rejected. It is template-managed, and the cap it
  already applies is the right one.
- **Only document `--baseline-root` and rename nothing.** Rejected. Every tool that
  copies the tree (clones, archives, backup software) would keep tripping on the same
  files.
- **A gate at 60-character slugs instead of 150 bytes.** Rejected. The generators already
  enforce the slug cap, and a byte limit on every tracked component also covers
  hand-named files.

## Consequences

- About 83 files get new names. Old PR descriptions and brag entries that link to a full
  old path no longer resolve; IDs still find every file.
- A tag cut after this lands is safe for `--baseline-rev`. v0.2.1 and v0.2.2 are not, and
  `just semver-check` avoids them.
- A hand-named file over 150 bytes now fails CI, naming the file.
