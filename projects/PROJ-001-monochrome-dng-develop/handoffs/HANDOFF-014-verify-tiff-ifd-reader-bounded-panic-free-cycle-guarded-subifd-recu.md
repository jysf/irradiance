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
  id: HANDOFF-014
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
  tokens_total: 7757521            # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 55
  branch: feat/spec-003-ifd-reader
  pr: null
  completed_at: 2026-08-21         # YYYY-MM-DD
  notes: "VERDICT: APPROVED at d284ff3. src/ byte-identical to b79c7ef, confirmed independently by sha256 9c965c48... on src/ifd.rs - the same digest HANDOFF-012's verify cycle recorded when it restored its own planted fault, so two verify cycles now agree the reader has not moved. TEN gates re-run by me, all green. BOTH red-proofs run by me. deny-fuzz red-proofed THREE directions with the mutation asserted to have changed the file each time (the exceptions=[...] ARRAY trap): exception removed -> exit 4 error[rejected] naming NCSA; fuzz/Cargo.toml license removed -> exit 4 error[unlicensed]; exception re-pointed at a crate not in the graph -> exit 4, which is the first direct proof that the named-exception-over-widened-allow reasoning has teeth. Library `just deny` unaffected (exit 0) under all three. CI parity checked too: cargo-deny resolves the ROOT deny.toml from a foreign cwd via --manifest-path, so the licenses-fuzz job uses the same policy file. FUZZ red-proof at a THIRD distinct site (build used payload, verify round 1 used read_ifd; I used walk_chain), lint-clean and tuned so that BOTH lint gates AND all 48 tests pass with the panic live - the fuzzer is the only thing that can see it. libFuzzer found it in ~3,600 execs from 22 seeds, synthesising MM/version-42/odd IFD0 offset 5 itself; crash-e794e4ea, exit 1. Restored byte-identical, DELIBERATE FAULT count 0, 13,053,759 runs in 61 s on the red run's own corpus INCLUDING the crash reproducer, zero artifacts, git status clean. Corpus facts re-measured by me rather than transcribed: 6 II / 1 MM from raw magic bytes; 4 uncompressed + 2 JPEG (7) + 1 vendor-private (65535). Pentax tier reclassification CORRECT - manifest says tier b, file is 37,669,430 bytes, and the tier-A synthetic at src/ifd.rs:1374 is a lib unit test that does run in CI. Provenance claim re-counted and extended: 49 top-level vendored files, 55 including libfuzzer/afl and libfuzzer/dataflow, all 55 carrying the LLVM Apache-2.0-with-exception header, and 0 of 56 files under libfuzzer/ mentioning NCSA or the University of Illinois. DEC-012's deferral MEASURED sound: touching src/ifd.rs makes just decisions-audit --changed name DEC-008 and DEC-012. SEVEN FOLLOW-UPS, none ship-blocking: FU-9 provenance-ledger Standing-decisions bullet for the declared-vs-carried case (the inverse direction of the ledger's own motivating example); FU-10 the new four-bodies/seven-files claim omits that all seven manifest entries are tier b and zero are tier a, so none of it runs in CI - the same defect the round corrected two paragraphs below it; FU-11 DEC-012's one-question rule does not predict its widest-blast-radius case, is_sensor_ifd propagates scalar() errors and sensor_candidates/sensor_ifd/sensor call it over EVERY IFD, so a malformed NewSubfileType on a thumbnail costs the plane; FU-12 SPEC-004's references.decisions is [] while DEC-012 aims its deferral at SPEC-004's first edit; FU-13 cost.totals is 10967269, exactly the first build session, where three sessions sum to 29737373; FU-14 oracle-must-be-shown-red is narrower in text than in practice now that three GATES are red-proofed; FU-15 the full-resolution-SubIFD phrasing FU-3 corrected survives at STAGE-002:84. tokens_total DEDUPED BY message.id and I say so: 120 usage objects, 54 distinct ids, raw 17,065,674 vs deduped 7,757,521 = 2.20x, 97.0% cache-read, a FLOOR. That is a SEVENTH measured factor and the band is now 1.61x-2.25x across seven observations - no fixed correction is valid on any raw figure. DID NOT run handback-sync per the handoff."
  synced_at: 2026-08-21   # stamped by hand, NOT by `just handback-sync`: this
                          # cycle's session is already hand-written into the
                          # spec (AGENTS.md §15), matching the precedent set on
                          # HANDOFF-011/012/013 at 93dcae0. The tool keys
                          # idempotence on this field alone and does not check
                          # cost.sessions, so leaving it null would let a later
                          # sync append a fourth duplicate. See feedback
                          # finding 15.
---

