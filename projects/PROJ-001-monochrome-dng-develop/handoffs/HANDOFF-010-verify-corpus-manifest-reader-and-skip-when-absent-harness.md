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
  id: HANDOFF-010
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  tokens_total: 6097683            # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 45
  branch: feat/spec-002-corpus-manifest-reader
  pr: null                         # not pushed — handoff said commit, do not merge
  completed_at: 2026-08-20         # YYYY-MM-DD
  notes: "APPROVED at 4516280 (reviewed at 112bd80). No ship-blocking findings; 7 follow-ups. SHA-256 verified exhaustively against an independent oracle - every length 0..600, exhaustive 2- and 3-way streaming splits, byte-at-a-time, and 2^32+1 bytes - plus K/H0 rederived from first principles and all 7 real corpus digests cross-checked with shasum -a 256. tokens_total is a transcript sum DEDUPED BY message.id (measured 1.95x raw inflation on this session; 97.0% cache-read)."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-010: Corpus manifest reader and skip-when-absent harness

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-002` for the **verify** cycle, at
`82fc390`. Independent session.

⚠ **ID note:** renamed `009` → `010` by hand. `just new-handoff` allocated `009`,
already held by SPEC-006's verify handoff on its branch — the command counts what
is visible in the current worktree, so parallel branches collide. **Second
occurrence.** Do not renumber it back.

## Context the Receiving Agent Needs

### Already reconciled — don't just repeat

- `just test 2>&1 | grep SKIP`, **no extra flags** → 8 lines (7 entries + summary),
  each naming the absent file. Criterion met.
- Real corpus present: **7/7**, 9 tests, 13.08 s. All gates green.

### The judgement call that most deserves scrutiny

**SHA-256 was hand-written from FIPS 180-4 rather than taken as a dependency.**
Hand-rolled crypto is normally a red flag, so weigh the argument, not the instinct:

- nothing in `std` hashes, and design budgeted exactly one dev-dep (`toml`);
- a hashing *crate* would be exercised **only where the corpus exists**, so a
  broken integration would be invisible in CI — the precise invisibility this spec
  exists to remove;
- NIST vectors run everywhere; verified against all 7 real files (~330 MB).

Corroboration: the manifest's `sha256` values were produced with `shasum -a 256`
**before this code existed**, and the suite checks against them — a wrong
implementation fails on real data. That is evidence, not proof: it exercises one
input class. **Consider what it does NOT cover** — empty input, multi-block
boundaries, lengths near the 55/56/64-byte padding edges, >4 GiB. Are the NIST
vectors well chosen? If you disagree, `sha2` is a clean swap and the builder says
so.

### Two more disclosed calls

1. The visible surface is an **`examples/` target**, not an `irr` subcommand —
   forced, because Cargo denies dev-deps to `[lib]`/`[[bin]]` (`DEC-010`). Is
   `examples/` the right home, or does it imply something user-facing that isn't?
2. CI's `rust / test` job also runs corpus-status, since it calls `cargo test`
   directly and would otherwise miss the lines.

### Also worth a look

- `just test` now takes **~12.6 s** with the full corpus (330 MB hashed in a debug
  build). `[profile.test] opt-level = 2` was deliberately **not** set, since
  profile changes affect every build. Right call?
- `DEC-010` must be explicit that the **library's zero-dependency claim is
  untouched** — that claim is load-bearing in this project's pitch.

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the criterion yourself with **no extra flags**, and confirm the negative case:
delete the corpus-status line and it must drop to **0** SKIP lines, or the check
has no teeth. The builder measured exactly that.

**Label every finding ship-blocking or follow-up.** A wrong hash accepted as
correct is ship-blocking; a slow test is a follow-up.

If **APPROVED**, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- SPEC-006's branch. ⚠ Both branches touch `app.just` and
  `.github/workflows/ci.yml`; that conflict is the orchestrator's to resolve.
- Re-opening `DEC-003`'s storage/schema decisions.

## Return Criteria — how to hand back

1. Verify cost session with a real `tokens_total`. ⚠ **Transcript sums
   double-count** — one jsonl line per content block repeats the same usage
   object. Deduplicate by `message.id` and **say that you did**, with cache-read
   share. This build measured the effect (~1.7x inflation) and updated the
   `token-counts-not-comparable` signal.
2. Fill `## Completion` and `handback:`; `handoff.status: completed`.
3. Verdict with SHA, every finding labelled ship-blocking or follow-up.
4. Commit on `feat/spec-002-corpus-manifest-reader`; do not merge.

