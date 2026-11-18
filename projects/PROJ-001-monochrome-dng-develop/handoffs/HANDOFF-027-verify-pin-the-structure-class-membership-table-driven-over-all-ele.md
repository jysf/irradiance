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
  id: HANDOFF-027
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # CONFIRMED, not inherited: `message.model` reads
                                   #   `claude-opus-5` on all 113 usage objects in this
                                   #   verify session's transcript. The dispatch hint was
                                   #   RIGHT this cycle — tier_map is now 2 for 7.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-03
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-009

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 7900000            # REAL combined count — what cost-audit reads
  estimated_usd: 18.00             # tokens_total × your rate, or your harness's number
  duration_minutes: 17
  branch: feat/spec-009-pin-structure-class-membership
  pr: null
  completed_at: 2026-09-04         # YYYY-MM-DD
  notes: "APPROVED at 55a25f8. Transcript-summed and DEDUPED BY message.id from this session's own JSONL (113 usage objects -> 61 distinct ids, 1.85x; 97.6% cache-read; all cache-creation on the 1h ephemeral tier, 5m tier zero). Measured floor at the time of writing: 6,660,367 (input 122 / output 34,406 / cache_read 6,499,334 / cache_write_1h 126,505), priced per-component at published OPUS rates ($15/$75/$30/$1.50 per M) = $16.13. Rounded UP to 7,900,000 / $18.00 to cover the turns spent writing this handback, per HANDOFF-020's precedent of capturing a floor before the session closes. `message.model` reads claude-opus-5 on all 113 objects, so the opus dispatch hint was correct this cycle. Did NOT run handback-sync and did NOT open the PR, per return criterion 6."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-027: Verify SPEC-009 — the Structure-class membership, at `55a25f8`

## Delegation Summary

Verify `SPEC-009` at **`55a25f8`** on `feat/spec-009-pin-structure-class-membership`
(pushed, not merged; `main` at `e6cc561`). It closes four `SPEC-008` findings and
is **STAGE-002's gate on its own inputs** — the next spec is the unpack, and the
hazard this closes is a `Compression` of `RATIONAL 2/2` reading `1`, passing
`require_uncompressed()`, and the unpack reading JPEG as raw samples.

**This is a strong build. Verify it accordingly — the risk here is not sloppiness,
it is a well-made thing with a gap nobody thought to look for.**

## What the orchestrator reconciled — reproduce, do not inherit

| claim | reconciled |
|---|---|
| branch + CI green on `55a25f8` and `3b50964` | ✅ read off the runs |
| 100 tests, 0 failed | ✅ summed across targets, corpus present |
| the table is **independent** of `is_structural_tag()` | ✅ `const STRUCTURAL_TAGS: [u16; 11]`, hand-written |
| `AC5`'s precondition assertion | ✅ `assert_eq!(c.sensor_candidates(), vec![1]);` present |
| `src/` behaviour change | ✅ **none** — the only non-test edit is the `malformed_tags` doc comment |

**⚠ The eleven-way red-proof, run by the orchestrator, not taken on report:**
every membership deleted in turn, each mutation asserted applied by `diff` and
asserted to compile, tree restored byte-identical after each.

```
control (unmutated)                    100 passed, 0 failed
TAG_NEW_SUBFILE_TYPE … TAG_STRIP_BYTE_COUNTS   1 failed each  (10 tags)
TAG_SUB_IFDS                                    2 failed
```

**Eleven for eleven.** `SPEC-008/FU-1` — "one of eleven enforced" — is closed.

## Where to actually look

The mechanical claims hold. Spend your round on judgement, not re-counting.

1. **`AC2`'s other direction is the load-bearing half.** The eleven-way proof
   shows each tag *rejects* `RATIONAL`. It would pass identically if `uints()`
   rejected `RATIONAL` **universally**, silently undoing `SPEC-007`.
   `an_interpretation_tag_still_accepts_a_rational` is the only thing standing
   between us and that. **Mutate it: make `uints()` reject `RATIONAL`
   unconditionally and confirm that test — and ideally only that test — dies.**
