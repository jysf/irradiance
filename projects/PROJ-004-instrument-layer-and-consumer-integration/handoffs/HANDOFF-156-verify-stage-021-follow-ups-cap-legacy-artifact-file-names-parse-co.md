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
  id: HANDOFF-156
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-18
  status: completed                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-074

project:
  id: PROJ-004
  stage: STAGE-022
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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 4661997               # REAL combined count — what cost-audit reads
  estimated_usd: 3.69              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 15
  branch: claude/spec-074-verify-0f9175
  pr: "https://github.com/jysf/irradiance/pull/259 (verdict comment; no new PR)"
  completed_at: 2026-11-18               # YYYY-MM-DD
  notes: "APPROVED at f56f5eb. 37 message ids, no subagents, opus-5-5 list rates with 1-hour cache writes; measured just before the handback commit."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-156: VERIFY: STAGE-021 follow-ups, bundled (SPEC-074)

## Delegation Summary

You are the **cold verifier** for SPEC-074. You did **not** implement it. Review it
against the spec, and return exactly one verdict.

- **The build:** **PR #259**, branch `claude/spec-074-build`, cut from `origin/main` at
  `073b594`. Its head is `f56f5eb`, in four commits:
  - `74021de` Part B: one Compression 7 header parse (`src/ljpeg.rs`, `src/plane.rs`,
    the pin at 5, CHANGELOG);
  - `c37a3b2` Part A: renames only (83 legacy artifacts) plus reference rewrites;
  - `9f3a724` Part A: `name-length-check`, its CI step, `semver-check`, AGENTS.md §6;
  - `f56f5eb` the handback.
- **Your branch:** your worktree is cut from `claude/spec-074-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-074-stage-021-follow-ups-cap-legacy-artifact-file-names-parse-co.md`.
  Read all of it: the probe findings, A1–A3, B1–B3, AC1–AC7, Non-Goals and the Notes.
- **Decision:** `decisions/DEC-067-…`. Also DEC-047 (shape binding first) and DEC-009
  (red-proof, control first).
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-155-build-stage-021-follow-ups-cap-legacy-artifact-file-names-parse-co.md`.
  Treat its claims as claims. The points to judge:
  - **Error identity in Part B.** Read the new `pub(crate)` functions and the two plane
    call sites. Confirm the order is unchanged: parse, then the shape check, then the
    scan from the same entropy offset. Confirm `decode_into` and `decode_window_into` are
    thin wrappers with unchanged behaviour. Look for any path where an error variant or
    field could now differ.
  - **The deviation:** `open_scan` kept its name with a new signature, because
    `tests/plane_tiles.rs` uses `fn open_scan` as the end anchor of its no-allocation
    source scan. Check that the new functions really sit inside that scanned region,
    and that the scan still means what it meant.
  - **The renames.** Re-derive the expected set yourself: every tracked
    `SPEC-`/`HANDOFF-`/`STAGE-`/`PATCH-`/`SPIKE-` file at `073b594` whose slug exceeds
    60 characters, with the new name per `scripts/_lib.sh` `slugify`. Diff that against
    commit `c37a3b2`'s renames (`git show -M --name-status c37a3b2`). Confirm:
    - the commit is pure renames plus reference rewrites (no content edits in renamed
      files, so rename similarity is 100% except where a reference inside it changed);
    - spec and timeline pairs share a slug;
    - no old name remains except the history quotes the handback lists.
  - **The gate.** Reproduce `name-length-check`'s red-proof yourself: 151 bytes red,
    150 green, control first. Confirm the CI step is in the `cost-data` job and ran on
    #259.
  - **AC3.** Run `cargo semver-checks --baseline-rev HEAD` and `just semver-check`.
- **Toolchain:**
  - AGENTS.md §5–6. `just test` takes about 3 minutes (the corpus profile plus the debug
    `*_alloc` binaries).
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.

## The verify contract

### Acceptance criteria and gates
- **Check that each of AC1–AC7 is met** by evidence that would fail if the property
  broke.
- **Run the gate suite yourself, with the corpus env set:**
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `lint-fuzz`
  - `fmt-check`
  - `validate`
  - `doc-check`
  - `deny`
  - `msrv`
  - `cost-audit`
  - `gate-audit`
  - `name-length-check`
  - `decisions-index --check`
- **Observe CI** green on the PR head with `gh pr checks 259`.

### The behavioural surface
- **Test edits are only the ones permitted.** `git diff 073b594 -- tests/` may show only
  `C7_ALLOCATIONS_PER_CHUNK`, its comment and its assertion message in
  `plane_units_alloc.rs`.
- **Every Compression 7 oracle and MD5 pin is unedited** and green, tier A and tier B.
- **No public item changes.**

**Mutate it yourself** (on an archived copy or local commits you never push):
- restore the double parse in one plane path only (strip, then tile): the pin goes red
  with the expected counts;
- swap the order so the scan runs before the shape check: does any test notice? If none
  does, report it as a gap (a finding, not necessarily a punch-list item);
- pass a wrong entropy offset (for example 0) into the new function: the oracles go red;
- in the gate, change the limit to 151: the 151-byte name passes, which shows the
  red-proof is what pins the limit.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved with no code changes:**
  - Do **not** open a PR that supersets #259.
  - Commit the filled HANDOFF-156 handback (and the timeline's verify line) on a branch
    named `claude/spec-074-verify-<suffix>`, and push it.
  - Post the verdict as a comment on PR #259 with the branch name.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` that supersets #259.
  - Reference HANDOFF-156, SPEC-074 and PR #259.
