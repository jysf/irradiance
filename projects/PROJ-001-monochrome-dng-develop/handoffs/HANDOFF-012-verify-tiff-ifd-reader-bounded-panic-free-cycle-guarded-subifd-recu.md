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
  id: HANDOFF-012
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  tokens_total: 9036505            # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 55
  branch: feat/spec-003-ifd-reader
  pr: null
  completed_at: 2026-08-20         # YYYY-MM-DD
  notes: "Verdict ⚠ PUNCH LIST at 644815f — one ship-blocker, documentation/config only, no code change. tokens_total is a transcript sum DEDUPED BY message.id (113 usage objects, 71 distinct ids, raw 14,592,470 vs deduped 9,036,505 = 1.61x); 97.9% cache-read; a FLOOR, written before the session closed."
  synced_at: 2026-08-20   # stamped by the orchestrator: this cycle
                         # is ALREADY in the spec (hand-written per
                         # AGENTS.md §15). handback-sync keys idempotence
                         # on this field alone and does NOT check existing
                         # cost.sessions, so without this it would append
                         # a duplicate. See feedback finding 15.
---

# HANDOFF-012: TIFF/IFD reader — bounded, panic-free, cycle-guarded, SubIFD recursion — plus its fuzz target

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-003` for the **verify** cycle, at
`d867403` (implementation `b79c7ef`). Independent session.

This is the first spec that parses attacker-influenced binary input, so the
panic-free constraint is now load-bearing rather than aspirational.

## Context the Receiving Agent Needs

### Already reconciled — don't just repeat

- **All nine gates green**, run by the orchestrator. 48 tests (31 lib + 9 corpus +
  8 ifd_reader). No `#[allow]` of any policy lint anywhere in `src/`. No fuzz
  artifacts left behind.
- **Criterion 5 verified independently, with a harder fault than the build's.**
  The build's planted fault was an unchecked index — but the lint policy *catches
  that at compile time* (`indexing_slicing`, `src/ifd.rs:704`, level from
  `src/lib.rs:48`), so it never reaches the fuzzer on a clean tree. The
  orchestrator instead planted a **lint-clean** `split_at(end)` that clippy passes,
  and libFuzzer found it: `deadly signal`, crash artifact written. So the fuzz
  target genuinely works on faults the lint policy cannot see — which is precisely
  the gap it exists to cover.

### ⚠ Two facts in MY spec were wrong; the build found both

Neither changed the design, but the record was wrong and a verifier would expect
different numbers:

1. **Byte order: SIX `II`, ONE `MM`** — not "two big-endian" as
   `HANDOFF-011`/the spec said. Confirmed on raw header bytes across all 7 files.
   Only `M2462362.DNG` is `MM`.
2. **`K3III.PEF` has NO SubIFD at all** — zero `SubIFD` mentions, no
   `NewSubfileType` tag, plane in `IFD0`, and it is the only file with a real IFD
   *chain* (`IFD0→IFD1→IFD2`). Confirmed with exiftool. So TIFF's **absent-means-0**
   default for `NewSubfileType` is load-bearing, not decorative — worth checking
   that the reader relies on it deliberately rather than by luck.

**Both durable docs still carry my wrong numbers** (`docs/conformance-matrix.md`
and the spec). The orchestrator corrects them at ship; flag it if you see the
error propagated anywhere else.

### What deserves scrutiny

1. **The guards.** Depth limit and cycle detection are the difference between a
   hostile file and an infinite loop. Are they on *every* recursion path, including
   the IFD *chain* (`next` pointers), not just SubIFD descent? The PEF is the only
   file with a real chain, so chain-walking has exactly one real-world test.
2. **`Error::UnsupportedCompression` on the three JPEG files** — they must be
   rejected but stay **tag-readable**. Is the boundary right?
3. **`DEC-011`** puts `libfuzzer-sys` in a separate `fuzz/` crate so
   `[dependencies]` stays empty. ⚠ The build discloses that **`cargo deny` does
   not reach `fuzz/`** — hand-checked only. Is that acceptable, or does the licence
   gate now have a hole?
