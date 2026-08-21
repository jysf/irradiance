---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-003
  type: story                      # epic | story | task | bug | chore
  cycle: ship                       # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved         # approved | punch-list | rejected — the OUTCOME of the verify
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
  to_agent: claude-opus-5          # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: 2026-08-20

references:
  decisions: [DEC-008, DEC-011, DEC-012]  # [DEC-NNN, DEC-MMM]
  constraints:                     # [constraint-id-1, constraint-id-2]
    - no-panics-on-untrusted-input
    - provenance-recorded-per-algorithm
    - no-copyleft-dependencies
    - test-before-implementation
    - oracle-must-be-shown-red
    - library-not-application
    - no-new-top-level-deps-without-decision
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-001, SPEC-002]                # blocking order, declared at frame

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "delivers the container half of the stage thesis"

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
      agent: claude-opus-5
      interface: other
      tokens_total: 10967269
      estimated_usd: null
      duration_minutes: 75
      recorded_at: 2026-08-20
      notes: "Build cycle for SPEC-003 (HANDOFF-011), commit b79c7ef on feat/spec-003-ifd-reader, not merged. All 7 acceptance criteria met; nine gates green and pasted in the handback. The fuzz target ships in this change and BOTH directions are pasted: a planted unchecked index in Container::payload gave exit 77 plus crash artifact crash-88173bfa in under a second ('range start index 64 out of range for slice of length 26'), and the input libFuzzer reported was our own count-overflow SEED - tag 273 StripOffsets, LONG, count 0xFFFFFFFF - so the hand-built tier-A corpus caught it on the seed pass; removing the fault gave 12,992,033 runs in 61 s with zero artifacts (an earlier clean run did 14,863,561). Tag extraction matches exiftool 13.55 on all 7 corpus files, read through SPEC-002's manifest reader with no hardcoded paths. NO #[allow] of any policy lint was needed anywhere - the panic-free constraint cost nothing and improved two decisions (a single checked u64 choke point for count x sizeof(type), and packed_bits() returning bits rather than bytes so DEC-008's remainder rule stays STAGE-002's). One new decision: DEC-011 (libfuzzer-sys in a separate fuzz crate; [dependencies] still empty). TWO MEASURED CORRECTIONS to this spec's own notes: only ONE corpus file is big-endian, not two (six II, one MM - checked on the raw header bytes and with exiftool -ExifByteOrder); and K3III.PEF has NO SubIFD at all - it is the only file with a real IFD chain (IFD0->IFD1->IFD2), its plane is in IFD0, and it writes no NewSubfileType tag anywhere, which is what makes TIFF's absent-means-0 default load-bearing rather than decorative. tokens_total is a transcript sum DEDUPED BY message.id and says so: 122 usage objects, 64 distinct ids, raw 19,980,303 vs deduped 10,967,269 = 1.82x inflation, 97.0% cache-read. It is a FLOOR - written before the session closed. Consistent with the 1.7x-2.25x SPEC-002 measured; SPEC-001's totals of 51,979,929 are still raw and should be re-summed."
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 9036505
      estimated_usd: null
      duration_minutes: 55
      recorded_at: 2026-08-20
      notes: "Verify cycle for SPEC-003 (HANDOFF-012), reviewing b79c7ef at HEAD 644815f on feat/spec-003-ifd-reader, not merged. VERDICT: PUNCH LIST - one ship-blocker, documentation and config only, no src/ change. Nine gates re-run by the reviewer and all green (48 tests; MSRV via the rustup shim - bare 'cargo +1.90.0' fails with 'no such command', the FIRST +toolchain trap, and it is the only gate with no just recipe). BOTH fuzz directions run personally: direction 1 planted a DIFFERENT lint-clean fault at a DIFFERENT site from the build's - split_at in Container::read_ifd (the walk path, not the payload path) - and the negative control was measured first, just lint exit 0 AND just lint-no-allow exit 0 WITH the fault in place, proving the fuzzer is the only thing that can see it; libFuzzer then found it from a ZERO-SEED corpus in ~38,900 execs ('mid > len', crash-decd0828, exit 1), synthesising II/version-42/IFD0-offset/entry-count-6608 itself, which is a stronger red than the build's seed-pass catch. Direction 2 restored byte-for-byte (sha256 9c965c48..., grep DELIBERATE FAULT = 0) and gave 16,832,041 runs in 61 s, zero artifacts, git status clean. SHIP-BLOCKER SB-1: DEC-011's licence table is wrong for the one crate it exists to sanction - libfuzzer-sys 0.4.13 declares '(MIT OR Apache-2.0) AND NCSA', not 'MIT OR Apache-2.0'; NCSA is not in deny.toml's allow list, so DEC-011's claim that no exception was needed is false; the enumeration also omits cfg-if, getrandom and r-efi, the last declaring 'MIT OR Apache-2.0 OR LGPL-2.1-or-later'; and the premise that cargo deny cannot reach fuzz/ is itself wrong - 'cargo deny --manifest-path fuzz/Cargo.toml check licenses' runs and catches exactly what the hand-check missed. Substance is fine (NCSA is permissive, nothing copyleft is linked); the RECORD is wrong, on a blocking constraint, in the only document standing in for an absent gate. Fix is four lines across DEC-011, deny.toml, fuzz/Cargo.toml (currently unlicensed) and constraints.yaml's enforcement field. Eight follow-ups: a THIRD wrong fact in the spec's corpus paragraph beyond the two known (three JPEG-compressed is wrong - two are JPEG code 7, K3III.PEF is code 65535 vendor-private - and HANDOFF-012 itself repeats it); the byte-order error also lives at CHANGELOG.md:31 (5 II / 1 MM / 1 PEF conflates byte order with container, the PEF is II too) and NOT in docs/conformance-matrix.md as the handoff said; 'full-resolution SubIFD' in AC6 and STAGE-001 is unsatisfiable for the PEF, whose plane is IFD0; conformance-matrix.md is stale in the way its own opening rule forbids - three held bodies now read end-to-end have no row, and its 'validates against ONE camera' section is false at the container level; malformed-tag policy is asymmetric and unstated (array() tolerates, SubIFDs via uints() is fatal to the whole container); the multi-strip gap is ASSERTED not merely uncovered (tests/ifd_reader.rs:352,443,448); 'no #[allow] anywhere in src/' is imprecise - two exist on cfg(test) modules, both sanctioned and invisible to the --lib-scoped gate; and the MSRV gate needs a just recipe. Guards verified on EVERY recursion path including the chain's next pointers - one shared visited vec threaded through the whole walk, depth checked at walk_chain entry, MAX_IFDS bounding the acyclic case, four shape-separated tests including a depth test that uses distinct offsets so the cycle guard cannot be what stops it. UnsupportedCompression boundary is structurally right: sensor() reads tags only and never dereferences StripOffsets, so rejection is a separate explicit require_uncompressed() and compressed files stay fully tag-readable. packed_bits() in bits is the right call and should stay - bytes would force the remainder decision that IS DEC-008's rule. tokens_total is a transcript sum DEDUPED BY message.id and says so: 113 usage objects, 71 distinct ids, raw 14,592,470 vs deduped 9,036,505 = 1.61x inflation, 97.9% cache-read. It is a FLOOR - written before the session closed. The 1.61x sits just under SPEC-002's 1.7x-2.25x band and the build's 1.82x, which confirms the factor is not a constant and no single correction should be applied to anyone's raw figure."
    - cycle: build
      agent: claude-opus-5
      interface: other
      tokens_total: 9733599
      estimated_usd: null
      duration_minutes: 60
      recorded_at: 2026-08-20
      notes: "Second BUILD cycle for SPEC-003 (HANDOFF-013) - the punch-list round closing the verify cycle's ship-blocker. Commit on feat/spec-003-ifd-reader, not merged. NO src/ CHANGE, as the handoff required: git status shows twelve modified files and one new decision, none under src/, and the reader is byte-identical to b79c7ef. SB-1 CLOSED, all three parts reproduced by me before fixing: cargo deny --manifest-path fuzz/Cargo.toml check licenses ran and FAILED on the untouched tree, on exactly two defects - libfuzzer-sys 0.4.13 declares (MIT OR Apache-2.0) AND NCSA, conjunctive, and irradiance-fuzz carried no license field at all, which is unlicensed and an error not a warning. Fixed by a NAMED per-crate exception in deny.toml rather than widening allow (allow is a standing graph-wide sanction and NCSA is here for one fuzz-only reason; a second NCSA crate should fail loudly), a license field on fuzz/Cargo.toml, and a DEC-011 licence table re-measured from cargo metadata rather than recollection - twelve packages, three of which the old table omitted, including r-efi 6.0.0 whose MIT OR Apache-2.0 OR LGPL-2.1-or-later is the only LGPL mention in either graph (disjunctive, so nothing is violated, but it is exactly what a provenance ledger exists to surface). NEW PROVENANCE FACT found while checking: libfuzzer-sys's README says the vendored libfuzzer/ directory is NCSA, but all 49 vendored .cpp/.h/.def files carry the post-2019 LLVM header Apache License v2.0 with LLVM Exceptions and NONE mentions NCSA or the University of Illinois - counted, not sampled. So the crate's declared SPDX expression and its own README are both stale against the code it ships; every reading is permissive and the gate enforces the stricter one. THE GATE IS NOW WIRED, which was the point: just deny-fuzz, CI job licenses-fuzz using cargo-deny-action's documented manifest-path input, AGENTS.md 6, and constraints.yaml's enforcement field naming both invocations. RED-PROOFED in both directions per oracle-must-be-shown-red: control green (exit 0), exception removed -> exit 4 rejected, license field removed -> exit 4 unlicensed, restored -> exit 0. Both halves of the fix are individually load-bearing. TEN GATES GREEN, run by me: fmt, clippy, test (48 - 31 lib + 9 corpus + 8 ifd_reader), just msrv, just deny, just deny-fuzz, lint-red-proof, lint-no-allow, cost-audit, decisions-index. Plus a fuzz smoke test because I touched the manifest cargo-fuzz reads: 5,757,081 runs in 21 s, zero artifacts, git status unchanged. THIRD +toolchain trap reproduced and closed: bare cargo +1.90.0 check exits 101 with no such command, just msrv exits 0; recipe added, trap documented, and MSRV is no longer the one gate of the ten with no recipe. THREE FACTUAL CORRECTIONS, each re-measured on the files rather than transcribed: byte order is 6 II / 1 MM (raw magic bytes of all seven; CHANGELOG:31's 5 II / 1 MM / 1 PEF conflated byte order with container - the PEF is II too); compression is 2 JPEG code 7 plus 1 vendor-private code 65535, not three JPEG (irr ifd on all five distinct shapes); and docs/conformance-matrix.md gained rows for THREE held bodies, not the one the handoff named - M Monochrom, M Monochrom Typ 246 and K-3 III Monochrome were all read end-to-end with no row, so fixing only one would have left the same defect twice. Its validates-against-ONE-camera section is corrected to the layer where it is now false (container: four bodies, seven files) while keeping it where it is still true (develop: one body), and its tier-A claim about the Pentax fixture is corrected to tier B - a 37 MB uncommitted file cannot gate anything; the tier-A synthetic in src/ifd.rs:1374 is what actually runs in CI. NEW DECISION DEC-012 states the malformed-tag rule: STRICT ON STRUCTURE, TOLERANT ON SHAPE - malformedness that changes what EXISTS (header, entry table, chain next, SubIFDs 330) is fatal to the container; malformedness that changes only what a known-optional fixed-length tag SAYS costs that tag and is reported in malformed_tags. The asymmetry is kept and is narrower than FU-5 framed: array() tolerates a wrong COUNT and nothing else - a wrong field type is fatal there too, via the ? on its uints() call. SubIFDs stays structural because a tolerant 330 yields a container that is structurally a lie, and SPIKE-001 measured how close the resulting wrong answer is - a Q2M SubIFD2 preview 56 px narrower than the plane. ALSO FIXED, not in the handoff's list but the same defect in the other durable doc: FU-3's full-resolution SubIFD in AC6 and STAGE-001, unsatisfiable on the PEF whose plane is IFD0. tokens_total is a transcript sum DEDUPED BY message.id and says so: 127 usage objects, 74 distinct ids, raw 17,087,494 vs deduped 9,733,599 = 1.76x. 97.9% cache-read. It is a FLOOR - computed before the session closed. The five measured factors are now 1.61x/1.76x/1.82x/1.86x/1.95x/2.25x: NOT a constant, so no fixed correction may be applied to any raw figure, including SPEC-001's cost.totals of 51,979,929 which is still a raw double-counted sum and must be re-summed with dedup, not divided. WARNING FOR THE ORCHESTRATOR, measured with --dry-run: just handback-sync SPEC-003 would append DUPLICATE build and verify sessions for HANDOFF-011 and HANDOFF-012, because both already have hand-written entries here yet both still read synced_at: null, and the script keys idempotence on synced_at alone with no check against existing sessions. I did not edit synced_at - the template says do not - so this entry is hand-written to match precedent. Stamp 011 and 012 synced, or delete the hand-written entries before syncing; do not run it as-is."
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 7757521
      estimated_usd: null
      duration_minutes: 55
      recorded_at: 2026-08-21
      notes: "Second VERIFY cycle for SPEC-003 (HANDOFF-014), reviewing d284ff3 on feat/spec-003-ifd-reader, not merged. VERDICT: APPROVED at d284ff3 - seven follow-ups, NO ship-blockers. SB-1 is closed and the gate that closed it has teeth. src/ byte-identical to b79c7ef: empty diff AND sha256 9c965c4842e82450... on src/ifd.rs, which is the SAME digest HANDOFF-012's verify cycle recorded when it restored its own planted fault, so two verify cycles a build round apart have independently hashed the reader and agree it has not moved. TEN GATES re-run by me, all green: fmt, clippy, test (48 - 31 lib + 9 corpus + 8 ifd_reader), just msrv (1.90.0 via the shim), just deny, just lint-red-proof (control 0 -> injection 101, all five lints at 4 distinct injected lines, still red without CI's -D warnings), just lint-no-allow, cost-audit, decisions-index --check, just deny-fuzz. DENY-FUZZ RED-PROOFED THREE DIRECTIONS, each with the mutation ASSERTED to have changed the file first - the exceptions=[...] ARRAY trap that silently no-opped the orchestrator's first attempt. RED 1 exception removed -> exit 4, error[rejected] naming '(MIT OR Apache-2.0) AND NCSA'. RED 2 fuzz/Cargo.toml license field removed -> exit 4, error[unlicensed]. RED 3 (MINE, and the one nobody had run) exception re-pointed at a crate absent from the graph -> exit 4: the first direct evidence that the NAMED-exception-over-widened-allow reasoning in deny.toml:59-67 has teeth, because it makes a behavioural claim ('a second crate arriving with NCSA fails loudly') that was until now only reasoned. Library just deny stayed exit 0 under all three. Control and restore both 0, deny.toml byte-identical. CI PARITY also measured, because a green local recipe says nothing about the job: the licenses-fuzz job passes manifest-path with no --config, and cargo-deny resolves the ROOT deny.toml for a manifest one directory down even from a foreign cwd (verified from /tmp, exit 0, diagnostics pointing at the repo-root deny.toml). FUZZ RED-PROOF at a THIRD distinct site - build used Container::payload, verify round 1 used Container::read_ifd, I used Container::walk_chain. Negative control FIRST and iterated: my first two faults (odd next; odd next inside a SubIFD chain) were both caught by ifd_survives_single_byte_corruption, which is a fact about the strength of the deterministic sweep, so I widened the trigger to require TWO odd offsets - beyond any single-byte flip. With the panic live: just lint exit 0, just lint-no-allow exit 0, cargo test --all-features exit 0 with 48 passed - the fuzzer the only thing that can see it. libFuzzer found it in ~3,600 execs past INITED from a FRESH empty corpus dir plus the 22 committed seeds, synthesising MM / version 42 / IFD0 at ODD offset 5 / a chain next that is ALSO odd; 'mid > len', crash-e794e4ea, EXIT=1. Direction 2 restored byte-identical (sha256 matches, grep DELIBERATE FAULT = 0) and ran 13,053,759 runs in 61 s on the red run's OWN corpus WITH the crash reproducer added back, zero artifacts, git status clean. CORPUS FACTS RE-MEASURED BY ME, not transcribed, because two prior rounds got them wrong: raw magic bytes of all seven give 6 II / 1 MM (M2462362.DNG the only 4d4d), and irr ifd gives 4 uncompressed (code 1) + 2 JPEG (code 7) + 1 vendor-private (65535). Both corrections confirmed. PENTAX TIER RECLASSIFICATION CORRECT: manifest says tier = b, the file measures 37,669,430 bytes, and the replacement claim checks out - the tier-A synthetic a_malformed_fixed_length_tag_costs_the_tag_not_the_file at src/ifd.rs:1374 is a LIB UNIT TEST, so it is inside the 31 that run on a bare CI runner with no corpus. A tier-A claim about an uncommitted 37 MB file was a claim that CI covered something it has never seen. SCOPE WIDENING RIGHT: three matrix rows rather than the one named, because conformance-matrix.md:3's own rule ('every camera gets a row the day it is known') was violated identically three times and fixing one would have left the defect twice; additive, disclosed, and src/ did not move a byte, which is what distinguishes this from scope creep. DEC-012 SOUND and its deferral MEASURED rather than taken on trust: I appended a newline to src/ifd.rs and just decisions-audit --changed named DEC-008 and DEC-012, then restored. affected_scope: src/ifd.rs is load-bearing - anyone editing the reader is told before committing, which is broader coverage than the doc comment the round was forbidden to write. Its narrowing of FU-5 verified against the code: array() at :810 carries a bare ? on uints(), so it tolerates a wrong COUNT and nothing else; tag 330 at :673 routes through uints() and :623 propagates with ?, so it is fatal to the container. PROVENANCE CLAIM RE-COUNTED AND EXTENDED: 49 top-level vendored .cpp/.h/.def (the round's number, correct), 55 including libfuzzer/afl and libfuzzer/dataflow, all 55 carrying the post-2019 LLVM Apache-2.0-with-exception header, and 0 of the 56 files under libfuzzer/ mentioning NCSA or the University of Illinois anywhere. SEVEN FOLLOW-UPS, none ship-blocking. FU-9: docs/provenance-ledger.md:42-49 needs a Standing-decisions bullet for the declared-vs-carried case - it is recorded in DEC-011:128-142 and constraints.yaml:45 but not in the document whose opening paragraph states that exact distinction as its reason to exist; and it runs the INVERSE direction from the ledger's own motivating example (demosaic declares permissive and carries copyleft; libfuzzer-sys declares stricter than it carries), which the ledger's framing quietly assumes cannot happen. FU-10: docs/conformance-matrix.md:24-48's new 'four bodies, seven files' claim omits that all seven manifest entries are tier b and ZERO are tier a, so none of that coverage runs in CI - the same defect the same edit corrected twenty lines below at :79-86, and in scope by the build's own widening rule. FU-11: DEC-012's one-question rule does not predict its widest-blast-radius case. is_sensor_ifd (src/ifd.rs:836-841) propagates scalar() errors with ?, and sensor_candidates :848-856, sensor_ifd :859-866 and sensor :873-880 all call it over EVERY IFD, so a malformed NewSubfileType, PhotometricInterpretation or SamplesPerPixel on a thumbnail or any unrelated chain link is fatal to sensor selection for the whole file. By the DEC's stated test that is interpret-phase and should cost 'that call only'; the outcome is much closer to costing the file. Matters now because SPEC-004 widens uints(), which is what scalar() calls - the exact inheritance DEC-012 exists to pre-decide. FU-12: SPEC-004's references.decisions is [] while DEC-012:126-131 aims its deferral at SPEC-004's first edit; AGENTS.md 15 build step 3 sends the agent to that empty list, and decisions-audit --changed is advisory, manual, not in CI and only fires on uncommitted changes. One line adds DEC-012 and DEC-008. FU-13: cost.totals.tokens_total is 10967269, exactly the FIRST build session, where three sessions sum to 29737373; introduced at d867403, inherited not caused by this round, harmless while unshipped (calibration lists only shipped specs; every shipped spec has totals == sum) but a wrong-but-plausible number is the exact failure mode this round was about. With this cycle: 37,494,894. FU-14: oracle-must-be-shown-red (constraints.yaml:52-56) says 'every ORACLE', but three GATES are now red-proofed (lint-red-proof, the fuzz target, deny-fuzz) and HANDOFF-013's own reflection asked for the rule to widen; right now that lives only in a handback, the least durable place in this repo, and no signal covers it. FU-15: the full-resolution-SubIFD phrasing FU-3 corrected in AC6 and STAGE-001:58 survives at STAGE-002:84 - defensible as scoped, but the identical wording and reachable by the same rule the build used to widen the matrix. MINOR, measured: just deny on the library graph now permanently emits warning[license-exception-not-encountered] for the libfuzzer-sys exception - one policy file, two graphs, so each reports the other's entries as unmatched; no effect on exit status, but that warning class can no longer carry signal on the library graph. tokens_total is a transcript sum DEDUPED BY message.id and says so: 120 usage objects, 54 distinct ids, raw 17,065,674 vs deduped 7,757,521 = 2.20x, 97.0% cache-read. It is a FLOOR - computed before the session closed. This is the SEVENTH measured factor and the band is now 1.61x / 1.76x / 1.82x / 1.86x / 1.95x / 2.20x / 2.25x - a 1.4x spread over seven observations, tracking how block-heavy a session is rather than how large it is, so NO fixed correction is valid on any raw figure. SPEC-001's cost.totals of 51,979,929 is still a raw double-counted sum and must be RE-SUMMED with dedup from its own transcript, not divided by anything. DID NOT run handback-sync on this spec, per the handoff; HANDOFF-014's synced_at is hand-stamped with the reason inline, matching the precedent set at 93dcae0 for HANDOFF-011/012/013."
  totals:
    tokens_total: 10967269
    estimated_usd: 0
    session_count: 1
