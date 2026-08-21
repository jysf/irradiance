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
  id: HANDOFF-011
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-20
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-003

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
  tokens_total: 10967269           # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 75
  branch: feat/spec-003-ifd-reader
  pr: null                         # not opened — the handoff says commit, do not merge
  completed_at: 2026-08-20         # YYYY-MM-DD
  notes: "All 7 acceptance criteria met; nine gates green; both fuzz directions pasted. tokens_total is a transcript sum DEDUPED BY message.id (122 usage objects, 64 distinct ids, raw 19,980,303 vs deduped 10,967,269 = 1.82x inflation, 97.0% cache-read) and is a FLOOR - written before the session closed. No #[allow] was needed to satisfy the panic-free policy. Two measured corrections to the spec/handoff: only ONE corpus file is big-endian, not two; and K3III.PEF has no SubIFD at all - its plane is in IFD0 with NO NewSubfileType tag, which is what makes TIFF's absent-means-0 default load-bearing."
  synced_at: 2026-08-20   # stamped by the orchestrator: this cycle
                         # is ALREADY in the spec (hand-written per
                         # AGENTS.md §15). handback-sync keys idempotence
                         # on this field alone and does NOT check existing
                         # cost.sessions, so without this it would append
                         # a duplicate. See feedback finding 15.
---

# HANDOFF-011: TIFF/IFD reader — bounded, panic-free, cycle-guarded, SubIFD recursion — plus its fuzz target

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-003` for the **build** cycle.

The first spec that actually reads a RAW container. It is also the first to touch
**attacker-influenced binary input**, so `no-panics-on-untrusted-input` stops being
a policy and starts being the work.

## Context the Receiving Agent Needs

### ⚠ Read the toolchain brief's "SECOND `+toolchain` trap" before you fuzz

`cargo fuzz` shells out to a bare `"cargo" "build"` which resolves to Homebrew's
**stable** cargo and rejects `-Zsanitizer`. Even
`~/.cargo/bin/cargo +nightly fuzz run` fails, because the *inner* call is what
breaks. Use:

```bash
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run <target>
```

**Proven at design**, so criteria 4 and 5 are known-achievable: `cargo fuzz init`
works, a target ran **32.9 M execs in 16 s**, and a planted unchecked index was
**caught** — exit 77 plus a crash artifact.

### SPIKE-001's code is DISCARDED — its measurements are not

Do **not** consult that decoder as an implementation; `test-before-implementation`
is why, and retro-fitting tests to working code yields tests that cannot fail.
Reusable facts:

- Sensor-IFD selection: `NewSubfileType == 0 && Photometric == 34892 &&
  SamplesPerPixel == 1` — **never largest dimensions**. `SubIFD2` is a
  full-resolution JPEG preview only **56 px** narrower than the plane.
- Guards required: depth limit, cycle detection on visited offsets, bounds-checked
  payload ranges.
- ⚠ Its version used bounds-check-**then-index** (`buf.get(..)?` then `s[0]`),
  which the lint policy **rejects**. Use `try_into` on the slice. Its "229 lines"
  is an underestimate for exactly that reason — not a target.

### The corpus shapes the tests

Seven files in `tests/corpus/manifest.toml`, read through the **SPEC-002 reader** —
do not hardcode paths, and let absent files skip visibly. Two are **big-endian
(`MM`)** against five `II`. Three are **JPEG-compressed** and must be **rejected
cleanly**, not decoded. The Pentax carries a `BlackLevelRepeatDim` tag dnglab
itself warns is malformed — a free regression fixture the reader must not panic on.

### Scope fence

Container only. `StripOffsets`/`StripByteCounts` are read **as tags**; reading the
strip is STAGE-002, where `DEC-008`'s two-path (`bits % 8`) unpack rule lands.

## Expected Deliverables

1. The IFD reader: byte-order handling, IFD chain walk, SubIFD (tag 330)
   recursion, typed errors on every bounds failure.
2. Depth and cycle guards, with tests that a self-referential SubIFD terminates.
3. **A fuzz target in this change**, seeded from tier-A including truncated and
   malformed inputs.
4. **Evidence the fuzz target works:** plant an unchecked index, show libFuzzer
   catching it (exit 77 + crash artifact), then remove it. Paste both.
5. Tag extraction matching `exiftool` on all 7 corpus files.
6. All nine gates green, output pasted.

## Out of Scope

- Any pixel decode or unpack — STAGE-002.
- Consulting SPIKE-001's decoder as an implementation.
- Hardcoding corpus paths; SPEC-002's reader exists for this.
- Widening the lint exceptions. If the panic-free policy makes something awkward,
  that awkwardness is the constraint working — say so in the handback rather than
  reaching for `#[allow]` (which `SPEC-006`'s gate will reject anyway).

