---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-065
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-019
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-060, DEC-058, DEC-059, DEC-018, DEC-024, DEC-051, DEC-052, DEC-003]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-017, SPEC-018, SPEC-062, SPEC-063, SPEC-064]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-063]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-019's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-019's caller-owned scratch: an allocation-free mono develop for the requester, and one allocation with P + A peak memory for every other caller."

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
  tokens_estimate: 26000000
  sessions:
    - cycle: build
      handoff: HANDOFF-137
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 24495029
      estimated_usd: 16.99
      duration_minutes: 65
      recorded_at: 2026-11-13
      notes: "TRUE cost re-summed at ship by the orchestrator from the ended build transcript 2a702dbf-8f01-4323-84fe-26eccdf7e734.jsonl (106 unique message.ids, all claude-opus-5-5, no subagents): input 216 + cache-creation 246,913 (1-hour) + cache-read 24,150,133 + output 97,767 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 20,612,106 / 14.65 before the handback commit, PR and CI turns."
    - cycle: verify
      handoff: HANDOFF-138
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 17393311
      estimated_usd: 12.95
      duration_minutes: 75
      recorded_at: 2026-11-13
      notes: "TRUE cost re-summed at ship by the orchestrator from the ended verify transcript e26a2c54-c30a-4f6f-9422-53b2d77050c3.jsonl (100 unique message.ids, all claude-opus-5-5, no subagents): input 247 + cache-creation 335,819 (1-hour) + cache-read 17,013,950 + output 43,295 (max per message), opus-5-5 rates. The handback floor was 15,435,140 / 10.10. Verdict APPROVED at fa2c01f with no code changes, posted as a comment on PR #224; 14 tier-A mutants all red; just test 655/0/1 with 12/12 corpus; fuzz develop 4,614,310 and opcode 3,985,931 runs in 66 s each, clean; CI 44/44. #224 merged as 7f233ff."
  totals:
    tokens_total: 41888340
    estimated_usd: 29.94
    session_count: 2
shipped_at: 2026-11-13
---

# SPEC-065: Caller-owned develop scratch: scratch_len plus develop_with_scratch_into, allocation-free

## Context

SPEC-065 is the third spec in STAGE-019 (PROJ-004 instrument layer C). It covers
monorail-000 / onyx-000 request 3 (`feedback/2026-11-11-monorail-000-requests.md`):
`scratch_len(sensor, &options)`, a variant taking `scratch: &mut [u16]`, a test proving no
allocation, and a typed error for wrong-length scratch. It depends on SPEC-063 (shipped),
and SPEC-064 (shipped) decides which warps are applied.

The requester's target files carry both `FixBadPixelsConstant` and `WarpRectilinear`, so
they budget today for the plane copy plus two active-area warp buffers. Their app wants to
own that memory (a pool, or GPU-shared buffers) and develop with no allocation at all.

**DEC-060** (minted with this design) fixes the API shape, the exact-length rule, the
private reuse layout, the allocation-free contract, and the shared core with
`develop_with_into`.

## Goal

Add `develop::scratch_len` and `develop::develop_with_scratch_into`. The second performs no
heap allocation and produces output identical to `develop_with_into` for every input and
options value. `develop_with_into` runs the same core with one internal allocation of
`scratch_len` samples.

## Design

### 1. Probe results (main at 12b0794, 2026-11-12; recorded so build is transcription)

`develop_into` was wrapped in a counting `#[global_allocator]` on each held file that
develops:

| File | Plane P | Active A | `FixBadPixelsConstant` | warp | allocations | bytes |
|---|---|---|---|---|---|---|
| Q2M L1021223, L1026016, L1026192 (each) | 8424×5632 = 47,443,968 | 8392×5632 = 47,263,744 | yes, constant 0 | non-identity | **6** | **283,943,504** |
| M Monochrom L1000622 | 5216×3472 | same | no | none | 0 | 0 |
| Typ 246 M2462362 | 5984×4000 | same | no | none | 0 | 0 |
| K-3 III | 6304×4224 | 6224×4160 | no | none | 0 | 0 |
| Canon 5D3, Nikon D300S | | | no | none | 0 | 0 |