## Handback

## Verdict

# ✅ APPROVED — at `4516280` (reviewed on `112bd80`)

**No ship-blocking findings.** Seven follow-ups, all labelled below. The one
call that most deserved scrutiny — hand-written SHA-256 — is **correct**, and I
established that against an independent oracle rather than by reading the code.
`task.cycle` → `ship`, `verify_verdict: approved`.

### Execution notes

- **Branch / PR:** `feat/spec-002-corpus-manifest-reader` — reviewed at
  `112bd80`, code as committed at `4516280`. Not pushed, no PR.
- **Completed at:** 2026-08-20
- **All acceptance criteria met?** **Yes, all six**, each re-run here rather
  than read off the handback.
- **For `verify`:** ✅ **APPROVED at `4516280`**

---

## 1. The criterion, run here with no extra flags

```
$ just test 2>&1 | grep SKIP
corpus: SKIP     LEICA-Q2-MONO/L1021223.DNG — MISSING at …/tests/corpus/tier-b/LEICA-Q2-MONO/L1021223.DNG
… 6 more, each naming its absent file …
corpus: 0/7 present, 7 MISSING — tier-B tests over those file(s) will SKIP. Set $IRRADIANCE_CORPUS_DIR to point at your corpus.
                                                              -> 8 lines
```

**The negative case — the check has teeth.** Deleted the one line
`cargo run --quiet --all-features --example corpus-status` from `app.just`'s
`test:` recipe:

```
mutated   just test 2>&1 | grep -c SKIP  -> 0     (criterion dies, as it must)
restored  just test 2>&1 | grep -c SKIP  -> 8
```

**The case the criterion actually exists for.** The full corpus is on this
machine at `…/crustimg_redo_plus/images`; I built a symlink farm missing exactly
one frame:

```
$ IRRADIANCE_CORPUS_DIR=<6-of-7> just test 2>&1 | grep SKIP
corpus: SKIP     LEICA-Q2-MONO/L1026016.DNG — MISSING at …/corpus-6of7/LEICA-Q2-MONO/L1026016.DNG
corpus: 6/7 present, 1 MISSING — …
```

and the no-false-alarm direction, all 7 present: **0 SKIP lines, 9 tests,
14.91 s** (builder measured 13.08 s — same machine, ordinary variance).

The design measurement reproduced exactly, on the same 6-of-7 tree:

```
cargo test                 -> 0 SKIP lines
cargo test -- --nocapture  -> 2
just test                  -> 2      <- no flags
```

## 2. SHA-256 — the judgement call, tested rather than argued

I take the builder's argument. A hashing crate's integration would execute
**only where the corpus exists**, which is nowhere in CI, so a broken wiring
would be invisible — and removing that exact invisibility is what this spec is
for. The NIST vectors run on every machine, every run. `sha2` would have been a
clean swap; it would also have been the weaker choice here. **I am not asking
for it.**

But the handoff is right that the real-corpus corroboration is *one input
class*, so I tested the code instead of weighing the argument. I built a
differential harness that `#[path]`-includes `tests/support/corpus.rs`
**verbatim** (not a copy) and compared it to Python's `hashlib` (OpenSSL) and to
`shasum -a 256`:

| probe | result |
|---|---|
| **Every length 0..=600**, one-shot, vs `hashlib` | **0 mismatches** — covers all 64 residues ≈9× each, incl. **55 / 56 / 57 / 63 / 64** |
| **Exhaustive 2-way splits** at n = 0,1,55,56,57,63,64,65,127,128,129,191,192,500,5000 | **0 mismatches** |
| **Exhaustive 3-way splits** (every `(i,j)`) at n = 64,65,128,200 | 33,042 splittings, **0 mismatches** |
| **Byte-at-a-time** `update()` at n = 1,55,56,64,65,200,1000 | matches one-shot |
| **> 4 GiB**: 2³²+1 zero bytes | `fbb82f7b…802c5c` — **matches `hashlib` exactly**; the 64-bit length field is sound |
| **K[64] and H0[8]** rederived from the cube/square roots of the first primes in `Decimal` | **all 72 constants exact** — written from the standard, not transcribed |
| **All 7 real corpus files** hashed with `shasum -a 256` vs the manifest pins | **7/7 identical** |