# HANDOFF-014: TIFF/IFD reader — bounded, panic-free, cycle-guarded, SubIFD recursion — plus its fuzz target

## Delegation Summary

Second verify cycle on `SPEC-003`, at `93dcae0` (fix `ff46fd9`). Independent
session.

Round 1 returned one ship-blocker — **documentation and config only, no `src/`
change** — and found the reader itself sound. That finding stands; do not re-derive
it.

## Context the Receiving Agent Needs

### Already reconciled by the orchestrator — don't just repeat

- **`src/` is byte-identical to `b79c7ef`.** Empty diff. The fix touched records
  and config only.
- **Ten gates green**, including the new `just deny-fuzz`.
- **SB-1's gate red-proofed by me, both directions:** removing the
  `libfuzzer-sys` exception → **exit 4**, `error[rejected]` naming
  `(MIT OR Apache-2.0) AND NCSA`; restored → 0; the library `just deny` gate
  unaffected. It has teeth.
- **The `handback-sync` hazard is handled** — `synced_at` stamped on
  HANDOFF-011/012/013 so the tool is a no-op. Without it, three hand-written
  sessions plus three transcriptions would have doubled `cost.totals`. Recorded as
  template finding 15. **Do not run `handback-sync` on this spec.**

### What deserves scrutiny

1. **The scope widening — was it right?** The handoff asked for one matrix row;
   the build wrote three, arguing that fixing only the named body satisfies the
   handoff and not the rule the handoff invoked. It also split the "validates
   against ONE camera" section and reclassified the Pentax fixture from tier A to
   tier B. **Check the tier reclassification especially** — a 37 MB uncommitted
   file gating nothing is a real correction, but it changes what CI is claimed to
   cover.
2. **`DEC-012` — "strict on structure, tolerant on shape."** A real decision made
   during a *fix* round, which is unusual. It narrows FU-5's framing (`array()`
   tolerates a wrong count and nothing else) and **defers its own implementation
   to SPEC-004's first edit** rather than editing `src/` here. Is that deferral
   sound, or does it leave an unstated rule live in shipped code?
3. **The provenance finding.** `libfuzzer-sys`'s README says its vendored
   directory is NCSA; all 49 vendored files carry `Apache-2.0 WITH LLVM-exception`
   and none mentions NCSA. So the crate's SPDX expression *and* its README are both
   stale against its own code. The gate enforces the stricter reading — right call,
   but does `docs/provenance-ledger.md` need a row, given that is exactly the
   declared-vs-carried distinction it exists for?
4. **The exception vs. widened `allow` choice.** Reasoned in `deny.toml`. Agree?
5. **`just msrv`** now exists (the third `+toolchain` trap). Does it use the shim
   correctly, and is the trap recorded in the toolchain brief?

### Settled — do not reopen

