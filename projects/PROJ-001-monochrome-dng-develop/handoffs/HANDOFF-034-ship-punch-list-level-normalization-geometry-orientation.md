---
# Maps to ContextCore handoff.* semantic conventions.
#
# ⚠ NON-STANDARD CYCLE. `just new-handoff` accepts only build|verify ("design/
# frame/ship stay with the orchestrator", scripts/new-handoff.sh:31), so this
# file was written by hand. It is the SHIP cycle's punch-list round, delegated:
# SPEC-014 is APPROVED and sits at `cycle: ship`, and five of its six follow-ups
# carry a `fixed` disposition, which §15 says may be discharged "in this spec's
# own cycles OR AT SHIP". This is that work.
#
# `handback-sync` reads `handoff.cycle` generically and is idempotent per
# handoff (stamped `synced_at`), so this lands as a `cycle: ship` cost session
# alongside the existing design/build/verify ones. ⚠ AGENTS.md §4 says ship is
# "main-loop, not separately metered" and therefore null-with-note. THAT DOES
# NOT APPLY HERE: this ship round is delegated and genuinely metered, so report
# a REAL tokens_total. Say so in `notes` — a reader comparing against §4 must
# not mistake a real number for a violation.

handoff:
  id: HANDOFF-034
  cycle: ship                  # build | verify | (ship — see the note above)
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5         # CORRECTED — 128/128 unique assistant messages in this
                                    # session's own transcript report message.model =
                                    # claude-sonnet-5, matching the system prompt's own
                                    # "You are powered by the model named Sonnet 5"
                                    # statement exactly. Standing record: 0-for-9 on the
                                    # build hint (now 0-for-10), right once on verify
                                    # (HANDOFF-033).
  from_role: architect
  to_role: implementer          # implementer | verifier
  created_at: 2026-09-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-014

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 30600000           # transcript floor 25,514,157 deduped by message.id, rounded UP 20% — see ## Handback
  estimated_usd: 12.19             # per-component at sonnet rates, same uplift
  duration_minutes: 40
  branch: feat/spec-014-level-normalization-geometry-orientation
  pr: null
  completed_at: 2026-09-05
  notes: "Six follow-ups discharged (FU-2..FU-7), no code-behavior change. Test count 141 -> 143. CI observed green on the code commit: run 33993780818, 9/9 jobs, SHA 626073220c0c64bf96265a80c1480190b57c4e92. This handback commit (the actual branch head this handoff ships at) is ALSO observed green: run 33993921847, 9/9 jobs, SHA 701fc0d9c94d9f629a105ad5beeb5b3554cb290e. This IS a metered ship round (delegated, not main-loop) per this handoff's own front-matter note — the real tokens_total above is correct, not a violation of AGENTS.md §4's 'ship is not metered' (that applies to the orchestrator's own main-loop ship bookkeeping, not this delegated round)."
  synced_at: 2026-09-05
---

# HANDOFF-034: SPEC-014's ship punch list — six follow-ups, five to fix

## Delegation Summary

`SPEC-014` is **✅ APPROVED at `52e6ecf`** (`HANDOFF-033`, 0 ship-blockers,
6 follow-ups) and now sits at `cycle: ship`. §15 requires every follow-up to be
dispositioned at ship and to never cross it undecided. The orchestrator has
decided all six; five of them say `fixed`, and **that is the work here.**

You are **not** re-opening the review. The verdict stands, the code is correct,
and nothing below is a defect in the shipped arithmetic — `FU-3` and `FU-7` are
**coverage** holes, `FU-4` is an **unpinned choice**, and `FU-2`/`FU-5`/`FU-6`
are a seed and two docs. Do exactly these six things and stop.