## Return Criteria — how to hand back

1. Paste all nine gates, plus both fuzz directions from deliverable 4.
2. Fill `## Completion` and `handback:`. ⚠ For `tokens_total`: **transcript sums
   double-count ~1.9x** — deduplicate by `message.id` and **say that you did**,
   with cache-read share. See `token-counts-not-comparable`.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Branch `feat/spec-003-ifd-reader` off `main`; commit; do not merge.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-003-ifd-reader`, commit `b79c7ef`. **Not merged**,
  no PR opened — per return criterion 4.
- **Completed at:** 2026-08-20
- **All acceptance criteria met?** **yes** — 7 of 7. Details below.
- **For `verify`:** n/a (this is the build cycle).

### The nine gates, on the tree as committed

```
1. cargo fmt --check                                       exit 0   clean
2. cargo clippy --all-targets --all-features -- -D warnings exit 0  No issues found
3. cargo test --all-features                               exit 0   48 passed, 0 failed
                                                                    (31 lib + 9 corpus + 8 ifd_reader)
4. cargo +1.90.0 check --all-targets --all-features (MSRV) exit 0
5. cargo deny check licenses                               exit 0   licenses ok
6. just lint-red-proof                                     exit 0   control clean (0) -> injection
                                                                    rejected (101) -> all five lints
                                                                    fired at the injected code, and
                                                                    still fire without -D warnings
7. just lint-no-allow  (clippy --lib -F x5)                exit 0
8. scripts/cost-audit.sh                                   exit 0   all shipped specs recorded
9. scripts/decisions-index.sh --check                      exit 0   no INDEX.md committed yet (11 DECs)
```

`just build` exit 0. `just test` still prints the corpus lines **before** the
suite, with no extra flags:

```
$ just test | head -9
corpus: root /Users/.../images (from $IRRADIANCE_CORPUS_DIR)
corpus: present  LEICA-Q2-MONO/L1021223.DNG
corpus: present  LEICA-Q2-MONO/L1026016.DNG
corpus: present  LEICA-Q2-MONO/L1026192.DNG
corpus: present  LEICA-M-MONOCHROM/L1000622.DNG
corpus: present  LEICA-M-MONOCHROM-TYP246/M2462362.DNG
corpus: present  PENTAX-K3III-MONO/K3III.DNG
corpus: present  PENTAX-K3III-MONO/K3III.PEF
corpus: 7/7 present — no tier-B test will skip

$ IRRADIANCE_CORPUS_DIR=/nonexistent just test 2>&1 | grep -c SKIP
8
```

With the corpus absent the tier-B lanes skip and the **hostile-input lane still
runs** — `ifd_rejects_hostile_input`, the guard tests, the truncation sweep and
the byte-corruption sweep need no corpus at all, so a machine with no camera
files still proves the guards.

### Deliverable 4 + 5 — the fuzz target, and BOTH directions

The `+toolchain` trap is real and the brief's fix is the one that works. `just
fuzz` now encodes it, and AGENTS §6 has been corrected — it documented
`~/.cargo/bin/cargo +nightly fuzz run <target>`, which is precisely the
invocation that **fails**, because the inner bare `"cargo" "build"` is what
breaks.

**Direction 1 — fault planted.** An unchecked index in `Container::payload`,
replacing the `.get(at..end).ok_or(Error::Truncated { .. })`:

```rust
// ── DELIBERATE FAULT — SPEC-003 acceptance criterion 5 ──────────
#[allow(clippy::indexing_slicing)]
Ok(&self.data[at..end])
```

```
$ PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
      fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60

INFO:      365 files found in fuzz/corpus/ifd
INFO:       22 files found in fuzz/seeds/ifd
INFO: seed corpus: files: 387 min: 1b max: 2643b total: 106329b rss: 36Mb

