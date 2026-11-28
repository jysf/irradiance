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
  id: HANDOFF-210
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-28
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-099

project:
  id: PROJ-006
  stage: STAGE-034
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-210: The per-plane WarpRectilinear at sensor geometry: red and blue resampled to green's geometry, and the consumer's warp named

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-099` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle. On a first `WarpRectilinear` with
`N = 3` whose red or blue set differs from green's, the six sensor-geometry entries
(`srgb_linear_plane_into`, `srgb_linear_plane_f32_into`, `mono_mix_plane_into` and their
`_rows_` forms) resample red and blue to green's geometry on the demosaiced camera triple,
before the matrix, through a delay ring between the fused traversal and `store`. A new
public accessor `opcode::sensor_geometry_warp` names the one warp the consumer applies
(green's set) and every set with its `CFAPlaneColor` colour. One new `Error` variant
(`UnsupportedWarpFold`), one new local-only SDK cross-check (`scripts/sdk-stage3.sh`), no
version bump. The developed colour output and the developed mix are untouched (SPEC-100).

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md` (onyx's answers of 2026-11-28)
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md` (Design Notes: onyx's rule 1 as amended)
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-099-the-per-plane-warprectilinear-on-the-developed-colour-output.md` (Design §1 to §7, Implementation Context, AC1 to AC10, Failing Tests, the rule-clause map). The spec is the contract.
- **Decision:** `./decisions/DEC-087-the-sensor-geometry-planes-resample-red-and-blue-to-green-geometry-and-leave-green-warp-to-the-consumer.md` (proposed). Item 3 is the exact arithmetic; implement it term by term.
- **Toolchain brief:** `./guidance/toolchain-brief.md`, and `AGENTS.md` §5's measured toolchain (the `+toolchain` trap for `cargo fuzz`).

### Decisions that apply

- `DEC-087`: this spec's decision: the accessor, when the CA step runs, the rule, where, the refusals and their order, what does not change.
- `DEC-086` items 5 and 8 (dated amendment lines): the sensor-geometry path and the accessor it deferred.
- `DEC-024`: the bilinear kernel and its clamp-to-edge rule, reused for the CA sample (clamped to the `ActiveArea`).
- `DEC-059` §3: `scale_bounds`, reused for the fold check on `[kr0, 3kr1, 5kr2, 7kr3]`.
- `DEC-044`: band invariance, which the rows forms' extra `H` rows rely on.
- `DEC-079`, `DEC-083`: the planes' and the mixer's arithmetic, unchanged; the CA step only replaces the camera triple they read.
- `DEC-069` §6: the SDK is run locally only; never vendored, built in CI or linked; none of its output is committed.
- `DEC-003`: tier-B files are never committed; tests skip by name when a file is absent.

### Constraints that apply

- `no-panics-on-untrusted-input`: checked arithmetic, `.get` reads; the table and ring are sized from validated geometry; the `warp_opcode` fuzz target gains the per-plane parser (AC10).
- `oracle-must-be-shown-red`: AC8 asserts its red-proof (green-only at least 0.30 px off); AC3's reference must be the test's own code from DEC-087's text.
- `test-before-implementation`: write the Failing Tests first and see each fail (or mismatch) against `main`.
- `provenance-recorded-per-algorithm`: one ledger row for the CA rule (class 1, derived from DNG 1.7.1.0's own transform).

### Prior related work

- SPEC-018 and SPEC-064 (`src/warp.rs`), SPEC-091, SPEC-092 (the entries), SPEC-093 (the developed mix, untouched here), SPEC-098 (`DevelopPath::SensorGeometry`, the pins this spec moves).
- HANDOFF-208/209 and PR 367/368: SPEC-098's build and verify. Its verify found five oracle gaps by mutation; the rule-clause map here is written to close that class up front.

### Facts the design probe measured (2026-11-28, release build, this machine)

- **The full tables are in the spec's Implementation Context:** the nine coefficient sets, the fold minima, the CA displacements, the table error, `H = 3`, the oracle per file and plane, and the three pre-registered `u16` digests.
- **The pre-registered `srgb_linear_plane_into` digests** (Q2 `0xd352bb75bf29d270`, Q3 `0xc6f67381a8b94593`, SL3 `0xbb9908b919ce2b08`) come from a probe that reproduced SPEC-098's pins with the step off. If yours differ, check DEC-087 item 3 term by term before anything else, and report; never edit a pin to match. The `f32` and mono pins are re-pinned (no independent prediction); say in the handback that the `u16` pins matched first.
- **Exactness route:** with the step off, the fused plane equals `to_srgb_linear_into(demosaic_into(n))` at every sample on all three files, `n` the test's own normalisation given in the spec. Use it for AC3 and AC7.
- **The oracle (AC8)** needs `dng_validate -3` on each file and on a scratch copy whose red and blue sets are overwritten by green's. The probe's patcher is `spec-099-design-patch-warp.py.txt`; turn it into `scripts/sdk-stage3.sh` (shell plus a short Python or Perl patch, writing only into its output directory). The SDK binary is at `~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`. Each SDK run takes a few seconds; the stage-3 TIFFs are 280–360 MB each.
- **Probe sources** (not in the repo), in `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b`: `spec-099-design-probe-coeffs.rs.txt` and `-coeffs-module.rs.txt` (the sets and the fold check), `spec-099-design-probe-oracle-camera.rs.txt` (the camera-RGB oracle with the Lucas–Kanade shift), `spec-099-design-probe-srgb-digests.rs.txt` (the digests, the exactness check and the sRGB-space attempt), `spec-099-design-patch-warp.py.txt`, and `spec-099-design-probe-output.txt`.
- **The corpus:** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. **Never copy any `-PB` file, SDK output or derivative pixel anywhere in the repo.**
- **Cost baseline:** the probe's naive CA (two planes, 60 MP, single thread) took 0.49–0.64 s; `srgb_linear_plane_into` is about 1.1 s today on the Q3 and SL3. Measure before and after on the Q3 and record it (not a gate).

## Expected Deliverables

- Code and doc changes implementing SPEC-099's Acceptance Criteria (Design §1 to §7), the six entries' docs saying the plane is CA-corrected and the consumer applies `sensor_geometry_warp(..).consumer()`.
- All Failing Tests written first, seen failing against `main`, then passing; SPEC-098's sensor-geometry pins in `tests/leica_colour.rs` moved as AC7 says.
- The rule-clause map's mutants, each with the test that turned red.
- The `warp_opcode` fuzz smoke with the new seed.
- `scripts/sdk-stage3.sh`, the `$IRRADIANCE_SDK_STAGE3_DIR` SKIP line in `corpus-status`, and AC8 run locally with its numbers in the handback.
- CHANGELOG `[Unreleased]` lines: the accessor, the CA step and the new error; `N = 2` and `N ≥ 4` now refused on the sensor-geometry entries.
- A PR against `main` from branch `feat/spec-099-sensor-geometry-ca`, referencing HANDOFF-210, SPEC-099, STAGE-034, PROJ-006, DEC-087, DEC-086, DEC-059 and DEC-024, with the provenance row and the oracle stated.

## Out of Scope

- `develop_color_*`, the developed mix, `warp_alpha` and `Framing` on them, their thread forms and optional-warp rules: SPEC-100.
- Any change to the mono develop, `parse_warp_rectilinear`, `apply_warp_into`, `demosaic_into`, `to_srgb_linear_into`, `DEVELOP_VERSION` or `MONO_MIX_VERSION`.
- Tangential warp terms when the CA step runs, and `N` other than 1 or 3.
- Committing, copying or redistributing any `-PB` corpus file or any SDK output.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