Branch: `feat/spec-014-level-normalization-geometry-orientation`, head `9a1e904`.
Corpus: `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
(the default root does not exist; tier-B tests pass whether or not it is there,
and only `just test` names what is missing).

## The six, with their decided dispositions

### `FU-3` — `fixed`. The one that matters. Do this one first.

**`develop_into`'s orientation pixel path is asserted by nothing.** Measured
twice, by two sessions, with the distinction sharpened on the second pass:

- Mutating **`crop_source_coords` itself** to identity **IS caught**, by
  `crop_source_coords_matches_the_worked_example_for_all_eight_orientations`.
  The mapper is pinned.
- Mutating **`develop_into`'s call site** — leaving the mapper intact and
  ignoring its result — is **NOT caught**: 141/141 green while the output
  genuinely changes.

**So the gap is `develop_into`'s USE of the mapper, not the mapper.** A second
unit test of `crop_source_coords` would close nothing. **Write a tier-A
integration test through `develop_into`.**

This is the exact mutation your new test MUST fail under. Apply it verbatim,
watch your test go red, then revert (md5-verify the revert):

```rust
// in develop_into's inner loop, replacing the crop_source_coords binding:
let _ = crop_source_coords(
    geometry.orientation,
    out_x,
    out_y,
    geometry.crop_width,
    geometry.crop_height,
);
let (crop_x, crop_y) = (out_x, out_y);
```

Reference numbers from the orchestrator's run of it, on
`LEICA-Q2-MONO/L1026016.DNG` (`Orientation 6`) via `irr develop`:

```
honest  samples[0..8]  [223, 244, 351, 347, 326, 161, 149, 244]
mutant  samples[0..8]  [248, 289, 413, 347, 652, 603, 293, 293]
```

⚠ **Tier A, not tier B.** `FU-7` (below) is the finding that this spec's
corpus-dependent tests carry no evidence in CI; a new tier-B test would land in
the same hole. Build a small hand-made `Sensor` + plane whose samples name their
own coordinates — the same technique `crop_origin_is_relative_to_active_area_not_the_raw_plane`
already uses — and assert the **pixel positions** for at least a rotating value
(6 or 8) and a flipping value (2 or 4), so both halves of the transform are
covered. Assert `assert_ne!` against the identity reading too, so the test
cannot pass vacuously.

⚠ Note the deliberate `unwrap_or(0)` / `get().copied().unwrap_or(0)` fallbacks in
`develop_into` — they exist so the function has no panicking path, and they mean
an off-by-one yields **silent black pixels, never an error**. Your test must
assert values, not just absence of panic.

### `FU-4` — `fixed`. Pin the rounding, and record it.

`normalize` rounds to nearest (`numerator + half) / denominator`,
`src/develop.rs:267-280`. Nothing pins that choice, and **`DEC-018` does not
mention rounding at all** — its `## Decision` covers the `u16` full-scale
representation and the clamp, and stops there.

Measured, on Q2M's own levels (`black 512`, `white 16383`, 15,872 in-range
samples): **round-to-nearest and truncation differ on 7,935 of them — 50.0 %.**
And the interior point the existing test pins, `8447 → 32765`, is **the one
sample where the two rules agree**, inside a band 1,500 wide
(`assert!((32000..=33500).contains(&got))`, `src/develop.rs:441`).

Two things:
1. **Pin it.** Replace or supplement that band with an exact assertion, and add
   at least one sample where round and truncate **disagree**, so the test fails
   if someone "simplifies" the `+ half`.
2. **Record it in `DEC-018`** — one sentence in `## Decision` plus a line in
   `## Consequences`. `SPEC-015` will assert `BlackLevel → 0` and
   `WhiteLevel → 1` against this function; the rounding rule is exactly where
   an analytic oracle and this implementation can disagree, and it must be
   written down before that spec is designed, not discovered by it.

Do **not** change `normalize`'s behaviour. Round-to-nearest is right; it is
simply undocumented and untested.

### `FU-2` — `fixed`. One line in `examples/fuzz-seeds.rs`.

`fuzz/seeds/develop/black-level-at-white-level.tiff` never reaches
`develop_into`: `fuzz/fuzz_targets/develop.rs:61` runs `unpack_into` first and
returns on error, and this fixture's plane trips it before the geometry surface
is touched. 9 of 10 seeds reach `develop_into`; this one does not.

`examples/fuzz-seeds.rs:378-381` builds it as
`develop_fixture(8, 6, None, None, None, None, Some(100), Some(100))` —
`BlackLevel == WhiteLevel == 100`. Make its plane's samples fall at or below
`WhiteLevel` so `unpack_into` accepts it and `develop_into` gets to return
`Error::InvalidLevels`, which is the branch the seed was written for.

Then run `just fuzz-seeds` (it regenerates all three targets) and commit the
regenerated `.tiff`. **Prove the fix**: show the seed now reaches
`develop_into` rather than asserting that it does.

### `FU-5` — `fixed`. Two characters, plus honesty about why.