On a Q2M frame, the six allocations are:
- the plane copy (`src.to_vec()`, 94,887,936 bytes);
- two active-area `vec!`s (94,527,488 bytes each);
- **three small ones totalling 592 bytes**:
  - the `Vec<Opcode>` that `parse_opcode_list` builds, once for `OpcodeList1` and once
    for `OpcodeList3`, inside `parse_fix_bad_pixels_constant` and
    `parse_warp_rectilinear`;
  - the neighbours `Vec::with_capacity(8)` in `apply_fix_bad_pixels_constant`.

The stage notes missed the opcode `Vec`s. Removing only the three big buffers would still
fail a zero-allocation test.

What follows from it:
- **Expected `scratch_len`** under DEC-060 §3:
  - Q2M at default options: P + A = **94,707,712** samples (189,415,424 bytes).
  - Q2M at `warp_alpha = 0`: P = **47,443,968** (the warp is skipped; the copy remains).
  - Every warp-free held file: **0**.
- **`develop_with_into` on a Q2M frame** goes from 6 allocations / 283,943,504 bytes to
  **1 allocation / 189,415,424 bytes**.

### 2. Public API (`src/develop.rs`), exactly DEC-060 §1

- `pub fn scratch_len(sensor: &Sensor, options: &DevelopOptions) -> Result<usize, Error>`.
- `pub fn develop_with_scratch_into(sensor: &Sensor, src: &[u16], dst: &mut [u16], scratch: &mut [u16], options: &DevelopOptions) -> Result<(), Error>`.
- **New `Error::DevelopScratchWrongLength { expected: u64, actual: usize }`**,
  `ErrorKind::CallerError`. Its `Display` names both values.
- **Doc comments state:**
  - the exact-length rule, and that `scratch_len` may be 0 (an empty slice is fine);
  - that scratch contents are unspecified on entry and return;
  - that `scratch_len` may change between versions (ask for it, do not hard-code it);
  - the no-allocation promise;
  - that the output equals `develop_with_into`'s;
  - mono only.

  Do **not** document the layout in public docs. It is private (DEC-060 §3); the doc can
  say what the scratch holds in general terms.

### 3. What `scratch_len` computes

It runs, in order:
1. the `warp_alpha` domain check (`InvalidWarpAlpha`);
2. `resolve_geometry` and `apply_options`, giving P from `sensor.width × height` and A
   from the active size;
3. `OpcodeList1`'s first `FixBadPixelsConstant` (`fbp`), via the allocation-free
   `parse_fix_bad_pixels_constant`;
4. `OpcodeList3`'s warp, lerped by `warp_alpha` with identity filtered (`warp`), via
   `non_identity_warp(sensor, alpha)`.

It returns DEC-060 §3's length, computed with checked arithmetic. An overflow is
`DevelopScratchWrongLength { expected: u64::MAX, actual: 0 }`.

It does **not** run `check_required_processing`, the levels check or the `FullActiveArea`
bounds check. It answers "how much scratch", not "will this develop succeed". Say so in
its doc. `develop_with_scratch_into` still runs all of them.

### 4. The shared core

Refactor `develop_with_into` into a private core that takes the scratch source: a
caller's `&mut [u16]`, or "allocate exactly what is needed". Both public functions call
it.

**Validation order:**
0. `warp_alpha`;
1. `check_required_processing`;
2. `src` length;
3. `resolve_geometry`, then `apply_options`;
4. levels;
5. `dst` length;
6. under `FullActiveArea`: parse `OpcodeList3`, lerp, and the bounds check (SPEC-064,
   unchanged);
7. parse `OpcodeList1` (`fbp`);
8. under `StandardCrop`: parse `OpcodeList3` and lerp;
9. **new:** compute the needed length.
   - In the caller-scratch form, refuse a mismatch with `DevelopScratchWrongLength`.
   - In the `develop_with_into` form, allocate one `vec![0u16; needed]`, or nothing when
     `needed == 0`.
10. Run the passes.

Steps 7 and 8 keep today's relative order (`OpcodeList1` before `OpcodeList3` under
`StandardCrop`). Today there is a plane copy between them, but it cannot fail, so moving
it after step 9 changes no error's precedence. Every existing precedence test must stay
green unedited.

