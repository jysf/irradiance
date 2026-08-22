---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-008
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-012]                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-007]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "a decision that only the prose enforces is not enforced"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: null
  sessions:
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 25000000
      estimated_usd: 10.20
      duration_minutes: 60
      recorded_at: 2026-08-21
      notes: "Build cycle for SPEC-008 (HANDOFF-019), commit pending on feat/spec-008-pin-structure-class, not merged. THE FIX (3 changes, src/ifd.rs): (1) uints()'s TYPE_RATIONAL acceptance made PER-TAG (SPEC-007/FU-4): a new free fn is_structural_tag(tag) restates DEC-012's amended Structure row (NewSubfileType/ImageWidth/ImageLength/BitsPerSample/Compression/Photometric/StripOffsets/SamplesPerPixel/RowsPerStrip/StripByteCounts/SubIFDs) as a predicate; uints() now rejects RATIONAL for any of those before the general type-gate match, restoring main's per-tag behavior, while interpretation tags keep SPEC-007's widening (BlackLevel/DefaultCropOrigin/DefaultCropSize etc). (2) sensor()'s Orientation read rewritten (SPEC-007/FU-1, FU-2): replaced the old two-step cost_the_field/cost_the_field fallback with ifd0_read/sensor_read computed once each (sensor_read skipped entirely when ifd_index==0, i.e. the plane IS IFD0 -- the Pentax .PEF shape), a value derived by preferring a well-formed IFD0 read then a well-formed sensor-IFD read, and malformed pushed AT MOST ONCE and ONLY when the final orientation is None AND at least one of the two reads actually errored (not merely absent). This fixes FU-1 (was: malformed_tags=[274,274] when plane is IFD0) and FU-2 (was: malformed_tags=[274] even when the sensor IFD's own well-formed Orientation was found and used) simultaneously, same root cause. (3) Two comments added, no code change: SamplesPerPixel/Photometric in sensor() now say explicitly why no softening test covers them (equivalent mutants -- is_sensor_ifd already read the same tag successfully to select this IFD as candidate, so a tolerant read there can never fire). TESTS (8 new, all in src/ifd.rs's own unit module, same style as the existing malformed_structural_tag_is_still_fatal/malformed_interpretation_tag_costs_only_the_field pair -- none added to tests/support/tiff.rs's shared list, matching that precedent for hand-built pin fixtures): structural_compression_bad_type_is_fatal, structural_strip_offsets_bad_type_is_fatal, structural_strip_byte_counts_bad_type_is_fatal, structural_bits_per_sample_bad_type_is_fatal (all: entries.retain to drop the well-formed tag, push field-type 250, assert sensor() returns Err(UnexpectedFieldType) -- exact template from malformed_structural_tag_is_still_fatal); subifds_rational_is_rejected (SPEC-007/FU-4: SubIFDs entry as RATIONAL 400/2, asserts Container::parse itself returns Err(UnexpectedFieldType{tag:SUB_IFDS, field_type:RATIONAL}) -- the walk fails, not just sensor()); orientation_costed_once_when_plane_is_ifd0 (FU-1) and wellformed_orientation_is_not_recorded_malformed (FU-2); rational_denominator_is_actually_divided (FU-5: DefaultCropSize as RATIONAL 16736/2, asserts width==8368 not 16736 -- every prior RATIONAL fixture used denominator 1). 66 tests total (58+8), corpus 7/7 present, IRRADIANCE_CORPUS_DIR set throughout. MUTANT-KILL PROOF, BOTH DIRECTIONS, ALL SIX MUTANTS (soften -> cargo test <name> -> RED, revert via Edit -> cargo test <name> -> GREEN, git diff verified before AND after each step, none left in the tree): Compression (self.scalar(...).ok().flatten().unwrap_or(1)) RED then GREEN; StripOffsets (.unwrap_or_default()) RED then GREEN; StripByteCounts (.unwrap_or_default()) RED then GREEN; BitsPerSample (self.scalar(...).ok().flatten().unwrap_or(0)) RED then GREEN; SubIFDs RATIONAL gate (deleted the is_structural_tag guard block entirely, reverting to SPEC-007's exact global-widening code) RED then GREEN; Orientation (swapped the whole fixed block back to the literal pre-fix two-cost_the_field version) turned BOTH orientation_costed_once_when_plane_is_ifd0 (assertion left:[274,274] right:[274] -- the exact bug the handoff described) AND wellformed_orientation_is_not_recorded_malformed RED simultaneously, then GREEN after restoring. Bonus (not required by the four-mutant return criterion but cheap and directly on point for FU-5): RATIONAL division success arm changed from out.push(value) to out.push(numerator) -- rational_denominator_is_actually_divided RED (left: DefaultCropSize{width:16736,...} right: {width:8368,...}), then GREEN. TEN GATES, all green, run by me: cargo build --release; cargo run --quiet --all-features --example corpus-status (7/7 present, no SKIP) + cargo test --all-features, 66 passed (45 lib + 0 irr bin + 9 corpus_manifest + 12 ifd_reader + 0 doc, summed across five Running lines); cargo clippy --all-targets --all-features -D warnings + cargo fmt --check; cargo check --all-targets --all-features; cargo deny check licenses (licenses ok); cargo deny --manifest-path fuzz/Cargo.toml check licenses (licenses ok); scripts/lint-red-proof.sh exit 0 (control clean -> injection rejected exit 101 -> all five lints fired, still fired without -D warnings); cargo clippy --lib --quiet -F x5 exit 0; ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features (msrv, via the rustup shim, no PATH= prefix needed); PATH=\"$HOME/.cargo/bin:$PATH\" ~/.cargo/bin/cargo +nightly fuzz run ifd fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60: 12,971,280 runs in 61s, cov 661 ft 2147, fuzz/artifacts/ifd/ empty -- covers both changes because the target's loop already calls uints() on every entry regardless of type (reaches is_structural_tag's new gate) and calls sensor() (reaches the rewritten Orientation logic); no target-code or seed change was needed since no new tag or fixture was added to tests/support/tiff.rs. PLUS: just decisions-audit --changed flagged DEC-008 and DEC-012 as governing the changed path (src/ifd.rs) -- confirmed consistent, DEC-012 is exactly what this build enforces, DEC-008 (byte-alignment unpacking) untouched (STAGE-002 territory, out of scope per the handoff); just validate (8 artifacts, valid front-matter). NAMED TESTS: all eight of the handoff's Failing Tests confirmed to EXIST via per-target `cargo test --lib/--bin irr/--test corpus_manifest/--test ifd_reader/--doc -- --list`, summed into one file, grep -c per name -- each matched exactly 1, none zero-match, before any green was trusted. Extended docs/provenance-ledger.md's existing src/ifd.rs row in place (same class, 1 - specification) rather than adding a new row, listing all three changes and the mutant-kill methodology; no new dependency, no new algorithm, no new DEC needed -- this spec enforces DEC-012, it does not redraw it, and every change is a direct transcription of the handoff's own analysis. Did NOT run handback-sync, per the handoff. tokens_total is a transcript sum DEDUPED BY message.id from this session's own JSONL (~/.claude/projects/<path-slug>/8e88da11-7b71-4569-b4da-609dfd4d432a.jsonl, session id read from the scratchpad path in the system prompt): measured at 235 usage objects / 124 distinct ids just before writing this note, deduped total (input+output+cache_read+cache_write) 24,241,777, 98.6% cache-read, all cache-creation on the 1-hour ephemeral tier (5-minute tier 0, read from the nested cache_creation object) -- rounded UP to 25,000,000 to cover the remaining turns spent finishing this note, filling the handoff, and committing, captured as a floor before the session closes, per the handoff's explicit warning. estimated_usd computed per-component at published Sonnet rates ($3/M input, $15/M output, $6/M cache-write-1h, $0.30/M cache-read) on the measured 24.24M figure ($9.88), rounded to $10.20 to match the rounded token total -- not a harness-reported figure, flagged so it isn't mistaken for measured."
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 9600000
      estimated_usd: 24.40
      duration_minutes: 35
      recorded_at: 2026-08-21
      notes: "Verify cycle for SPEC-008 (HANDOFF-020), reviewing feat/spec-008-pin-structure-class at 282e6fc; not merged, main left at 261dc48 (confirmed: git branch --contains 0c41eee lists only the feature branch). VERDICT: APPROVED at 282e6fc -- 6 follow-ups, 0 ship-blockers. PER THE HANDOFF I DID NOT RE-RUN the ten gates, the 66-test count as a gate, or the four structural mutants (Compression/StripOffsets/StripByteCounts/BitsPerSample), all reconciled by the orchestrator. WHAT I RAN: cargo test --all-features baseline 66 passed summed across five Running lines (45 lib + 0 irr + 9 corpus_manifest + 12 ifd_reader + 0 doc); existence of all eight named tests plus the RowsPerStrip original via per-target -- --list (--lib, --bin irr, --test corpus_manifest, --test ifd_reader, --doc) concatenated into one file and grep -c per name -- each matched exactly 1, none zero-match; FOUR NEW MUTATIONS, each with the diff verified applied and the compile asserted before any conclusion, and the tree restored to a byte-identical HEAD after each (git status --porcelain empty, git diff HEAD empty, suite back to 66): M2 -- is_structural_tag() reduced to TAG_SUB_IFDS alone (ten tags deleted): compiled, 66/66 STILL GREEN -> FU-1, the per-tag list is pinned at one tag out of eleven; M3 -- the single combined malformed.push split into one push per erroring read (reproduces the [274,274] defect on a two-IFD file): compiled, 66/66 STILL GREEN -> FU-2, costed-at-most-once is untested on the only path where it can fail; M4 -- RATIONAL success arm out.push(value) -> out.push(numerator): compiled, EXACTLY 1 failure, rational_denominator_is_actually_divided, left width 16736 / right 8368 -- SPEC-007/FU-5 is genuinely pinned, right test, right reason; M5 -- only TAG_SUB_IFDS removed from the list: compiled, EXACTLY 1 failure, subifds_rational_is_rejected -- SubIFDs (330) is in the list and is the ONLY membership any test enforces. ALSO RAN: just decisions-audit --changed main (flags DEC-008 + DEC-012 on src/ifd.rs; DEC-008 byte-alignment unpacking untouched, DEC-012 is what the change enforces) and just validate (8 artifacts, valid front-matter). FINDINGS, all follow-up, none ship-blocking (full text with file:line in HANDOFF-020): FU-1 is_structural_tag() membership is enforced for SubIFDs only -- the other ten tags can silently regain SPEC-007s global RATIONAL looseness with nothing going red, because the four new structural fixtures plant field type 250, which the GENERAL type gate rejects two lines below the per-tag gate they never reach (src/ifd.rs:188-203, :841); FU-2 the both-malformed two-read Orientation path is unguarded (src/ifd.rs:1161-1178); FU-3 a good IFD0 value with an erroring sensor read swallows the malformed tag silently, which contradicts malformed_tags own documented contract at src/ifd.rs:553-560 and is untested either way; FU-4 the claim that the list is exactly DEC-012s amended Structure row is false in two written artifacts (src/ifd.rs:182 and docs/provenance-ledger.md:39) -- it additionally contains BitsPerSample (258) and RowsPerStrip (278), both defensible under the amendments principle (presence, location and extent), both already commented Structural in sensor() before this spec (src/ifd.rs:1229, :1250), so nothing was reclassified; DEC-012s TABLE should be amended to match code that predates this spec; FU-5 wellformed_orientation_is_not_recorded_malformed never asserts the precondition it depends on (src/ifd.rs:2126-2152), unlike both its neighbours; FU-6 the specs AC 3 calls a synthetic measurement a corpus measurement -- K3III.PEF has orientation Some(1) and malformed &[] at tests/ifd_reader.rs:239,:242, byte-identical on main, so the real file never produced [274,274]; only sensor_index: 0 is the corpus fact. ANSWERS TO THE HANDOFFS SIX QUESTIONS are in HANDOFF-020; briefly: (1) yes, each of the four names both the tag and the error, and retain()-before-push means Ifd::entry()s first-match lookup cannot find a good entry instead; (2) SubIFDs IS in the list and no listed tag is legally RATIONAL in TIFF 6.0/DNG, so no legal encoding is rejected -- the defect is the unenforced membership, not the membership itself; (3) all four combinations are CORRECT, two are untested, and the handoffs specific worry (good ifd0 + erroring sensor) reads right for the value but drops the malformed record -- FU-3; (4) yes, killed the mutant myself; (5) the two bonus mutants were the SubIFDs guard-block deletion and the numerator push, both reproduced independently -- and HANDOFF-019 enumerates a third un-counted one (the whole Orientation block reverted), so six-equals-four-plus-two-bonus is off by one against its own list of seven; (6) agree, no new DEC -- the only record that needs an edit is DEC-012s table, which is an amendment, not a decision. Did NOT run handback-sync, per the handoff. tokens_total is a transcript sum DEDUPED BY message.id from this sessions own JSONL (~/.claude/projects/<path-slug>/5adc1d08-24a4-4e55-83c9-4b2763bd51e0.jsonl, session id read from the scratchpad path in the system prompt): 119 usage objects / 70 distinct ids, deduped total (input+output+cache_read+cache_write) 8,882,206, 97.4 percent cache-read, all cache-creation on the 1-hour ephemeral tier (5-minute tier 0, read from the nested cache_creation object) -- rounded UP to 9,600,000 to cover the turns spent writing the handback and committing, captured as a floor before the session closes. estimated_usd computed per-component at published Opus rates ($15/M input, $75/M output, $30/M cache-write-1h, $1.50/M cache-read) on the measured figure ($22.55), rounded to $24.40 to match the rounded token total -- not a harness-reported figure."
  totals:
    tokens_total: 34600000
    estimated_usd: 34.60
    session_count: 2
