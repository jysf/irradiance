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
  id: HANDOFF-013
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
  tokens_total: 9733599            # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 60
  branch: feat/spec-003-ifd-reader
  pr: null
  completed_at: 2026-08-20         # YYYY-MM-DD
  notes: "SB-1 closed and the fuzz licence gate WIRED (just deny-fuzz + CI job licenses-fuzz), red-proofed both ways: exception removed -> exit 4, license field removed -> exit 4, honest tree -> exit 0. TEN gates green, pasted. NO src/ change - git status shows no file under src/. Three factual corrections re-measured on the files, not transcribed; the conformance matrix needed THREE rows, not the one named. New DEC-012 states the malformed-tag rule (strict on structure, tolerant on shape). tokens_total is a transcript sum DEDUPED BY message.id: 127 usage objects, 74 distinct ids, raw 17,087,494 vs deduped 9,733,599 = 1.76x, 97.9% cache-read, a FLOOR. Six measured factors now span 1.61x-2.25x - NOT a constant, no fixed correction is valid on any raw figure. ⚠ ORCHESTRATOR: just handback-sync SPEC-003 would DOUBLE-COUNT HANDOFF-011 and HANDOFF-012 - measured with --dry-run; both have hand-written sessions in the spec but synced_at: null, and the script keys idempotence on synced_at alone."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-013: <Task Title — same as the spec's title>

## Delegation Summary

Second build cycle on `SPEC-003` — the punch-list round. Verify returned
⚠ PUNCH LIST with **one ship-blocker**, and it is **documentation and config only:
no `src/` change**. The reader itself was found sound.

## Context the Receiving Agent Needs

### 🚫 SB-1 — the licence record is wrong on a blocking constraint

All three parts **independently reproduced by the orchestrator**:

1. **`libfuzzer-sys` declares `(MIT OR Apache-2.0) AND NCSA`** — verified in its
   own `Cargo.toml`. `AND` is **conjunctive**. `DEC-011:81` records it as
   `MIT OR Apache-2.0`, which is wrong, and `DEC-011:85`'s claim that "no
   exception entry was needed" is therefore false.
2. **NCSA is not in `deny.toml`'s allow list** (MIT, Apache-2.0, Apache-2.0 WITH
   LLVM-exception, BSD-2/3-Clause, Zlib, 0BSD, Unicode-3.0).
3. **The premise everyone accepted is false.**
   `cargo deny --manifest-path fuzz/Cargo.toml check licenses` **runs** — and
   **FAILS** today. The gate was never absent; it was never invoked.

Substance is fine: NCSA is permissive and nothing copyleft is linked. **The record
is wrong**, on `no-copyleft-dependencies`, in the document standing in for a gate
that turns out to exist. That is why it blocks.

Also missing from `DEC-011:42`'s table: **`cfg-if`, `getrandom`, and `r-efi 6.0.0`**
— the last being `MIT OR Apache-2.0 OR LGPL-2.1-or-later`, the only crate in the
graph that mentions LGPL at all. Disjunctive, so permissive is selectable and
nothing is wrong — but an unrecorded LGPL mention in *this* repo is precisely what
the ledger exists to surface.

### The fix

- `DEC-011` — correct the licence table; add the three missing crates; retract the
  "no exception needed" claim.
- `deny.toml` — allow `NCSA` (or a targeted per-crate exception for
  `libfuzzer-sys`; pick one and say why).
- `fuzz/Cargo.toml` — **currently has no `license` field at all.** Add one.
- `guidance/constraints.yaml:45` — the `enforcement:` field is now inaccurate.
- **Wire the fuzz licence check as a real gate**, since it works. That converts a
  hand-check into a mechanism, which is the whole point.

### Three factual corrections (two are the orchestrator's errors)

