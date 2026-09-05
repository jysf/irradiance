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
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map, not a measurement
                                    # (signal `tier-map-predicts-what-it-should-record`).
                                    # CORRECT THIS to whatever your own system prompt
                                    # reports as `message.model` before handing back.
                                    # Standing record: 0-for-9 on the build hint,
                                    # right once on verify (HANDOFF-033).
  from_role: architect
  to_role: implementer          # implementer | verifier
  created_at: 2026-09-05
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-014

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — see the cycle note above
  estimated_usd: null
  duration_minutes: null
  branch: feat/spec-014-level-normalization-geometry-orientation
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null
  synced_at: null                  # stamped by `just handback-sync` — do not edit
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

- **Branch / SHA:**
- **Completed at:** YYYY-MM-DD
- **All six follow-ups discharged?** yes/no — one line each
- **Test count:** was 141, now ___
- **CI:** run id, job count, SHA

### Red-proofs watched

- **`FU-3`** — mutation applied, test red, output delta:
- **`FU-4`** — `+ half` removed, assertion red:

### Cost self-report

- **Tokens (total):**
- **Estimated USD:**
- **Duration (minutes):**
- **Source of the number:**

### New findings

- `FU-8` … (or "none")

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