thread '<unnamed>' (29178155) panicked at src/ifd.rs:686:26:
range start index 64 out of range for slice of length 26
==24519== ERROR: libFuzzer: deadly signal
SUMMARY: libFuzzer: deadly signal
0x49,0x49,0x2a,0x0,0x8,0x0,0x0,0x0,0x1,0x0,0x11,0x1,0x4,0x0,0xff,0xff,0xff,0xff,0x40,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
II*\000\010\000\000\000\001\000\021\001\004\000\377\377\377\377@\000\000\000\000\000\000\000
artifact_prefix='.../fuzz/artifacts/ifd/'; Test unit written to
    .../fuzz/artifacts/ifd/crash-88173bfac05e9a2e88b5f1c1267ab3b619af5c4e
Base64: SUkqAAgAAAABABEBBAD/////QAAAAAAAAAA=

Error: Fuzz target exited with exit status: 77
```

Worth reading the 26 bytes it found: `II`, version 42, IFD0 at 8, one entry,
tag `0x0111` = **273 `StripOffsets`**, type 4 `LONG`, count `0xFFFFFFFF`, offset
64. That is the `count-overflow` seed almost verbatim — the hand-built tier-A
corpus caught it on the seed pass, before libFuzzer generated anything.

**Direction 2 — fault removed.** `src/ifd.rs` restored byte-for-byte
(`sha256 55fec492e93245fa5e26af4a3d560f51376412a92a3ce0240fe04522e6e176a0`,
verified against the copy taken before planting; `grep -c "DELIBERATE FAULT"` →
0):

```
$ PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
      fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60

#388  INITED cov: 559 ft: 1911 corp: 243/62Kb exec/s: 0 rss: 43Mb
...
Done 12992033 runs in 61 second(s)
exit=0        fuzz/artifacts/ifd/ : 0 files
```

An earlier clean run before the plant did 14,863,561 runs in 61 s. The crash
artifact was archived out of the tree after capture, so the committed tree has
no artifacts and `git status` is clean after a fuzz run (`fuzz/corpus/` and
`fuzz/artifacts/` are gitignored; `fuzz/seeds/` is committed).

### Deliverable 6 — every tag against `exiftool 13.55`, all 7 files

Read through SPEC-002's manifest reader; no path is hardcoded anywhere. The
table below is `irr ifd` output against `exiftool` output, and it is also pinned
as the `EXPECTED` table in `tests/ifd_reader.rs` so it runs on every commit.

| file | bo | IFDs | sensor | W x H | bits | compr | black | white | ActiveArea | crop origin/size | orient | opcodes | malformed |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| L1021223.DNG | II | 4 | #1 sub | 8424x5632 | 14 | 1 | 512 | 16383 | 0 0 5632 8392 | 12 24 / 8368 5584 | 1 | 1,3 | — |
| L1026016.DNG | II | 4 | #1 sub | 8424x5632 | 14 | 1 | 512 | 16383 | 0 0 5632 8392 | 12 24 / 8368 5584 | **6** | 1,3 | — |
| L1026192.DNG | II | 4 | #1 sub | 8424x5632 | 14 | 1 | 512 | 16383 | 0 0 5632 8392 | 12 24 / 8368 5584 | 1 | 1,3 | — |
| L1000622.DNG | II | 2 | #1 sub | 5216x3472 | 16 | 1 | 220 | 16383 | **none** | 2 2 / 5212 3468 | 1 | none | — |
| M2462362.DNG | **MM** | 2 | #1 sub | 5984x4000 | 12 | **7** | 0 | 3750 | none | 4 4 / 5976 3992 | 1 | none | — |
| K3III.DNG | II | 3 | #1 sub | 6304x4224 | 14 | **7** | 64 | 16378 | 34 26 4194 6250 | 28 24 / 6192 4128 | 1 | none | **50713** |
| K3III.PEF | II | **3 chained** | **#0 IFD0** | 6304x4224 | 14 | **65535** | none | none | none | none | 1 | none | — |

Every cell matches `exiftool`. Three points that are not decoration:

- **The rotated frame does rotate.** `L1026016` reports `Orientation 6`
  (Rotate 90 CW) where its two siblings report 1 — same body, same firmware.
  Orientation is per-frame and the reader gets it from the file every time.
- **The layer-0 arithmetic closes exactly** on all four uncompressed planes,
  with no oracle tooling: `8424 x 5632 x 14 = 664,215,552 bits` against
  `StripByteCounts x 8 = 664,215,552`, and `5216 x 3472 x 16 = 289,759,232`
  likewise. Asserted in code on a typed error path (`Sensor::packed_bits`), not
  only in a test.
- **The Pentax's malformed tag costs the tag, not the file.**
  `malformed_tags: [50713]` — `BlackLevelRepeatDim` with `count = 1` where DNG
  requires 2, exactly what `dnglab` warns about. Everything else on that IFD
  still reads, and nothing panics.

Compressed planes are rejected cleanly and stay tag-readable:

```
compression     7 (JPEG — not decodable by PROJ-001)
unpackable      no — compression 7 is not supported by this library
compression     65535 (vendor/other — not decodable by PROJ-001)
unpackable      no — compression 65535 is not supported by this library
```

### Two measured corrections to the spec and handoff

Both were caught by measuring before writing code, and neither changed the
design — but a verifier reading the spec would otherwise expect different
numbers.

1. **"TWO are big-endian (MM)" is wrong — exactly ONE is.** Measured on the
   first four bytes of each file and confirmed with `exiftool -ExifByteOrder`:
   six `II`, one `MM` (`M2462362.DNG`). Big-endian support is still fully
   exercised — the unit tests and the fuzz seeds build every fixture in both
   orders, and `header_reads_both_byte_orders` would catch the byte-swap bug
   that one real file cannot.
2. **`K3III.PEF` has no `SubIFD` at all.** It is the only corpus file with a
   real IFD **chain** (IFD0 → IFD1 → IFD2), its sensor plane lives in **IFD0**,
   and it writes **no `NewSubfileType` tag anywhere**. So TIFF 6.0's
   absent-means-0 default is load-bearing for the selection rule, not
   decorative. It is also the file that exercises the chain walk — the six DNGs
   all have `next = 0` on IFD0.

A third, smaller one: `docs/measured-q2m-dng.md` and the handoff both describe
the selection rule correctly, but the *tag numbers* around it are easy to get
wrong — `50718` is `DefaultScale`, `50719` is `DefaultCropOrigin` and `50720` is
`DefaultCropSize`. Re-derived from the real files' bytes and cross-checked
against `exiftool`'s labels before a line of the reader was written.

### On the panic-free policy making things awkward

It didn't, and that is worth recording because the handoff invited the
complaint. **No `#[allow]` of any policy lint was added anywhere**, and none was
wanted. `.get(range)` + `try_into()` + `checked_mul` is not more verbose than
bounds-check-then-index — it is the same length and it cannot be got wrong the
way SPIKE-001's version could. Two places where the constraint actively improved
the design:

- `Container::payload` computes `count x sizeof(type)` **once**, in `u64`, with
  `checked_mul`, because `arithmetic_side_effects` would not let it be written
  any other way. That single choke point is what the planted fault proved is
  load-bearing.
- `packed_bits()` returns **bits**, not bytes, because writing `/ 8` would have
  forced a decision about the remainder — which is DEC-008's two-path rule and
  belongs to STAGE-002, not here. The lint pushed the scope fence into the type
  signature.

### Cost self-report

- **Tokens (total):** **10,967,269**
- **Estimated USD:** null
- **Duration (minutes):** ~75
- **Source of the number:** transcript sum. `/cost` is a client-side slash
  command the assistant cannot execute, so this is summed from this session's
  own `usage` objects
  (`~/.claude/projects/-Users-...-irradiance/4fbd25f9-c014-4c2a-95ac-e3c241cc21c5.jsonl`).

⚠ **It is DEDUPED BY `message.id`, and I say so because the raw sum is not
comparable to one that isn't.** A transcript writes one JSONL line per content
block and repeats the same `usage` object on each, so a naive sum double-counts
every multi-block message. Measured on this session:

```
usage objects (raw lines) : 122
distinct message.id       : 64
raw sum                   : 19,980,303
DEDUPED sum               : 10,967,269
inflation factor          : 1.82x
```