- **Rejected:** comment on #259 with the reasons, and push the handback branch.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Stage explicit paths.
- Fill this handoff's `handback:` front matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

## Out of Scope

- Changing `slugify` or any template-managed script.
- Renaming DECs.
- A semver CI gate.
- Any change to the scan reader or decode output.

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

*Filled in by the cold verifier (claude-opus-5-5), 2026-11-18.*

### Execution notes

- **Branch / PR:** `claude/spec-074-verify-0f9175` (this handback and the timeline's
  verify line). The verdict is a comment on PR #259. There is no new PR: the build
  needed no code change.
- **Completed at:** 2026-11-18
- **All acceptance criteria met?** Yes. AC1–AC7 were checked independently; the
  evidence follows.
- **For `verify`:** ✅ **APPROVED at `f56f5eb`** (the PR #259 head; the build's last
  code commit is `9f3a724`).

#### Part B: error identity

- `decode_into` and `decode_window_into` are now `parse_headers`, then
  `decode_parsed_into` / `decode_parsed_window_into`.
- The back halves are the old bodies verbatim, with the `Nf=2` / `Nf=1` split and the
  `LjpegMissingMarker { marker: MARKER_DHT }` fallback. `open_scan(data, entropy_at)`
  keeps the single slice-or-`Truncated { at, len: 0 }` site.
- Both plane paths still run, in order:
  1. `parse_headers(chunk)?`
  2. `check_strip_shape(..)?`
  3. the scan from the same `entropy_at`.
- The old second parse was the same pure function on the same bytes, so it could
  never fail where the first one passed. No error variant or field can differ.
- `src/lib.rs` (the fuzz entry), `fuzz/` and every public item are untouched, and
  semver-checks gives 202/202.

#### The `open_scan` deviation is accepted

- `tests/plane_tiles.rs` `the_tile_path_allocates_nothing` scans `src/ljpeg.rs` from
  `pub(crate) struct Window` (line 1301) to the first `fn open_scan` (line 1898). That
  text occurs exactly once.
- Both new functions (lines 1846 and 1873) and both wrappers sit inside that region.
  The scan still covers every decode entry point, now plus the from-parsed ones.
- `open_scan` and `parse_headers` stay outside it, as before. `open_scan` itself still
  allocates nothing.

#### Renames (`c37a3b2`)

- **Rename set.** I re-derived it independently: at `073b594`, every tracked
  `SPEC`/`HANDOFF`/`STAGE`/`PATCH`/`SPIKE` file whose slug (after the cycle word and
  before `-timeline`) exceeds 60, with the new name from `scripts/_lib.sh`'s own
  `slugify`.
  - That gives 83 files, and they equal the commit's 83 renames exactly (`diff` empty).
  - There are no collisions.
  - The 16 spec and timeline pairs share a slug.
- **Pure renames plus reference rewrites.**
  - 76 renames are R100 and 7 are R098/R099. The other 33 files are modified.
  - Applying the old-to-new name map to each of the 44 removed lines reproduces the 44
    added lines exactly, with 0 mismatched hunks, so there are no content edits.