---

# SPEC-003: TIFF/IFD reader — bounded, panic-free, cycle-guarded, SubIFD recursion — plus its fuzz target

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-003]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

Every later stage reads tags, so a wrong container walk silently poisons all of
them. RAW is attacker-influenced binary (`no-panics-on-untrusted-input`), and
this is the first spec to touch it.

## Goal

A bounded, panic-free TIFF/IFD reader with SubIFD recursion, plus its fuzz
target **in the same change** (AGENTS.md §12 — a retrofitted fuzz target tests
the shape the code already has). Depth-limited, cycle-guarded on visited
offsets, every offset and length bounds-checked into a typed error.

Sensor-IFD selection keys on `NewSubfileType == 0 && Photometric == 34892 &&
SamplesPerPixel == 1` — **never on largest dimensions**; `SubIFD2` is a
full-resolution JPEG preview only 56 px narrower than the plane.

⚠ SPIKE-001 built a working version and it is **discarded** — do not consult it
as an implementation. Re-derive test-first.

## Inputs

*(Filled at build from what was actually read — 2026-08-20.)*

- **Files to read:** `guidance/toolchain-brief.md` (the two `+toolchain` traps),
  `guidance/constraints.yaml`, `AGENTS.md` §11/§12/§13,
  `docs/measured-q2m-dng.md` (the tag set), `tests/corpus/manifest.toml`,
  `tests/support/corpus.rs` (SPEC-002's reader — the only route to a corpus
  path).