That last row closes the loop the handoff asked about: the manifest digests came
from `shasum` before this code existed, and I re-derived them with `shasum`
myself — then the suite verifies the same 330 MB with the hand-written code and
passes. Two independent implementations agree on real data *and* on 600
synthetic lengths *and* past the 4 GiB boundary.

**Empty input, multi-block boundaries, the 55/56/64 padding edges and >4 GiB —
all four of the handoff's named gaps are covered and all four are correct.**

### Are the NIST vectors well chosen? No — see F1.

## 3. The oracle goes red, and it goes red *selectively* (§15 check 9)

Observed personally, not read off a handback:

| deliberate fault | `corpus_hash_mismatch_fails` | `corpus_truncation_fails_by_size` |
|---|---|---|
| `if actual != self.sha256` → `if false` (corpus.rs:349) | **FAILED** ✅ | ok |
| `if meta.len() != self.bytes` → `if false` (corpus.rs:330) | ok | **FAILED** ✅ |

Each red-proof is wired to its own claim and to nothing else — that is stronger
than a red. The negative control inside `corpus_hash_mismatch_fails` (the intact
file must pass first) is the DEC-009 lesson correctly transplanted.

## 4. Gates, re-run here

```
1. cargo fmt --check                                        ✅ clean
2. cargo clippy --all-targets --all-features -- -D warnings  ✅ 0 issues
3. just test (corpus absent)                                 ✅ 11 passed (2 lib + 9 corpus)
   just test (corpus present, 7/7)                           ✅ 9 passed, 14.91 s, 0 SKIP
4. cargo +1.90.0 check --all-targets --all-features (MSRV)   ✅ exit 0
     ⚠ via ~/.cargo/bin/cargo — the §5 `+toolchain` trap, again
5. cargo deny check licenses                                 ✅ licenses ok
6. just lint-red-proof                                       ✅ control clean → injection
                                                                rejected → all five lints fired
7. cost-audit ✓ / decisions-index --check ✓ / decisions-audit --changed ✓
```

## 5. The §15 verify checks

| # | check | result |
|---|---|---|
| 1 | acceptance criteria met and tested | ✅ all six, re-run above |
| 2 | spec's failing tests now pass | ✅ all three, incl. the `grep SKIP` one |
| 3 | no drift from referenced decisions | ✅ `decisions-audit --changed` flags DEC-000/003/004/009/010; each consistent |
| 4 | no constraint violations | ✅ all six binding constraints hold (§7 below) |
| 5 | non-trivial choices have a `DEC-*` | ⚠ **F3** — the biggest one is a bullet inside DEC-010 |
| 6 | reflection answered, not mailed in | ✅ names a real bug, a real process gap, a real git lesson |
| 7 | cost.sessions has prior-cycle entries | ✅ build present; no design entry — consistent with SPEC-001, design is un-metered |
| 8 | **runtime** behaviour exercised, not shape | ✅ this is the whole review — mutation + differential, not reading |
| 9 | **did the oracle go red?** | ✅ personally observed, and selectively (§3) |
| 10 | **fuzz target exists and ran?** | **N/A** — §12 bar 2 binds a *parser spec adding an input surface*. `src/` is untouched (`git diff dd4eb42..HEAD -- src/` is empty), TOML is parsed by a third-party crate over a **committed, trusted** file, and SHA-256 is dev-only test support. SPEC-003 is the parser spec and names its target. Design specified none, correctly. |
| 11 | provenance row, honest class, permissive source | ✅ FIPS PUB 180-4, class 1 (published specification), US Government work. Honest: written from the standard, and the ledger row says so |
| 12 | new dependency permissive, not a RAW decoder | ✅ verified independently of DEC-010's table: `cargo tree -e normal,build,dev` = **11 crates**, every licence MIT / Apache-2.0 / MIT-OR-Unlicense / Unicode-3.0. `toml` is a TOML parser |

## 6. The handoff's four "also worth a look" questions, answered