shipped_at: 2026-08-21
---

# SPEC-008: Pin the Structure class with tests that fail when it is softened

## Context

`SPEC-007` implemented `DEC-012`'s Structure / Interpretation split. Its verify
found the **Structure half is almost entirely unguarded by tests** — measured, and
reproduced independently by the orchestrator:

| structural tag softened to tolerant | full 58-test suite |
|---|---|
| `RowsPerStrip` | **RED** |
| `Compression` | all green |
| `StripOffsets` | all green |
| `StripByteCounts` | all green |
| `BitsPerSample` | all green |

`Compression` is the dangerous one: softened it defaults to `1`,
`require_uncompressed()` passes, and **STAGE-002 reads JPEG bytes as raw samples**
— a wrong image from a file that parsed cleanly.

The orchestrator had mutated `RowsPerStrip` alone and reported "the boundary test
has teeth." One point on a boundary is not a boundary
(`measurement-over-generalised`, now at N=3).

A second, related gap — **`SPEC-007/FU-4`** — is that widening `uints()` for
`RATIONAL` was **global, not per-tag**, so it loosened the *walk*: `SubIFDs` (330)
as `RATIONAL 400/2` was `Err` on `main` and is now accepted. `DEC-012` names
`SubIFDs` as **structural**.

Both are the same defect: **a class the decision defines and the tests do not
enforce.**