**The passes, by case** (the layout from DEC-060 §3):
- **Neither:** `crop_orient_normalize_into(src → dst)`, as today.
- **`fbp` only:** copy `src → scratch[0, P)`, run `apply_fix_bad_pixels_constant` on it,
  then `crop_orient_normalize_into(scratch[0, P) → dst)`.
- **`warp` only:** `normalize_active_area_into(src → scratch[0, A))`, then
  `apply_warp_into(scratch[0, A) → scratch[A, 2A))`, then crop and orient from
  `scratch[A, 2A)` into `dst`.
- **Both:**
  1. copy `src` into `scratch[0, P)` and fix it;
  2. normalize it into `scratch[P, P + A)`;
  3. warp into `scratch[0, A)`, reusing the dead plane copy;
  4. crop and orient into `dst`.

Use `split_at_mut` to get the disjoint halves. No `unsafe` (the crate forbids it), and
no indexing that clippy's panic lints reject.

### 5. Allocation-free opcode lookups (`src/opcode.rs`)

Re-implement `parse_fix_bad_pixels_constant` and `parse_warp_rectilinear` as walkers
over the raw framing (`read_raw_opcode`), with no `Vec`.

**Their results must be identical to today's `parse_opcode_list(bytes)` + `find_map`,
errors included.** That means the walker still has to:
- read every opcode in the list, not stop at the first match;
- parse the parameter block of **every** recognised opcode, so a malformed later
  `WarpRectilinear` or `FixBadPixelsConstant` is still `MalformedOpcodeParams`;
- refuse an unrecognised mandatory ID with `UnsupportedMandatoryOpcode`;
- skip optional unknowns without copying their params;
- return the first instance of the wanted opcode.

`parse_opcode_list` itself is unchanged; it is public and returns owned `Opcode`s.
`apply_fix_bad_pixels_constant` swaps its neighbours `Vec` for `[u16; 8]` plus a count.
It sorts the filled prefix with `sort_unstable` and takes `lower_median` of it, so the
result is identical.

### 6. Fuzz (`fuzz/fuzz_targets/develop.rs`)

After the existing calls, add a differential:
- compute `scratch_len` with the same options;
- if it is `Ok(n)` and `n` is within `MAX_PIXELS × 3`, run `develop_with_scratch_into`
  with a scratch prefilled with `0xA5A5`;
- assert its `Result`, compared by `Debug`, and its `dst` equal `develop_with_into`'s.

Also run it once with scratch of length `n + 1`, and assert `DevelopScratchWrongLength`
whenever `develop_with_into` returned `Ok`.

The `opcode` target gains a differential between the new walkers and
`parse_opcode_list` + `find_map` (Debug-equal results).

No new seeds are required. The SPEC-063/064 develop seeds already cover all four cases.
Confirm that in the handback.

### 7. Docs

- **CHANGELOG `[Unreleased]`:**
  - Added: `scratch_len`, `develop_with_scratch_into` and `DevelopScratchWrongLength`.
  - Changed: `develop_into` / `develop_with_into` now make at most one allocation and use
    P + A in place of P + 2A samples at peak on a file with both opcodes. Quote the Q2M
    figures. Output is unchanged.
  - Changed: `parse_fix_bad_pixels_constant` and `parse_warp_rectilinear` no longer
    allocate.
- **Module doc of `src/develop.rs`:** replace the SPEC-018 "allocation notes" paragraph,
  and `develop_into`'s doc on what it allocates, with the new statement.
- **Provenance ledger:** none needed (no new algorithm). Say so in the handback.
- `DEVELOP_VERSION` stays `"4"`.

## Inputs

- **Files to read:**
  - `src/develop.rs`: `develop_with_into` ~1060 onward (the FBP copy ~1143, the warp
    buffers ~1184/1194), `apply_fix_bad_pixels_constant` ~670, the three passes, and
    `non_identity_warp`;
  - `src/opcode.rs`: `read_raw_opcode`, `parse_opcode_list` ~416,
    `parse_warp_rectilinear` ~469, `parse_fix_bad_pixels_constant` ~505, `check_list`
    (the walker pattern to mirror);
  - `src/lib.rs`: `Error`, `kind()`.