Composition (deduped): input 128 · output 107,473 · cache-write 226,759 ·
cache-read 10,632,909 — **97.0% cache-read**. This confirms the 1.7x–2.25x range
SPEC-002's two cycles measured (signal `token-counts-not-comparable`), and it is
a **floor**: written before the session closed, so the true figure is somewhat
higher. SPEC-001's `cost.totals` of 51,979,929 is still a raw double-counted sum
and should be re-summed with dedup rather than left standing.

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-011` — `libfuzzer-sys` lives in a separate `fuzz/` crate, outside the
    library's graph. Sanctioned by `no-new-top-level-deps-without-decision` as
    narrowed by DEC-004 rule 4 (dev-only, permissive, not a RAW decoder, DEC
    authored in the same pass). `[dependencies]` is still **empty** and
    `[dev-dependencies]` still holds exactly one entry. ⚠ Recorded limit:
    `cargo deny` evaluates the graph rooted at the library and therefore does
    **not** reach `fuzz/` — its dependencies' licences were checked by hand and
    tabulated in the DEC.

- **Deviations from spec:** none in scope or approach. Four small things done
  that the spec did not ask for, each because leaving it would have shipped a
  known defect:
  1. **Fixed AGENTS §6's fuzz command**, which documented the invocation that
     does not work. §6's stated contract is that nothing in its block is
     unrunnable, so this was a live defect in the one place a cold agent looks.
     Added `just fuzz` / `just fuzz-seeds` to match.
  2. **Added `*.PEF` / `*.pef` to `.gitignore`.** The manifest has indexed a
     37 MB `K3III.PEF` since 2026-08-18 and the never-commit list did not cover
     it — a `git add -A` in a directory holding one would have put it in history
     permanently, which is exactly the failure the RAW-extension block exists
     to prevent.
  3. **Replaced the provenance ledger's placeholder row** — it literally read
     *"(first entry lands with STAGE-001's IFD reader)"*, and this is that
     reader.
  4. **`irr ifd`** as the reader for the parsed tags (AGENTS §11: a parsed tag
     with no consumer is not parsed). It is also what produced the exiftool
     comparison above.

- **Follow-up work identified:**
  - **F1 — the corpus has no file that exercises a multi-strip plane.** All
    seven are single-strip (`RowsPerStrip == ImageLength`), so
    `strip_offsets`/`strip_byte_counts` have never been read with more than one
    element on real data. Synthetic coverage exists; real coverage does not.
    Worth a `[[wanted]]` row rather than a spec.
  - **F2 — `uints()` refuses `RATIONAL`, and a real DNG may use it** for
    `BlackLevel` (the spec permits SHORT/LONG/RATIONAL). None of our seven do —
    the only RATIONAL in any sensor IFD is the Pentax's `DefaultScale`, which
    nothing reads. SPEC-004 owns the wider type model; flagging it so it is not
    discovered as a bug.
  - **F3 — `cargo deny` does not cover `fuzz/`.** Either add a second
    `cargo deny --manifest-path fuzz/Cargo.toml` job, or accept the hand-check
    and say so in the constraint's `enforcement:` field. Recorded in DEC-011
    rather than silently.
  - **F4 — the fuzz target is not in CI.** Deliberate (it needs nightly and
    wall-clock), but that means nothing re-runs it automatically. A short
    `-max_total_time=30` nightly-only job is cheap if wanted.
  - **F5 — `docs/measured-q2m-dng.md`'s tag list is prose, not numbers.** Every
    structural claim in it reproduced exactly against three Q2M frames, but the
    DNG tag *numbers* are nowhere in it, and getting `50719`/`50720` wrong is a
    one-character bug with a plausible-looking result. A numbered table would
    have saved a measurement round here.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing structural; the handoff was unusually well-armed, and the fuzz
   PATH fix being *proven at design* removed what would otherwise have been the
   expensive loop. The one real cost was that two stated corpus facts were
   wrong (two big-endian files; the implied "every file has a SubIFD"), and I
   only found that by measuring all seven files before writing code. That
   measuring pass took ~10 minutes and paid for itself twice over — it is also
   where the `50718`/`50719`/`50720` tag-number trap surfaced.

2. **Was there a constraint or decision that should have been listed but
   wasn't?**
   — `provenance-recorded-per-algorithm` was not named in the handoff's
   "Constraints checked" framing, yet this spec ships the ledger's **first real
   row** and the placeholder was sitting there waiting for it. Also: DEC-004
   rule 4's dev-dependency carve-out clearly covers `libfuzzer-sys`, but the
   handoff mandated a fuzz target without saying that the dependency it
   requires is pre-sanctioned — a more literal implementer could reasonably
   have stopped to ask.

3. **If you did this task again, what would you do differently?**
   — Dump every corpus file's sensor-IFD entries *with tag number, field type
   and count* as the very first action, before reading any prose about them.
   I did that second, after reading the docs, and briefly built a tag map from
   memory that was off by one on `DefaultCropOrigin` — caught immediately
   because the decoded values disagreed with `exiftool`, but the check should
   have come first. The types mattered too: `ActiveArea` is SHORT on the Leicas
   and LONG on the Pentax, `BlackLevel` is SHORT on the Q2M and LONG elsewhere.
   A reader written against one file's types would pass on that file and fail
   on the next.