1. **"three JPEG-compressed" is wrong.** Measured: `M2462362.DNG` and `K3III.DNG`
   are compression **7** (JPEG); `K3III.PEF` is **65535** (vendor-private). The
   spec and `HANDOFF-012` both say three. `CHANGELOG.md:34` already says it
   correctly — fix the spec to match.
2. **`CHANGELOG.md:31` conflates byte order with container:** "5 `II` / 1 `MM` /
   1 PEF". The PEF is **`II` too** — it is 6 `II` / 1 `MM` across 7 files.
3. **`docs/conformance-matrix.md` has no row for the Leica M Monochrom
   (Typ 246)**, against that file's own opening rule that every camera gets a row
   the day it is known. Three bodies now read end-to-end; make the matrix say so.

### Also in scope

- **A third `+toolchain` trap:** bare `cargo +1.90.0` fails with `no such command`,
  and MSRV is **the only gate with no `just` recipe**. Add one, and record the trap
  in `guidance/toolchain-brief.md`.
- **`array()` tolerates malformed tags while `SubIFDs` via `uints()` is fatal to
  the whole container**, with no stated rule. Either state the rule or make them
  consistent — a reader that survives one malformed tag and dies on another needs
  to say which is which on purpose.

## Expected Deliverables

1. SB-1 closed: `DEC-011`, `deny.toml`, `fuzz/Cargo.toml`,
   `constraints.yaml:45`, **and a working licence gate over `fuzz/`**. Paste the
   gate passing.
2. The three factual corrections above.
3. A `just msrv` recipe; the third `+toolchain` trap in the toolchain brief.
4. The malformed-tag rule stated (or the behaviour made consistent).
5. All nine gates green **plus** the new fuzz-licence gate — output pasted.
6. ⚠ **No `src/` change is expected.** If you find yourself editing the reader,
   stop and say why in the handback — verify found it sound.

## Out of Scope

- The reader's logic. Verify approved it; this round is records and config.
- Pixel decode / unpack — STAGE-002, `DEC-008`.
- The multi-strip corpus gap (follow-up, recorded).

## Return Criteria — how to hand back

1. Paste all gates including the new fuzz-licence one.
2. Fill `## Completion` and `handback:`. ⚠ `tokens_total`: deduplicate by
   `message.id` and **say so**, with cache-read share. Note: measured inflation
   factors so far are **1.61× / 1.82× / 1.86× / 1.95× / 2.25×** — it is **not** a
   constant, so no fixed correction may be applied to a raw figure.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Commit on `feat/spec-003-ifd-reader`; do not merge.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-003-ifd-reader`, on top of `ee0a3f0`
  (implementation still `b79c7ef`). **Not merged**, no PR opened — return
  criterion 4.
- **Completed at:** 2026-08-20
- **All acceptance criteria met?** **yes** — all five deliverables, plus two
  in-scope extras named below. Every fact I was handed was re-measured before I
  acted on it; two of them turned out to need a wider fix than the handoff
  described, and I say which and why.
- ✅ **NO `src/` CHANGE.** `git status` lists thirteen paths and not one is under
  `src/`. The reader is byte-identical to `b79c7ef`. Where the round genuinely
  wanted something written *near* the code — the malformed-tag rule — it went to
  `DEC-012` with `affected_scope: src/ifd.rs` instead, so
  `just decisions-audit --changed` surfaces it the moment anyone edits the
  reader. See the note under **Deviations**.

---

## SB-1 — closed

### All three parts reproduced on the untouched tree, before touching anything

```
$ cargo deny --manifest-path fuzz/Cargo.toml check licenses

error[unlicensed]: irradiance-fuzz = 0.0.0 is unlicensed
error[rejected]: failed to satisfy license requirements
  ┌─ libfuzzer-sys-0.4.13/Cargo.toml:36:36
  │ license = "(MIT OR Apache-2.0) AND NCSA"
  │                                    rejected: license is not explicitly allowed
  ├ NCSA - University of Illinois/NCSA Open Source License:
  ├   - OSI approved
  ├   - FSF Free/Libre
