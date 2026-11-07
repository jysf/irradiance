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
  id: HANDOFF-102
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-07
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-048

project:
  id: PROJ-002
  stage: STAGE-012
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
  tokens_total: 217733             # REAL combined count — what cost-audit reads
  estimated_usd: 1.10              # tokens_total × your rate, or your harness's number
  duration_minutes: 20
  branch: claude/jovial-margulis-f54357
  pr: 143                          # verify PR against main
  completed_at: 2026-11-07         # YYYY-MM-DD
  notes: "APPROVED. tokens_total is this harness's context-window occupancy at handback (get_usage: 217,733 of 1M, 22%); it counts unique/context-equivalent tokens, NOT a cache-inclusive transcript sum, so it is a floor and is not directly comparable to the build's 18.08M figure. No cache-inclusive /cost sum is exposed to this agent."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-102: demosaic interior fast path (verify)

## Delegation Summary

One sentence: `claude-opus-5` (acting as `architect`) hands `SPEC-048` to
`claude-opus-4-8` (acting as `verifier`) for the **verify** cycle: an independent
cold review of the build in PR #142 (DEC-041 — a row-wise interior fast path for
`demosaic_into` that reads interior neighbours by direct index into per-row
slices, keeps the 2-pixel border on the reflecting path, reuses the kernel
arithmetic verbatim, and is therefore bit-identical).

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-048-demosaic-interior-fast-path.md` — Acceptance Criteria + Failing Tests are the contract.
- **Decision:** `./decisions/DEC-041-demosaic-interior-fast-path-and-the-stage-012-perf-discipline.md` — the fast-path structure, why the border stays reflecting, and the bit-identical + measured discipline.
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-101-build-demosaic-interior-fast-path.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

### Decisions that apply

- `DEC-041` — the interior fast path + the STAGE-012 perf discipline (bit-identical
  proof + measured before/after + its own DEC). **This is the spec.**
- `DEC-032` — the demosaic kernel (zero-alloc two-pass green); its maths and the
  two-pass structure must be untouched.
- `DEC-040` — the ActiveArea-origin phase offset (SPEC-047); the fast path must
  preserve it.
- `DEC-016` — caller-owned src/dst buffers; no new allocation.

### Constraints that apply

- `no-panics-on-untrusted-input` — the fast path must be panic-free; per-row slices
  make the interior in-bounds by construction (no `unsafe`, no per-read `get`).
- `no-new-top-level-deps-without-decision`, `one-spec-per-pr`.
- (`oracle-must-be-shown-red` does not cleanly apply — output-invariant change;
  the proof is the `dst_after == dst_before` equivalence test plus the frozen
  golden hashes, not a red→green.)

### Prior related work

- `HANDOFF-101` (build) / `PR #142` — the build under review.
- `SPEC-047` / `DEC-040` (shipped, PR #139) — the ActiveArea-origin phase offset.

## Expected Deliverables — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? The equivalence test NOT
  tautological (its reference genuinely independent)?
- Decision drift — `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- Bit-identity demonstrated (not asserted); scope held (only neighbour fetch
  changed); panic-free; five-lint deny policy intact; measured before/after present.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Any change to the demosaic kernel, its arithmetic, or the output bytes.
- The colour-develop memory / f32 work (P2), threading (P3), the colour-geometry
  crop follow-up — separate STAGE-012 specs.
- `unsafe` or SIMD.

## Return Criteria — how to hand back

Completed: `handback:` front-matter filled (with a real `tokens_total`), this
`## Handback` section filled, `handoff.status`/`handback.status` → completed,
verdict returned.

---

## Handback

### Execution notes

- **Branch / PR:** verify branch `claude/jovial-margulis-f54357`; verify PR
  opened against `main` (PR #143). Build under review: PR #142,
  squash-merged to `main` as `b7606130ca075c071f4e6bcb3fec060662082a0f` (which my
  verify branch is a superset of — HEAD == origin/main == this SHA at review start).
- **Completed at:** 2026-11-07
- **All acceptance criteria met?** yes — all four, verified below.
- **For `verify`:** the verdict —
  **✅ APPROVED (at commit `b7606130ca075c071f4e6bcb3fec060662082a0f`).**

#### Verdict detail — ✅ APPROVED

Reviewed cold and adversarially on a verify branch that is a superset of the
shipped build (`b760613`). This is an output-invariant perf change, so the core
question was "did the output really not move, and is the proof real." It did not,
and the proof is real.

**Acceptance criteria — all met.**

1. **Bit-identical across all four phases + interior/border sizes.** The new
   `interior_fast_path_matches_the_reference_demosaic` compares production
   `demosaic_into` (which now takes the fast path in the interior) against
   `local_hamilton_adams(.., false)` byte-for-byte over RGGB/BGGR/GRBG/GBRG at
   5×5, 6×6, 9×7, 11×9, 12×10 and 8×13; passes. The hand-computed golden tables
   `EXPECTED_HA`/`BGGR`/`GRBG`/`GBRG` (the frozen pre-change bytes) still pass
   unchanged, and the provenance ledger records a 24 MP full-buffer checksum
   (`2359785445034`) identical before and after. ✓
2. **Measured before/after recorded** in `docs/provenance-ledger.md`'s new
   "STAGE-012 performance measurements" section: 24.0 MP RGGB, 533.0 → 316.2 ms
   median of 7 release runs, ≈1.69×, with an honest "conservative floor" caveat
   (the LCG mosaic is a worst case for the directional classifier). ✓
3. **Panic-free, no per-element bounds check in the interior.** Each interior
   neighbour is read from a fixed-size window (`src.get(lo..hi)?.try_into()` to a
   `[u16; N]`), destructured by constant index; the `?` returns `None` on the
   impossible out-of-range case and the caller falls back to the reflecting path,
   so the function cannot panic. `#![forbid(unsafe_code)]` holds; `just lint-ci`
   (the five-lint deny policy as CI sees it) green. ✓
4. **Determinism held; no new dependency; no public API change.** Pure integer
   arithmetic, no RNG; `Cargo.toml`/`Cargo.lock` untouched; `demosaic_into`'s
   signature unchanged. ✓

**The equivalence test is NOT tautological — adversarially confirmed.**
`local_hamilton_adams` is a wholly independent i64 reimplementation (its own
`reflect`/`s()`/`rnd4`/`rnd8`/`clamp` closures and its own two-pass loops), shares
no code with `src/demosaic.rs`, and reflects *every* pixel (no fast path), so it is
a genuine `dst_before`. I neutralized the fast path by injecting a one-index
off-by-one into `pass1_green_interior` (an interior N-neighbour read `window(up1)[2]`
→ `[3]`, reading `(x+1, y-1)` instead of `(x, y-1)`) and re-ran: the equivalence
test went **red** with the precise "diverged from the all-reflecting reference …
no longer bit-identical" message (first at 5×5), as did two other interior-exercising
tests. (The 6×6 RGGB golden table happened to stay green for that particular fault
because it only perturbs vertical-axis pixels — which is exactly why the
varied-mosaic equivalence test across six sizes earns its place.) Probe reverted;
working tree clean.

**Scope held.** The diff changes only *how a neighbour is fetched*: the kernel
arithmetic lives in single shared functions (`green_from_neighbours`,
`diagonal_from_neighbours`, `colour_difference`) that both the reflecting border
path (`green_at_site` / `diagonal_directed` / `interpolate_red_blue`) and the new
interior path (`pass1_green_interior` / `pass2_red_blue_interior`) call, so
bit-identity is structural, not re-verified by eye. The two-pass structure, the
public signature and the dependency set are unchanged. The interior/border split
is correct: interior = every pixel with `2 ≤ x < w-2` and `2 ≤ y < h-2`
(`col_interior_end = w.saturating_sub(2)`, `row_interior_end = h.saturating_sub(2)`),
which is exactly the region where the ±2 max neighbour offset cannot leave the
plane; axes shorter than 5 leave the whole pass on the reflecting border path
(still bit-identical). The DEC-040 ActiveArea phase offset is preserved: the fast
path's `top_odd`/`left_odd` XOR parity is algebraically equal to the slow path's
`wrapping_sub` parity (bit 0 of a subtraction is unaffected by borrow), so a
fast-path interior pixel and a slow-path border pixel are assigned the same colour.

**Gates — all green.** `just test` (full suite, Tier-B corpus present via
`$IRRADIANCE_CORPUS_DIR` — "corpus: 9/9 present — no tier-B test will skip"):
425 passed, 0 failed, 1 ignored (the pre-existing SPEC-017 `q2m_frames_have_at_
least_one_replaced_pixel`, a documented corpus finding unrelated to this spec);
real exit 0 confirmed without a `tail` pipe. `just lint-ci` (clippy pinned
`+stable`, `-D warnings`, all targets/features): exit 0, zero warnings. `just
fmt-check`: clean, exit 0. `just validate` (52 artifacts). `just cost-audit` (clean).
`just decisions-audit` (0 structural errors; 37 pre-existing advisory scope
warnings only — DEC-040/DEC-041 both govern `src/demosaic.rs` but do not
contradict: DEC-040 is the phase offset DEC-041 explicitly preserves, DEC-041 is
the fetch restructuring; no supersession needed).

**Build reflection honesty.** The build notes name a real mechanical move —
extracting the kernel into single shared functions so bit-identity is structural,
and passing nine neighbours as a `[i32; 9]` array to stay under clippy's
`too_many_arguments` — rather than claiming nothing was unclear. Consistent with
the diff; no undisclosed deviations found.

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from.

- **Tokens (total):** 217,733 — this harness's context-window occupancy at
  handback (`get_usage`: 217,733 of a 1M window, 22%). It counts unique /
  context-equivalent tokens, **not** the cache-inclusive transcript sum the build
  reported (18.08M), so it is a floor and is not directly comparable. A
  cache-inclusive `/cost` cumulative is not exposed to this agent, and the
  per-session budget meter reset mid-session so this occupancy read is the honest
  figure.
- **Estimated USD:** ~1.10 (order-of-magnitude, Opus list rate on the measured
  occupancy; a floor for the same reason).
- **Duration (minutes):** ~20
- **Source of the number:** harness report (`get_usage` context-window occupancy).
  No `/cost` sum available.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-041 (design-cycle) governs; the build and
  verify raised no new decision.
- **Deviations from spec:** none affecting behaviour. One disclosed mechanical
  implementation choice (extract the kernel into shared functions; pass neighbours
  as an array) — no DEC needed, no contract change.
- **Follow-up work identified:** none new. The P2 (colour-develop memory / f32
  entry point), P3 (threading) and the colour-geometry crop follow-up remain the
  already-planned separate STAGE-012 specs; SIMD/`unsafe` stays out of scope.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The one thing worth checking hard was whether the
   equivalence test was a genuine `dst_before` or a dressed-up copy of the fast
   path; reading `local_hamilton_adams` (an independent i64 reimplementation that
   reflects every pixel) settled it, and the injected-off-by-one probe confirmed it
   bites.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-041, DEC-032, DEC-040, DEC-016 and the no-panic / no-new-deps
   constraints were all named and all applied.

3. **If you did this task again, what would you do differently?**
   — Nothing material. For an output-invariant change, the decisive move is to
   neutralize the optimisation and watch the equivalence gate go red; I'd reach for
   that first, as the one check that separates a real bit-identity proof from a
   tautology.
