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
  id: HANDOFF-020
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5    # what ACTUALLY ran (SPEC-007/FU-6): Opus 5, 1M-context variant
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-21
  status: completed                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-008

project:
  id: PROJ-001
  stage: STAGE-001
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
  status: completed                     # completed | blocked | rejected
  tokens_total: 9600000               # REAL combined count — what cost-audit reads
  estimated_usd: 24.40              # tokens_total × your rate, or your harness's number
  duration_minutes: 35
  branch: feat/spec-008-pin-structure-class
  pr: null
  completed_at: 2026-08-21               # YYYY-MM-DD
  notes: "APPROVED at 282e6fc — 6 follow-ups, 0 ship-blockers. tokens_total rounded up from a measured 8,882,206 floor (deduped by message.id, 119 usage objects / 70 distinct ids); see the Cost self-report. Did NOT run handback-sync per the return criteria."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-020: Pin the Structure class with tests that fail when it is softened

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-008` for the **verify** cycle. Independent
session.

This spec exists because SPEC-007's verify proved a boundary was guarded at **one
point in five**. Its whole value is that the guard is now real — so the scrutiny
that matters is whether the new tests fail for the **right reasons**, not whether
they fail.

## Context the Receiving Agent Needs

### Already reconciled — and this time across the whole class, not one point

Ten gates green, 66 tests, `main` untouched, one commit ahead, tree clean.

**All four structural mutants mutation-tested by the orchestrator**, each asserted
to compile *and* apply before concluding:

| structural tag → tolerant | before SPEC-008 | now |
|---|---|---|
| `Compression` | 0 failures | **1 — killed** |
| `StripOffsets` | 0 failures | **1 — killed** |
| `StripByteCounts` | 0 failures | **1 — killed** |
| `BitsPerSample` | 0 failures | **1 — killed** |

`RowsPerStrip` was already covered. The gap SPEC-007's verify found is closed.

Also verified in code: `is_structural_tag()` (`src/ifd.rs:188`) is a real per-tag
list gating `TYPE_RATIONAL` at `:841`, and `sensor()`'s `Orientation` now records
at most once and **only when no valid value was found anywhere** — which is FU-1
and FU-2 together.

### What deserves scrutiny — the tests, not the mutants

1. **Do the new tests fail for the RIGHT reason?** Each kills its mutant, but a
   test can be red for an unrelated cause. Check the assertion actually names the
   tag and error it claims to (`UnexpectedFieldType { tag, field_type }`), the way
   the `RowsPerStrip` original does.
2. **Is `is_structural_tag()`'s list right?** Compare it tag-for-tag against
   `DEC-012`'s amended Structure row. A tag missing from the list silently regains
   the global RATIONAL looseness; a tag wrongly added rejects a legal encoding.
   ⚠ Confirm `SubIFDs` (330) is in it — FU-4's whole point.
3. **The `Orientation` logic has four combinations** (`ifd0` ok/err ×
   `sensor` ok/err). The fix reads correctly for the two the fixtures cover.
   Are the other two right — in particular a *good* `ifd0` value with an
   *erroring* sensor read?
4. **FU-5 — is the division actually pinned?** The complaint was that every
   well-formed RATIONAL fixture used denominator `1`, so a mutant pushing the
   numerator and ignoring the quotient survived. Verify a denominator ≠ 1 fixture
   now exists **and** kills that mutant.
5. **Two "bonus" mutants** were claimed beyond the required four. What were they,
   and do they cover anything the four do not?
6. **`docs/provenance-ledger.md` extended in place, "no new DEC needed."** Agree?

### One process note worth recording

`HANDOFF-019` was the first handoff written with `to_agent: null` per
`SPEC-007/FU-6`, and the build filled it with what **actually ran**
(`claude-sonnet-5`). The fix worked on first use — no prediction, no mismatch.

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

You do **not** need to re-run the four mutants — the orchestrator did, and pasted
the numbers. Spend the cycle on whether the tests are *honest*: right reason, right
assertion, complete class.

⚠ Traps that have each produced wrong answers here: zero-match `cargo test <name>`
**exits 0**; **sum across targets**; and **assert a mutation compiled and applied**
before concluding (five failures on this project).

Label findings `SB-N` / `FU-N` for **this** spec. If APPROVED, set
`task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- Redrawing `DEC-012`'s Structure/Interpretation line.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.

## Return Criteria — how to hand back

1. Paste the ten gates and whatever you re-ran.
2. Fill `## Completion` and `handback:`; set `to_agent` to what **actually ran**;
   `tokens_total` deduplicated by `message.id`, captured **before** the session
   closes.