- **External APIs:** none. TIFF 6.0 (1992) §2 and the Adobe DNG Specification
  1.7.1.0 are the sources, as published specifications — provenance class 1.
- **Oracle tools, run and never linked:** `exiftool 13.55` for the tag
  cross-check, `dnglab 0.7.2` for the pinned `raw_checksum` the manifest
  already holds.
- **Related code paths:** `src/lib.rs` (the crate's `Error` type and the
  panic-free lint policy).

## Outputs

*(Filled at build — 2026-08-20.)*

- **Files created:**
  - `src/ifd.rs` — the reader.
  - `tests/ifd_reader.rs` — corpus + hostile-input tests.
  - `tests/support/tiff.rs` — hand-built tier-A byte fixtures, shared by the
    test lane and the fuzz-seed writer.
  - `fuzz/Cargo.toml`, `fuzz/fuzz_targets/ifd.rs`, `fuzz/seeds/ifd/*` — the
    fuzz target and its committed seed corpus.
  - `examples/fuzz-seeds.rs` — regenerates those seeds.
  - `decisions/DEC-011-*.md` — `libfuzzer-sys` in a separate crate.
- **Files modified:** `src/lib.rs` (13 new `Error` variants, `pub mod ifd`),
  `src/bin/irr.rs` (the `ifd` subcommand), `app.just` + `AGENTS.md` §6 (the
  `fuzz` recipes, and a correction — §6 documented the invocation that does not
  work), `docs/provenance-ledger.md` (first real row), `.gitignore` (`*.PEF`),
  `CHANGELOG.md`.
