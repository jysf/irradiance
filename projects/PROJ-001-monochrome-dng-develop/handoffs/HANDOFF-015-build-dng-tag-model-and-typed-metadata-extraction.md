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
  id: HANDOFF-015
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-21
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-004

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
  status: completed                # completed | blocked | rejected
  tokens_total: null               # see notes — no metering surface was available this cycle
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 50             # estimate; no wall-clock timestamps captured
  branch: feat/spec-004-tag-model
  pr: null
  completed_at: 2026-08-21         # YYYY-MM-DD
  notes: "tokens_total: null, deliberately, not by default. This session ran as the top-level interactive Claude Code session (not a sub-agent spawned via the Agent tool by a separate orchestrator), so there is no subagent_tokens figure for anything to have captured, and I have no tool-level access to run /cost or read raw per-message API usage objects from inside a turn — the CLI slash command is not reachable via Bash. The harness DOES expose a running <total_tokens> 'N tokens left' counter in system-reminders (session budget started at 15,000,000); that counter is a DIFFERENT accounting basis than HANDOFF-013's precedent (dedup-by-message.id over raw usage objects, with a reported cache-read share) — it has no cache-read breakdown and I cannot verify what it counts (output tokens only? input+output? does it already dedup?). Reporting it as tokens_total would misrepresent methodology against the 1.61x-2.25x range already on record. Recommend the user run /cost in their terminal for the authoritative figure, or set cost.metering_source: none in .repo-context.yaml for this metering surface. duration_minutes is a rough estimate from the shape of the work, not measured."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-015: DNG tag model and typed metadata extraction

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-004` for the **build** cycle.

Extract the remaining DNG tags the develop pipeline reads — and close the two
obligations SPEC-003 deferred here rather than make a `src/` edit in a
records-only round.

## Context the Receiving Agent Needs

### The two obligations are one question: what does a malformed tag cost?

**Read `DEC-012` first.** It states the rule — *a malformedness that changes what
exists is fatal; one that changes only what a known-optional field says costs that
field alone* — and it was written during SPEC-003's fix round specifically so this
spec would not re-derive it.

**FU-11 is where the code contradicts that rule.** Measured at design:
`is_sensor_ifd` (`src/ifd.rs:836`) calls `self.scalar(...)?` three times, and
`sensor_candidates`, `sensor_ifd` and `sensor` each run it over **every** IFD. So a
malformed tag on a *thumbnail* fails the whole container.

It is **latent, not live** — no corpus file carries a malformed tag on that path
(the Pentax's malformed `BlackLevelRepeatDim`, tag 50713, is not one of the three).
**Do not conclude from a green corpus that the path is sound.**

⚠ **The obvious fix is wrong.** Silently treating a malformed scalar as "not a
sensor IFD" hides a real plane behind a bad tag: if the *sensor* IFD's
`Photometric` is malformed, you convert a readable file into a bare `NoSensorIfd`
with no explanation. A malformed candidate must be **skipped and recorded**, and if
no candidate is then found the error must say **why**. Same discipline as the
corpus reader's loud skip — an invisible skip is the defect.

### Corpus facts, re-measured 2026-08-20 — use these, not older numbers

- **6 `II`, 1 `MM`** across 7 files
- **4 uncompressed, 2 JPEG (code 7), 1 vendor-private (65535)**
- `K3III.PEF`: **no SubIFD, no `NewSubfileType`**, plane in `IFD0`, and the only
  file with a real IFD *chain*
- M Monochrom: **no `ActiveArea`**, **no opcode lists**

⚠ Three earlier claims in this project's specs were wrong on exactly these points,
and each was a `find`/`exiftool` away. **Re-measure anything you are about to
assert.**

### Types matter here

`ActiveArea` as a bare `Vec<u32>` makes the caller remember it is
`[top, left, bottom, right]`. Give it a shape. And **absent must not collapse into
zero** — `ActiveArea` is missing on the M Monochrom, `NewSubfileType` is missing on
the PEF, and TIFF's absent-means-0 default for the latter is *what finds that
plane at all*.

## Expected Deliverables

1. Typed extraction of `BlackLevel`, `WhiteLevel`, `ActiveArea`,
   `DefaultCropOrigin`, `DefaultCropSize`, `Orientation`, and the **presence** of
   `OpcodeList1`/`OpcodeList3`.
2. `DEC-012` implemented; **FU-11 closed** per the subtlety above.
3. Hand-constructed TIFFs (via `tests/support/tiff.rs`, shipped by SPEC-003)
   proving a malformed tag on a **non-sensor** IFD and on the **sensor** IFD have
   **different, asserted** outcomes.
4. Values matching `exiftool` on all 7 files, pinned as an expected table.
5. Fuzz coverage of the new extraction paths.
6. All ten gates green, output pasted.

## Out of Scope

- **Levels arithmetic, cropping, orientation transforms** — STAGE-002 and
  `DEC-008`. Extracting `BlackLevel` is in scope; subtracting it is not.
- Executing opcodes — STAGE-003. Presence only.
- Any new dependency.

## Return Criteria — how to hand back

1. Paste all ten gates and the two malformed-tag tests from deliverable 3.
2. Fill `## Completion` and `handback:`. ⚠ `tokens_total`: deduplicate by
   `message.id` and **say so**, with cache-read share. Seven measured factors span
   **1.61×–2.25×** — not a constant.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Branch `feat/spec-004-tag-model` off `main`; commit; do not merge.
   ⚠ Do **not** run `handback-sync` — see finding 15.