3. `handoff.status: completed`; spec `task.cycle: verify` (or `ship`).
4. Commit on `feat/spec-008-pin-structure-class`; do not merge. Do not run
   `handback-sync`.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-008-pin-structure-class` @ `282e6fc` (local; not
  pushed, no PR, not merged — `main` is at `261dc48` and does **not** contain
  `0c41eee`, confirmed with `git branch --contains`).
- **Completed at:** 2026-08-21
- **All acceptance criteria met?** Yes — all six.
- **For `verify`:** ✅ **APPROVED at `282e6fc`** — **6 follow-ups, 0
  ship-blockers.**

The fix is real and the four required mutants are dead. What this cycle found is
that **the same "one point is not a boundary" recursion has moved one level in**:
the *per-tag list* that replaces the global widening is itself pinned at one tag
out of eleven, and the *"costed at most once"* rule is pinned only on the path
where the second read does not exist. Both are the spec's own thesis applied to
the spec's own fix. Neither ships bad data at `282e6fc` — the code is correct;
it is the guard against softening it again that is one point wide.

### What I ran (and what I did not)

**Not re-run, per the handoff** — reconciled by the orchestrator and taken as
given: the ten gates, `main` untouched, and the four structural mutants
(`Compression`, `StripOffsets`, `StripByteCounts`, `BitsPerSample`), each
asserted to compile and apply.

**Run by me, this cycle:**

| what | result |
|---|---|
| `cargo test --all-features` (baseline) | **66 passed**, 0 failed — summed across five `Running` lines: 45 lib + 0 `irr` + 9 `corpus_manifest` + 12 `ifd_reader` + 0 doc |
| Existence of all 8 named tests + the `RowsPerStrip` original, per-target `-- --list`, summed into one file, `grep -c` per name | **each matched exactly 1**, none zero-match (`named-tests-can-pass-vacuously`) |
| **M2** — `is_structural_tag()` reduced to `TAG_SUB_IFDS` alone (ten tags deleted) | compiled, diff verified applied, **66/66 still green** → finding `FU-1` |
| **M3** — the single combined `malformed.push` split into one push per erroring read (re-introduces `[274, 274]`) | compiled, diff verified applied, **66/66 still green** → finding `FU-2` |
| **M4** — RATIONAL success arm `out.push(value)` → `out.push(numerator)` | compiled, **1 failure**, `rational_denominator_is_actually_divided`, `left: width 16736 / right: 8368` — right test, right reason |
| **M5** — only `TAG_SUB_IFDS` removed from the list | compiled, **1 failure**, `subifds_rational_is_rejected` — right test, right reason |
| tree restored after every mutation | `git status --porcelain` empty, `git diff HEAD` empty, suite back to **66** |
| `just decisions-audit --changed main` | flags `DEC-008` + `DEC-012` on `src/ifd.rs`; `DEC-008` (byte-alignment unpacking) untouched, `DEC-012` is what the change enforces |
| `just validate` | ✓ 8 artifacts, valid front-matter |

### The six questions, answered

**1. Do the new tests fail for the RIGHT reason?** — **Yes.** Every one of the
four names both halves it claims: `Err(Error::UnexpectedFieldType { tag:
TAG_<X>, field_type: 250 })`, the same shape as the `RowsPerStrip` original
(`src/ifd.rs:1787`). Three properties make that honest rather than decorative:
each fixture `entries.retain()`s the well-formed tag away **before** pushing the
malformed one, so `Ifd::entry()`'s first-match lookup (`src/ifd.rs:401`) cannot
find a good entry instead; the tag consts are `const` paths, so they are
equality patterns, not catch-all bindings; and none of the four tags is read by
`is_sensor_ifd`, so a failure cannot be `NoSensorIfdCandidatesMalformed` wearing
the wrong name. Under the intended softening mutant `sensor()` returns `Ok`, the
`matches!` is `false`, and the assertion fails — for the reason it advertises.
`orientation_costed_once_when_plane_is_ifd0` additionally asserts
`sensor_candidates() == vec![0]`, which pins the *precondition* the test is
about. See `FU-5` for the one sibling that does not.

**2. Is `is_structural_tag()`'s list tag-for-tag correct?** — **`SubIFDs` (330)
is in it** (`src/ifd.rs:201`), and `subifds_rational_is_rejected` is the only
test that dies when it is removed (M5) — FU-4's point is closed. Nothing in the
list is wrongly added in the sense that matters: TIFF 6.0 / DNG define all
eleven as BYTE/SHORT/LONG/IFD, so **no legal encoding is rejected**. Two tags
are in the list that `DEC-012`'s amended Structure row does not name —
`BitsPerSample` (258) and `RowsPerStrip` (278) — see `FU-4`. And the *membership
of ten of the eleven is enforced by nothing* — see `FU-1`.

**3. The four `Orientation` combinations.** — The logic is correct in all four;
two of them are untested, and one of those two is a behaviour question rather
than a coverage gap. Full matrix, for a plane that is **not** IFD0 (so both
reads exist):

| `ifd0` read | sensor read | → `orientation` | → `malformed_tags` | pinned by |
|---|---|---|---|---|
| `Ok(Some v)` | `Ok(None)` | `Some(v)` | `[]` | `orientation_comes_from_ifd0_when_the_plane_is_a_subifd` |
| `Err` | `Ok(Some v)` | `Some(v)` | `[]` | `wellformed_orientation_is_not_recorded_malformed` (FU-2) |
| `Err` | `Ok(None)` | `None` | `[274]` | `malformed_orientation_on_ifd0_keeps_the_plane` |
| `Err` | `Err` | `None` | `[274]` **once** | **nothing** → `FU-2` |
| **`Ok(Some v)`** | **`Err`** | `Some(v)` | **`[]` — swallowed** | **nothing** → `FU-3` |

Plus the plane-is-IFD0 path (`sensor_read` is `None` by construction), pinned by
`orientation_costed_once_when_plane_is_ifd0`. The handoff's specific worry — a
**good `ifd0` value with an erroring sensor read** — reads correctly as far as
the *value* goes (`Some(v)`, IFD0 wins, the plane survives), but the malformed
sensor-IFD entry is dropped without a record; that is `FU-3`.
`plane_is_ifd0 = ifd_index == 0` is sound: `ifd0()` is `self.ifds.first()`
(`src/ifd.rs:688`) and `ifd_index` indexes the same `Vec`.

**4. FU-5 — is the division actually pinned?** — **Yes, and I killed the mutant
myself.** `rational_denominator_is_actually_divided` (`src/ifd.rs:1989`) is the
first fixture with a denominator ≠ 1 (`16736/2`, asserted as `8368`); the base
`sensor()` fixture carries no `DefaultCropSize`, so the RATIONAL entry is the
only one read. M4 (`out.push(value)` → `out.push(numerator)`) produced **exactly
one failure — that test** — with `left: DefaultCropSize { width: 16736, .. }`
against `right: { width: 8368, .. }`. Precise pin, right reason, no collateral.

**5. The "bonus" mutants.** — The two labelled bonus are (a) **delete the
`is_structural_tag` guard block** in `uints()`, reverting to `SPEC-007`'s global
widening → kills `subifds_rational_is_rejected`; (b) **`out.push(numerator)`** →
kills `rational_denominator_is_actually_divided`. I reproduced both
independently (M5, M4), each killing exactly one test. `HANDOFF-019`'s handback
actually enumerates a **third** un-counted mutant — reverting the whole
`Orientation` block to the pre-fix two-`cost_the_field` version, which turned
*both* orientation tests red — so its arithmetic ("six = the required four plus
two bonus") is off by one against its own list of seven. Do they cover anything
the four do not? **Yes, and necessarily so:** the four structural fixtures plant
field type `250`, which the *general* type gate rejects — they never touch the
per-tag RATIONAL gate, the `Orientation` accounting, or the division. Each bonus
mutant covers exactly one of the three code changes the four cannot reach. What
none of the seven covers is `FU-1` and `FU-2`.

**6. `docs/provenance-ledger.md` extended in place, "no new DEC needed."** —
**Agree**, on both halves. Extending the existing `src/ifd.rs` row is the
precedent `SPEC-004` and `SPEC-007` both set on this same row, and the class is
right: no new dependency, no new algorithm, no implementation consulted — a
per-tag gate and a control-flow correction to an existing spec-derived rule.
`just decisions-audit --changed main` agrees (`DEC-008` untouched, `DEC-012`
enforced). The one written artefact that does need an edit is `DEC-012`'s
**table**, which is a doc amendment to match code that already existed, not a
new decision — `FU-4`.

### Findings

**0 ship-blockers.** Six follow-ups.

**`FU-1` — the per-tag list is pinned at one tag out of eleven.**
`src/ifd.rs:188-203` (the list), `:841` (the gate).
Measured: deleting all ten tags except `TAG_SUB_IFDS` leaves **66/66 green**
(M2); deleting only `TAG_SUB_IFDS` kills exactly one test (M5). So `SubIFDs` is
the only membership any test enforces. `Compression`, `StripOffsets`,
`StripByteCounts`, `BitsPerSample`, `RowsPerStrip`, `ImageWidth`, `ImageLength`,
`NewSubfileType`, `Photometric` and `SamplesPerPixel` can each silently regain
`SPEC-007`'s global RATIONAL looseness with nothing going red. The four new
structural fixtures cannot catch it — they plant field type `250`, which the
general gate rejects two lines further down; the per-tag gate is never on their
path. The consequence is the one the spec's own Context names: a `Compression`
encoded `RATIONAL 2/2` would read as `1`, `require_uncompressed()` would pass,
and STAGE-002 would read JPEG bytes as raw samples — and `StripByteCounts` as
`RATIONAL 28/2` would silently read `[14]` (the build measured that shape itself,
pre-fix). Not ship-blocking: the behaviour at `282e6fc` is correct, and AC 2
required the change to be made and written down, not a test per tag.
**Fix:** one table-driven test over all eleven tags asserting `uints()` returns
`UnexpectedFieldType` for a RATIONAL entry, and a paired interpretation tag
asserting it still reads. This is `SPEC-008`'s own thesis at N=4 and should be
the next spec.

**`FU-2` — "costed at most once" is untested on the only path where it can
fail.** `src/ifd.rs:1161-1178`.
Measured: splitting the single combined `if` into one push per erroring read —
which reproduces the exact `[274, 274]` defect `SPEC-007/FU-1` existed to kill,
on a two-IFD file where both `Orientation` entries are malformed — leaves
**66/66 green** (M3). `orientation_costed_once_when_plane_is_ifd0` cannot catch
it, because its `sensor_read` is `None` by construction. The fix is proven for
the shape it was written for and unproven for the shape where two reads actually
exist.
**Fix:** a fixture with a malformed `Orientation` on **both** IFD0 and the
SubIFD plane, asserting `malformed_tags == vec![TAG_ORIENTATION]` — one element,
not two.

**`FU-3` — a good IFD0 value with an erroring sensor read swallows the malformed
tag silently.** `src/ifd.rs:1168-1178`; contract at `src/ifd.rs:553-560`.
When IFD0's `Orientation` is well-formed and the sensor IFD's own entry is
malformed, `orientation` is `Some(v)` (correct — IFD0 wins) and `malformed_tags`
is **empty**. But `Sensor::malformed_tags` is documented as "Tags that are
**present but shaped wrong**, recorded rather than rejected", and that entry is
present, shaped wrong, and not recorded. Identical to pre-fix behaviour, so no
regression, and defensible under the new rule ("only when no valid value was
found anywhere") — but the rule and the field's documented contract now
disagree and nothing says which wins. Untested either way.
**Fix:** decide it (record-what-was-ignored, or value-found-means-silence),
write the chosen one beside `malformed_tags`, and pin it.

**`FU-4` — "exactly `DEC-012`'s amended Structure row" is not exact, in two
places.** `src/ifd.rs:182` (doc comment) and `docs/provenance-ledger.md:39`.
`DEC-012`'s amended row names: header, entry tables, chain `next`, `SubIFDs`,
`StripOffsets`, `StripByteCounts`, `ImageWidth`/`Length`, `Compression`,
`NewSubfileType`, `Photometric`, `SamplesPerPixel`. `is_structural_tag()`
additionally contains **`BitsPerSample` (258)** and **`RowsPerStrip` (278)**.
Both are defensible under the amendment's *principle* — "what exists is the
plane — its presence, its **location** and its **extent**" — both were already
commented Structural in `sensor()` before this spec (`src/ifd.rs:1229`, `:1250`),
and AC 1 names `BitsPerSample` as structural itself, so **nothing was
reclassified here**. What is wrong is the claim of exactness, in the very
artefact AC 2 asks for ("either way it is written down").
**Fix:** either amend `DEC-012`'s table to name the two tags — a doc amendment
to match code that predates this spec, not a redraw — or drop "exactly" and say
which two come from the principle rather than the table. Prefer the former: a
predicate and a decision that disagree tag-for-tag is how the next reader
reintroduces `FU-1`.

**`FU-5` — `wellformed_orientation_is_not_recorded_malformed` does not pin its
own precondition.** `src/ifd.rs:2126-2152`.
The test only means anything if the sensor plane is the SubIFD, so that the two
`Orientation` reads are distinct — but unlike both of its neighbours
(`malformed_orientation_on_ifd0_keeps_the_plane`,
`orientation_costed_once_when_plane_is_ifd0`, which each assert
`sensor_candidates()`), it never asserts it. It holds today only because IFD0
carries `NewSubfileType = 1`. If that fixture ever drifted, the test would keep
passing while testing nothing.
**Fix:** one line — `assert_eq!(c.sensor_candidates(), vec![1]);`.

**`FU-6` — the spec's AC 3 calls a synthetic measurement a corpus measurement.**
`SPEC-008` AC 3 says `malformed_tags = [274, 274]` was "measured … on the Pentax
`.PEF`, a real corpus shape". The corpus expectation for `K3III.PEF` is
`orientation: Some(1)`, `malformed: &[]` (`tests/ifd_reader.rs:239`, `:242`) —
**byte-identical on `main`** — so the real file, whose `Orientation` is
well-formed, never produced `[274, 274]`; only the *shape* (`sensor_index: 0`)
is a corpus fact, which the spec's own Notes section states correctly. The build
did the right thing (a synthetic fixture with that shape). Raised because
"measured" is a load-bearing word in this repo and this is the second spec in a
row where a measurement claim needed narrowing.
**Fix:** narrow AC 3's wording at ship.

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 9,600,000 (rounded up from a measured floor) — a transcript sum **deduplicated by
  `message.id`** from this session's own JSONL
  (`~/.claude/projects/<path-slug>/5adc1d08-24a4-4e55-83c9-4b2763bd51e0.jsonl`,
  session id read off the scratchpad path the harness gave me). Measured just
  before committing: 119 `usage` objects across 70
  distinct message ids; deduped total (input + output + cache-read +
  cache-write) 8,882,206, of which 97.4% is
  cache-read, all cache-creation on the 1-hour ephemeral tier (5-minute tier 0,
  read from the nested `cache_creation` object). Rounded **up** to cover the
  turns spent writing this handback and committing — **a floor**, captured
  before the session closes.
- **Estimated USD:** 24.40 — computed per component at published Opus
  rates ($15/M input, $75/M output, $30/M cache-write-1h, $1.50/M cache-read) on
  the measured figure, then rounded with the token total. Not a harness-reported
  number; flagged so it is not mistaken for a metered one.
- **Duration (minutes):** ~35 (session start to this handback, wall clock; not
  separately instrumented).
- **Source of the number:** transcript `usage` objects, read directly — `/cost`
  is not available from inside a turn. Same methodology as `SPEC-004`'s verify
  and `SPEC-007`'s and `SPEC-008`'s builds.

### Drift and new artifacts

- **New decisions emitted:**
  - None. Concur with the build: this spec enforces `DEC-012` and does not
    redraw it. `FU-4` asks for an **amendment to `DEC-012`'s table** so it names
    the two tags the code has treated as structural since `SPEC-007` — a
    correction to a record, not a new decision.
- **Deviations from spec:**
  - None. All six acceptance criteria met; all eight named failing tests exist
    (confirmed per-target via `-- --list`, summed, `grep -c` — each exactly 1)
    and pass.
- **Follow-up work identified:**
  - `FU-1` and `FU-2` should become **one spec**, not six: both are "the fix is
    right and the guard on it is one point wide", which is the same defect this
    spec was written to close one level out. `FU-3` (decide it) and `FU-4`
    (amend `DEC-012`'s table) belong in the same spec's scope; `FU-5` and `FU-6`
    are one-line edits that can ride along.
  - Unchanged from the build's own note: the structural classification of
    `RowsPerStrip`/`Compression`/`StripOffsets`/`StripByteCounts`/
    `BitsPerSample` remains unverified by **real** data — every corpus file is
    single-strip, and the three compressed ones are rejected before unpacking.
    That is `DEC-012`'s accepted gap, not a new finding.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The handoff's own framing did the work: "spend the cycle on
   whether the tests are honest, not whether they fail" is what turned this from
   a re-run into two new findings. The one thing I had to derive rather than
   read was which mutants the *existing* fixtures already kill — that is what
   separates `FU-1`/`FU-2` (nothing kills them) from the seven the build
   reported (each kills something).

2. **Was there a constraint or decision that should have been listed but
   wasn't?**
   — `DEC-012`'s amended table should have listed `BitsPerSample` and
   `RowsPerStrip` (`FU-4`). It did not, so the build restated it as "exactly"
   the row while adding two tags — the right code, and a written claim that a
   tag-for-tag reader will find false. That is a record defect that predates
   this spec and got inherited by it.

3. **If you did this task again, what would you do differently?**
   — Run the *complement* mutation first. `M2` (delete ten of eleven list
   entries) took one run and produced the cycle's main finding; I reached it
   only after reading the four fixtures closely enough to notice that field type
   `250` never exercises the per-tag gate. "Which mutant does each new test
   actually kill, and what is left over?" is a cheaper first question than
   "is each test correct?", and it generalises: this is the third consecutive
   spec where the finding was **what the new tests do not reach**.