2. **`DEC-015` chose Option B with zero code change.** Read whether the narrowed
   contract (`src/ifd.rs:553-569`) actually states the property `DEC-014`'s
   oracle exemption depends on: *a tag named in `malformed_tags` is one whose
   value the reader genuinely does not have, never one it recovered.* If that
   sentence is true, the exemption is sound; if it is only nearly true, `DEC-014`
   inherits the gap. This is the coupling the finding could not have known about
   when it was raised.
3. **`AC3`'s fixture.** `orientation_malformed_on_both_ifds_is_costed_once` is
   the guard `SPEC-008/FU-2` never had. Does it die when the combined
   `malformed.push` is split into one per erroring read? Measured on `main`
   before this spec: that split compiles and leaves everything green.
4. **A twelfth tag.** Adding one to `is_structural_tag()` without a table row is
   the *strict* direction and is explicitly a non-goal — but confirm the table
   would not silently drift out of sync with the predicate, and say whether that
   matters.

## ⚠ One finding that is the ORCHESTRATOR's, disclosed so you do not file it against the build

The build could not self-report `tokens_total` and asked the orchestrator to run
`/cost`. **That is not the build's fault.** `HANDOFF-024` named the transcript
method five times and its build self-reported without difficulty; **this
handoff mentioned it zero times.** Same requirement, method dropped between two
handoffs written by the same author.

Recovered from the build's own transcript instead: **20,412,565** deduped by
`message.id` (196 usage objects → 105 distinct ids, 1.86×, 98.4 % cache-read),
priced per-component at **Sonnet** rates because `message.model` reads
`claude-sonnet-5` on all 196 — so `tier_map` is now **1 for 6**. At Opus rates
the same session computes $43.45 (5.0×); at the repo's flat rate, $134.72
(15.5×).

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you and pasted; sum across all six
   targets. **Observe CI green on the SHA you approve.**
2. **Watch a red-proof fail yourself** (§15 check 9) — the eleven-way, or at
   minimum the `AC2` mutation in point 1 above, which nobody has run.
3. **Fuzz** (§15 check 10) — build claims 13.5 M runs, seeds unchanged.
4. Every mutation: **assert it changed the file and compiled** before concluding.
   Stage your work first.
5. Handback with a real `tokens_total` **deduped by `message.id`** — read your
   own transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`, the session
   id is in the scratchpad path in your system prompt. Price per-component at the
   rates for the model `message.model` reports.
6. **Correct `handoff.to_agent`.** Do **not** run `handback-sync` or open the PR.
7. Findings `SB-N`/`FU-N` from 1, each with a §15 disposition. ⚠ A `spec:`
   disposition must **name an AC in that spec which would fail if the finding
   were left undone**.
8. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

### ✅ APPROVED at `55a25f8`

`feat/spec-009-pin-structure-class-membership`, pushed, not merged; `main` at
`e6cc561`. Two follow-ups, **zero ship-blockers**. `src/` is byte-identical
between `55a25f8` and this branch's `HEAD` (`4709c92`) — the delta is docs only
— so everything below was run against the exact code being approved.

### What I ran, and what it said

**Eleven gates + `just lint-ci`, all green, all summed across all six targets.**

| gate | result |
|---|---|
| `test` (corpus set, tools present) | **100 passed, 0 failed** — 49 lib + 0 `irr` + 9 `corpus_manifest` + 12 `ifd_reader` + 30 `metadata_oracle` + 0 doc |
| `fmt --check` | 0 |
| `clippy` local (0.1.97) | 0 |
| **`lint-ci` (0.1.98 — CI's floating stable, PATH-prefixed)** | 0 |
| `lint-no-allow` | 0 |
| `lint-red-proof` | 0 — control clean → injection rejected (101) → all five lints fired |
| `typecheck` | 0 |
| `build --release` | 0 |
| `msrv` (cargo 1.90.0, verified by `--version`) | 0 |
| `deny check licenses` | 0 |
| `deny --manifest-path fuzz/Cargo.toml` | 0 |
| `fuzz` (60s) | **10,695,240 runs, zero crashes**, seeds byte-unchanged |

⚠ **Zero `SKIP` lines in the whole run.** The tier-B halves genuinely executed:
`exiftool` and `dnglab` were on `PATH` and `$IRRADIANCE_CORPUS_DIR` resolved, so
`metadata_matches_exiftool_on_every_corpus_file`, `dnglab_scalars_agree_on_the_six_dng_files`
and both oracle red-proofs are real greens, not loud skips.

**CI observed green on the SHA I am approving** — run `33842552614`,
`headSha 55a25f856bc25f37cce9e41d19ff07ace1926f15`, **all 9 jobs** including
`rust / test` and `rust / lint policy red-proof`. Also green on `3b50964`
(`33842214431`, 9/9). The docs-only commit did **not** get a reduced job set.

`just validate` 17 artifacts valid · `just cost-audit` clean ·
`just decisions-audit --changed main` flags DEC-000/008/012/015 — DEC-008
(byte-alignment unpacking) untouched, DEC-012 is what the change enforces, and
the DEC-012/DEC-015 same-scope warning is **not** a contradiction: DEC-012's own
rows (`:48`, `:143`) already say the value is *dropped* **and** the tag recorded,
which is exactly Option B.

### Red-proofs I watched fail myself (§15 check 9)

Every mutation below was asserted applied by `git diff --numstat`, asserted to
compile, and the tree restored to `md5 56d43e6f2e05609e45e1d64c75059bb9` with
`git status --porcelain` empty after each. Work was staged first.

**1. The eleven-way, reproduced independently — eleven for eleven.**

```
control (unmutated)                          49 passed, 0 failed
TAG_NEW_SUBFILE_TYPE …                        1 failed  (10 tags)
  each: every_structural_tag_rejects_a_rational