4. **Disclosed follow-up: no corpus file exercises a multi-strip plane.** All four
   uncompressed planes are single-strip. Real gap or acceptable for STAGE-001?
5. `packed_bits()` returns *bits* rather than bytes, deliberately, so
   `DEC-008`'s remainder decision stays in STAGE-002. Right call?

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the fuzz red-proof yourself (check #9). ⚠ Two traps:
- `cargo fuzz` needs the rustup shim **first on PATH** —
  `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd`
  (see the toolchain brief's "second `+toolchain` trap").
- **An unchecked index will not reach the fuzzer** — the lint policy rejects it at
  compile time. Plant something lint-clean (`split_at`, `unreachable!`, `assert!`)
  or you will be testing clippy, not the fuzzer.

**Label every finding ship-blocking or follow-up.** A panic reachable from
attacker-controlled bytes is ship-blocking. A missing corpus shape is a follow-up.

If **APPROVED**, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- Pixel decode / unpack — STAGE-002 and `DEC-008`.
- Re-opening the `-F` gate, `DEC-009`, or the corpus manifest design.

## Return Criteria — how to hand back

1. Paste the gates and both fuzz directions you ran yourself.
2. Fill `## Completion` and `handback:`. ⚠ `tokens_total`: transcript sums
   **double-count ~1.9x** — deduplicate by `message.id` and **say so**, with
   cache-read share.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Commit on `feat/spec-003-ifd-reader`; do not merge.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-003-ifd-reader` at `644815f` (implementation
  `b79c7ef`). **Not merged**, no PR opened — per return criterion 4.
- **Completed at:** 2026-08-20
- **All acceptance criteria met?** **yes, 7 of 7** — every one re-checked against
  the tree as committed, not against the build's report. The punch list below is
  about *records*, not about the reader.
- **For `verify`:** ⚠ **PUNCH LIST** at `644815f`. One ship-blocker, and it is
  documentation + config only — no line of `src/` changes.

---

## The gates, run by me on `644815f`

`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`, 7/7
corpus files present, `git status` clean before and after.

```
1. cargo fmt --check                                        exit 0
2. cargo clippy --all-targets --all-features -- -D warnings exit 0
3. cargo test --all-features                                exit 0   48 passed, 0 failed
                                                                     (31 lib + 9 corpus + 8 ifd_reader)
4. ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features    exit 0   (MSRV)
5. cargo deny check licenses                                exit 0   "licenses ok"
6. just lint-red-proof                                      exit 0   control clean (0) → injection
                                                                     rejected (101) → all five lints
                                                                     fired at the injected lines, and
                                                                     still fire without CI's -D warnings
7. just lint-no-allow  (clippy --lib -F ×5)                 exit 0
8. scripts/cost-audit.sh                                    exit 0   all shipped specs recorded
9. scripts/decisions-index.sh --check                       exit 0   no INDEX.md committed (11 DECs)
```

Nine green, confirming the build's report. Two extras I ran that the build did
not:

- `just fuzz-seeds` then `git status` → **empty**. The committed seed corpus
  regenerates byte-identically from `tests/support/tiff.rs`, so the "a fixture
  added to the hostile-input test lands in the fuzz lane too" claim in
  `app.just:108` is true and not merely intended.
- `just decisions-audit --changed main` → DEC-000/004/008/009/010/011 govern the
  touched paths; I read each against the change and found no drift (§15 check 3).
  ⚠ Note for whoever runs this next: bare `--changed` compares *uncommitted*
  changes and printed "No changed files in scope" on this clean tree — the base
  ref is required to see anything.

⚠ **Trap the brief did not mention, hit on gate 4.** `cargo +1.90.0 check` fails
with `error: no such command: +1.90.0` — the *first* `+toolchain` trap, same root
cause as the fuzz one. Homebrew's cargo is first on PATH and does not understand
`+toolchain` at all. `~/.cargo/bin/cargo +1.90.0 …` is what works. The MSRV gate
is the one gate in the nine with no `just` recipe wrapping it, so it is the one
that hands you the raw command. Worth a recipe.

---

## Both fuzz directions, run by me (§15 check 9)

Both traps in the handoff are real and both bit exactly as advertised. The PATH
fix is load-bearing; `just fuzz` encodes it correctly.

### Direction 1 — fault planted → RED

I planted a **different** fault from the build's, at a **different site**, so the
proof covers a path the build's did not. The build faulted `Container::payload`
(the payload path). I faulted `Container::read_ifd` (the **walk** path), replacing
the bounds-checked entry table fetch at `src/ifd.rs:649-655`:

```rust
// ── DELIBERATE FAULT — verify cycle, AGENTS.md §15 check 9 ──────────
let (head, _) = self.data.split_at(entries_end);
let (_, table) = head.split_at(entries_at);
```

**First, the negative control the second trap demands** — proof this fault is
invisible to the lint policy, so the fuzzer is the only thing that can catch it:

```
$ just lint            # clippy --all-targets --all-features -D warnings, + fmt
just lint exit=0
$ just lint-no-allow   # clippy --lib -F unwrap/expect/indexing/panic/arithmetic
just lint-no-allow exit=0
```

Both green **with the fault in place**. `split_at` is not `indexing_slicing`, not
`panic!`, not `arithmetic_side_effects` — the whole policy waves it through, and
it still panics `mid > len`. That is the gap the fuzz target exists to cover, and
it is now measured rather than argued.

**Then the fuzzer — with ZERO seed files**, pointed at an empty corpus directory,
so libFuzzer had to synthesise the crashing input itself rather than find it on
the seed pass:

```
$ PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
      /tmp/.../empty-corpus -- -max_total_time=60

INFO:        0 files found in /tmp/.../empty-corpus
#38889  NEW    cov: 169 ft: 181 corp: 12/281b ... MS: 5 ChangeBit-CrossOver-ChangeBit-ChangeBit-CMP-

thread '<unnamed>' (29779582) panicked at src/ifd.rs:655:35:
mid > len
==21491== ERROR: libFuzzer: deadly signal
SUMMARY: libFuzzer: deadly signal
artifact_prefix='.../fuzz/artifacts/ifd/'; Test unit written to
    .../fuzz/artifacts/ifd/crash-decd0828da8174bbbcaa9400a21c59c179fa53f3
Base64: SUkqAE8AAAAAIQAAAAA...
EXIT=1
```

From an empty corpus, in ~38,900 executions. The 125-byte input it built:

```
00000000: 4949 2a00 4f00 0000  II*.O...      II, version 42, IFD0 @ 0x4F = 79
0000004e: 71d0 196d            q..m          entry count @79 = 0x19d0 = 6608
```

6608 entries × 12 = 79,296 bytes of table demanded from a 125-byte file →
`split_at(79377)` on a 125-byte slice → panic. libFuzzer discovered the TIFF magic,
a valid version word, a plausible IFD0 offset **and** a hostile entry count with no
seed corpus at all. This is a strictly stronger red than the build's, whose
crashing input was one of our own seeds.

### Direction 2 — fault removed → GREEN

`src/ifd.rs` restored from the pre-plant copy and verified byte-for-byte:
`sha256 9c965c4842e82450109b7b5d3b09bd5ca93509030607de70c382037acc21b655`,
`grep -c "DELIBERATE FAULT"` → **0**, `git diff` → empty.

```
$ just fuzz 60
INFO:      469 files found in fuzz/corpus/ifd
INFO:       22 files found in fuzz/seeds/ifd
INFO: seed corpus: files: 491 min: 1b max: 3371b total: 167322b
Done 16832041 runs in 61 second(s)
EXIT=0        fuzz/artifacts/ifd/ : 0 files
```

**16,832,041 executions, zero artifacts, exit 0.** The crash artifact from
direction 1 was archived out of the tree; `fuzz/artifacts/ifd/` is empty and
`git status` is clean.

---

## The five things flagged for scrutiny

### 1. Guards on **every** recursion path, including the chain's `next` pointers — YES

Verified by reading, not inferred. `walk_chain` (`src/ifd.rs:584`) threads **one**
`visited: &mut Vec<u32>` through the entire walk rather than one per chain, and
the chain loop checks-and-pushes before every read (`:601`, `:604`). So the guard
covers all three shapes with one mechanism: a chain that points at itself, a
SubIFD that points at itself, and a SubIFD pointing back at an ancestor. Depth is
checked at function entry (`:591`) and chain siblings correctly inherit their
parent's depth rather than incrementing — which is right, a chain is not nesting.
`MAX_IFDS` (`:606`) bounds the acyclic-but-enormous case that the cycle guard
alone would let run. Native stack depth is bounded by `MAX_IFD_DEPTH = 8`, which
is what makes writing the SubIFD descent as recursion safe.

Four unit tests cover the shapes **separately**, which is the part that matters —
`a_chain_that_points_at_itself_terminates` (`src/ifd.rs:1210`),
`a_self_referential_subifd_terminates` (`:1187`), `a_two_hop_subifd_cycle_terminates`
(`:1198`), and `a_long_chain_stops_at_the_ifd_limit` (`:1240`). Best of them is
`nesting_past_the_depth_limit_terminates` (`:1222`), which puts every nested IFD at
a **distinct offset** specifically so the cycle guard cannot be what stops it. A
depth test that a cycle guard would also pass is not a depth test; this one isn't.

Real-world chain coverage is one file, as the handoff says. Confirmed live:

```
$ irr ifd .../PENTAX-K3III-MONO/K3III.PEF
ifds            3
  #0 @8 depth 0 chain — 21 entries, next 122890
  #1 @122890 depth 0 chain — 8 entries, next 123008
  #2 @123008 depth 0 chain — 8 entries, next 0
sensor_matches  [0]
```

One real chain, and the absent-`NewSubfileType` default is visibly what finds the
plane in `IFD0`. Thin, but it is the only chain in existence in this corpus and
the synthetic tests cover the hostile shapes the real file cannot.

### 2. The `UnsupportedCompression` boundary — right, and structurally so

`Container::sensor()` reads **tags only** and never dereferences `StripOffsets`,
so it succeeds on compressed files; rejection is a separate, explicit
`Sensor::require_uncompressed()` (`src/ifd.rs:477`). That is the correct seam: the
*container* has no opinion about compression, the *unpacker* does, and STAGE-002
inherits a clean boundary instead of a special case.
`tests/ifd_reader.rs:373` asserts both halves in one match — the typed error
carries the code, **and** `sensor.width` still reads afterwards. Verified live on
both compressed shapes (`compression 7 (JPEG …)`, `compression 65535
(vendor/other …)`), with every other tag still printing.

One correction to the framing, which is a real error and not a nitpick — see
**FU-2**: it is **two JPEG + one Pentax PEF**, not three JPEG.

### 3. `cargo deny` does not reach `fuzz/` — **a hole, and it is the ship-blocker**

Not because the limit is undisclosable. DEC-011 discloses it honestly and that is
to its credit. It is a hole because **(a)** the hand-check that replaces the gate
got the licences wrong on its first and only use, and **(b)** the gate reaches
`fuzz/` fine with one flag, so the premise that a hand-check was necessary does
not hold. Full detail in **SB-1**.

### 4. No corpus file exercises a multi-strip plane — follow-up, and worse than "untested"

Confirmed: all seven are single-strip. See **FU-6** — the shape is not merely
uncovered, it is *asserted*, in three places.

### 5. `packed_bits()` returning bits, not bytes — right call, keep it

Returning bytes forces `/ 8`, which forces a remainder decision, which **is**
DEC-008's `bits % 8` two-path rule and belongs to STAGE-002. Returning bits states
the invariant (`w × h × bits == StripByteCounts × 8`) while deciding nothing, and
it is the only form correct for *both* DEC-008 branches — a 14-bit plane and a
16-bit plane agree in bits and disagree about what a byte count means. The build's
note that the lint policy pushed the scope fence into the type signature is a fair
description of what happened.

Two caveats, neither blocking: `Error::ValueOverflow` from `packed_bits()` always
reports `tag: TAG_IMAGE_WIDTH` (`src/ifd.rs:498`) even when height or bit depth
overflowed; and the identity does not generalise to multi-strip (FU-6).

---

## Punch list

### 🚫 SHIP-BLOCKING

**SB-1 — `DEC-011`'s licence table is wrong for the one crate it exists to
sanction, and it is the sole enforcement of a `blocking` constraint.**
`decisions/DEC-011-libfuzzer-sys-in-a-separate-fuzz-crate.md:81`, `:42`, `:85`.

Three separate defects, measured:

**(a) The declared licence is wrong.** DEC-011:81 records `libfuzzer-sys` as
`MIT OR Apache-2.0`. The crate declares:

```
libfuzzer-sys-0.4.13/Cargo.toml:36:  license = "(MIT OR Apache-2.0) AND NCSA"
```

`AND` is conjunctive — the NCSA terms apply **in addition**, not as an
alternative. `NCSA` appears nowhere in DEC-011, and it is **not** in `deny.toml`'s
`allow` list (`deny.toml:21-30`). So DEC-011:85 — *"All are already in
`deny.toml`'s `allow` list … so no exception entry was needed and none was
added"* — is false. Conversely, the `Apache-2.0 WITH LLVM-exception` the DEC says
was needed for the vendored C++ is reported by `cargo deny` as an **unencountered**
allowance: nothing in the graph declares it.

**(b) The crate enumeration is incomplete, and omits the one crate that mentions
LGPL.** DEC-011:42 enumerates *"`arbitrary`, `cc`, `libc`, `jobserver`, `shlex`
and `find-msvc-tools`"*. The real graph (`cargo metadata --manifest-path
fuzz/Cargo.toml`) also contains `cfg-if 1.0.4`, `getrandom 0.4.3`, and:

```
r-efi   6.0.0   MIT OR Apache-2.0 OR LGPL-2.1-or-later
```

Disjunctive, so a permissive option exists and the constraint is not violated —
but `no-copyleft-dependencies` (`guidance/constraints.yaml:41-45`, severity
**blocking**) names LGPL explicitly and says *"including dev-dependencies"*. A
hand-check whose table omits the only crate in the graph carrying LGPL in its
expression is not the check it claims to be.

**(c) The gate reaches `fuzz/` after all.** The premise that a hand-check was
necessary is false. Measured:

```
$ cargo deny --manifest-path fuzz/Cargo.toml check licenses
error[unlicensed]: irradiance-fuzz = 0.0.0 is unlicensed
error[rejected]: failed to satisfy license requirements
  ┌─ libfuzzer-sys-0.4.13/Cargo.toml:36:36
  │ license = "(MIT OR Apache-2.0) AND NCSA"
  │                                    rejected: license is not explicitly allowed
licenses FAILED
```

It runs, and it catches exactly what the hand-check missed. (`irradiance-fuzz`
itself has no `license` field — `fuzz/Cargo.toml:1-6` — which is its own finding
from the same run.)

**Substance is fine and I want to be plain about that:** NCSA is permissive
(OSI-approved, FSF Free/Libre), every crate in the graph can be taken under a
permissive licence, and nothing copyleft is linked. **The record is what is
wrong**, on a blocking constraint, in the document that is the only thing standing
in for an automated gate — in a repo whose corpus manifest says licence provenance
*"cannot be reconstructed later"* and is *"the one field with no second chance"*,
and whose provenance ledger exists specifically to make the permissive claim
*defensible rather than merely asserted*. A defensible claim resting on a table
with a wrong row is an asserted one.

Why ship-blocking rather than follow-up: SPEC-003 is where this gap was
introduced, DEC-011 is the compensating control, and shipping it means the next
dependency added to `fuzz/` inherits a process already demonstrated to fail. It is
also cheap — **no code change**:

1. `DEC-011:81` — correct the row to `(MIT OR Apache-2.0) AND NCSA`, add the three
   missing crates, and note `r-efi`'s LGPL option and why the disjunction is fine.
2. `deny.toml:21-30` — add `"NCSA"` to `allow`, or a per-crate `exceptions` entry
   for `libfuzzer-sys` with the reason. (Also reconsider the now-unencountered
   `Apache-2.0 WITH LLVM-exception`.)
3. `fuzz/Cargo.toml` — add `license = "MIT OR Apache-2.0"`; a package with no
   licence field is `unlicensed` to the gate.
4. `guidance/constraints.yaml:45` — `enforcement:` reads *"cargo deny check
   licenses in CI"*, which is now inaccurate. Either add
   `cargo deny --manifest-path fuzz/Cargo.toml check licenses` as a second gate
   (recommended — it is one line and it already works) or say in the field that
   `fuzz/` is hand-checked under DEC-011.

This also closes the build's own **F3**, which proposed exactly this and left it
open.

### 📋 FOLLOW-UP

**FU-1 — the byte-order error is propagated in one place the handoff does not
name, and *not* in the place it does.**
- `CHANGELOG.md:31` — *"all **7** corpus files (5 `II` / 1 `MM` / 1 PEF)"*.
  This conflates byte order with container format: `K3III.PEF` is `II` too. I
  measured the raw header bytes of all seven myself — **6 `II`, 1 `MM`**
  (`M2462362.DNG`), matching the build.
- `SPEC-003:262` — *"Two are big-endian (`MM`) where five are `II`"* — the one the
  handoff names.
- ⚠ **`docs/conformance-matrix.md` does not carry this error.** I checked the
  whole file; it makes no byte-order claim about the corpus at all. The handoff's
  pointer at it is wrong — what *is* wrong in that file is FU-4, which is a bigger
  problem than the one it was blamed for.

**FU-2 — a THIRD wrong fact in the same spec paragraph, missed at build and
propagated into HANDOFF-012 itself.**
`SPEC-003:262-263` — *"**Three** are JPEG-compressed and must be rejected
cleanly."* Only **two** are JPEG (`Compression == 7`: `M2462362.DNG`,
`K3III.DNG`). The third, `K3III.PEF`, is `Compression == 65535` — a vendor-private
Pentax scheme, not JPEG. Verified live: `irr ifd` reports
`compression 65535 (vendor/other — not decodable by PROJ-001)`.
The code and the changelog both get this right (`src/ifd.rs:474-476`,
`CHANGELOG.md:34`); the spec does not, and **HANDOFF-012's own scrutiny item 2
repeats it** ("three JPEG files must be rejected"). It matters because "three
JPEG" implies one unsupported-compression class where there are two, and PROJ-003
scopes lossless-JPEG and PEF decompression as different problems.

**FU-3 — the "full-resolution SubIFD" phrasing carries the PEF error into two
durable docs.**
- `SPEC-003:174` (acceptance criterion 6) — *"the reader reaches the
  full-resolution **SubIFD**"*.
- `projects/PROJ-001-.../stages/STAGE-001-...md:58` — *"walk its IFD tree to the
  full-resolution **SubIFD**"*.

`K3III.PEF` has no SubIFD; its plane is `IFD0`. Read literally, AC6 is
unsatisfiable on 1 of the 7 files it names. The implementation and the tests get
this right throughout — `sensor_ifd`, `ifd_reaches_sensor_plane` — so this is
purely the prose. "sensor IFD" is the phrase that is true for all seven.

**FU-4 — `docs/conformance-matrix.md` is stale in the exact way its own opening
rule forbids.**
Line 3: *"**Every camera gets a row the day it is known, files or not.**"* Line 5:
the coverage column is what turns *"we never thought about X"* into a declared
row. AGENTS §15 (ship) requires confirming this file is current, and calls a
camera that gained support without gaining a row *"the 'unread field' defect in
its most expensive form."*

The matrix (`:10-19`) still carries **one** monochrome row — Leica Q2 Monochrom.
Three further bodies are held, manifested, and now read end-to-end by this spec,
with none:
- **Leica M Monochrom** — 16-bit, uncompressed, `L1000622.DNG` (also DEC-008's
  16-bit branch evidence)
- **Leica M Monochrom Typ 246** — 12-bit, `MM`, JPEG-compressed, `M2462362.DNG`
  (the corpus's only big-endian file)
- **Pentax K-3 III Monochrome** — `K3III.DNG` (14-bit, JPEG) **and** `K3III.PEF`
  (14-bit, compression 65535) — the only IFD chain, and the malformed
  `BlackLevelRepeatDim` fixture

Related, same file: `:21` *"⚠ PROJ-001 validates against ONE camera"* is no longer
true at the container level. This spec validates the reader against **four bodies
and seven files** — which is precisely the *"prove the **container reader** is not
Leica-shaped, which is STAGE-001's job"* that `:34-35` describes as the cheap half
of the value. That job is done and the section still says it is pending.
Also `:46-49` calls the Pentax malformed tag *"a tier-A regression fixture"*; it is
tier B (a 37 MB uncommitted file).

**FU-5 — the malformed-tag policy is asymmetric, and the rule is not stated.**
`src/ifd.rs:801-819` (`array()`) *tolerates* a present-but-wrong-shape tag: the
value is dropped, the tag number is recorded in `malformed_tags`, the file still
reads. That is what keeps the Pentax's one-element `BlackLevelRepeatDim` from
costing the file, and it is the right instinct.

But `src/ifd.rs:670-676` (`sub_ifd_offsets_of_last` → `uints`) makes the **same
class** of defect on tag 330 fatal to the whole container: a `SubIFDs` entry with
an unreadable field type (`RATIONAL`, a signed type) or a payload offset past EOF
aborts `Container::parse` entirely through `?`, rather than yielding an IFD0-only
walk with `330` recorded as malformed. Both cases are "an optional tag is present
but shaped wrong"; the module applies opposite policies and states no rule for
which applies when.

Not a panic, not wrong today — no corpus file trips it, and strict-is-safe is a
defensible default. Flagging it because SPEC-004 widens the type model directly on
top of `uints()`, and the boundary should be *decided* before it is inherited.
Adjacent to the build's **F2**.

**FU-6 — the multi-strip gap: the single-strip shape is not merely untested, it is
asserted in three places.**
Confirmed — all seven corpus files are single-strip. But:
- `tests/ifd_reader.rs:448` — `assert_eq!(sensor.strip_offsets.len(), 1, …)`
- `tests/ifd_reader.rs:443` — `assert_eq!(sensor.rows_per_strip, Some(expect.height), …)`
- `tests/ifd_reader.rs:352` — `strip_byte_counts` compared against `vec![<one value>]`

So a multi-strip corpus file arriving later **fails these tests** rather than
exercising a new path. That is the right way round — loud, not silent — but it
means the `[[wanted]]` row the build proposed should say so explicitly, or someone
will read the failure as a reader bug. `Sensor::packed_bits()`'s layer-0 identity
also compares against a single `StripByteCounts` and does not generalise. Agreed
follow-up, exactly as the handoff frames it; no corpus file is needed to *fix*
anything today.

**FU-7 — "no `#[allow]` of any policy lint anywhere in `src/`" is imprecise.**
There are two, both on `#[cfg(test)] mod tests`: `src/lib.rs:223` (pre-existing,
SPEC-001) and `src/ifd.rs:934` (new here). Both are the sanctioned exception, both
are invisible to `just lint-no-allow` because it scopes to `--lib` deliberately
(`app.just:80-83`), and I confirmed the policy is doing real work — my planted
fault had to be `split_at` precisely because an unchecked index cannot compile.
The accurate sentence is *"no `#[allow]` on any **non-test** path in `src/`"*.
Worth correcting only because the imprecise version is what a future agent greps
for, and `src/lib.rs:34-35` already states the limitation correctly.

**FU-8 — the MSRV gate has no `just` recipe.** Eight of the nine gates are one
`just` word; gate 4 hands you a raw `cargo +1.90.0 …` that fails with
`error: no such command: +1.90.0` under the default PATH — the same trap class the
repo has already documented twice for `cargo fuzz`. `just msrv` wrapping
`~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features`, plus a line in
AGENTS §6, closes the last uncovered one.

---

### Cost self-report

Mirrors the `handback:` front-matter.

- **Tokens (total):** **9,036,505**
- **Estimated USD:** null — no published rate applied; the build left this null
  too and a made-up number is worse than none.
- **Duration (minutes):** ~55
- **Source of the number:** transcript sum. `/cost` is a client-side slash command
  the assistant cannot execute, so this is summed from this session's own `usage`
  objects (`~/.claude/projects/-Users-…-irradiance/a604e646-….jsonl`).

⚠ **DEDUPED BY `message.id`, and I say so.** A transcript writes one JSONL line per
content block and repeats the same `usage` object on each, so a naive sum
double-counts every multi-block message:

```
usage objects (raw lines) : 113
distinct message.id       : 71
raw sum                   : 14,592,470
DEDUPED sum               :  9,036,505
inflation factor          : 1.61x
```

Composition (deduped): input 142 · output 34,174 · cache-write 159,902 ·
cache-read **8,842,287** — **97.9% cache-read**. It is a **floor**: computed before
the session closed, so the true figure is somewhat higher.

The 1.61× inflation sits just under SPEC-002's measured 1.7×–2.25× band and the
build's 1.82×, which is consistent with signal `token-counts-not-comparable`: the
factor tracks how block-heavy a session is, so it is not a constant and a single
"×1.9" correction should not be applied to anyone's raw figure. SPEC-001's
`cost.totals` of 51,979,929 is still a raw double-counted sum and should be
re-summed with dedup — the build flagged this and it is still true.

### Drift and new artifacts

- **New decisions emitted:** none. SB-1 amends `DEC-011` rather than superseding
  it — the decision itself (a separate `fuzz/` crate, `[dependencies]` stays
  empty) is sound and I would keep it; only its licence table and its claim about
  gate reachability are wrong.
- **Deviations from spec:** none found. All 7 acceptance criteria met. The four
  unrequested things the build did (AGENTS §6 fuzz-command fix, `*.PEF` in
  `.gitignore`, the provenance-ledger row, `irr ifd`) are each defensible and I
  would have flagged their absence.
- **Follow-up work identified:** FU-1 … FU-8 above. Of the build's own five,
  **F3 is promoted to ship-blocking** (SB-1); F1 becomes FU-6 with the added point
  that the shape is asserted, not merely uncovered; F2 is broadened into FU-5;
  F4 (fuzz not in CI) and F5 (`measured-q2m-dng.md` has no tag numbers) I confirm
  and leave as stated.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing that slowed me down; the handoff was well-armed and both fuzz traps
   were called correctly. What cost me time was a trap it *didn't* call: gate 4's
   `cargo +1.90.0` fails under the default PATH for the identical reason
   `cargo fuzz` does, and it is the only one of the nine gates with no `just`
   recipe hiding the fix (FU-8). More usefully: the handoff pointed at
   `docs/conformance-matrix.md` as carrying the two wrong corpus facts, and it
   doesn't — checking that pointer is what led me to the real defect in that file,
   which is that four of the seven held files belong to three bodies with no row.
   A wrong pointer that provokes a real read is not the worst kind of wrong.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `no-copyleft-dependencies` was not in the handoff's scrutiny list as a
   *constraint*, only as scrutiny item 3's question about `cargo deny`'s reach.
   Framing it as "is this hole acceptable?" invites a yes/no about the hole and
   not a re-check of the table the hole made load-bearing — which is where the
   actual defect was. When a gate is disclosed as not running, the right question
   is not "is that acceptable" but "then show me the substitute working."

3. **If you did this task again, what would you do differently?**
   — Run `cargo deny --manifest-path fuzz/Cargo.toml` in the first five minutes.
   I reached it by reasoning about whether the disclosed hole was acceptable, when
   one command answers both that question and the deeper one. More generally: for
   any "we hand-checked this because the tool can't reach it," try the tool first.
   It took one flag. I would also plant the fuzz fault before reading the reader
   rather than after — the negative control (`just lint` green with the fault in)
   is the single most informative measurement in this cycle and it takes two
   minutes.