- **New public API:** `ifd::Container::{parse, ifds, ifd0, payload, uints,
  scalar, required_scalar, values, is_sensor_ifd, sensor_candidates,
  sensor_ifd, sensor}`, `ifd::{Ifd, Entry, Sensor, Compression, ByteOrder}`,
  the `TAG_*` constants, and `MAX_IFD_DEPTH` / `MAX_IFDS` / `MAX_TAG_VALUES`.
- **New flags / options:** `irr ifd [--entries] <file>` — `--entries` defaults
  to **off** and adds a per-entry tag/type/count dump.
- **Database changes:** none.

## Acceptance Criteria

1. A TIFF/IFD reader walks IFD0's chain and recurses `SubIFDs` (tag 330), reading
   entry tags, types, counts and payloads.
2. **Every** offset and length read is bounds-checked and returns a **typed
   error** — no `unwrap`, no indexing, no unchecked arithmetic on any parse path
   (constraint `no-panics-on-untrusted-input`; the lint policy makes this
   mechanical, so it should be a compile-time property, not a review one).
3. **Depth-guarded and cycle-guarded.** A SubIFD chain that points at itself, or
   nests arbitrarily, terminates with an error rather than recursing forever.
4. **A fuzz target ships in this change** (§12 bar 2 — not retrofitted), seeded
   from tier-A fixtures including truncated and malformed inputs.