TAG_SUB_IFDS                                  2 failed
  every_structural_tag_rejects_a_rational + subifds_rational_is_rejected
control again (restored)                     49 passed, 0 failed
```

**2. `AC2`'s other direction — the mutation nobody had run (handoff point 1).**
`uints()`'s gate reduced to `if entry.field_type == TYPE_RATIONAL {` — RATIONAL
rejected **universally**, silently undoing `SPEC-007`. Result: **3 failures**,
`an_interpretation_tag_still_accepts_a_rational` among them.

⚠ **This corrects the handoff's own framing, in the build's favour.** The handoff
says that test "is the only thing standing between us and that." It is not — the
pre-existing `rational_default_crop_size_reads_or_costs_the_field` and
`rational_denominator_is_actually_divided` die too. The widening is guarded at
three points, not one.

**3. `AC3`'s fixture (handoff point 3).** I ran the split **twice**, because the
naive split is stronger than the one design measured green on `main`:

- naive (guard dropped entirely) → 3 failures, including `orientation_malformed_on_both_ifds_is_costed_once`;
- **faithful** (the `orientation.is_none()` guard kept, split only across the two
  reads — the variant that compiled green on `main` at `024eaae`) →
  **exactly 1 failure**, and the assertion is verbatim `SPEC-007/FU-1`'s defect:

```
assertion `left == right` failed
  left: [274, 274]
 right: [274]
```

`AC3`'s fixture is genuinely load-bearing, and it is the only thing that catches
that split.

**4. The stated hazard, end to end — a probe the ACs do not cover directly.**
Every AC tests `uints()` in isolation; none tests the spec's actual headline
hazard through `sensor()`. I wrote a throwaway probe (`Compression` as
`RATIONAL 2/2`, then `Container::parse(..).sensor()`) and ran it **with a
negative control**:

- on `55a25f8`: `Err(UnexpectedFieldType { tag: 259, field_type: 5 })` — fatal, correct;
- with `TAG_COMPRESSION` dropped from `is_structural_tag()`:
  `Ok(Sensor { .. compression: Uncompressed, .. malformed_tags: [] })`.

That is the hazard verbatim — a `RATIONAL 2/2` `Compression` reading as
uncompressed, `malformed_tags` **empty**, the file parsing cleanly, and
`require_uncompressed()` waving it through to the unpack. The probe was removed;
`every_structural_tag_rejects_a_rational` is what stands between STAGE-002 and
that today, and it does stand.

### Handoff point 2 — is `DEC-014`'s exemption actually sound?

**Yes, and the narrowed contract states the right property.** I traced this
rather than taking it on report.

The exemption is narrower than the spec's prose implies: `malformed_tags` is
consulted at exactly one place, `compare_optional`'s `ToolValue::Unreadable(_)`
arm (`tests/support/tools.rs:521`). It never suppresses a comparison against a
tool value we *have* — it only rules "we agree" when the tool **also** could not
read the tag. So the soundness condition is precisely the sentence at
`src/ifd.rs:566-569`: *a tag named here is one whose value this reader genuinely
does not have.*

All three sites that can push to `malformed` satisfy it, and each does so
structurally, in the same branch that drops the value:

| site | pushes | returns |
|---|---|---|
| `array::<N>()` `:982` | wrong length | `Ok(None)` |
| `cost_the_field()` `:1123` | any `Err` | `None` |
| `sensor()`'s `Orientation` `:1178-1182` | only under `orientation.is_none()` | — |

Six of the seven compared fields have exactly one source, so for them "value
lost" and "read errored" are the same event and the property is unbreakable
without restructuring. `Orientation` is the only two-source field, it is the only
one where the two can diverge, and it is the one `DEC-015` decides and
`a_malformed_sensor_orientation_with_a_good_ifd0_value_is_silently_dropped` pins.
**`DEC-014` does not inherit a gap.**

### Handoff point 4 — the twelfth tag, measured (this is `FU-1`)

Confirmed: the table **can** drift out of sync with the predicate, and the drift
is partial in a way worth naming. Measured on the **full 100-test suite**, not
just `--lib`, so the live tier-B oracle against `exiftool`/`dnglab` on all seven
corpus files had its chance to object:

| tag added to `is_structural_tag()` | full suite |
|---|---|
| `TAG_BLACK_LEVEL` | **red** — `an_interpretation_tag_still_accepts_a_rational` |
| `TAG_DEFAULT_CROP_SIZE` | **red** — `rational_default_crop_size_reads_or_costs_the_field`, `rational_denominator_is_actually_divided` |
| **`TAG_DEFAULT_CROP_ORIGIN`** | **100 passed, 0 failed** |
| `TAG_ACTIVE_AREA` / `TAG_ORIENTATION` | 100 passed, 0 failed |

`ActiveArea` and `Orientation` are silent-and-fine: DNG does not permit either as
`RATIONAL`, so a row for them would pin nothing. `DefaultCropOrigin` is
different. `is_structural_tag()`'s **own doc comment** (`src/ifd.rs:185-187`)
names exactly three tags as the reason the widening exists — "*`BlackLevel`,
`DefaultCropOrigin`, `DefaultCropSize` and friends may legally be RATIONAL in
DNG*". Two of those three are pinned. The middle one is not, and no corpus file
encodes it as `RATIONAL`, so the live oracle cannot catch it either.

**Does it matter? Not much, and not for this spec.** It is the strict direction,
which `SPEC-009`'s Non-Goals exclude by name, and it fails **closed** — a typed
`Error::UnexpectedFieldType`, loud, never a wrong image. `AC2` asked for "a
paired interpretation tag" (singular) and the build delivered exactly that. This
is a finding against the spec's scoping, not against the build.

What it *does* touch is a claim. `SPEC-009`'s Context argues the
`SPEC-007`→`008`→`009` recursion terminates "because of the **shape of the
fix**: one table-driven test over all eleven memberships has no 'one point' left
to be narrow at." That is true of the **rejection** direction and not of the
**acceptance** direction added in the same breath, which is pinned at one point
out of three — the same one-point-guard shape, one level up.

### Findings

| id | finding | disposition |
|---|---|---|
| `FU-1` | `an_interpretation_tag_still_accepts_a_rational` pins the RATIONAL-acceptance direction at **one** tag; adding `TAG_DEFAULT_CROP_ORIGIN` to `is_structural_tag()` leaves the full 100-test suite green, though the predicate's own doc comment (`src/ifd.rs:185-187`) names it as one of exactly three legally-RATIONAL DNG tags. Strict direction, fails closed, explicitly a `SPEC-009` Non-Goal. | `signal: measurement-over-generalised` — evidence, **instance 6**, and the closest match yet to instance 3 ("mutated ONE structural tag … asserted 'the boundary test has teeth'"). ⚠ **Not `closed`:** I measured that no corpus file encodes `DefaultCropOrigin` as `RATIONAL`, so there is no test that would fail — a close here would be the "someone remembers" kind AGENTS.md rejects. ⚠ **Not `spec:`:** no existing spec has an AC that fails if this is left undone, which is return criterion 7's bar. If ship prefers, the concrete fix is cheap enough to take as `fixed`: make that test table-driven over the three tags the doc comment names, mirroring `AC1`. **Ship decides.** |
| `FU-2` | `SPEC-009` carries **two** `cycle: build` sessions — the build's null-numeric one asking the orchestrator to fill it in, and the orchestrator's recovered `20,412,565` appended **beside** it rather than into it. Dollars are not double-counted (nulls sum as 0) but `totals.session_count: 3` counts one cycle twice, and a shipped spec keeps a null-numeric metered session — the loophole AGENTS.md §4 exists to close. | `signal: cost-field-has-two-owners` — evidence, **N=2 → N=3**, and from a third direction: the two prior instances were the field going *empty* because each side assumed the other had it; this one is both sides writing and the record double-counting. Trivially `fixed` at ship by deleting the null entry (its provenance note is duplicated verbatim in the filled one) and setting `session_count`. |

### §15 checklist

1. **ACs met and tested?** Yes, all seven, each re-measured above. 2. **Failing
tests pass?** Yes — all five named tests exist exactly once across the six
targets (`-- --list`, `grep -c`, no zero-match). 3. **Decision drift?** None;
DEC-012/DEC-015 confirmed consistent. 4. **Constraint violations?** None —
`oracle-must-be-shown-red` satisfied by red-proofs I watched, `library-not-application`
by an unchanged public API, `test-before-implementation` trivially (test-only
change). 5. **Non-trivial choices carry a `DEC-*`?** Yes, `DEC-015`, and it
argues Option B from `DEC-014`'s stakes rather than from the doc comment's
letter. 6. **Reflection mailed in?** No — it is substantive, and Q1 surfaces the
`AC5`/`## Failing Tests` naming disagreement rather than silently picking one.
That was the right call and the right resolution: `AC5`'s own paragraph is the
more specific text. 7. **`cost.sessions` populated?** Yes — see `FU-2`. 8.
**Behavioral surface exercised?** No AC claims runtime behavior; all seven are
test-shaped, and I exercised the one *behavioral* claim the ACs leave implicit
(the `require_uncompressed()` hazard) myself, with a control. 10. **Fuzz ran?**
Yes, 10.7M runs. 11. **Provenance row honest?** Yes — class 1, specification,
and it states "**zero** new tags and **zero** src/ read logic", which matches the
diff exactly. 12. **New dependency?** None; `Cargo.toml` and `Cargo.lock` are
byte-identical to `main`.

### Verdict

**✅ APPROVED at `55a25f8`.** Two follow-ups, both routed to existing signals,
neither ship-blocking, neither a defect in the build.

The handoff called this "a well-made thing with a gap nobody thought to look
for," and the honest answer after a round of looking is that the gap is in the
**spec's** scoping, not the build's execution: the acceptance direction is pinned
at one point out of three while the rejection direction is pinned at eleven out
of eleven. The build did what `AC2` asked, and `AC2` asked for one. Everything
the spec set out to close is closed, and I watched the hazard itself — a
`RATIONAL 2/2` `Compression` reading as `Uncompressed` with an empty
`malformed_tags` — appear and disappear under my own hands.

**Not done, per return criterion 6:** `handback-sync` not run, PR not opened,
nothing committed, nothing merged.