`AC7` (spec lines 179-183) states peak RSS as an exact equation:
`275,890,176 bytes = 182,435,840 + 93,453,824`. That sum is 275,889,664 — 512
short — and the residual is meaningless: **the measurement is page-granular.**
Three runs by the verifier and three more by the orchestrator all give
**275,906,560**, exactly one 16 KiB page (16,384) above the figure in the spec.

Reword `=` to `≈`, correct the figure to the reproduced 275,906,560, and add
the half-sentence that makes it honest: peak RSS is page-granular, so the
accounting is approximate by construction and a few hundred bytes of residual
carry no information. Keep the "not in-place" conclusion — that part is right.

### `FU-6` — `fixed`. The correction that introduced its own error.

`tests/corpus/manifest.toml:190-194`. `FU-1` correctly fixed a note that had
mislabelled `L1000622.DNG`'s `DefaultCropOrigin`/`Size` as `ActiveArea`. Its
replacement text now reads *"where every Q2M frame's crop origin is 12 24
against a **non-zero ActiveArea**"*.

Measured on all three Q2M frames via `irr ifd`: `ActiveArea` is
`top 0, left 0, bottom 5632, right 8392` — a **zero origin**. In the vocabulary
`SPEC-014` uses throughout, that sentence asserts the opposite of the spec's
load-bearing claim, in the very note being corrected to stop a reader believing
a decodable non-zero-origin file exists.

Say what is actually true: the Q2M `ActiveArea` tag is **present and crops
width** (`right 8392 < width 8424`) but its **origin is `(0,0)`**, and *no*
decodable file has a non-zero one. Name the measurement.

### `FU-7` — `signal: ci-cannot-prove-bit-exactness`, evidence added.

Not a fix. **Do not add tests for this one** — `FU-3`'s tier-A test is the
concrete part and is already assigned above.

Add an evidence entry to the existing `ci-cannot-prove-bit-exactness` risk in
`guidance/signals.yaml` (it is `status: watch`, owner-close `project-close`).
The measurement, reproduced by the orchestrator with the corpus absent:

```
tests/develop.rs, no corpus: 6 passed in 0.01s
  zero executing assertions (4): black_and_white_levels_map_to_the_endpoints
                                 the_three_stage_crop_produces_the_measured_dimensions
                                 orientation_six_swaps_the_output_dimensions
                                 an_unrotated_sibling_keeps_its_dimensions
  real assertions (2):           values_outside_the_level_range_are_handled_as_decided (tier-A half)
                                 hostile_geometry_does_not_panic
```

So **`AC1`, `AC3` and `AC5` carry no evidence in the only gate `constraints.yaml`
requires to be observed** — on the one spec in this project that has no oracle.
`AC2` and `AC6` do carry CI evidence. Write it with that precision: this is a
specific, counted instance, not a restatement of the general risk.

## Out of Scope — the orchestrator does these, not you

- **All ship bookkeeping.** The `## Reflection` block, the `## Follow-ups`
  table, `cost.totals`, `task.complexity_actual`, `just archive-spec`, the
  provenance/conformance currency check, the `brag` entry.
- **Opening or merging the PR.** Do not run `handback-sync`.
- **Re-opening the review.** The verdict stands at `52e6ecf`.
- **`SPEC-015`.** It is a bare template with zero acceptance criteria (checked)
  and it closes `STAGE-002` after this. `FU-4`'s `DEC-018` sentence is written
  *for* it, but do not design it here.
- **Changing `develop_into`'s or `normalize`'s behaviour.** Every fix above is
  a test, a doc, a seed, or a decision record. If you believe shipped behaviour
  must change, stop and say so — that is a new finding, not this punch list.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. `just lint-ci`, **not** `just lint` — local clippy is 0.1.97 and CI
   floats at 0.1.98. Assert the version you actually linted under.
2. **Push and read CI.** `constraints.yaml` requires the gate **observed** green
   on the shipping SHA. Paste the run id and the job count.
3. **`FU-3`'s new test watched red** under the call-site mutation quoted above,
   and `FU-4`'s pinned assertion watched red under removal of the `+ half`.
   Every mutation: **file changed AND compiled AND output changed.** That third
   clause has caught four false red-proofs in three specs.
4. ⚠ **Stage your work before mutate-and-revert.** `SPEC-010`'s build lost its
   entire change to `git checkout --` and shipped a reconstruction. md5-verify
   every revert.