5. **The fuzz target is shown to WORK**, not merely to exist: a deliberately
   unchecked index, planted temporarily, must be found by libFuzzer and produce a
   crash artifact. Paste that. A fuzz target that has never caught anything is
   the "green oracle that cannot fail" in another costume.
6. On the real corpus, the reader reaches the full-resolution **sensor IFD** —
   *not* "SubIFD": `K3III.PEF` has none and keeps its plane in `IFD0` — and reports
   dimensions, bit depth, compression, levels, `ActiveArea`, `DefaultCrop`,
   `Orientation` and opcode-list presence, matching `exiftool` on all 7 files.
7. All gates stay green. ⚠ **Nine at design; TEN as shipped.** The punch-list
   round added `just deny-fuzz` — the licence gate over `fuzz/`, which DEC-011
   had recorded as unreachable and hand-checked instead. It reaches fine, and it
   was failing the whole time (HANDOFF-013, SB-1).

## Failing Tests

```bash
# reader reaches the sensor IFD on every corpus file that is present
cargo test --all-features ifd_reaches_sensor_plane

# hostile input: truncated header, cyclic SubIFD, absurd offsets/counts
cargo test --all-features ifd_rejects_hostile_input

# the fuzz target exists, builds, and runs
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd -- -max_total_time=60
```