licenses FAILED
```

It runs. It fails. The gate was never absent — it was never invoked, for the
length of a whole build cycle, while a hand-written table stood in for it on a
`blocking` constraint.

And the full graph, from `cargo metadata --manifest-path fuzz/Cargo.toml
--all-features` rather than from what the fuzz build happened to print:

```
arbitrary        1.4.2     MIT OR Apache-2.0
cc               1.4.3     MIT OR Apache-2.0
cfg-if           1.0.4     MIT OR Apache-2.0          <- absent from DEC-011:42
find-msvc-tools  0.1.11    MIT OR Apache-2.0
getrandom        0.4.3     MIT OR Apache-2.0          <- absent from DEC-011:42
irradiance       0.1.0     MIT OR Apache-2.0
irradiance-fuzz  0.0.0     (none)                     <- the unlicensed error
jobserver        0.1.35    MIT OR Apache-2.0
libc             0.2.189   MIT OR Apache-2.0
libfuzzer-sys    0.4.13    (MIT OR Apache-2.0) AND NCSA
r-efi            6.0.0     MIT OR Apache-2.0 OR LGPL-2.1-or-later   <- absent from DEC-011:42
shlex            2.0.1     MIT OR Apache-2.0
```

### The fix, and the one judgement call the handoff left open

**A named per-crate `exception`, not a widened `allow`** — the handoff asked me to
pick one and say why:

> `allow` is a **standing, graph-wide sanction**. NCSA is in this repo for exactly
> one reason — a fuzz-only test dependency's vendored C++ — and it has no business
> being pre-approved for the *library's* graph, which is the graph carrying the
> embeddable-permissive promise consumers actually rely on. An exception is
> **named**: a second crate arriving with NCSA fails loudly and gets its own
> decision. A widened `allow` would wave it through in silence. `deny.toml`'s own
> comment already defines exceptions as *"a license OUTSIDE `allow`, knowingly
> accepted for ONE named crate"* — this is that, exactly.

Everything else in the handoff's fix list, done: `license = "MIT OR Apache-2.0"`
on `fuzz/Cargo.toml` (it had none, and no-licence is an **error**, not a warning);
`DEC-011`'s table re-measured and the "no exception needed" claim retracted in
place rather than quietly swapped; `guidance/constraints.yaml:45`'s `enforcement:`
now names **both** invocations and states the gate's real scope.

### ⚠ A provenance fact I found while checking, which the ledger should want

`libfuzzer-sys`'s README says *"All files in the `libfuzzer` directory are licensed
NCSA."* They are not:

```
files=49  apache-llvm-header=49  ncsa-header=0
```

All **49** vendored `.cpp`/`.h`/`.def` files carry the post-2019 LLVM header —
*"Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions"* —
and **none** mentions NCSA or the University of Illinois. Counted, not sampled. So
the crate's declared SPDX expression *and* its own README are both stale against
the code it actually ships. Nothing is at risk: every reading is permissive, and
the gate enforces the declared expression, which is the stricter one. It also
settles the handoff's aside about the now-unencountered
`Apache-2.0 WITH LLVM-exception` allowance — it stays, because it is what the
vendored source really claims; cargo-deny reports it unencountered only because
vendored C++ is not a cargo package. DEC-011's old parenthetical was right about
the source headers and wrong about what the gate can see.

### The gate is wired — and shown red, because this repo does not accept a gate that has never failed

`just deny-fuzz`, CI job `licenses-fuzz` (via `cargo-deny-action`'s documented
`manifest-path` input — I fetched the action's `action.yml` rather than assuming
the input exists), `AGENTS.md` §6, and `constraints.yaml`. Then the red-proof,
because `oracle-must-be-shown-red` applies to a gate as much as to an oracle
(DEC-009's whole argument):

```
=== CONTROL: honest tree ===                                       exit=0
=== RED 1: exception removed (NCSA no longer sanctioned) ===
  error[rejected]: failed to satisfy license requirements
  rejected: license is not explicitly allowed
  licenses FAILED                                                  exit=4
