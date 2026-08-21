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
  id: HANDOFF-008
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-20
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-002

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
  tokens_total: 9498150            # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 30
  branch: feat/spec-002-corpus-manifest-reader
  pr: null                         # not pushed — handoff said commit, do not merge
  completed_at: 2026-08-20         # YYYY-MM-DD
  notes: "Reader + visible skip shipped; 7/7 gates green. One dev-dep (toml, DEC-010) - [dependencies] still empty. SHA-256 written from FIPS 180-4 instead of a second dep, with a ledger row. tokens_total is a transcript sum DEDUPED BY message.id and so is NOT comparable to SPEC-001's figures, which double-count multi-block messages ~1.7x - measured, and filed on signal token-counts-not-comparable."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-008: Corpus manifest reader and skip-when-absent harness

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-002` for the **build** cycle.

Build the corpus manifest reader and make an absent tier-B file **visibly**
skipped. Storage and schema are already settled by `DEC-003`;
`tests/corpus/manifest.toml` ships seeded with 7 entries and **nothing reads
it** — its own header records that as a debt owned by this spec.

## Context the Receiving Agent Needs

### Two design-time measurements — transcribe, do not re-derive

**1. The `toml` dev-dependency.**

| config | crates | parses? |
|---|---|---|
| `toml = "0.8"` | 12 | yes |
| `default-features = false, features = ["parse"]` | **11** | yes |
| `default-features = false` | 6 | **NO** — `Value: FromStr` unsatisfied |

The last row is a trap: `cargo check` passes because nothing calls the API. **Use
`features = ["parse"]`.** With the dep present, `cargo +1.90.0 check
--all-targets` → 0 and `cargo deny check licenses` → licenses ok.

It is **dev-only**, so the library's zero-dependency claim is untouched. Your
`DEC-*` must say that explicitly — "irradiance has no dependencies" appears in the
README-facing story and must stay true as written.

**2. `eprintln!` inside a passing test is INVISIBLE.** Measured:
`cargo test` → 0 SKIP lines; `cargo test -- --nocapture` → 2.

So "skip loudly" **cannot** be satisfied inside the test harness. Recommended: a
small corpus-status step that `just test` runs **before** the suite, printing one
line per manifest entry (present / MISSING + path). The in-harness skip returns
early; the loudness lives where it can be seen with no flags.

⚠ Do **not** make `just test` pass `--nocapture` globally — that buries the signal
in full test output instead of surfacing it.

### Constraints that bind

- `no-new-top-level-deps-without-decision` — DEC-004 rule 4 sanctions a trivial
  **dev-only** permissive dep **with its DEC authored in the same pass**. `toml`
  qualifies; write the DEC.
- `DEC-003` — tier-B files are never committed; CI cannot run them.
- A silent skip is the same defect class as an oracle that cannot go red.

## Expected Deliverables

1. Manifest reader: path, `sha256`, `oracle.raw_checksum` for all 7 entries.
2. `$IRRADIANCE_CORPUS_DIR` resolution, defaulting to `tests/corpus/tier-b/`.
3. sha256 verified on present files; mismatch fails loudly naming the file.
4. **A visible skip** — `just test 2>&1 | grep SKIP` prints, naming the absent
   file, with no extra flags. Paste that output.
5. A `DEC-*` for the `toml` dev-dependency, stating the dev-only scope.
6. All existing gates green, output pasted.

## Out of Scope

- Any decoding — SPEC-003 onward.
- The `#[allow]` bypass — that is **SPEC-006**, no shared files.
- Re-opening `DEC-003`'s storage/schema decisions.
- A runtime (non-dev) dependency of any kind.

## Return Criteria — how to hand back

1. Paste the gates **and** the visible-skip output from deliverable 4.
2. Fill `## Completion` and `handback:`. For `tokens_total`: if `/cost` is
   unavailable, sum transcript usage objects and **say so**, with cache-read share.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Branch `feat/spec-002-corpus-manifest-reader` off `main`; commit; do not merge.

## Handback

### Execution notes