**The red-proof for criterion 5** — plant an unchecked index, run the target, and
libFuzzer must produce a crash artifact under `fuzz/artifacts/`.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- **Any pixel decode or unpack.** STAGE-002, where `DEC-008`'s two-path
  (`bits % 8`) rule lands. `StripOffsets`/`StripByteCounts` are read here as
  *tags*; reading the strip they point at is not in scope. Held at build:
  `Sensor::packed_bits()` deliberately returns **bits**, not bytes, so the
  remainder question stays STAGE-002's to answer.
- **The typed tag model.** `SPEC-004`. This module widens `BYTE`/`SHORT`/`LONG`
  to `u32` and returns `Error::UnexpectedFieldType` for `RATIONAL` and the
  signed types rather than guessing.
- **A live metadata oracle.** `SPEC-005` diffs parsed tags against
  `dnglab analyze` and `exiftool` at run time. Here the `exiftool` answers are
  *pinned* as an expected table, checked by hand at build.
- **Decoding the DNG opcode streams.** Presence only (`OpcodeList1/2/3`);
  `WarpRectilinear` and `FixBadPixelsConstant` are STAGE-003.
- **Lossless JPEG (SOF-3) or Pentax PEF decompression.** Three corpus files
  need one of these and are rejected cleanly instead; PROJ-003.