**Is `examples/` the right home?** **Yes — it is forced *and* idiomatic.**
`examples/` is the only Cargo target kind that both receives dev-dependencies
and produces a runnable binary; the alternative was promoting `toml` to a
runtime dep, which DEC-010 rightly refuses. The "implies something user-facing"
worry is real but narrow, and it has a date on it — see **F4**.

**CI's `rust / test` job also running corpus-status.** Correct, and necessary:
CI calls `cargo test` directly, so without that step the handoff's "and in CI
logs" would have been false. Well-commented at `ci.yml:57-64`, including the
"do NOT replace this with `--nocapture`" instruction that keeps the next
maintainer from undoing it.

**~12.6 s and `[profile.test] opt-level` deliberately unset.** **Right call**,
and here is the number it was made without: 4 GiB hashed in **144.5 s debug vs
9.7 s release — 14.9×**, so the corpus pass is ~14 s unoptimised and would be
~1 s optimised. Still right to skip: the cost lands only on machines holding
330 MB of RAW, never on CI, and `[profile.test]` is a repo-wide lever pulled for
one test. **If it ever becomes friction, the narrower lever is
`[profile.test.package.irradiance] opt-level = 2`, not a blanket
`[profile.test]`.**

**Is DEC-010 explicit that the LIBRARY's zero-dependency claim is untouched?**
**Yes — and it does better than asked.** The claim is stated in the title, the
Decision paragraph, Alternative A, and Validation. It also names the failure
mode nobody asked it to: *"the first crack in 'zero dependencies' as a slogan…
anyone repeating the claim must say **the library** has no dependencies, not
**the repo**"*, with a README check queued. I verified the technical claim
independently: `cargo tree -e normal` → `irradiance v0.1.0` alone, and
`git diff dd4eb42..HEAD -- src/` is empty. Nothing to add.

## 7. Constraints

| constraint | verdict |
|---|---|
| `no-new-top-level-deps-without-decision` | ✅ dev-only + DEC authored in the same pass = DEC-004 rule 4 exactly |
| `no-copyleft-dependencies` | ✅ re-measured myself, 11 crates, all permissive |
| `provenance-recorded-per-algorithm` | ✅ ledger row, class 1, honest |
| `library-not-application` | ✅ `[dependencies]` empty, `src/` untouched |
| `oracle-must-be-shown-red` | ✅ observed red, and selectively |
| `test-before-implementation` | ✅ spec's `## Failing Tests` written at design; build added four more |

**Criterion 1 ("no test hardcodes a corpus path") holds.** The `.DNG` strings in
`tests/corpus_manifest.rs` are *manifest lookup keys* passed to `Manifest::get`,
not filesystem paths; every path is built by `CorpusFile::resolve`.

**The `#![allow(dead_code)]` is justified, and I checked rather than took it.**
Removing it, every warning is emitted against
`examples/../tests/support/corpus.rs` (the example, which never touches
`sha256`, `at`, `get`, `require`, `verify`) plus `origin` against the test
binary. Every reported item **is** used — by the other consumer. It masks
nothing.

---

## Findings — every one labelled

### Ship-blocking

**None.**

### Follow-up

**F1 — the NIST vector set misses the 55-byte padding edge, and so does the real
corpus.** `tests/corpus_manifest.rs:305-333`
The four vectors have lengths 0, 3, 56, 1,000,000 → residues mod 64 of
**{0, 3, 56, 0}**; the streaming test (`:340`) adds residue 8. Mutating
`tests/support/corpus.rs:474` `>` → `>=` makes every message of length
≡ 55 (mod 64) hash wrong, and **all 9 committed tests still pass** — I ran it.
The real corpus does not cover it either: the seven manifest sizes have residues
{0,0,0,0,0,54,36}, none of them 55. So the one surviving mutant in the padding
branch is invisible to both the vectors *and* the 330 MB corroboration.
*The shipped code is correct at that edge* — proven exhaustively above. This is
regression detection, not a live bug. **Fix: add lengths 55, 57 and 63 to
`sha256_matches_published_vectors`.** One line, and it kills the mutant.
(For contrast, the mutant the builder already fixed *is* caught: removing the
`if !data.is_empty()` guard at `:454` reddens `sha256_streaming_matches_one_shot`
and nothing else. That test earned its place, exactly as the reflection says.)