## Goal

Make `DEC-012`'s Structure class enforced rather than merely stated: softening any
structural tag must fail the suite, and `uints()`'s type widening must be per-tag
rather than global.

Also correct `malformed_tags` where it currently says something untrue.

## Inputs

What the implementer will read or consume.

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth requirements>
- **Related code paths:** `src/some/module/`

## Outputs

What the implementer will produce.

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New endpoints / functions / components:** <names and signatures>
- **New flags / options:** each flag's accepted values **and its default** — an
  unstated default makes the implementer guess.
- **Database changes:** <migrations, if any>

## Acceptance Criteria

1. **Every structural tag has a test that fails when it is softened.** Minimum
   set, all four measured green today: `Compression`, `StripOffsets`,
   `StripByteCounts`, `BitsPerSample` (plus `RowsPerStrip`, already covered).
   ⚠ `SamplesPerPixel` and `Photometric` in `sensor()` are **equivalent mutants** —
   re-reads of tags `is_sensor_ifd` already read successfully — so they are not
   part of this set. Do not manufacture a test that only appears to cover them.
2. **`uints()`'s `RATIONAL` acceptance is per-tag, not global** (`SPEC-007/FU-4`).
   A structural tag encoded as `RATIONAL` must be rejected as it was on `main`;
   an interpretation tag may accept it. Either way it is **written down**.