=== RED 2: license field removed from fuzz/Cargo.toml ===
  error[unlicensed]: irradiance-fuzz = 0.0.0 is unlicensed
  licenses FAILED                                                  exit=4
=== RESTORED ===                                                   exit=0
```

Both halves of the fix are **individually load-bearing**. Neither is decoration.

---

## The ten gates, run by me

`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`, 7/7 corpus
files present.

```
 1. cargo fmt --check                                        exit 0
 2. cargo clippy --all-targets --all-features -- -D warnings exit 0
 3. cargo test --all-features                                exit 0   48 passed, 0 failed
                                                                      (31 lib + 9 corpus + 8 ifd_reader)
 4. just msrv                          ← NEW RECIPE          exit 0   1.90.0, --all-targets --all-features
 5. just deny            (library graph)                     exit 0   "licenses ok"
 6. just lint-red-proof                                      exit 0   control clean (0) → injection
                                                                      rejected (101) → all five lints
                                                                      fired at the injected lines, and
                                                                      still fire without CI's -D warnings
 7. just lint-no-allow   (clippy --lib -F ×5)                exit 0
 8. scripts/cost-audit.sh                                    exit 0   all shipped specs recorded
 9. scripts/decisions-index.sh --check                       exit 0   no INDEX.md committed (12 DECs)
10. just deny-fuzz      (fuzz graph)   ← THE NEW GATE        exit 0   "licenses ok"
```

**Ten green.** And one extra, because I edited the manifest `cargo-fuzz` itself
reads and a green `cargo deny` says nothing about whether the target still builds:

```
$ just fuzz 20
INFO: seed corpus: files: 653 min: 1b max: 3371b total: 211819b
Done 5757081 runs in 21 second(s)
EXIT=0        fuzz/artifacts/ifd/ : 0 files      git status: unchanged
```

---

## The third `+toolchain` trap — reproduced, then closed

```
$ cargo +1.90.0 check --all-targets --all-features
error: no such command: `+1.90.0`
help: invoke `cargo` through `rustup` to handle `+toolchain` directives
                                                             exit 101

$ just msrv     # ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
    Finished `dev` profile ... in 2.46s                      exit 0