The reader's logic (approved round 1) · the `-F` gate · `DEC-009` · the corpus
manifest design · the multi-strip gap (follow-up, recorded) · the cost figures.

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the fuzz red-proof yourself (check #9) — ⚠ two traps: `cargo fuzz` needs the
rustup shim **first on PATH**, and **an unchecked index will not reach the
fuzzer** (the lint policy rejects it at compile time), so plant something
lint-clean.

Also red-proof `just deny-fuzz` yourself. ⚠ The exception is an
`exceptions = [...]` **array**, not `[[licenses.exceptions]]` blocks — the
orchestrator's first mutation targeted the wrong form, was a silent no-op, and
nearly produced a false "no teeth" finding. **Assert your mutation changed the
file before you run the check.**

**Label every finding ship-blocking or follow-up.** If the gate is sound and the
remainder are follow-ups, **approve** — this spec has had two build rounds and the
reader was found sound in round 1.

If **APPROVED**, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- The reader's logic; pixel decode/unpack (STAGE-002, `DEC-008`).
- Running `handback-sync` on this spec — it would duplicate.

## Return Criteria — how to hand back

1. Paste the gates and both red-proofs you ran yourself.
2. Fill `## Completion` and `handback:`. ⚠ `tokens_total`: deduplicate by
   `message.id` and **say so**, with cache-read share. Six measured factors now
   span **1.61×–2.25×** — not a constant, so no fixed correction is valid on a raw
   figure.
3. `handoff.status: completed`; spec `task.cycle: verify` (or `ship` if approved).
4. Commit on `feat/spec-003-ifd-reader`; do not merge.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-003-ifd-reader`, reviewed at `d284ff3`. **Not
  merged**, no PR opened — return criterion 4.
- **Completed at:** 2026-08-21
- **All acceptance criteria met?** **yes** — all seven, including AC7's
  *"nine at design; TEN as shipped"*.
- **Verdict:**

# ✅ APPROVED at `d284ff3`

Implementation byte-identical to `b79c7ef`; SB-1's fix is `ff46fd9`; records
`93dcae0`. **Seven follow-ups, none ship-blocking.**

I did **not** run `handback-sync` on this spec.

---

## `src/` really is untouched — and I can show it two ways

```
$ git diff --stat b79c7ef..HEAD -- src/
(empty)

$ shasum -a 256 src/ifd.rs
9c965c4842e82450109b7b5d3b09bd5ca93509030607de70c382037acc21b655
```

That digest is the one **`HANDOFF-012`'s verify cycle recorded** when it restored
its own planted fault ("restored byte-for-byte, sha256 9c965c48…"). So the
identity is not just an empty diff against a commit — two independent verify
cycles, a build round apart, have now hashed the same file and got the same
answer. Worth saying because "no `src/` change" is the load-bearing claim of the
whole punch-list round.

---

## The ten gates, run by me

`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`, 7/7
corpus files present.

```
 1. cargo fmt --check                                         exit 0
 2. cargo clippy --all-targets --all-features -- -D warnings  exit 0
 3. cargo test --all-features                                 exit 0   48 passed, 0 failed
                                                                       (31 lib + 9 corpus + 8 ifd_reader)
 4. just msrv                                                 exit 0   1.90.0 via the rustup shim
 5. just deny            (library graph)                      exit 0   "licenses ok"
 6. just lint-red-proof                                       exit 0   control clean (0) → injection
                                                                       rejected (101) → all five lints
                                                                       fired at 4 distinct injected lines,
                                                                       and still fire without CI's -D warnings
 7. just lint-no-allow   (clippy --lib -F ×5)                 exit 0
 8. scripts/cost-audit.sh                                     exit 0   all shipped specs recorded
 9. scripts/decisions-index.sh --check                        exit 0   no INDEX.md committed (12 DECs)
10. just deny-fuzz       (fuzz graph)                         exit 0   "licenses ok"
```

**Ten green.**

---

## Red-proof 1 — `just deny-fuzz`, three directions, mutation asserted each time

⚠ The handoff's trap is real and I defended against it explicitly: the exception
is an `exceptions = [ … ]` **array** (`deny.toml:49-72`), so I asserted the file
had actually changed before believing any exit code. A silent no-op here
manufactures a "no teeth" finding.

```
=== CONTROL: honest tree ===                                    just deny-fuzz exit=0

=== RED 1: the libfuzzer-sys/NCSA exception removed from the ARRAY ===
  MUTATION ASSERTION — 71d70
    <     { name = "libfuzzer-sys", allow = ["NCSA"] },
  ✓ file CHANGED; occurrences of name = "libfuzzer-sys" now: 0
    error[rejected]: failed to satisfy license requirements
    36 │ license = "(MIT OR Apache-2.0) AND NCSA"
       │                                    rejected: license is not explicitly allowed
       ├ NCSA - University of Illinois/NCSA Open Source License:
    licenses FAILED                                             just deny-fuzz exit=4
  and the LIBRARY gate under the same mutation:                 just deny      exit=0

=== RED 2: license field removed from fuzz/Cargo.toml ===
  MUTATION ASSERTION — 11d10
    < license = "MIT OR Apache-2.0"
    warning[unlicensed]: a valid license expression could not be retrieved
    error[unlicensed]: irradiance-fuzz = 0.0.0 is unlicensed
    licenses FAILED                                             just deny-fuzz exit=4

=== RED 3 (mine): is the exception really NAMED? ===
  MUTATION ASSERTION — 71c71
    <     { name = "libfuzzer-sys", allow = ["NCSA"] },
    >     { name = "some-other-crate", allow = ["NCSA"] },
    error[rejected]: failed to satisfy license requirements
    licenses FAILED                                             just deny-fuzz exit=4

=== RESTORED ===  deny.toml byte-identical, git status clean
                                                                just deny-fuzz exit=0
                                                                just deny      exit=0
```

**RED 3 is the one the round did not run, and it is the one that tests the
*argument* rather than the fix.** `deny.toml:59-67` justifies a named exception
over a widened `allow` on the ground that *"a second crate arriving with NCSA
fails loudly"*. That is a claim about behaviour, and until now it was only
reasoned. It is now measured: NCSA sanctioned for a crate that is not the one
carrying it fails exactly as the comment promises. Both halves of the fix are
individually load-bearing **and so is the choice between remedies**.

**CI parity, since a green local recipe says nothing about the job:** the
`licenses-fuzz` job runs `cargo-deny-action` with `manifest-path: fuzz/Cargo.toml`
and no `--config`, so the whole thing rests on cargo-deny finding the **root**
`deny.toml` for a manifest one directory down. Measured, from a foreign cwd:

```
$ cd /tmp && cargo deny --manifest-path <repo>/fuzz/Cargo.toml check licenses
   ┌─ /Users/…/irradiance/deny.toml:42:6      <- the ROOT policy file, resolved
licenses ok                                                     exit=0
```

One policy file, two graphs, both invocations — as `deny.toml:15-27` claims.

---

## Red-proof 2 — the fuzz target, at a third distinct site

Sites so far: the **build** planted an unchecked index in `Container::payload`;
**verify round 1** planted `split_at` in `Container::read_ifd`. I used
`Container::walk_chain` — the recursion driver neither previous round touched.

The fault, at `src/ifd.rs:610-618` under the fault, immediately after
`read_ifd` returns:

```rust
// TIFF offsets are word-aligned by convention, so "assume it" — the classic
// shape of a parser bug. Lint-clean on purpose: no unwrap, no expect, no
// indexing, no `panic!`, no arithmetic that can trap. `split_at` panics all
// by itself when mid > len.
if offset & 1 == 1 && next & 1 == 1 {
    let (_head, _tail) = self.data.split_at(self.data.len().wrapping_add(1));
}
```

**The negative control came first, and I iterated on the fault until the fuzzer
was the only thing that could see it.** My first two versions (`next` odd; `next`
odd inside a SubIFD chain) were both caught by
`ifd_survives_single_byte_corruption` — a genuinely good result for that test,
and the reason DEC-011 calls the deterministic sweep a *complement* rather than a
substitute. Requiring **two** odd offsets puts it beyond a single-byte flip:

```
just lint          (clippy -D warnings)   exit 0   <- BLIND
just lint-no-allow (-F ×5)                exit 0   <- BLIND
cargo test --all-features                 exit 0   <- BLIND, 48 passed / 0 failed
```

A live `mid > len` panic in the reader, and every cheaper gate green. Then:

```
=== DIRECTION 1: fault planted → RED ===
INFO: seed corpus: files: 22 min: 4b max: 466b total: 2472b
#23     INITED cov: 400 ft: 650 corp: 20/2440b
#3595   NEW    cov: 493 ft: 1200 corp: 101/16Kb
thread '<unnamed>' panicked at src/ifd.rs:617:48:
mid > len
==1877== ERROR: libFuzzer: deadly signal
Test unit written to fuzz/artifacts/ifd/crash-e794e4ea19abc368d3027981c87948fd75cea6a2
                                                                          EXIT=1
```

Found in **~3,600 executions** past `INITED`, from a **fresh empty corpus dir**
plus the 22 committed seeds. What it synthesised is the point:

```
00000000: 4d4d 002a 0000 0005  MM.*....     <- MM, version 42, IFD0 at offset 5 (ODD)
00000050: 0000 0000 0100 4a01  ......J.     <- an IFD whose `next` is ODD too
```

It reconstructed a big-endian TIFF header, put IFD0 at an **odd** offset, then
built a chain pointer that is **also odd** — the exact two-value conjunction the
fault needed and that no single-byte mutation of any seed can reach.

```
=== DIRECTION 2: fault removed → GREEN ===
restored sha256 = 9c965c4842e82450109b7b5d3b09bd5ca93509030607de70c382037acc21b655
expected        = 9c965c4842e82450109b7b5d3b09bd5ca93509030607de70c382037acc21b655
grep 'DELIBERATE FAULT' = 0        git status --porcelain = 0 paths
INFO: seed corpus: files: 108      <- the red run's own corpus, crash reproducer INCLUDED
Done 13053759 runs in 61 second(s)
artifacts: 0                       git status: 0 paths            EXIT=0
```

Direction 2 re-runs on the corpus **the red run itself built, with the crashing
input added back in**. A green that has never been fed the input that killed it
is a weaker green than it looks.

---

## The five scrutiny items

### 1. The scope widening — right, and the tier reclassification is correct

**The reclassification, checked against the sources rather than the argument:**

```
tests/corpus/manifest.toml:222-223   path = "PENTAX-K3III-MONO/K3III.DNG"
                                     tier = "b"
                             :227    bytes = 37669430
$ ls -l …/PENTAX-K3III-MONO/K3III.DNG   37669430          ✓ 37 MB, and tier b in the manifest
```

And the thing that replaces it: `src/ifd.rs:1374`
`a_malformed_fixed_length_tag_costs_the_tag_not_the_file` is a **lib unit test**,
so it is inside the 31 that `cargo test` runs on a bare runner with no corpus.
The matrix's corrected claim at `:79-86` — real file is the *discovery*, synthetic
is the *regression test* — is exactly right, and the line reference is accurate.
**A tier-A claim about an uncommitted 37 MB file was a claim that CI covered
something it has never seen. Correcting it makes the matrix say less and mean
more.** I agree with the reclassification without reservation.

**The three-rows-not-one widening: right, and for the stated reason.** The rule
the handoff invoked is `docs/conformance-matrix.md:3` — *"Every camera gets a row
the day it is known."* Two of the three bodies were unlisted on identical grounds.
Fixing one and leaving two would have satisfied the instruction while leaving the
defect it was aimed at in place twice over. A build that notices the difference
between the instruction and its reason, says so, and widens **additively** is
doing the job. This is not scope creep; scope creep changes the shipped artifact,
and `src/` did not move a byte.

**The split of the "validates against ONE camera" section: right too** — deleting
it would have thrown away a warning that is still true at the develop layer,
keeping it whole would have kept one that is false at the container layer. But
see **FU-10**: the new half inherits the very defect the same edit corrected two
paragraphs below it.

### 2. `DEC-012` — the deferral is sound, and I measured the mechanism it rests on

A decision emitted during a *fix* round is unusual, and the right test is not
"was it in scope" but **"does it leave an unstated rule live in shipped code?"**
It does not, and the reason is mechanical rather than promissory:

```
$ printf '\n' >> src/ifd.rs && just decisions-audit --changed
Decisions governing your uncommitted changes:
⚠ DEC-008 — Sample unpacking branches on byte alignment, not on bit depth
⚠ DEC-012 — Strict on structure, tolerant on shape — where a malformed tag …
      re-read this decision before committing; your change touches:
        src/ifd.rs
```

I ran that, and restored the file. `affected_scope: src/ifd.rs` is not
decoration: **anyone who edits the reader is told about DEC-012 before they
commit.** That is a better outcome than a doc comment would have been, because a
doc comment is only seen by whoever opens that function, while this fires on the
whole file. Writing the rule into `src/` this round would also have broken the
one constraint the round was given, for a strictly weaker result.

The decision itself checks out against the code:

- `array()` tolerating a wrong **count** and nothing else — `src/ifd.rs:810`
  carries a bare `?` on `uints()`, so a wrong field type is fatal there too.
  DEC-012's narrowing of FU-5's framing is accurate, and FU-5 did overstate it.
- Tag 330 fatal to the container — `:673` routes it through `uints()`, `:623`
  propagates with `?`. Correct.
- The Option A rejection is the strongest part of the record: a tolerant 330
  returns a container that is *structurally a lie*, and SPIKE-001's measurement
  (a Q2M `SubIFD2` preview 56 px narrower than the plane) makes that concrete
  rather than rhetorical.

Confidence 0.85 with *"no held file exercises a broken 330, so this is reasoned,
not measured"* stated in the record is the right calibration, and `## Validation`
naming a real corpus file as the only thing that should move it is exactly what
`AGENTS.md §16` asks for. **Two gaps, both follow-ups: FU-11 and FU-12.**

### 3. The provenance finding — yes, the ledger wants it. **FU-9.**

I re-counted rather than transcribing, and the fact is slightly bigger than the
round recorded:

```
libfuzzer-sys-0.4.13/Cargo.toml:36   license = "(MIT OR Apache-2.0) AND NCSA"
README.md:105                        "All files in the `libfuzzer` directory are licensed NCSA."

vendored .cpp/.h/.def, top level               : 49    Apache-2.0-WITH-LLVM header: 49
vendored .cpp/.h/.def, incl. afl/ + dataflow/  : 55    Apache-2.0-WITH-LLVM header: 55
ALL files under libfuzzer/ (any extension)     : 56    mentioning NCSA or U. of Illinois: 0
```

The round's **49** is right for the top level; the two subdirectories add six more
files that say the same thing, and **nothing anywhere under `libfuzzer/` mentions
NCSA at all.** So both the SPDX expression and the README are stale against the
shipped code, confirmed independently. Detail in the punch list.

### 4. Per-crate exception vs widened `allow` — **agree**, and now measured

The reasoning at `deny.toml:59-67` is right on the merits: `allow` is a standing
graph-wide sanction, and NCSA is in this repo for exactly one fuzz-only reason,
so pre-approving it for the **library's** graph — the graph that carries the
embeddable-permissive promise — would be sanctioning something no consumer is
exposed to. What made me comfortable signing off is that the argument's testable
half is now tested (RED 3 above): a second crate arriving with NCSA **does** fail
loudly. Also confirmed: the exception is inert where it should be — `just deny`
reports it as `license-exception-not-encountered` on the library graph and the
fuzz graph does not, which is precisely the containment the choice was made for.

### 5. `just msrv` — correct shim use, trap recorded

`app.just:156-157` is `~/.cargo/bin/cargo +1.90.0 check --all-targets
--all-features` — the shim as the **outer** command and, correctly, **no `PATH=`
prefix**, because unlike `cargo fuzz` nothing shells out to an inner bare `cargo`.
`guidance/toolchain-brief.md:71-98` records it as *"The THIRD `+toolchain` trap"*
and states that distinguishing detail explicitly. `AGENTS.md §6` carries the raw
command with the same warning, and CI's `msrv` job correctly uses
`dtolnay/rust-toolchain@1.90.0` + bare `cargo check` (no shim needed on a
rustup-managed runner). Exit 0 for me.

The generalisation in the brief — *"a gate documented as a raw command is a gate
that will be run wrong"* — is the most valuable line the round produced, and it is
evidenced: MSRV was the only one of the ten without a recipe and the only one that
cost two agents a loop apiece.

---

## Punch list — seven follow-ups, no ship-blockers

### 📋 FOLLOW-UP

**FU-9 — the provenance ledger has no row for the one declared-vs-carried case
this repo has actually found.**
`docs/provenance-ledger.md:42-49` (the *Standing decisions* list).

The fact is recorded twice — `DEC-011:128-142` and
`guidance/constraints.yaml:45` — and neither is the ledger, whose opening
paragraph (`:6-11`) states this exact distinction as its reason to exist:
*"the licence a crate declares and the provenance its code carries are different
things, and `cargo deny` only sees the former."* That is a verbatim description of
libfuzzer-sys.

A **table row would be wrong** — the table is per-algorithm and this is neither an
algorithm nor a decoder in `irradiance` — but the *Standing decisions* list is
already where crate-level facts live (`demosaic`, `rawler`/`rawloader`, patents).
One bullet, with the counts above.

⚠ And one sentence beyond what the round found, because it changes what the
ledger is for: **this case runs the opposite direction from the ledger's own
motivating example.** `demosaic` declares permissive and *carries* copyleft —
declared better than carried. `libfuzzer-sys` declares `AND NCSA` and carries
Apache-2.0-WITH-LLVM-exception — declared **stricter** than carried. The ledger's
framing quietly assumes the first direction. Recording the second is what stops
someone later reading a stale-but-stricter declaration as evidence of risk, or —
worse — "correcting" `deny.toml` toward a README that its own source contradicts.

**FU-10 — the new "four bodies, seven files" claim inherits the exact defect this
round corrected two paragraphs below it.**
`docs/conformance-matrix.md:24-48`.

The rewritten section asserts the container reader is *"now exercised against
four bodies, three makes-worth of firmware and seven files, all read end-to-end
and cross-checked against `exiftool 13.55`"* — with no note of where. Measured on
`tests/corpus/manifest.toml`: **seven `[[file]]` entries, `tier = "b"` on all
seven, zero tier-A entries.** None of that coverage runs in CI, ever, by design
(`DEC-003`).

The same file gets this right for the Pentax fixture at `:79-86` — *"a 37 MB
uncommitted file cannot gate anything… the tier-B file cannot gate anything"* —
and carries the general caveat at `:135-137`. So the correction and the
uncorrected claim are in the same file, from the same edit, twenty lines apart.
By the build's own widening rule — *fixing only the named instance satisfies the
handoff and not the rule it invoked* — this one was in scope too.

Cheap fix: one clause on `:29`, *"…on a machine holding the corpus; CI sees none
of it."* Not ship-blocking, because the caveat exists elsewhere in the same file
and as signal `ci-cannot-prove-bit-exactness`.

**FU-11 — `DEC-012`'s one-question rule does not predict its widest-blast-radius
case.**
`src/ifd.rs:836-841` (`is_sensor_ifd`), `:848-856` (`sensor_candidates`),
`:859-866` (`sensor_ifd`), `:873-880` (`sensor`).

`is_sensor_ifd()` reads three tags through `scalar()` and propagates every error
with `?`. All three selection entry points call it **over every IFD in the
container**. So a malformed `NewSubfileType`, `PhotometricInterpretation` or
`SamplesPerPixel` on **any** IFD — a thumbnail, an unrelated chain link, an IFD
that could never be the plane — is fatal to sensor selection for the whole file.

By `DEC-012`'s stated test — *"does this change what exists, or only what a field
says?"* — that is squarely **interpret**-phase and the rule predicts it "costs
that call only". The table is not wrong about the code (`sensor()` *is* the call);
the **rule** doesn't reach the case, and the practical outcome is much closer to
"costs the file" than to "costs the tag".

This matters now rather than later for the reason DEC-012 itself gives: `SPEC-004`
widens the type model directly on top of `uints()`, and `uints()` is what
`scalar()` calls. The boundary should be decided before it is inherited — which is
the decision's own stated purpose. One extra table row, or a sentence under
`## Consequences`, closes it. Not ship-blocking: no corpus file trips it, and
strict-is-safe remains defensible.

**FU-12 — `SPEC-004` does not reference `DEC-012`, and `DEC-012` aims its
deferral at SPEC-004's first edit.**
`projects/…/specs/SPEC-004-dng-tag-model-and-typed-metadata-extraction.md:34` —
`references.decisions: []`.

`DEC-012:126-131` says *"SPEC-004's first edit should be a one-line pointer to
DEC-012 above `array()` and `sub_ifd_offsets_of_last()`"*, and HANDOFF-013's
handback repeats it. Neither location is one `AGENTS.md §15` sends a build agent
to: build step 3 is *"read every `DEC-*` listed in the handoff's references"*, and
that list is empty.

The backstop is real and I measured it (scrutiny item 2) — but it is **advisory,
manual, and not in CI**, and it only fires on *uncommitted* changes. Adding
`DEC-012` and `DEC-008` to SPEC-004's `references.decisions` costs one line and
removes the dependence on someone remembering to run an advisory command. That is
the difference between a deferral that is defensible and one that is *sound*.
SPEC-004 is still `cycle: frame`, so its design pass is the natural moment.

**FU-13 — `cost.totals.tokens_total` is three sessions stale and looks
computed.**
`projects/…/specs/SPEC-003-….md:98-99`.

```
cost.sessions   build 10,967,269 · verify 9,036,505 · build 9,733,599   sum 29,737,373
cost.totals     10,967,269      <- exactly the FIRST build session
```

Introduced at `d867403` (build round 1), not by this round; neither later cycle
touched it. Nothing reads it while the spec is unshipped — I checked: every
shipped spec has `totals == sum(sessions)`, and `just calibration` lists only the
three shipped specs. So ship will recompute it.

Flagged anyway because **a wrong-but-plausible number is the exact failure mode
this whole round was about**: a ship agent seeing a non-null `totals` may take it
for a computed figure. Either `null` it until ship, or make it right. With this
cycle it is **37,494,894**.

**FU-14 — `oracle-must-be-shown-red` is narrower in its text than in its
practice, and HANDOFF-013's own reflection asked for this.**
`guidance/constraints.yaml:52-56`.

The rule says *"Every **oracle** ships with a deliberate-fault test proving it
FAILS on a broken input."* Three **gates** are now red-proofed: `lint-red-proof`
(DEC-009 argues the point at length), the fuzz target (AC5), and — this round —
`deny-fuzz`. The build volunteered that last one and then wrote in its reflection
that it *"should have been a stated deliverable rather than something I decided to
add"*, with the right generalisation: *"any spec that adds a gate should inherit
that constraint automatically."*

I agree, and I want it out of prose. Either widen the rule to *"every oracle **or
gate**"*, or record it in `guidance/signals.yaml` (`type: lesson`) per `§15`'s
ship step — right now it exists only inside a handback, which is the least
durable place in this repo. There is no signal covering it: the nearest,
`unrun-docs-carry-errors`, is about docs, not gates.

**FU-15 — the "full-resolution SubIFD" phrasing FU-3 corrected survives in a
third durable doc.**
`projects/PROJ-001-monochrome-dng-develop/stages/STAGE-002-…md:84` — *"Locating
the full-resolution SubIFD's strip and reading it."*

The round fixed this at `SPEC-003` AC6 and `STAGE-001:58`, both now *"sensor
IFD"*. This third instance is **defensible as scoped** — both uncompressed corpus
planes really are in SubIFDs, and the PEF is out of STAGE-002's reach — which is
why it is last and smallest. But it is the identical wording, and the rule the
build invoked to widen the matrix from one row to three reaches it as well. Fix
it, or write the one clause saying why it is exempt.

### Minor — measured, no action needed

`just deny` on the library graph now emits, permanently:

```
warning[license-exception-not-encountered]: license exception was not encountered
```

One policy file, two graphs (`deny.toml:15-27`), so each invocation reports the
other's entries as unmatched. It does not affect exit status and it is the
*correct* containment — but that warning class can no longer carry signal on the
library graph, which is worth knowing before someone tries to use it.

---

### Cost self-report

Mirrors the `handback:` front-matter.

- **Tokens (total):** **7,757,521**
- **Estimated USD:** null — no published rate applied; all three prior cycles left
  this null, and an invented number is worse than none.
- **Duration (minutes):** ~55
- **Source of the number:** transcript sum. `/cost` is a client-side slash command
  the assistant cannot execute, so this is summed from this session's own `usage`
  objects (`~/.claude/projects/-Users-…-irradiance/cdc06a2c-….jsonl`).

⚠ **DEDUPED BY `message.id`, and I say so.** A transcript writes one JSONL line per
content block and repeats the same `usage` object on each, so a naive sum
double-counts every multi-block message:

```
usage objects (raw lines) : 120
distinct message.id       :  54
raw sum                   : 17,065,674
DEDUPED sum               :  7,757,521
inflation factor          : 2.20x
```

Composition (deduped): input 108 · output 45,584 · cache-write 184,245 ·
cache-read **7,527,584** — **97.0% cache-read**. It is a **floor**: computed
before the session closed, so the true figure is somewhat higher.

**On the correction factor.** This is the **seventh** measured observation, and at
**2.20×** it sits near the top of the band: **1.61× / 1.76× / 1.82× / 1.86× /
1.95× / 2.20× / 2.25×**. A 1.4× spread over seven measurements is not noise around
a constant — the factor tracks how block-heavy a session is, and this one was
tool-heavy (ten gates, six mutations, two fuzz runs), which is exactly why it
landed high. **No fixed correction is valid on any raw figure.** That still applies
to `SPEC-001`'s `cost.totals` of **51,979,929**, which remains a raw
double-counted sum: it must be **re-summed with dedup from its own transcript**,
not divided by anything. Flagged at build, at verify, at the punch-list round, and
again here.

### Drift and new artifacts

- **New decisions emitted:** none. `DEC-012` is the punch-list round's and I would
  keep it as written; my two additions to it (FU-11, FU-12) are refinements, not
  a supersession.
- **Deviations from spec:** none found. All seven acceptance criteria met,
  including AC7's ten-gate form. The build's three scope widenings — three matrix
  rows, the FU-3 fix, and the malformed-tag rule going to a `DEC` rather than a
  doc comment — are each additive, each disclosed, and each correct; I would have
  flagged the absence of the first two.
- **Follow-up work identified:** FU-9 … FU-15 above. Of the punch-list round's
  own list, I confirm and leave as stated: FU-6 (multi-strip, now in
  `CHANGELOG.md`'s Known gaps at `:64-66`), FU-7 (`#[allow]` wording), F4 (fuzz
  not in CI — still worth its own spec), F5 (`measured-q2m-dng.md` has no tag
  numbers), and the `handback-sync` double-count, whose hand-stamp workaround is
  filed as template finding 15 with three ordered fixes.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing was unclear, and the pre-reconciliation was worth more than usual:
   being handed *"`src/` is byte-identical, ten gates green, SB-1 red-proofed both
   directions"* meant I spent no time re-establishing what was already settled and
   all of it on the two things the orchestrator could not settle from outside —
   whether the widening was *right*, and whether the gate's teeth survive
   directions nobody had tried. What did cost me time was self-inflicted and worth
   recording: **my first two planted faults were caught by
   `ifd_survives_single_byte_corruption`**, so I had to design the fault upward
   until only the fuzzer could see it. That is a fact about the test suite, not an
   obstacle — the deterministic sweep is stronger than I assumed — but "plant a
   lint-clean fault" is only half the requirement, and the other half (*plant one
   the deterministic tests cannot reach*) is not written down anywhere. It should
   be, next to the two `+toolchain` traps.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `DEC-003` — the two-tier corpus policy. It is not in the handoff's list, yet
   it decides two of the five scrutiny items: the Pentax reclassification is
   *literally* a DEC-003 tier question, and FU-10 exists only because tier B is
   invisible to CI by DEC-003's design. Reading the tier reclassification without
   DEC-003 in hand, you can check the arithmetic (37 MB, uncommitted) but not the
   thing that makes it matter — that this repo deliberately accepted a corpus CI
   cannot see, and therefore has to be scrupulous about which claims are gated and
   which are merely true on one machine. The handoff asked me to check the
   reclassification "especially, because it changes what CI is claimed to cover" —
   which is the right instinct, and DEC-003 is where that instinct is written down.

3. **If you did this task again, what would you do differently?**
   — Test the *argument* before testing the fix. The handoff asked "per-crate
   exception vs widening `allow` — agree?", and my first instinct was to read the
   reasoning in `deny.toml` and judge it, which is what the question invites. The
   reasoning is good, but it makes a **behavioural** claim — *"a second crate
   arriving with NCSA fails loudly"* — and behavioural claims are testable in one
   command. I only thought to run RED 3 after RED 1 and RED 2 were done, as an
   afterthought, and it turned out to be the most informative of the three: it is
   the only one that distinguishes the chosen remedy from the rejected one. The
   general form, and it is the same lesson one level up from the one this whole
   ship-blocker taught: **a rationale that predicts an outcome is an oracle, and
   this repo's rule for oracles is that they must be shown red.** I should reach
   for the mutation first and the reading second, not the other way round.
