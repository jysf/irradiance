---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-130
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-11
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-058

project:
  id: PROJ-004
  stage: STAGE-015
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---


# HANDOFF-130: VERIFY: tiled plane decode, tile tags, placement and edge tiles (SPEC-058)

## Delegation Summary

You are the **cold verifier** for SPEC-058. You did **not** implement it. Review it
against SPEC-058 and DEC-056 and return exactly one verdict.

- **The build:** **PR #202**, branch `claude/spec-058-build-5f1a2f`, cut from
  `origin/main` at `e5480ef`. The head when this handoff was minted is `3692e9b`;
  its last commit is the handback PR link only.
- **Your branch:** your worktree is cut from `claude/spec-058-verify-base`. That is
  the build head plus this handoff and the spec's `cycle: verify`, so the code under
  review is the build's code.

The build delivers:
- **Tiled planes in `plane::unpack_into` and `validate`:**
  - tile tags 322–325 on `Sensor`;
  - layout dispatch and a shared tile-table check;
  - uncompressed tiles copied row by row.
- **Compression-7 tiles in `src/ljpeg.rs`:** they decode through a clipped,
  strided destination window (DEC-056). This is the change with the most risk: the
  bit-exact reconstruct loops changed shape, and the strip path must come out
  byte-identical.
- **New `Error` variants** with `ErrorKind` classification.
- **Tests:** `tests/plane_tiles.rs`, plus additions to `plane_oracle`.
- **Fuzz seeds, corpus manifest entries** for the three Pixel files, and docs.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-058-tiled-plane-decode-tile-tags-placement-and-edge-tiles-uncomp.md`.
  Read all of it, especially the design-time probe table, Design §1–10 and
  AC1–AC11.
- **Decisions:**
  - `./decisions/DEC-056-tiled-planes-decode-in-place-through-a-clipped-strided-window.md`
  - DEC-016: `unpack_into` allocates nothing.
  - DEC-047: frame shape is bound to the plane, here to the tile.
  - DEC-050: validate before allocate, and a shared validate pass.
  - DEC-051: wildcard-free `Error::kind`.
  - DEC-049: required and mandatory opcodes.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-129-build-tiled-plane-decode-tile-tags-placement-and-edge-tiles-uncomp.md`.
  Treat its claims as claims and check them yourself. Judge whether each declared
  deviation is sound:
  - AC5 overflow asserted on `validate` only;
  - the transposed tile is `Ok` from `validate` and `LjpegPlaneShapeMismatch` from
    `unpack_into`;
  - AC8 refuses with `UnsupportedMandatoryOpcode { id: 9 }`;
  - the window red-proof lives in `tests/plane_oracle.rs`;
  - the Pixel files are skipped by name in `metadata_oracle`;
  - two legacy tests were rewritten.

  The build also says the Compression-7 path allocates `HuffTable` `Vec`s in
  `parse_headers`, an issue that predates this spec. Confirm that this spec added no
  allocation and the old one is unchanged.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test`
  already passes `--all-features`; add no flag. For tier B, set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. The three
  Pixel files are in `GOOGLE-PIXEL-4A`, `GOOGLE-PIXEL-3A` and `GOOGLE-PIXEL-7-PRO`.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC11 met by a test that would fail if the property broke?
- Is the build reflection honest?
- Run the gate suite yourself, with the corpus env set:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `fmt-check`
  - `validate`
  - `cost-audit`
  - `gate-audit`
  - `doc-check`
  - `deny`
- Run `just decisions-audit --changed`.

### The behavioural surface (this is where defects escape)

**The Pixel oracle runs, and is independent.**
- Run `cargo test --all-features --test plane_tiles --test plane_oracle -- --nocapture`
  with the corpus set. Confirm no SKIP line appears for the Pixel files.
- Independently recompute one Pixel checksum outside the test code. For example,
  dump `unpack_into`'s plane as LE `u16` with a few lines of Rust or Python run with
  `-I`, MD5 it, and compare it with `dnglab analyze --raw-checksum` on the same file.
  The pinned values are in the spec's probe table.
- Confirm the expected values are dnglab's and are not derived from irradiance's own
  output.

**The strip path is byte-identical.** This is the core risk of DEC-056.
- Every held strip file's `raw_checksum` oracle must pass unchanged. Check in
  `tests/corpus/manifest.toml` that no strip expectation was edited:
  `git diff origin/main -- tests/corpus/manifest.toml` should be additions only.
- Check the LJ92 round-trip suite and the held Compression-7 files (Nf=2, the
  Pentax with APPn/COM).
- Read the window code in `src/ljpeg.rs`. Confirm a fully visible window takes
  exactly the old arithmetic: stride equals width, origin (0, 0), no discard.

**The window clip is right at every edge.** Mutate it yourself, outside any test
knob, and confirm a test goes red for each mutation:
- one extra or one fewer visible column;
- one extra or one fewer visible row;
- not discarding a right-of-plane difference (the bitstream desyncs);
- applying `Pt` to discarded samples, if that is observable;
- an off-by-one in the tile origin or stride;
- for `Nf = 2`, an odd visible width.

Use writer fixtures where the Pixel files cannot reach a case. Pixel edges may be
whole-tile or not; check which. The writer-built Compression-7 fixtures must have
partial right **and** bottom tiles, in II and MM, for Nf=1 and Nf=2.

**Hostile tile tables are refused before any write.** For each Design §3 case,
confirm the typed error comes from both `validate` and `unpack_into`, where the
deviation list does not explain otherwise, and that `dst` is untouched on refusal:
- zero `TileWidth` or `TileLength`;
- count mismatch;
- tile count overflow in `u64`;
- both strip and tile tags present;
- a tile out of file bounds;
- an uncompressed byte-count mismatch;
- counts summing past `file.len()`;
- a transposed frame;
- a clipped-size edge frame;
- a non-whole-byte tile row.

Confirm no arithmetic can panic or wrap: tile origin times stride, tile index
products, `u32` to `usize` conversions.

**No allocation added.**
- `git diff origin/main -- src/` must add no `Vec`, `Box`, `vec!`, `collect`,
  `to_vec` or `String` on the decode path.
- Check what AC6's grep test actually pins, and that it would catch a new
  allocation.

**Error surface.**
- Every new variant is in `Error::kind()` with no wildcard.
- Each has `Display` and a test.
- `Sensor` is still `#[non_exhaustive]`, and the new fields are documented.
- `CHANGELOG [Unreleased]` names the additions.

**Develop refuses the Pixel files (AC8).** It must be a refusal, not a silent
decode that ignores the GainMap.

**Fuzz.**
- Seeds in `fuzz/seeds/{plane,develop,demosaic}` include tiled fixtures.
- Run `just fuzz-smoke`, or at least the plane and ljpeg targets for 60s each, if
  the toolchain allows. Report what you ran.

**CI is observed green on the PR head** with `gh pr checks 202` or `gh run view`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are
  fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes
  (DEC-003).
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript (`basis: transcript`, dedupe by
  `message.id`, max output per message, opus-5-5 rates). Fill the `## Handback`
  section too.
- Open a NEW PR against `main` from `claude/spec-058-verify-<suffix>` carrying the
  handback, and any punch-list fixes. It must superset #202 and reference
  HANDOFF-130, SPEC-058, DEC-056 and PR #202.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