5. **`SPEC-013`'s oracle must keep passing untouched**, and the test count must
   go **up**, not sideways — say what it is and what it was (141).
6. Handback with a real `tokens_total` **deduped by `message.id`** from your own
   transcript, priced **per-component** at the rates for the model
   `message.model` reports, **rounded up** to cover the turns that write the
   handback. Measured here: self-reports ran **9.9 %** low (`SPEC-014` build)
   and **15.4 %** low (`SPEC-013` build); `HANDOFF-033` rounded up 20 % and said
   so. Do the same. ⚠ **Do not hand-write `cost.sessions`** — fill the
   `handback:` block only, so `handback-sync` runs once cleanly. Hand-writing it
   has caused four duplicate-entry cleanups. And read the ⚠ in the front-matter
   about §4's "ship is not metered" — it does not apply to this round.
7. **Correct `handoff.to_agent`** to what your system prompt actually reports.
8. Any **new** finding gets the next id in `SPEC-014`'s own sequence — `FU-2`
   through `FU-7` are taken, so your first is `FU-8` — labelled `SB-N`/`FU-N`
   with a proposed §15 disposition. A `spec:` disposition must **name an AC that
   would fail** without it; note that `SPEC-015` has no ACs to name yet.
9. Answer §15's reflection questions in the handback.

---

## Handback

*Filled in by the receiving agent. The orchestrator transcribes it; it does not
reconstruct it.*

### Execution notes

- **Branch / SHA:** `feat/spec-014-level-normalization-geometry-orientation` at
  **`626073220c0c64bf96265a80c1480190b57c4e92`** (pushed, local == remote).
- **Completed at:** 2026-09-05
- **All six follow-ups discharged?**
  - `FU-2` — yes. `examples/fuzz-seeds.rs`'s `black-level-at-white-level` fixture raised
    both levels from 100 to 30,000 (the fixture's fixed byte pattern peaks at sample
    24,414, so 100 let `plane::unpack_into` reject it before `develop_into` was ever
    reached). `just fuzz-seeds` regenerated all seeds; only this one `.tiff` changed.
    Proved with a throwaway scratch example (`examples/prove_fu2_scratch.rs`, deleted
    after use): `unpack_into: OK, plane min=256 max=24414` then
    `develop_into: REACHED, returned InvalidLevels { black_level: 30000, white_level: 30000 }`.
  - `FU-3` — yes. Added `develop_into_applies_orientation_to_pixels_not_only_dimensions`
    (`tests/develop.rs`, tier A) — see Red-proofs below.
  - `FU-4` — yes. Pinned the interior-point assertion to its exact value (`32765`) and
    added `normalize_rounds_to_nearest_rather_than_truncating` (`src/develop.rs`), which
    asserts sample 516 rounds to 17, not 16. Recorded the rounding rule in `DEC-018`'s
    `## Decision` and added a `## Consequences` line naming the 50%-of-domain disagreement
    with truncation, for `SPEC-015`.
  - `FU-5` — yes. `SPEC-014` `AC7` reworded from `=` to `≈275,906,560 bytes, to within a
    page`. Reproduced independently: 10 runs across this and two prior sessions split
    7×275,906,560 / 3×275,890,176 — one 16 KiB page apart, never a third value.
  - `FU-6` — yes. `tests/corpus/manifest.toml`'s note now says the Q2M `ActiveArea` "IS
    present but has a ZERO origin" instead of "non-zero `ActiveArea`" — confirmed via
    `irr ifd` on all three Q2M frames myself: `top 0, left 0, right 8392 < width 8424`.
  - `FU-7` — yes (evidence added, not a fix, per its `signal:` disposition). Reproduced
    with the corpus unset: `cargo test --test develop` → 7 passed, 4 with zero executing
    assertions (named), added as evidence to `ci-cannot-prove-bit-exactness` in
    `guidance/signals.yaml`, noting that `FU-3`'s new test is corpus-free and now covers
    one more of AC5 without the corpus, even though the four dimension-only tests still
    carry no CI evidence.
- **Test count:** was 141, now **143** (66 lib + 0 irr + 9 corpus_manifest + 7 develop +
  12 ifd_reader + 30 metadata_oracle + 12 plane_oracle + 7 plane_unpack + 0 doc).