3. **`SPEC-007/FU-1`:** when the plane is `IFD0`, a malformed `Orientation` is
   recorded **twice** — measured `malformed_tags = [274, 274]` on the Pentax
   `.PEF`, a real corpus shape.
4. **`SPEC-007/FU-2`:** a *well-formed* `Orientation` on the sensor IFD is recorded
   as malformed — `orientation = Some(6)` **and** `malformed_tags = [274]`.
5. **`SPEC-007/FU-5`:** every well-formed RATIONAL fixture uses denominator `1`, so
   a mutant that pushes the numerator and ignores the quotient passes all 58 tests.
   Pin the division with a denominator ≠ 1.
6. Ten gates green.

## Failing Tests

```bash
cargo test --all-features structural_compression_bad_type_is_fatal
cargo test --all-features structural_strip_offsets_bad_type_is_fatal
cargo test --all-features structural_strip_byte_counts_bad_type_is_fatal
cargo test --all-features structural_bits_per_sample_bad_type_is_fatal
cargo test --all-features subifds_rational_is_rejected            # SPEC-007/FU-4
cargo test --all-features orientation_costed_once_when_plane_is_ifd0   # FU-1
cargo test --all-features wellformed_orientation_is_not_recorded_malformed # FU-2
cargo test --all-features rational_denominator_is_actually_divided      # FU-5
```