- **Tests to mirror:**
  - `tests/previews_alloc.rs`: the counting allocator, its thread-local switch and the
    negative control;
  - `tests/develop_options.rs` and `tests/warp_alpha.rs`: the tier-B gating, MD5 pins and
    sentinel probes;
  - `tests/opcode.rs`: the opcode fixtures;
  - `tests/error_kind.rs`.
- **Decisions:** DEC-060 (this spec), DEC-058, DEC-059, DEC-018, DEC-024, DEC-051,
  DEC-052, DEC-003.

## Outputs

- **Files modified:**
  - `src/develop.rs`: the two functions, the core, and the allocation-free neighbours;
  - `src/opcode.rs`: the two walkers;
  - `src/lib.rs`: the new variant;
  - `fuzz/fuzz_targets/develop.rs` and `fuzz/fuzz_targets/opcode.rs`;
  - `CHANGELOG.md`;
  - `tests/error_kind.rs`.
- **Files created:**
  - `tests/develop_scratch.rs` (behaviour);
  - `tests/develop_scratch_alloc.rs` (its own binary, holding the counting allocator).
- **New public items:**
  - `develop::scratch_len`;
  - `develop::develop_with_scratch_into`;
  - `Error::DevelopScratchWrongLength`.
- **No new option fields.**

## Acceptance Criteria

- [x] **AC1. `scratch_len` values.**
  - **Tier A.** One fixture per case (neither, `fbp` only, warp only, both), non-square,
    with an `ActiveArea` border so P ≠ A. The function returns 0, P, 2A and P + A
    respectively, under both framings.
  - **Lerp.** On the warp-only and both fixtures, `warp_alpha = 0` returns 0 and P
    respectively.
  - **Tier B.** Each Q2M frame returns 94,707,712 at default options and 47,443,968 at
    `warp_alpha = 0`. The five warp-free held files return 0. A missing file is reported
    by name.
- [x] **AC2. Byte identity.** `develop_with_scratch_into`, given exactly `scratch_len`
  samples, produces the same `dst` as `develop_with_into`:
  - on all four tier-A fixtures;
  - under `StandardCrop` and `FullActiveArea`;
  - at `warp_alpha` 0, 0.5 and 1;
  - with `apply_orientation` true and false.

  On tier B, the SPEC-063 MD5 pins hold through `develop_with_scratch_into` at default
  options on all eight files. On L1021223, the output also equals `develop_with_into`'s
  under `FullActiveArea` at α = 0.5.
- [x] **AC3. The scratch path allocates nothing** (`tests/develop_scratch_alloc.rs`, the
  `previews_alloc` pattern).
  - On each tier-A fixture, under every option combination in AC2, there are zero
    allocations between the switches. The `Sensor`, `src`, `dst` and `scratch` are built
    before counting starts.
  - The same holds on the error paths: a wrong-length scratch, a refused over-unity warp
    under `FullActiveArea`, and an out-of-domain α.
  - **Negative control:** a `Vec` allocated inside the switches is counted.
  - **Tier B:** zero allocations on L1021223 at default options. A missing file is
    reported by name.
- [x] **AC4. `develop_with_into` allocates at most once.**
  - On the "both" fixture: exactly 1 allocation of `scratch_len × 2` bytes. On the
    neither fixture: 0.
  - On L1021223: exactly 1 allocation of 189,415,424 bytes, against 6 / 283,943,504
    before. This pins the peak-memory saving.
- [x] **AC5. Wrong length is typed, and nothing is written.** For the "both" fixture,
  scratch of length `n − 1`, `n + 1` and 0 each give:
  - `DevelopScratchWrongLength { expected: n, actual }`;
  - `kind() == CallerError`;
  - `dst` and `scratch` both unchanged (sentinel fills).

  For the neither fixture (`n == 0`), a scratch of length 1 is refused the same way, and
  an empty one succeeds.
- [x] **AC6. Output never depends on scratch contents.** On the "both" fixture, and on
  L1021223, prefilling the scratch with 0x0000, 0xFFFF and a ramp gives identical `dst`.
- [x] **AC7. Error precedence.** `develop_with_scratch_into` with a correct-length
  scratch returns, for every hostile fixture in `tests/develop.rs`,
  `tests/develop_options.rs` and `tests/warp_alpha.rs` that `develop_with_into` refuses,
  the same error (Debug-equal). Run this as one table-driven test over those fixtures.
  When the scratch is **also** wrong:
  - a malformed `OpcodeList1` is reported, not the scratch length;
  - an out-of-domain α is reported first.

  Every existing precedence test stays green unedited.