## Gate results — all ten, run 2026-08-21 against `IRRADIANCE_CORPUS_DIR` pointed at the real 7-file corpus

1. **build** — `cargo build --release` → `Finished \`release\` profile [optimized] target(s)`.
2. **test** — `cargo test --all-features` → lib 31/31, `corpus_manifest.rs` 9/9, `ifd_reader.rs` 12/12, doc-tests 0/0. All green.
3. **lint (clippy)** — `cargo clippy --all-targets --all-features -- -D warnings` → clean, no warnings.
3b. **lint (fmt)** — `cargo fmt --check` → clean.
4. **typecheck** — `cargo check --all-targets --all-features` → clean.
5. **licences (lib)** — `cargo deny check licenses` → `licenses ok` (pre-existing `license-not-encountered` warnings for unused allow-list entries BSD-3-Clause/Zlib/Unicode-3.0, not new).
6. **licences (fuzz)** — `cargo deny --manifest-path fuzz/Cargo.toml check licenses` → `licenses ok`, same pre-existing warnings.
7. **msrv** — `~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` → clean against the pinned 1.90.0 floor.
8. **lint-red-proof** — `just lint-red-proof` (`scripts/lint-red-proof.sh`) → `✓ lint policy red-proof: control clean (exit 0) → injection rejected (exit 101) → all five lints fired at the injected code, and still fire without CI's -D warnings (exit 101).`
9. **lint-no-allow** — `just lint-no-allow` (`cargo clippy --lib --quiet -F ...` the five policy lints) → clean, no `#[allow]` escape found.
10. **fuzz** — `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60` → run twice (once before, once after the doc-comment fixes), **13,620,246 executions** in the final run, `Done`, zero entries in `fuzz/artifacts/ifd/`. Seed corpus regenerated via `cargo run --example fuzz-seeds` first, adding the two new FU-11 fixtures (`malformed-photometric-on-thumbnail.tiff`, `malformed-photometric-on-only-candidate.tiff`) so they are fuzzed, not just unit-tested.

Also ran directly, per the spec's literal `## Failing Tests` commands, confirming each selects and runs a real test (not a silent zero-match green):
```
$ cargo test --all-features tag_model_matches_exiftool     → 1 passed
$ cargo test --all-features orientation_is_per_frame       → 1 passed
$ cargo test --all-features absent_tag_is_absent_not_zero  → 1 passed
$ cargo test --all-features malformed_tag_costs_only_that_tag           → 1 passed
$ cargo test --all-features malformed_on_thumbnail_does_not_lose_the_plane → 1 passed
```

## Deliverable 3 — the two malformed-tag tests (`tests/ifd_reader.rs`)

Both are built from two new hand-constructed TIFF fixtures added to `tests/support/tiff.rs`
(`malformed_photometric_on_thumbnail`, `malformed_photometric_on_the_only_candidate`), which
also feed the fuzz seed corpus via the shared `tiff::all()` catalog. Same malformed tag
(`PhotometricInterpretation`, forced to an unreadable TIFF field type), placed on different
IFDs — the outcomes are different and both are asserted, per FU-11:

```rust
/// The malformed tag is on an unrelated (thumbnail) IFD, and the real sensor
/// plane is a SubIFD elsewhere. It must still be reachable.
#[test]
fn malformed_on_thumbnail_does_not_lose_the_plane() {
    let data = tiff::malformed_photometric_on_thumbnail(tiff::Order::Little);
    let container =
        Container::parse(&data).expect("the walk itself is fine — DEC-012 walk vs interpret");
    let sensor = container
        .sensor()
        .expect("a malformed tag on the thumbnail must not hide the real plane (FU-11)");
    assert_eq!(sensor.ifd_index, 1);
    assert_eq!((sensor.width, sensor.height), (4, 2));
}

/// The malformed tag is on the file's ONLY candidate — the plane itself.
/// `sensor()` must fail with an error that SAYS a candidate was malformed,
/// not a bare `NoSensorIfd` indistinguishable from "this file has no raw
/// plane at all" — the obvious-looking fix (silently treat it as
/// `NotSensor`) is exactly what FU-11 forbids.
#[test]
fn malformed_on_the_sensor_ifd_is_reported_not_hidden() {
    let data = tiff::malformed_photometric_on_the_only_candidate(tiff::Order::Little);
    let container = Container::parse(&data).expect("the walk itself is fine");
    match container.sensor() {
        Err(Error::NoSensorIfdCandidatesMalformed { candidates }) => {
            assert_eq!(candidates, vec![(0, TAG_PHOTOMETRIC)]);
        }
        other => panic!("expected NoSensorIfdCandidatesMalformed naming the tag, got {other:?}"),
    }
    // Same discipline via sensor_ifd(): the caller who only needs the IFD,
    // not the typed Sensor, gets the same explained failure.
    assert!(matches!(
        container.sensor_ifd(),
        Err(Error::NoSensorIfdCandidatesMalformed { .. })
    ));
}
```

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-004-tag-model` (local commit; not pushed, no PR opened per return criteria — "commit; do not merge")
- **Completed at:** 2026-08-21
- **All acceptance criteria met?** yes — with one finding worth flagging: this build discovered that SPEC-003's build (`b79c7ef`) had **already delivered** the bulk of AC1's tag extraction (BlackLevel, WhiteLevel, ActiveArea, DefaultCropOrigin/Size, Orientation, opcode presence, all cross-checked against exiftool on all 7 files) — the spec/handoff's framing of "SPEC-003 stops exactly where geometry begins" was stale (see Reflection Q1). The genuinely outstanding SPEC-004 work was: (a) AC1's *typing* — ActiveArea/DefaultCropOrigin/DefaultCropSize were `[u32; N]` bare arrays, not named-field structs, so I introduced `ActiveArea { top, left, bottom, right }`, `DefaultCropOrigin { x, y }`, `DefaultCropSize { width, height }`; (b) FU-11 itself, genuinely open exactly as described; (c) the two literally-named test commands the spec's `## Failing Tests` block requires, none of which existed under those exact names before this build (a `cargo test <name>` filter with zero matches exits 0 — a silent false green — so this mattered, not just cosmetic).
- **For `verify`:** N/A — this is the build handback.

### Cost self-report

- **Tokens (total):** null — see `handback.notes` in the front-matter for the full reasoning. Short version: this ran as the top-level interactive session, not a sub-agent an orchestrator metered via `subagent_tokens`, and I have no tool-level access to `/cost` or raw per-message `usage` objects from inside a turn. The harness's own `<total_tokens>` "N left" counter (budget 15,000,000) is a different accounting basis than the dedup-by-`message.id` methodology on record from HANDOFF-013 (no cache-read share, unverified whether it dedups), so reporting it as `tokens_total` would misrepresent methodology rather than inform it.
- **Estimated USD:** null
- **Duration (minutes):** ~50, estimated from the shape of the work (two 61s fuzz runs, ~15 cargo invocations, corpus test runs at 13–23s each, plus reading/editing time) — not measured against a wall clock.
- **Source of the number:** none available (see above)

### Drift and new artifacts