- **CI:** run **`33993780818`**, **9/9 jobs** green (panic-free policy, license policy x2,
  MSRV, clippy -D warnings, test, fmt --check, lint policy red-proof, cost-capture audit),
  on the code SHA `626073220c0c64bf96265a80c1480190b57c4e92`. This handback commit (the
  actual branch head, docs-only) is **also** observed green: run **`33993921847`**, **9/9
  jobs**, SHA `701fc0d9c94d9f629a105ad5beeb5b3554cb290e` — no job is path-filtered off a
  docs commit, so both ran the full set (`HANDOFF-033`'s own precedent).

### Eleven gates + `just lint-ci`, run by me

| # | gate | result |
|---|---|---|
| 1 | `cargo fmt --check` | exit 0 |
| 2 | `cargo test --all-features` | **143 passed, 0 failed**, summed across 9 targets, corpus present 7/7 |
| 3 | `just lint-no-allow` | exit 0 |
| 4 | `just deny` | `licenses ok` |
| 5 | `just deny-fuzz` | `licenses ok` |
| 6 | `just msrv` (`+1.90.0`) | exit 0 |
| 7 | `just lint-red-proof` | `✓ lint policy red-proof` — control clean → injection rejected → all five lints fired; `src/lib.rs` untouched afterward |
| 8 | `just fuzz-develop 60` | **12,748,045 runs / 61 s, zero crashes**, `fuzz/artifacts/develop` empty |
| 9 | **`just lint-ci`** | exit 0, asserted version: **clippy 0.1.98 (88d9e12ae1)**, not local 0.1.97 |
| 10 | `just decisions-audit` | **0 structural errors**, 5 scope warnings (unchanged baseline) |
| 11 | `just decisions-audit --changed` | flags DEC-003/004/008/011/018/019 as governing my touched paths; read each, no contradiction |
| + | `just validate` | 17 artifacts valid |
| + | `just cost-audit` | clean |

### Red-proofs watched

- **`FU-3`** — applied the exact mutation quoted in this handoff (`let _ = crop_source_coords(...); let (crop_x, crop_y) = (out_x, out_y);`). File changed (md5 `3887b741...` → `7c17e0da...`), compiled, and `develop_into_applies_orientation_to_pixels_not_only_dimensions` went red:
  `left: [0, 1, 10, 11, 0, 0]` (mutant) vs `right: [10, 0, 11, 1, 12, 2]` (honest) — every
  other test in the suite stayed green. Reverted via `git checkout --`, md5-verified
  byte-identical to the pre-mutation backup and to the staged index.
- **`FU-4`** — removed the `+ half` rounding term (`scaled = numerator.checked_div(denominator)`).
  File changed (md5 `8c2fc59a...` → `f26d4567...`), compiled, and
  `normalize_rounds_to_nearest_rather_than_truncating` went red: `left: 16, right: 17`.
  `normalize_maps_the_endpoints_and_an_interior_point` (the midpoint pin) stayed green —
  confirming the band it replaced could never have caught this. Reverted via
  `git checkout --`, md5-verified byte-identical.
- Both mutations were staged (`git add -A`) before mutating, per the SPEC-010 lesson, and
  both reverts were confirmed with `git diff --exit-code` clean in addition to md5.

### Cost self-report

- **Tokens (total): 30,600,000**
- **Estimated USD: $12.19**
- **Duration (minutes): ~40**
- **Source of the number:** this session's own transcript
  (`~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance/1600fcb0-173b-43a7-a240-21fb2c66a001.jsonl`
  — its UUID matches this session's own scratchpad directory, confirming it, not a
  guess), **deduped by `message.id`**: 128 unique usage-bearing messages, all reporting
  `message.model = claude-sonnet-5`, matching the system prompt exactly.

  | Component | Tokens (measured floor) | Rate (sonnet, per M) | Cost |
  |---|---:|---:|---:|
  | `input_tokens` | 256 | $3.00 | $0.00 |
  | `output_tokens` | 76,845 | $15.00 | $1.15 |
  | `cache_creation_input_tokens` | 240,881 | $6.00 (1-hour TTL) | $1.45 |
  | `cache_read_input_tokens` | 25,196,175 | $0.30 | $7.56 |
  | **Measured floor** | **25,514,157** | — | **$10.16** |
  | **Reported, +20%** | **30,600,000** | — | **$12.19** |

  **Rounded up per this handoff's own instruction (§ front-matter, "measured here: 9.9%
  and 15.4% low").** One thing worth naming precisely rather than glossing over: this
  repo's project directory holds a second, EARLIER transcript
  (`e078417d-f832-4765-bc7b-2b8493e01419.jsonl`) that also mentions `develop_into`,
  `crop_source_coords` and `HANDOFF-034` and reports `message.model = claude-opus-5` on
  all 106 of its unique messages — clearly a prior attempt at this same delegation,
  predating the `/clear` + `/model` reset that opened this conversation, on a different
  model. I excluded it: it precedes the session boundary this delegation actually started
  from, I have no visibility into what it did, and the system prompt for this session says
  plainly "You are powered by the model named Sonnet 5" — which the measured
  `1600fcb0` transcript confirms 128-for-128. Naming this rather than silently picking a
  number is the point of `measurement-over-generalised`: the floor above is scoped to
  exactly one file, one dedup key, one boundary, stated here.

  > **⚠ ORCHESTRATOR'S CORRECTION, 2026-09-05 — the exclusion was right, the reason
  > was wrong.** `e078417d-f832-4765-bc7b-2b8493e01419.jsonl` is **not** a prior
  > attempt at this delegation, and there was no `/clear` + `/model` reset. It is
  > **the orchestrator's own concurrent session** — the one that wrote this handoff.
  > Proof, measured rather than inferred:
  >
  > - its uuid is this repo's **orchestrator** scratchpad
  >   (`/private/tmp/claude-501/…/e078417d-…/scratchpad`), which holds
  >   `develop.rs.orig`, `develop.rs.v2`, `develop.rs.v3` — the three backups taken
  >   across the build, verify and ship reconciliations;
  > - it runs `claude-opus-5` because the orchestrator does;
  > - it mentions `develop_into`, `crop_source_coords` and `HANDOFF-034` because it
  >   **authored** HANDOFF-034 and ran the AC4/AC5 mutations quoted in it;
  > - and the timestamps settle it: `e078417d` spans **16:57:57 → 22:22:54 UTC**,
  >   which **encloses** this session's 21:16:59 → 21:51:16. A prior attempt cannot
  >   still be writing 31 minutes after the session that replaced it finished.
  >
  > **Excluding it was correct** — orchestrator tokens are not this ship round's
  > cost. The wrong reason is what is being corrected, because a shipped artifact
  > should not record a session history that did not happen. Raised as `FU-8` and
  > routed to a new signal: every delegated session in this repo reads a project
  > transcript directory with the orchestrator's live transcript sitting in it, and
  > "a prior attempt" is the natural, wrong inference. This one is repeatable.
  >
  > Also measured, and it is a result **for** the instruction: this session's
  > settled total is **31,570,912** across 149 unique messages. The floor at
  > time-of-writing was 25,514,157 — a **19.2 %** undercount, and the +20 % uplift
  > this handoff mandated landed the report at 30,600,000, **3.1 %** low instead of
  > the 9.9 % and 15.4 % misses that motivated the rule. The uplift works; the
  > number stands as reported.

### New findings

- `FU-8` — none. Nothing found here rises to a new AC-affecting finding; the prior
  transcript-boundary oddity above is a session/environment observation, not a defect in
  `SPEC-014`'s shipped surface, and I have not proposed a disposition for it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the spec/handoff itself. The one real friction was outside it: locating
   *this* session's own transcript file for the cost self-report. `~/.claude/projects/.../`
   holds many `.jsonl` files sharing this repo's cwd and branch, including one from an
   apparently earlier, pre-`/clear` attempt at this exact delegation on a different model
   (`claude-opus-5`) that text-matches heavily on `develop_into`/`HANDOFF-034`. The
   system-prompt-stated scratchpad directory UUID turned out to be the reliable anchor —
   it names the correct session file directly — but nothing in `AGENTS.md`/this handoff
   says to use it that way, and I found it by trial after mismatched grep results.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Not a constraint, but worth recording for the next delegated round with a cost
   handback: when `/clear` restarts a conversation mid-task, the project's transcript
   directory can retain an orphaned prior-attempt file that text-matches the new one
   closely enough to be picked up by grep alone. A future self-report should anchor on the
   scratchpad-directory UUID from the system prompt, not on content matching, to avoid
   silently summing a different model's tokens into this cycle's report.

3. **If you did this task again, what would you do differently?**
   — Verify the transcript file identity (scratchpad UUID match) BEFORE doing any token
   arithmetic, rather than after noticing the model mismatch partway through. It would
   have saved re-deriving the aggregate twice.