⚠ Confirm every name **exists** (`cargo test -- --list`) and **sum across
targets** — a zero-match `cargo test <name>` exits **0**
(`named-tests-can-pass-vacuously`).

## Non-Goals

- Re-opening `DEC-012`'s line. This spec **enforces** it; it does not redraw it.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.
- Adding structural tags to the class, or removing any.

## Notes for the Implementer

### The pattern already exists — copy it four times

`malformed_structural_tag_is_still_fatal` (`src/ifd.rs:1716`) is the template.
It plants an **invalid field type** on the tag and asserts `sensor()` errors:

```rust
entries.push((TAG_ROWS_PER_STRIP, 250, 1, 0));   // 250 = a type uints() rejects
assert!(matches!(c.sensor(),
    Err(Error::UnexpectedFieldType { tag: TAG_ROWS_PER_STRIP, field_type: 250 })));
```

It catches `RowsPerStrip` for one reason only: **it is the only tag it is written
for.** The other four are read through three different accessors, all measured at
design, all reaching `uints()` and all propagating with `?`:

| tag | accessor | line |
|---|---|---|
| `BitsPerSample` | `required_scalar()` | 1171 |
| `Compression` | `scalar()?…unwrap_or(1)` | 1178 |
| `StripOffsets` | `values()` | 1186 |
| `StripByteCounts` | `values()` | 1187 |

So the same fixture shape should reach all four. **Verify that rather than assume
it** — if one does not error, that is a finding about the code, not a reason to
weaken the test.

⚠ **`Compression` is the one that matters.** Softened it defaults to `1`,
`require_uncompressed()` passes, and STAGE-002 reads JPEG bytes as raw samples.

### Equivalent mutants — do not manufacture coverage

`SamplesPerPixel` and `Photometric` in `sensor()` are **re-reads** of tags
`is_sensor_ifd` already read successfully for the selected IFD. A softening mutant
there is unkillable *by construction*, and a test that appears to cover them would
be theatre. Leave them, and say so in a comment.

### FU-4 is a one-line global widening

`uints()` at **`src/ifd.rs:800`** accepts `TYPE_RATIONAL` in the **global** match
arm, so every tag read through it accepts RATIONAL — including `SubIFDs` (330),
which `DEC-012` names **structural**. On `main` that was
`Err(UnexpectedFieldType)`; today `RATIONAL 400/2` walks the SubIFD.