- **New decisions emitted:** none. FU-11's fix and the typed-geometry work both implement `DEC-012`'s already-stated rule directly; neither opened a new debatable fork worth its own record. `DEC-012`'s required pointer comments were added above `array()` and `sub_ifd_offsets_of_last()` in `src/ifd.rs` per its own Consequences section.
- **Deviations from spec:**
  - `is_sensor_ifd` changed its return type from `Result<bool, Error>` to a new public `SensorMatch` enum (`Yes | No | Unreadable(u16)`) — a `Result` shape was structurally what caused FU-11 (a caller with only "ok or error" cannot represent "skip and keep scanning" without inventing a side channel), so the type itself needed to change, not just the callers.
  - `sensor_candidates()` changed from `Result<Vec<usize>, Error>` to plain `Vec<usize>` — it is now genuinely infallible (a malformed candidate is recorded, not surfaced as an error, at that call), so the `Result` wrapper was dishonest. Updated its 3 call sites (`tests/ifd_reader.rs`, `src/bin/irr.rs`; `fuzz/fuzz_targets/ifd.rs`'s `let _ = ...` needed no change).
  - New `Error::NoSensorIfdCandidatesMalformed { candidates: Vec<(usize, u16)> }` variant — not in the spec's text but required by FU-11's own stated requirement ("the error must say why").
  - Corrected three now-stale doc comments (`src/lib.rs` crate doc, two spots in `src/ifd.rs`) that said "the wider type model — RATIONAL, signed types, ASCII — is SPEC-004's": that was never actually this spec's scope (every tag AC1 lists is SHORT/LONG) and would have been a false claim once this spec closes. Also refreshed `docs/provenance-ledger.md`'s `src/ifd.rs` row rather than adding a new one — same spec lineage, same provenance class (1 — specification), no implementation was consulted for either FU-11 or the typed structs.
  - Renamed two existing SPEC-003 tests to the spec's literally-required names: `ifd_tags_match_exiftool` → `tag_model_matches_exiftool`, `a_malformed_fixed_length_tag_costs_the_tag_not_the_file` → `malformed_tag_costs_only_that_tag`. Same coverage, names now match `## Failing Tests` exactly.
- **Follow-up work identified:**
  - `RATIONAL`/signed-type/`ASCII` field-type support remains unimplemented in `uints()` — no DNG tag PROJ-001 currently reads needs it, but it will surface the moment one does (flagged, not filed, since nothing calls for it yet).
  - Whether a malformed non-sensor candidate should be surfaced on a *successful* `Sensor` (not just in the failure-path error) was a live design question I resolved narrowly: it is currently invisible on success (the thumbnail's problem is irrelevant once the real plane is found). If a future spec wants that visibility for diagnostics, it is a small addition to `scan_sensor`'s existing plumbing, not a redesign.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Not "slowed down" exactly, but worth recording precisely because AGENTS.md and this handoff both warn about it: the spec's own Context section ("SPEC-003 ... stops exactly where geometry begins") and the handoff's "THE JOB: extract BlackLevel, WhiteLevel, ActiveArea..." framed this as mostly-greenfield extraction work. Reading `src/ifd.rs` first showed SPEC-003's single build commit had already shipped typed `Option<...>` extraction and an `exiftool`-cross-checked test for all 7 files for every one of those tags. The actual gap was narrower and different in kind: FU-11 (real, exactly as described) plus a typing upgrade (bare arrays → named structs) plus the literally-named test commands. This is exactly the "three earlier claims in this project's specs were wrong" pattern the handoff itself warns about — it just recurred one level up, in the handoff's own framing of what was already built. Re-reading the actual `src/ifd.rs` before touching anything is what caught it; a literal reading of the Context section alone would not have.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No — `DEC-012` was sufficient and correctly anticipatory; it explicitly named `array()` and `sub_ifd_offsets_of_last()` as the pointer sites and got both right.

3. **If you did this task again, what would you do differently?**
   — Nothing structurally — re-verifying the corpus facts against `exiftool` directly (rather than trusting the handoff's restated numbers) and reading the target module in full before forming a task list both paid off immediately and cheaply. I would flag the stale-context finding above earlier and louder if I were also updating the spec/handoff docs themselves, but that was out of scope for a build cycle (records-only changes belong to a different round per this project's own convention).