- **Widening the lint exceptions.** None was needed — see the handback.

## Notes for the Implementer

### ⚠ `cargo fuzz` DOES NOT WORK with the default PATH — measured at design

This is a hard blocker and it is not obvious. `cargo fuzz` shells out to a bare
`"cargo" "build"`, and that inner `cargo` resolves to **Homebrew's stable cargo**,
which rejects `-Zsanitizer=address`:

```
error: 1 nightly option were parsed
Error: failed to build fuzz script
```

Even `~/.cargo/bin/cargo +nightly fuzz run` fails, because the *inner* invocation
is what breaks. **The fix is to put the rustup shim first on PATH:**

```bash
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run <target>
```

Verified end to end at design: `cargo fuzz init` works; a target then built and ran
**32.9 M executions in 16 s**; and a deliberately unchecked index was **found**,
producing `Error: Fuzz target exited with exit status: 77` and a crash artifact.
So criteria 4 and 5 are both known-achievable — the mechanism is proven before you
start.

### What SPIKE-001 established — as facts, not as code to copy

Its decoder is **discarded on an unmerged branch and must not be consulted as an
implementation** (`test-before-implementation`); re-derive test-first. What it
*measured* is reusable:

- Selection: `NewSubfileType == 0 && Photometric == 34892 (LinearRaw) &&
  SamplesPerPixel == 1` — **never by largest dimensions**; `SubIFD2` is a
  full-resolution JPEG preview only 56 px narrower than the plane.