```

Recorded in `guidance/toolchain-brief.md` as **"The THIRD `+toolchain` trap"**,
with the detail that distinguishes it from the second: **no `PATH=` prefix is
needed here**, because nothing shells out to an inner bare `cargo`. Three traps,
one root cause, and the fix differs by one detail each time — which is precisely
why each got rediscovered.

The generalisable half is in the brief and in `app.just`: **a gate documented as a
raw command is a gate that will be run wrong.** MSRV was the only one of the ten
with no recipe, and it is the only one that cost two agents a loop apiece.

---

## The three factual corrections — each re-measured, and one needed a wider fix

I did not transcribe any of these. I measured them.

**1. Byte order — 6 `II` / 1 `MM`.** Raw two-byte magic of all seven:

```
L1021223.DNG 4949   L1026016.DNG 4949   L1026192.DNG 4949   L1000622.DNG 4949
M2462362.DNG 4d4d   K3III.DNG    4949   K3III.PEF    4949
```

`CHANGELOG.md:31`'s *"5 `II` / 1 `MM` / 1 PEF"* conflated **byte order** with
**container** — they are independent axes and the PEF is `II` too. It now reads
6 `II` / 1 `MM` **and** 6 DNG / 1 PEF, stated as two axes so the conflation cannot
recur.

**2. Compression — two JPEG, one vendor-private.** Via `irr ifd`:

```
L1021223.DNG  Little   compression 1     (uncompressed)
L1000622.DNG  Little   compression 1     (uncompressed)
M2462362.DNG  Big      compression 7     (JPEG — not decodable by PROJ-001)
K3III.DNG     Little   compression 7     (JPEG — not decodable by PROJ-001)
K3III.PEF     Little   compression 65535 (vendor/other — not decodable by PROJ-001)
```

The spec's corpus paragraph is rewritten with all three of its wrong facts
corrected **and left visible as corrections** rather than silently swapped — a
design that asserted corpus numbers without measuring them is the part worth
remembering.

**3. `docs/conformance-matrix.md` needed THREE rows, not one.** ⚠ The handoff
named the Leica M Monochrom (Typ 246). Checking against the file's own opening
rule — *"Every camera gets a row the day it is known"* — the **M Monochrom** and
the **Pentax K-3 III Monochrome** are equally unlisted, equally held, and equally
read end-to-end by this spec. Fixing only the one named would have left the
identical defect in place twice, so I added all three, each carrying what makes it
distinct (16-bit and the only non-zero `ActiveArea` origin; the only `MM` file and
the only 12-bit one; the only real IFD chain, the only file with no `SubIFDs` tag,
two containers of one scene, and the malformed-tag discovery).

Two more defects in that file, both of which the punch list flagged and both of
which the opening rule makes mandatory once you are editing the row:

- **§"PROJ-001 validates against ONE camera"** was false where it mattered and
  still true where it mattered. Split, not deleted: the **container** half is done
  (four bodies, seven files — with the specific coverage each one bought spelled
  out), the **develop** half is untouched and inherits the caveat undiminished.
  Deleting the section would have thrown away a true warning; leaving it would
  have kept a false one.
- **The Pentax fixture is tier B, not tier A.** A 37 MB uncommitted file cannot
  gate anything. The corrected text names what *actually* pins the behaviour in
  CI: the tier-A synthetic at `src/ifd.rs:1374`
  (`a_malformed_fixed_length_tag_costs_the_tag_not_the_file`). The real file is the
  **discovery**; the synthetic is the **regression test**. Confusing them is how a
  gate that runs nowhere gets believed.

---

## The malformed-tag rule — stated, as `DEC-012`

**Strict on structure, tolerant on shape.**

| Phase | What is read | Malformed → |
|---|---|---|
| **Walk** — `Container::parse` | header, entry tables, chain `next`, `SubIFDs` (330) | fatal to the **container** |
| **Interpret** — `sensor()`, `scalar()`, `values()`, `array()` | every other tag | fatal to **that call** |
| **Interpret**, narrow | `array::<N>()` where `count != N` | costs the **tag**; recorded in `malformed_tags` |

Reading the code closely, **the asymmetry is real but narrower than FU-5 framed
it.** `array()` tolerates a wrong **count** and nothing else: its `uints()` call
carries a bare `?`, so a wrong *field type* is fatal there too. The rest of the
apparent contradiction is a **phase** difference, not a policy difference — tag
330 is read during the walk, where a failure means you do not know what IFDs
exist, and everything else is read after, where you do.

I kept both behaviours rather than making them consistent, and the argument is in
the DEC's alternatives. The short form: a tolerant `SubIFDs` returns a container
that is **structurally a lie** — on four of seven corpus files the sensor plane
*is* in a SubIFD — and SPIKE-001 already measured how close the resulting wrong
answer is: a Q2M's `SubIFD2` is a full-resolution **JPEG preview only 56 px
narrower than the plane**. Degrading a structural read into a plausible wrong
answer is the one failure mode this reader exists to prevent. Making `array()`
strict instead is measurably worse on real input: a shipping camera writes that
tag malformed and `dnglab` tolerates it.

Confidence 0.85, and the reason is stated in the record: **no held file exercises
a broken 330**, so this is reasoned, not measured. What would move it is a real
corpus file, not an argument.

---

### Cost self-report

Mirrors the `handback:` front-matter.

- **Tokens (total):** **9,733,599**
- **Estimated USD:** null — no published rate applied; both prior cycles left this
  null and an invented number is worse than none.
- **Duration (minutes):** ~60
- **Source of the number:** transcript sum. `/cost` is a client-side slash command
  the assistant cannot execute, so this is summed from this session's own `usage`
  objects (`~/.claude/projects/-Users-…-irradiance/2f791a1f-….jsonl`).

⚠ **DEDUPED BY `message.id`, and I say so.** A transcript writes one JSONL line per
content block and repeats the same `usage` object on each, so a naive sum
double-counts every multi-block message:

```
usage objects (raw lines) : 127
distinct message.id       :  74
raw sum                   : 17,087,494
DEDUPED sum               :  9,733,599
inflation factor          : 1.76x
```

Composition (deduped): input 148 · output 45,774 · cache-write 156,731 ·
cache-read **9,530,946** — **97.9% cache-read**. It is a **floor**: computed before
the session closed, so the true figure is somewhat higher.

**On the correction factor, since the handoff asked me to be explicit.** The
measured factors are now **1.61× / 1.76× / 1.82× / 1.86× / 1.95× / 2.25×**, a
1.4× spread across six observations. The factor tracks how block-heavy a session
is — how many tool calls and thinking blocks a message carries — which varies with
the *shape* of the work, not with its size. So **no fixed correction is valid on
any raw figure.** That applies directly to `SPEC-001`'s `cost.totals` of
**51,979,929**, which is still a raw double-counted sum: it must be **re-summed
with dedup from its own transcript**, not divided by anything. Flagged at build,
flagged again at verify, still true.

### ⚠ A finding about the cost tooling itself, measured

`just handback-sync SPEC-003 --dry-run`:

```
  [dry-run] HANDOFF-011 → cost.sessions[build]  tokens=10967269
  [dry-run] HANDOFF-012 → cost.sessions[verify] tokens=9036505