- **Branch / PR:** `feat/spec-002-corpus-manifest-reader` @ **`4516280`** — committed, **not
  pushed and no PR**, per "commit; do not merge".
  ⚠ **Read this before anything else:** the SPEC-002 *design* commit `412cb1b`
  was sitting on the branch `feat/spec-006-allow-attribute-gate`, not on this
  spec's branch. `feat/spec-002-corpus-manifest-reader` was still at `dd4eb42`
  and **HANDOFF-008 did not exist in its tree** — I could not read my own
  handoff. `412cb1b`'s parent *is* `dd4eb42`, so I fast-forwarded this branch
  onto it (no rewrite, no cherry-pick, nothing discarded) and built on top.
  `feat/spec-006-allow-attribute-gate` still points at `412cb1b` and now holds
  SPEC-002 design work under a SPEC-006 name; the orchestrator should reset or
  delete it before SPEC-006 starts. This is DEC-004's
  `shared-tree-subagent-corruption` failure mode ("a design commit landed on the
  wrong branch"), recurring.
- **Completed at:** 2026-08-20
- **All acceptance criteria met?** **Yes, all six.** Evidence below.

#### Deliverable 4 — the visible skip, in three corpus states

The criterion, run with **no extra flags**, no corpus present (this is the CI
state and the default root):

```
$ just test 2>&1 | grep SKIP
corpus: SKIP     LEICA-Q2-MONO/L1021223.DNG — MISSING at …/tests/corpus/tier-b/LEICA-Q2-MONO/L1021223.DNG
corpus: SKIP     LEICA-Q2-MONO/L1026016.DNG — MISSING at …/tests/corpus/tier-b/LEICA-Q2-MONO/L1026016.DNG
corpus: SKIP     LEICA-Q2-MONO/L1026192.DNG — MISSING at …/tests/corpus/tier-b/LEICA-Q2-MONO/L1026192.DNG
corpus: SKIP     LEICA-M-MONOCHROM/L1000622.DNG — MISSING at …/tests/corpus/tier-b/LEICA-M-MONOCHROM/L1000622.DNG
corpus: SKIP     LEICA-M-MONOCHROM-TYP246/M2462362.DNG — MISSING at …/tests/corpus/tier-b/LEICA-M-MONOCHROM-TYP246/M2462362.DNG
corpus: SKIP     PENTAX-K3III-MONO/K3III.DNG — MISSING at …/tests/corpus/tier-b/PENTAX-K3III-MONO/K3III.DNG
corpus: SKIP     PENTAX-K3III-MONO/K3III.PEF — MISSING at …/tests/corpus/tier-b/PENTAX-K3III-MONO/K3III.PEF
corpus: 0/7 present, 7 MISSING — tier-B tests over those file(s) will SKIP. Set $IRRADIANCE_CORPUS_DIR to point at your corpus.
```

All-absent is the easy case, so I also ran the case the criterion actually
exists for — **6 of 7 present, one file removed** (all seven are on this
machine, so I pointed `$IRRADIANCE_CORPUS_DIR` at a link farm missing the
rotated frame):

```
$ IRRADIANCE_CORPUS_DIR=<6-of-7> just test 2>&1 | grep SKIP
corpus: SKIP     LEICA-Q2-MONO/L1026016.DNG — MISSING at …/corpus-partial/LEICA-Q2-MONO/L1026016.DNG
corpus: 6/7 present, 1 MISSING — tier-B tests over those file(s) will SKIP. Set $IRRADIANCE_CORPUS_DIR to point at your corpus.
```

And the no-false-alarm direction, all 7 present:

```
$ IRRADIANCE_CORPUS_DIR=<real corpus> just test 2>&1 | grep -c SKIP
0
```

**The design measurement reproduced exactly** (6 present / 1 absent, same tree):

```
cargo test                 -> 0 SKIP lines
cargo test -- --nocapture  -> 2 SKIP lines
just test                  -> 2 SKIP lines     <- no flags
```

**Adversarial mutation — the criterion has teeth.** Delete the one line
`cargo run --quiet --all-features --example corpus-status` from `app.just`'s
`test:` recipe and the criterion dies, which is what makes it a criterion and
not decoration:

```
mutated  `just test 2>&1 | grep SKIP` -> 0 lines   (criterion FAILS, as it must)
restored `just test 2>&1 | grep SKIP` -> 2 lines
```

#### The seven gates, on the tree as committed

```
1. cargo fmt --check                                      ✅ clean
2. cargo clippy --all-targets --all-features -- -D warnings ✅ No issues found
3. cargo test --all-features                              ✅ 11 passed, 0 failed
                                                             (2 lib + 9 corpus)
4. cargo +1.90.0 check --all-targets --all-features (MSRV) ✅ exit 0
5. cargo deny check licenses                              ✅ licenses ok
6. just lint-red-proof (SPEC-001's gate)                  ✅ control clean → injection
                                                             rejected → all five lints fired
7. cost-audit.sh / decisions-index.sh --check             ✅ both ✓
```

Test names, run with the corpus absent:

```
running 9 tests
test corpus_manifest_parses ... ok
test corpus_root_defaults_and_is_overridable ... ok
test corpus_hash_mismatch_fails ... ok
test corpus_truncation_fails_by_size ... ok
test corpus_absent_file_is_missing_not_an_error ... ok
test manifest_rejects_entries_missing_provenance ... ok
test sha256_matches_published_vectors ... ok
test sha256_streaming_matches_one_shot ... ok
test corpus_files_match_their_pinned_sha256 ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

#### Deliverables 1–3, 5–6

1. **Reader** — `tests/support/corpus.rs`. Reads all **7** entries and exposes
   `path`, `sha256` and `oracle.raw_checksum` (plus `tier`, `bytes`, `licence`,
   `source`, `pgm_bytes`, `strip_bytes`). All seven required keys are required
   *loudly*: an entry missing one is rejected **by name**, which makes DEC-003's
   "EVERY entry MUST carry `licence` and `source`" mechanical rather than
   remembered. Nothing hardcodes a corpus path.
2. **`$IRRADIANCE_CORPUS_DIR`** — resolved by `CorpusRoot::resolve()`, default
   `<crate root>/tests/corpus/tier-b` via `CARGO_MANIFEST_DIR` so the working
   directory cannot change the answer. Empty-string is treated as unset.
3. **sha256 verified, mismatch fails loudly by name** — and *measured against
   the real corpus*, not just synthetically: all **7/7** real files (21–86 MB,
   ~330 MB total) verify against their pinned digests in 12.6 s.
   The red-proof plants a **same-length, one-byte-flipped** copy so the size
   check cannot be what catches it, and asserts an intact file passes first —
   the DEC-009 negative-control lesson applied to a second oracle. Truncation is
   covered separately and reports `size mismatch`.
5. **`DEC-010`** — `decisions/DEC-010-toml-as-a-dev-only-dependency.md`. States
   the dev-only scope explicitly and re-measures design's table (11 crates, MSRV
   1.90 holds, licenses ok, every crate MIT/Apache-2.0 or MIT).
6. Gates above.

### Cost self-report

- **Tokens (total):** **9,498,150**
- **Estimated USD:** null (no rate recorded in this repo; prior sessions also
  left this null)
- **Duration (minutes):** ~30
- **Source of the number:** transcript `usage` objects. `/cost` is a
  client-side slash command the assistant cannot execute, so I summed this
  session's own transcript
  (`~/.claude/projects/-Users-…-irradiance/dbdeb6a8-….jsonl`) — the same data
  `/cost` derives from. It is a **FLOOR**: written before the session ends.
- **Composition:** input 150 + output 65,303 + cache-write 159,959 +
  cache-read 9,272,738 — **97.6% cache-read**.

⚠ **This number is not comparable to any SPEC-001 figure, and the reason is a
bug in how those were measured, not a difference in effort.** A Claude Code
transcript writes **one jsonl line per content block**, and every line of a
multi-block assistant message repeats the *same* `usage` object. Summing usage
objects therefore double-counts. Measured on SPEC-001's own verify-4 transcript
(`e17489a8`): **116 raw usage objects, 67 distinct `message.id`s, 47
duplicated** — raw sum **14,177,812** vs deduped **8,053,949**, against a
recorded **10,962,512** (a partial raw sum). My figure above is **deduped by
`message.id`**; every transcript-summed figure in SPEC-001 is inflated ≈1.7×,
which makes its `cost.totals: 51,979,929` not a real number. Signal
`token-counts-not-comparable` updated with the measurement and the one-line fix
(key the summing loop by `message.id`). SPEC-001's four transcript-based
sessions should be re-summed rather than left standing.

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-010` — `toml` is a dev-dependency; the library keeps zero dependencies.
- **Deviations from spec:** none in substance. Three judgement calls worth
  naming, because each could reasonably have gone the other way:
  1. **SHA-256 is written, not depended on.** The handoff budgeted exactly one
     dev-dependency and design measured only `toml`, but nothing in std hashes.
     Rather than silently add a second unmeasured dep (`sha2` + its ~6-crate
     chain) I implemented SHA-256 from **FIPS 180-4** — provenance class 1,
     ledger row added, proven against the published NIST vectors on every run.
     The deciding argument is this spec's own logic: a hashing crate would be
     exercised *only* on a machine holding the corpus, so a broken integration
     would be **invisible in CI** — the exact invisibility this spec exists to
     remove. The vectors run everywhere. ~110 lines. If verify disagrees, `sha2`
     is a small, clean swap.
  2. **The visible surface is an `examples/` target, not an `irr` subcommand.**
     Forced, not chosen: Cargo gives dev-dependencies to tests/examples/benches
     but **not** to `[lib]` or `[[bin]]`, so `irr` cannot parse the manifest
     without promoting `toml` to a runtime dep. Recorded in DEC-010.
  3. **CI's `rust / test` job now runs `corpus-status` too.** The handoff's
     wording ("visible with no flags **and in CI logs**") only holds if CI runs
     it, and CI calls `cargo test` directly, not `just test`. One added step,
     same ordering as `app.just`.
- **One `#[allow]` added, and it is not SPEC-006's:** `#![allow(dead_code)]` at
  the top of `tests/support/corpus.rs`. That file is compiled into **two**
  crates via `#[path]` and each uses a different subset, so `dead_code` reports
  items that *are* used — I checked the full list (11 items; every one is used
  by the other consumer). It is `dead_code`, in `tests/`, on code never compiled
  into the library — not one of the five panic lints, and it shares no file with
  SPEC-006.
- **Follow-up work identified:**
  - **Branch hygiene, do this first:** reset/delete
    `feat/spec-006-allow-attribute-gate`, which currently holds SPEC-002's
    design commit.
  - **Re-sum SPEC-001's cost sessions with dedup** (signal above). Cheap now,
    misleading forever if left.
  - `[[wanted]]` and `[[available]]` in the manifest still have **no reader**.
    In scope for nothing today; I documented it in the manifest header so it is
    a declared gap rather than a silent one. If they are ever meant to surface,
    that is a spec.
  - `just test` now costs **~12.6 s** on a machine with the full corpus (330 MB
    hashed in a debug build). Acceptable, and it is the check working. If it
    becomes friction, `[profile.test] opt-level = 2` is the lever — I did not
    take it, since profile changes affect every build and nobody asked.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Not the spec: the two measurements transcribed exactly and build really was
   near bit-for-bit transcription, which is the §12 claim holding up. What cost
   real time was **finding the handoff at all** — it was on another branch, and
   the first three minutes went on reconciling git state instead of reading. The
   spec's `## Inputs` / `## Outputs` / `## Non-Goals` were also still template
   placeholders (`path/to/new.ext`); I filled them from what was actually built,
   but that is design's section and it shipped empty.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Two. (a) **Nothing said how to hash.** The spec requires sha256
   verification, budgets one dev-dependency, and never says where the hash comes
   from — the largest genuine decision in the cycle, and it was implicit. (b)
   **`provenance-recorded-per-algorithm` was not in the handoff's "Constraints
   that bind"**, yet writing SHA-256 triggers it. I found it by reading
   `constraints.yaml` per §15, but a build cycle that skipped that step would
   have shipped an unlogged algorithm. Worth adding: *if the spec's acceptance
   criteria name a checksum, hash, or codec, the provenance constraint binds.*

3. **If you did this task again, what would you do differently?**
   — Run `git log --all --oneline` **before** `cat`-ing the handoff. I lost the
   loop assuming my branch contained my own handoff. Given DEC-004 rule 1 already
   says *trust git/disk over any self-report*, the same instinct should apply to
   the handoff's own location — reconcile where the work is, then read it. On the
   code: I would write `sha256_streaming_matches_one_shot` first. It caught a real
   bug (`buf_len` reset to 0 when a call was fully absorbed by the partial-block
   buffer) that the NIST vectors **all passed through**, because every vector is a
   single `update()` call — the multi-block 10⁶-'a' vector included. The bug only
   appears when a chunk boundary lands mid-block, which is exactly what the 1 MiB
   file reader does.