- [x] **AC8. The opcode walkers are equivalent and allocation-free.**
  - **Equivalence.** For every opcode-list fixture in `tests/opcode.rs`, plus fixtures
    with a malformed second `WarpRectilinear`, a malformed second `FixBadPixelsConstant`,
    an optional unknown with params, and an unknown mandatory after the match:
    `parse_warp_rectilinear(b)` and `parse_fix_bad_pixels_constant(b)` give results
    Debug-equal to `parse_opcode_list(b)` followed by `find_map`.
  - **Allocation-free.** Both functions allocate nothing, in the alloc binary.
- [x] **AC9. `apply_fix_bad_pixels_constant` allocates nothing.** All its existing tests
  pass unedited, and the alloc binary shows 0 allocations on a plane with bad pixels.
- [x] **AC10. Error surface.**
  - `DevelopScratchWrongLength` is in `Error::kind()` with no wildcard and has a row in
    `tests/error_kind.rs`.
  - A test pins that its `Display` contains both numbers.
  - `scratch_len` overflow is pinned by a unit test or documented as unreachable on
    64-bit, with the reason.
- [x] **AC11. API hygiene.**
  - Both functions are documented as §2 says.
  - `cargo doc` has no broken links.
  - CHANGELOG `[Unreleased]` is updated as §7 says.
- [x] **AC12. Fuzz.** Both differentials in §6 are in place, and 60 s runs of the
  `develop` and `opcode` targets are clean. Report the run counts.
- [x] **AC13. No regression.**
  - Every existing test passes with no assertion changed, other than `error_kind`'s rows
    and counts.
  - `just test` passes with the corpus env set. So do `lint-ci`, `lint-no-allow`,
    `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny` and `msrv`.
  - No new dependency.
  - `DEVELOP_VERSION == "4"`.

## Failing Tests

- **`tests/develop_scratch.rs`:**
  - `scratch_len_per_case_and_alpha`: AC1, tier A.
  - `scratch_len_on_held_files`: AC1, tier B.
  - `scratch_output_equals_develop_with_into`: AC2, tier A.
  - `scratch_output_matches_the_pins`: AC2, tier B.
  - `wrong_length_scratch_is_refused_before_any_write`: AC5.
  - `output_does_not_depend_on_scratch_contents`: AC6.
  - `scratch_variant_reports_the_same_errors_in_the_same_order`: AC7.
  - `scratch_error_display_names_both_lengths`: AC10.
- **`tests/develop_scratch_alloc.rs`** (its own binary):
  - `counter_counts`: the negative control.
  - `scratch_develop_allocates_nothing`: AC3, tier A, including the error paths.
  - `scratch_develop_allocates_nothing_on_q2m`: AC3, tier B.
  - `develop_with_into_allocates_once`: AC4, tier A and tier B.
  - `opcode_lookups_allocate_nothing`: AC8.
  - `fix_bad_pixels_allocates_nothing`: AC9.
- **`tests/opcode.rs`:** `walkers_equal_parse_opcode_list_then_find_map` (AC8).
- **`tests/error_kind.rs`:** the new row (AC10).

**Red-proof expectations** (verify mutates these; each must turn a named test red):
- Any `vec!` or `to_vec` left in the core's scratch path → AC3.
- The opcode walker built on `parse_opcode_list` again → AC8's allocation test.
- The walker stops at the first match (it skips validating later opcodes) → AC8's
  malformed-second-opcode cases.
- The walker skips the unknown-mandatory check → AC8.
- No reuse in the "both" case (layout P + 2A) → AC1 and AC4 (the lengths).
- The warp writes into the plane copy **before** normalize has read it (wrong order in the
  "both" case) → AC2.
- A read of scratch before it is written (for example, the warp source taken from the
  wrong half) → AC6 and AC2.
- The scratch length check moved after a write → AC5's sentinels.
- The scratch length check moved before the `OpcodeList1` parse → AC7.
- "At least `n`" accepted → AC5's `n + 1` case.
- The neighbours array not sorted, or `lower_median` over the whole array → the existing
  `apply_fix_bad_pixels_constant` tests.