- **No old name remains** at HEAD, with or without `.md`. This includes the history
  quotes, which use `…`.
- At HEAD, 0 slugs are over 60.
- `git log --follow` on the renamed HANDOFF-064 reaches `4afc018` (PR #37).

#### Gate (AC4)

Reproduced in a scratch worktree, control first:
- The tree is green: 833 paths.
- A 150-byte name is green.
- A 151-byte name is red with exit 1, and the output names it.
- Mutant `LIMIT=151` lets the 151-byte name pass, so the red-proof is what pins the
  limit.
- The script from `9f3a724`, run on the `073b594` tree, is red on 13 names.
- The step is in the `cost-data` job (`cost-capture audit`). It ran green on #259 run
  44421850791 at `f56f5eb`: 832 paths.

#### AC3: semver

Both checks use cargo-semver-checks 0.51.0.
- `cargo semver-checks --baseline-rev HEAD` gets past the clone: 202 checks, 202 pass.
- `just semver-check` against crates.io 0.2.2: 202 pass.

#### AC5–AC6: mutants

All mutants ran on a scratch worktree with its own target dir, control first; none
were pushed.
- **Double parse, strip path only:** red, 40 against 20 ("c7 multi-strip band 1").
- **Double parse, tile path only:** red, 120 against 60 ("c7 tiles band 1").
- **Scan before shape check, strip:** red in `plane::tests::compression7_frame_dimension_mismatch_is_typed_error`
  and `ljpeg_shape_binding::line_count_mismatch_is_a_shape_mismatch`.
- **Scan before shape check, tile:** red in `plane_tiles::clipped_size_edge_frame_is_refused`
  and `tile_frame_shape_is_bound_to_the_tile`.

  The order is pinned in both paths, so there is no gap to report.
- **Entropy offset 0, both paths:** red across `plane_oracle` (MD5 pins), `plane_units`,
  `ljpeg_shape_binding` (including `corpus_comp7_files_still_bit_exact`), `develop_*`,
  `black_levels`, `dng_writer` and `plane_tiles`.
- **Control after the mutants:** green.

#### Tests and gates (AC6–AC7)

- **Test edits.** `git diff 073b594 -- tests/` touches only `plane_units_alloc.rs`:
  the constant (10 to 5), its doc comment and the assertion message. No oracle or MD5
  pin was edited.
- **`just test`** (corpus 12/12 present): 710 passed, 0 failed, 1 ignored, 3 min 11 s.
- **Gates:** `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`,
  `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`, `name-length-check` and
  `decisions-index --check` all exit 0.
- **`just ljpeg-timing`** (medians, ms): 105.5 (K3III) / 93.3 (M246) / 61.3 (Pixel 4a).
  The build measured 107.3 / 93.7 / 61.1, so there is no measurable change.
- **CI on #259 (`f56f5eb`):** 22/22 pass.

### Cost self-report

- **Tokens (total):** see front matter (`tokens_total`).
- **Estimated USD:** see front matter (`estimated_usd`), at opus-5-5 list rates with
  1-hour cache writes.
- **Duration (minutes):** see front matter.
- **Source of the number:** the transcript `ec107918-d4ee-4d06-b07d-3d543d80ea5d.jsonl`,
  found by the scratchpad UUID. Each `message.id` is counted once at its max output.
  There are no subagents. Measured just before the handback commit, so the last few
  messages are not counted.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none of my own. I accept the build's three deviations:
  - The pin's assertion message was reworded. It is a test edit the handoff permits.
  - `open_scan` kept its name; see above.
  - There is no separate red-proof script for the gate. The spec asked only for a shown
    red-proof, and I reproduced it.
- **Follow-up work identified:** none blocking.
  - `just id-audit` is red on main from stale worktrees, as the build reported. It is
    not a CI gate, and I did not re-run it here.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The handoff's mutant list mapped one-to-one onto edits.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The handoff lists `decisions-index --check` beside the recipe names. Passing it
   quoted as one argument to `just` fails as an unknown recipe, which looks like a red
   gate.
3. **If you did this task again, what would you do differently?**
   — Derive the rename set and the line-level rewrite check as one script up front.
   That was the strongest evidence, and it was cheap.