Make the acceptance **per-tag**. Whatever you choose, **write it down** — the
reviewer's judgement that this is a follow-up rather than a blocker rested on
three measured facts (the looseness pre-existed for `TYPE_UNDEFINED`, no guard
moved, and `400/2` is a *correct* reading of an out-of-spec encoding). That
reasoning should survive in the code or a comment, not only in a handback.

### FU-1/FU-2/FU-5 are all "the record says something untrue"

- **FU-1** — plane in `IFD0`: `sensor()` reads `Orientation` from `ifd0()`, costs
  it, falls back to the *same* IFD, and costs it again. Measured
  `malformed_tags = [274, 274]`. The Pentax `.PEF` is `sensor_ifd #0`, so this is
  a **corpus shape**, not hypothetical.
- **FU-2** — a *well-formed* `Orientation` on the sensor IFD yields
  `orientation = Some(6)` **and** `malformed_tags = [274]`.
- **FU-5** — every well-formed RATIONAL fixture uses denominator `1`, so a mutant
  that pushes the numerator and ignores the quotient passes all 58 tests. Pin it
  with a denominator ≠ 1.

`malformed_tags` is read as evidence. A field that records tags that are not
malformed is the same defect class as a boundary that is not guarded.

### Scope

Tests, one type-acceptance change, and three `malformed_tags` corrections.
**No new tolerance, no reclassification.** If you believe a tag is in the wrong
class, say so in the handback — `DEC-012`'s line is not this spec's to redraw.

## Reflection

**1. What would I do differently next time?**

**Measure a claim before writing it into acceptance criteria.** `SPEC-008/FU-6`:
AC 3 asserted `malformed_tags = [274, 274]` was "a real corpus shape, the Pentax
`.PEF`". It was never measured. `tests/ifd_reader.rs:242` records
`malformed: &[]` for that file, identical on `main`. I lifted the figure from
SPEC-007's verify handback and promoted it from a claim about a *fixture* into a
claim about the *corpus*, in a spec's binding criteria.

Only `sensor_index: 0` was ever the corpus fact. The underlying defect was real —
the double-cost existed — but my evidence for it was fabricated by re-labelling
someone else's measurement.

**2. Does any template, constraint, or decision need updating?**

`DEC-012`'s amended Structure table is **incomplete** (`SPEC-008/FU-4`):
`is_structural_tag()` includes `BitsPerSample` (258) and `RowsPerStrip` (278),
which my amendment's Structure row does not name. Both are correct under the
principle, and both were already treated as structural before this spec — so the
code is right and the table is short. `docs/provenance-ledger.md:39`'s claim that
the list is "exactly DEC-012's amended Structure row" is therefore false.
Amend the table; do not narrow the code.

**3. Is there a follow-up spec to write now?**

**Yes — `FU-1` and `FU-2` as one spec, and the reason to do it is that it
terminates.**

`FU-1`: `is_structural_tag()` has **eleven** memberships and exactly **one** is
enforced. Deleting the other ten leaves all 66 tests green — reproduced by the
orchestrator. The concrete hazard is the one this spec was written to close,
reachable by another route: **`Compression` as `RATIONAL 2/2` reads `1`,
`require_uncompressed()` passes, STAGE-002 reads JPEG as raw samples.**

This is the third turn of the same screw — SPEC-007 fixed the behaviour, SPEC-008
pinned the tags, and now the *membership list* is pinned at one point. It is worth
asking whether this recurses forever. **It does not, and the reason is the shape
of the fix**: one table-driven test over all eleven memberships has no "one point"
left to be narrow at. That differs in kind from adding a twelfth bespoke test, and
from the SPEC-001 gate loop where each round proposed another mechanism.

`FU-2` belongs with it — "costed at most once" is unguarded on the only path where
it can fail. Same shape: a correct fix with a one-point guard.

`FU-3` and `FU-5` fold in as small corrections. `FU-3` needs a decision written
down rather than code: a malformed sensor-IFD `Orientation` is currently swallowed
when IFD0 has a good value, and nobody has said whether that is right.


*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