- AC2's byte identity against `develop_with_into` cannot see a pass-order or wrong-buffer
  bug inside the shared core → `each_case_equals_its_composition_from_public_parts`
  (public `apply_fix_bad_pixels_constant` composed with the warp-only develop). *(Added at
  ship; the build found the gap with red-proofs 6 and 7b.)*
- The output pass reads `normalized` instead of the warped region, in the warp-only and
  "both" cases → `tests/warp.rs`, `tests/warp_alpha.rs` and the composition oracle.
  *(Added at ship; two extra mutants from the cold verify.)*

## Non-Goals

- **Warping straight into `dst`** to save another A (DEC-060 Alternatives; additive later).
- **A scratch variant of the colour path**, or of `unpack_into`.
- **Making `parse_opcode_list` allocation-free.** It returns owned data by design.
- **CFA-as-mono** (SPEC-066), **the blocked transpose** (SPEC-067), **`irr` CLI flags**,
  and a release.

## Notes for the Implementer

- **`split_at_mut` on the "both" case:** `scratch.split_at_mut(P)` gives
  `(plane, rest)`. Normalize from `plane` into `rest[..A]`, then warp from `rest[..A]` into
  `plane[..A]`. Check that the borrow shapes work before writing the passes; this is the
  only tricky borrow.
- **AC4's byte count.** `vec![0u16; n]` is one `alloc_zeroed` of `2n` bytes. Count
  `alloc_zeroed` in the allocator (the `previews_alloc` one already does).
- **Tier-B alloc test.** Read the file and unpack the plane **before** switching counting
  on. `Container::parse` and the unpack allocate, legitimately.
- **The walker.** Mirror `check_list`'s loop over `read_raw_opcode`, but dispatch params
  exactly as `parse_opcode_list` does. The easy mistake is returning on the first match.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Two changes, both about the design's oracle and coverage claims.
   - **Name each oracle's independence from the code it guards.** I assigned the
     pass-order and wrong-buffer red-proofs to AC2: byte identity with
     `develop_with_into`. After the refactor both entries run the same core, so the
     relation moves with any bug inside it. Only the tier-B MD5 pins caught those mutants,
     and CI has no corpus. The build added an independent composition oracle in tier A.
   - **Classify the fuzz seeds before saying "no new seeds are required."** No develop
     seed carried `FixBadPixelsConstant`, so the P and P + A layouts were never fuzzed
     until the build added five seeds.

2. **Does any template, constraint, or decision need updating?**
   — **DEC-060** held as written. Every measured figure matched the design probe:
   - Q2M `scratch_len` 94,707,712, and 47,443,968 at α = 0;
   - 0 allocations on the scratch path;
   - 1 allocation of 189,415,424 B in `develop_with_into`.

   **Signal bumped:** `shared-path-oracle-tautology` goes from N=2 to N=3 in
   `guidance/signals.yaml`. This case has a new shape: two public entries on one core,
   rather than a round-trip. The bar was already met, so codifying it waits for a stage
   that can land it.

   **The verify-return practice from SPEC-064 worked again.** The approval came back as a
   PR comment plus a docs branch, and #224 squash-merged with no conflicting verify PR.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. The verify found two `[XS]` test strengthenings, now in the STAGE-019
   backlog:
   - **The median fixture.** Its neighbours already arrive sorted, so under "neighbours
     unsorted" only `develop_warp_branch_actually_warps` goes red.
   - **AC3's alloc test** has no develop-level row for a malformed `OpcodeList1`.

   DEC-060 records two further options as additive later: warping straight into `dst`,
   and a scratch variant of the colour path.

4. **Where was the worst defect caught?**
   — build

5. **What can a user do now that they couldn't before?**
   — Before: a Leica Q2 Monochrom develop made 6 heap allocations totalling 283,943,504
   bytes, with no way to reuse buffers across frames. After: `develop_with_scratch_into`
   develops it with 0 allocations into a caller-owned 189.4 MB scratch, sized by
   `scratch_len`. `develop_into` itself now makes 1 allocation of 189,415,424 bytes, a
   saving of 94.5 MB and five allocations per frame.