**F2 — "the loudness cannot live in the harness" is false as recorded; it is the
print *macros* that are captured, not stderr.** `tests/support/corpus.rs:306-310`
Measured here: `writeln!(std::io::stderr(), …)` inside a **passing** test is
**not** captured by libtest and prints under bare `cargo test` with no flags.
`eprintln!` is captured. Proved in this repo — swapping `require()`'s
`eprintln!` for a direct stderr write takes
`cargo test --test corpus_manifest` from **0 SKIP lines to 8**, no `just`, no
`--nocapture`; restored, back to 0.
*Consequence today:* a developer who types `cargo test` instead of `just test`
still gets a silent skip — this spec's own defect, in the residual case.
*Why it is not ship-blocking:* the spec's executable criterion is
`just test 2>&1 | grep SKIP` and it passes; CI is covered; and `corpus-status` is
independently better than in-harness output (it lists **present** files too, and
runs before the suite). Keep it. **But the claim is written as "measured" in five
places** — the spec's `## Notes for the Implementer`, `app.just:30-42`,
`ci.yml:57-64`, `tests/support/corpus.rs:19-27`, `examples/corpus-status.rs:7-13`
— and a wrong *measured* fact propagating through the record is the kind of thing
this repo exists to catch. **Fix: make `require()` write direct-to-stderr (belt
and braces, `just test` unchanged) and correct the five comments to say
"`eprintln!`/`println!` are captured; direct `stderr()` writes are not."**

**F3 — the cycle's largest judgement call has no `DEC-*` of its own.**
`decisions/DEC-010-toml-as-a-dev-only-dependency.md` (Consequences → "Neutral")
Hand-writing SHA-256 is recorded only as a consequence bullet inside a decision
*titled* "`toml` is a dev-dependency", whose `tags:` are
`[dependencies, testing, corpus, licensing]` — no `hashing`, no `crypto`. The
builder's own reflection calls it "the largest genuine decision in the cycle",
and I agree; it is the one I spent most of this review on. The mechanical hook
does work — DEC-010's `affected_scope` includes `tests/support/**`, and
`decisions-audit --changed` surfaced it for me — so this is discoverability, not
a broken audit trail. **Fix: split it into its own DEC** (alternatives: `sha2`,
`ring`, hand-write; the CI-invisibility argument; revisit-if: a second hash
algorithm is needed, at which point take the dependency).

**F4 — `examples/corpus-status.rs` ships inside a published crate.**
`Cargo.toml:14`
`cargo package --list --allow-dirty` includes `examples/corpus-status.rs`. On
crates.io and docs.rs an `examples/` entry reads as *a usage example of the
library*, and this one never touches the library — it `#[path]`s into
`tests/support/`. Harmless today because `publish = false`; wrong the moment
that flips (STAGE-004 or a later publish spec). **Fix, at that moment:
`exclude = ["examples/corpus-status.rs", "tests/corpus/**"]`, or put the example
behind `required-features`.** Filed now because the flag is a one-word change
someone will make without thinking about `examples/`.

**F5 — the recorded build `tokens_total` is a floor that is now measurably
~25 % low.** spec `cost.sessions` build entry; HANDOFF-008 `handback.tokens_total`
Re-summing the build transcript (`dbdeb6a8`) **after** the session closed,
deduped by `message.id`: **12,644,814** (167 usage objects, 91 distinct ids, 76
duplicated, 98.0 % cache-read) against the recorded **9,498,150**. The builder
labelled it a floor written before the session ended — accurate, honest, and now
quantified. **I independently confirmed the double-counting mechanism** on three
transcripts: inflation **1.86×** (build), **1.84×** and **2.25×** (two others),
and **1.95×** on my own. The `token-counts-not-comparable` signal's diagnosis is
correct; only the specific multiplier varies by session shape.
**Fix: when SPEC-001's four sessions are re-summed, re-sum SPEC-002's build too
— the same one-line dedup, run after the session ends rather than during it.**

**F6 — `..` escapes the corpus root while `/` is rejected.**
`tests/support/corpus.rs:201-206` (guard) and `:276-282` (`resolve`)
The parser rejects an absolute `path` citing DEC-003's "manifest paths are
relative to `$IRRADIANCE_CORPUS_DIR`", but `resolve()` pushes `..` components
verbatim. Verified: `path = "../../../etc/hosts"` parses clean and resolves to
`/tmp/corpus-root/../../../etc/hosts`. The manifest is committed and trusted, so
this is a completeness gap in a guard, not a vulnerability — but the guard states
an invariant it does not actually enforce. **Fix: reject any component equal to
`..` in the same check.** One line, same error message shape.