- The guards needed: depth limit, cycle detection on visited offsets,
  bounds-checked payload ranges.
- ⚠ Its version used **bounds-check-then-index** (`buf.get(..)?` then `s[0]`),
  which the lint policy **rejects**. Use `try_into` on the slice. Its measured
  "229 lines" is therefore an underestimate; do not treat it as a target.

### Corpus facts that shape the tests

Seven files, `tests/corpus/manifest.toml`, read via the SPEC-002 reader — **do not
hardcode paths**.

⚠ **Three facts in this paragraph were wrong as designed. All three are corrected
below, each measured on the files themselves.** Build caught the byte-order count
and the PEF's missing `SubIFDs` (HANDOFF-011); verify caught the compression count
— and caught that HANDOFF-012 had repeated it (FU-2). They are left visible rather
than silently swapped, because a design that asserted corpus numbers without
measuring them is the thing worth remembering.

- **Byte order: SIX `II`, ONE `MM`** — not "two big-endian where five are `II`".
  Only `M2462362.DNG` (M Monochrom Typ 246) is `MM`. Read off the raw two-byte
  magic of all seven, three times now by three agents.
- **Compression: TWO are JPEG, not three.** `M2462362.DNG` and `K3III.DNG` are
  `Compression == 7`. The third undecodable file, `K3III.PEF`, is
  `Compression == 65535` — a **vendor-private Pentax scheme, not JPEG**. All
  three must be rejected cleanly, and the distinction is load-bearing rather than
  pedantic: PROJ-003 scopes lossless-JPEG (SOF-3) and PEF decompression as two
  different problems, so "three JPEG" implies one unsupported-compression class
  where there are two.
- **`K3III.PEF` has no `SubIFDs` tag at all** — no `SubIFD`, no `NewSubfileType`
  anywhere in the file, its plane in `IFD0`, and it is the only file in the corpus
  with a real IFD **chain** (`IFD0 → IFD1 → IFD2`). So TIFF's *absent-means-0*
  default for `NewSubfileType` is load-bearing, and "the full-resolution SubIFD"
  is unsatisfiable on 1 of the 7 files this spec names. **"sensor IFD"** is the
  phrase that is true for all seven; the code and tests already use it
  (`sensor_ifd`, `ifd_reaches_sensor_plane`).

One (Pentax `K3III.DNG`) carries a `BlackLevelRepeatDim` tag that dnglab itself
warns is malformed — a natural regression fixture, and the reader must not panic
on it. ⚠ It is a **tier-B** file (37 MB, uncommitted), so it is a fixture only
where the corpus is present; `docs/conformance-matrix.md` previously called it
tier A.

### Scope

Container only. **No pixel decode, no unpack** — that is STAGE-002, where
`DEC-008`'s two-path (`bits % 8`) rule lands. Reading `StripOffsets`/
`StripByteCounts` as *tags* is in scope; reading the strip is not.

## Reflection

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