```

Both of those sessions **are already in the spec**, hand-written by the agents
that ran those cycles. `handback-sync` keys idempotence on `synced_at` alone
(`scripts/handback-sync.sh:68`) and does no check against existing
`cost.sessions`, and all three handbacks still read `synced_at: null` — so running
it as-is **double-counts two cycles**, and `cost.totals` feeds `just calibration`
and `cost-audit`.

I did not edit `synced_at` — the template says *"stamped by `just handback-sync` —
do not edit"*, and quietly stamping a sync that did not run is exactly the class
of thing this repo treats as the defect. So my session is **hand-written to match
the precedent the two prior cycles set**, and the choice is yours:

- **stamp `synced_at: 2026-08-20`** on 011/012/013 by hand, since the
  transcription demonstrably already happened; or
- **delete the three hand-written sessions** and let `handback-sync` write them.

Either is fine. Running it as-is is not. The deeper fix, if you want one: make
`handback-sync` skip a handoff whose `id` already appears in `cost.sessions`
notes, so the two paths stop depending on nobody using both.

### Drift and new artifacts

- **New decisions emitted:**
  - **`DEC-012` — Strict on structure, tolerant on shape: where a malformed tag
    costs the tag and where it costs the file.** `affected_scope: src/ifd.rs`, so
    `just decisions-audit --changed` raises it for SPEC-004.
  - `DEC-011` **amended in place**, not superseded — the decision itself (a
    separate `fuzz/` crate; `[dependencies]` stays empty) is sound and I would
    keep it. Only its licence table and its claim about gate reachability were
    wrong, and both corrections are marked as corrections. `affected_scope` gained
    `**/deny.toml`, since it now governs a named exception there.

- **Deviations from spec / handoff scope** — three, all additive, all disclosed:
  1. **The conformance matrix got three rows, not one** (reasoned above). Fixing
     one of three identical omissions would have satisfied the handoff and not the
     rule the handoff invoked.
  2. **FU-3 fixed too** — *"the full-resolution SubIFD"* in `SPEC-003`'s AC6 and
     `STAGE-001`, unsatisfiable on the PEF whose plane is `IFD0`. Not on the
     handoff's list, but I was already correcting the PEF's structure in the same
     paragraph, and leaving the same error standing two lines below the correction
     would have been worse than either fixing or not touching it. Both now say
     **"sensor IFD"**, which is true for all seven and is what the code has always
     called it.
  3. **The malformed-tag rule went to a `DEC`, not to a doc comment.** A one-line
     pointer above `array()` and `sub_ifd_offsets_of_last()` is where a reader
     would most want it, and that is a `src/` edit this round forbade. I did not
     make it. **Recommendation: it should be SPEC-004's first edit** — the DEC says
     so explicitly under Consequences, so the instruction survives this handback.

- **Follow-up work identified:**
  - **FU-6 (multi-strip) is now recorded in `CHANGELOG.md`'s Known gaps** with the
    point verify added — the single-strip shape is *asserted* in three places, so a
    multi-strip file will fail those tests rather than silently take a new path.
    Right way round, but it is a test to update, not a reader bug, and the
    `[[wanted]]` row should say so when it is written.
  - **`handback-sync` double-count** (above) — worth a patch-lane fix.
  - **FU-7** — "no `#[allow]` anywhere in `src/`" is imprecise; the accurate
    sentence is *"no `#[allow]` on any **non-test** path"*. Not corrected here: it
    lives in prose in prior handbacks rather than in a durable doc, and
    `src/lib.rs:34-35` already states the limitation correctly. Left as verify
    filed it.
  - **F4** — the fuzz target still does not run in CI (needs nightly). Unchanged
    and still worth its own spec.
  - **F5** — `docs/measured-q2m-dng.md` has no tag numbers. Unchanged.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing was unclear; this handoff was the best-armed of the three, because it
   had already *reproduced* its own ship-blocker. What that changed is worth
   naming: I spent no time establishing whether SB-1 was real and all of it on the
   two things the handoff could not settle from outside — which remedy to pick
   (exception vs `allow`), and whether the fix was *complete*. Both of the places
   I widened scope came out of the second question. The one thing I would have
   wanted stated: the handoff said "the licence record is wrong on a blocking
   constraint" but not "and therefore check whether every *other* record this
   round touches was measured or remembered." That reframing is what turned the
   matrix from a one-row fix into a three-row one.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `oracle-must-be-shown-red`. The handoff said *"wire the fuzz licence check as
   a real gate since it works"* — and "it works" was demonstrated by watching it
   **fail** on the broken tree, which is only half a proof. A gate that has only
   ever been observed failing is the mirror image of the green oracle that cannot
   fail: you have not shown that passing means anything. This repo already has the
   discipline (DEC-009, and `lint-red-proof`'s negative control exists for exactly
   this reason), so the red-proof I ran should have been a stated deliverable
   rather than something I decided to add. **Any spec that adds a gate should
   inherit that constraint automatically.**

3. **If you did this task again, what would you do differently?**
   — Read `libfuzzer-sys`'s vendored source **before** writing the corrected
   licence table, not after. I wrote the NCSA row from the SPDX expression, then
   went looking for the vendored C++ to explain *why* NCSA was in the expression at
   all — and found 49 files that say Apache-2.0-WITH-LLVM-exception and zero that
   say NCSA, which is a better fact than anything in the table above it and forced
   a rewrite. The general form is the lesson this whole ship-blocker teaches, one
   level up: **a declared licence is a claim, not an observation.** DEC-011 got
   caught believing a remembered table; I nearly got caught believing a declared
   expression. The ledger exists because those are different things — that is
   literally what `provenance-recorded-per-algorithm`'s rationale says — and I
   should have reached for the source on the first pass rather than the second.