**F7 — quantifying the `[profile.test]` decision (no action needed).**
Recorded so the next person does not re-measure: **14.9× debug→release** on this
hash (4 GiB: 144.5 s vs 9.7 s). The corpus pass is ~14 s today, ~1 s optimised.
Agree with leaving it unset; if it is ever taken, take
`[profile.test.package.irradiance]`, not the blanket profile.

---

### Cost self-report

- **Tokens (total):** **6,097,683**
- **Estimated USD:** null (no rate recorded in this repo; consistent with prior
  sessions)
- **Duration (minutes):** ~45
- **Source of the number:** transcript `usage` objects. `/cost` is a client-side
  slash command the assistant cannot execute, so I summed this session's own
  transcript (`~/.claude/projects/-Users-…-irradiance/f437e7b3-….jsonl`) — the
  same data `/cost` derives from. **FLOOR**: written before the session ends.
- **⚠ DEDUPLICATED BY `message.id`, and I say so because the handoff required
  it.** This session: 97 usage objects, **51 distinct `message.id`s**, 46
  duplicated → raw 11,900,593 vs deduped **6,097,683**, an inflation of
  **1.95×**. Comparable to SPEC-002's build figure (also deduped); **not**
  comparable to any SPEC-001 figure.
- **Composition:** input 102 + output 41,634 + cache-write 141,361 + cache-read
  5,914,586 — **97.0 % cache-read**.

### Drift and new artifacts

- **New decisions emitted:** none. F3 proposes one; authoring it is the
  orchestrator's call, not a verify-cycle act.
- **Deviations from spec:** none. The build added four tests beyond the spec's
  three (`corpus_truncation_fails_by_size`,
  `corpus_absent_file_is_missing_not_an_error`,
  `manifest_rejects_entries_missing_provenance`,
  `sha256_streaming_matches_one_shot`) — additive, and the last one caught a real
  bug.
- **Follow-up work identified:** F1–F6 above, plus the two the builder already
  filed and which I confirm still stand:
  - **Branch hygiene, still outstanding:** `feat/spec-006-allow-attribute-gate`
    still points at `412cb1b` and holds SPEC-002's design commit under a SPEC-006
    name. Reset or delete it **before** SPEC-006 starts.
  - `[[wanted]]` / `[[available]]` still have no reader — declared in the
    manifest header, which is the right handling.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing was unclear; the handoff was unusually good at naming *what to
   distrust*, which is what a verify handoff is for. The one thing that cost
   time was locating the tier-B corpus: the handback says "all seven are on this
   machine" but not **where**, and the default root does not exist. I found it
   under `…/crustimg_redo_plus/images` with `mdfind`. A handback that relies on
   the reviewer reproducing a corpus-present measurement should paste the
   `IRRADIANCE_CORPUS_DIR` it used.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The builder's answer (b) is right and worth adopting verbatim: *if a spec's
   acceptance criteria name a checksum, hash, or codec, then
   `provenance-recorded-per-algorithm` binds* — it was not in HANDOFF-008's
   "Constraints that bind" and the builder found it only by reading
   `constraints.yaml` per §15. I would add a second: **§15 check 10 (fuzz
   target) needs a stated negative case.** I had to reason my way to "N/A"
   because it is one of the four checks where a wrong answer is an automatic ❌,
   and the rule as written ("for any spec touching a parser") does not say
   whether *consuming a third-party parser over a trusted committed file*
   counts. It does not — but that should be written down, not re-derived.

3. **If you did this task again, what would you do differently?**
   — Reach for mutation testing sooner. I spent the first pass reading the
   SHA-256 by eye and satisfied myself it was right — which is exactly the
   self-report DEC-004 rule 1 warns about. The findings that matter (F1, F2) both
   came from *breaking* something and watching what failed to notice: F1 from
   mutating the padding branch, F2 from doubting a measurement the record
   asserted five times. Reading confirms; mutating discriminates. On a hand-rolled
   primitive, the differential harness should be the **first** move, not the
   third — it took ten minutes and it is the entire basis for the approval.
