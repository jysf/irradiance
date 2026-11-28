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
  id: HANDOFF-212
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-28
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-100

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

# HANDOFF-212: The per-plane warp on the developed colour output and the developed mix

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-100` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle. The held Leica Q2, Q3 and SL3 (a
mandatory three-set `WarpRectilinear`) stop refusing on the two developed outputs:
- the colour develop (`develop_color_into`, `develop_color_f32_into`, both threaded forms)
  applies a **mandatory** first warp as SPEC-099's CA step, then green's set `W`
  (`sensor_geometry_warp(..).consumer()`) on each camera plane **before** the matrix, then
  the crop and orientation; optional warps stay skipped;
- the developed mix applies the first warp **whatever its flags** as the mono develop of
  SPEC-099's CA-corrected mixed plane with `W` at `warp_alpha` and `Framing`, the CA ring
  inside each strip and in the caller's scratch.
No new public item, no version bump, no output change for any file that develops today.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md` (onyx's answers)
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-100-the-per-plane-warp-on-the-developed-colour-output-and-the-de.md` (Design §1 to §7, Implementation Context, AC1 to AC10, Failing Tests, the rule-clause map). The spec is the contract.
- **Decision:** `./decisions/DEC-088-the-developed-outputs-apply-the-ca-step-then-green-warp-before-the-matrix-for-colour-over-the-mixed-plane-for-the-mix.md` (proposed). Items 2 to 5 are the contract for the code.
- **Toolchain brief:** `./guidance/toolchain-brief.md` and `AGENTS.md` §5 and §6.

### Decisions that apply

- `DEC-088`: this spec's decision: one rule, the colour develop (DNG order, mandatory only, no options, threaded fallback), the developed mix (`W`, whatever the flags, CA ring per strip, no allocation), scratch, refusals and order, versions.
- `DEC-087`: the CA step (items 2 to 5), reused unchanged; item 6 has a dated amendment line.
- `DEC-084`: the developed mix; items 1, 3, 6 and 7 have dated amendment lines. Rule 1 is the oracle.
- `DEC-086` item 6: replaced (dated line).
- `DEC-059`: `warp_alpha`'s lerp and `FullActiveArea`'s bounds rule, now read on `W`.
- `DEC-060` §2: `develop_with_scratch_into` allocates nothing; `develop_with_into` allocates once.
- `DEC-044` (bands, threads, the 5-to-8 fallback), `DEC-043` (the colour crop scatter), `DEC-024` (bilinear), `DEC-058` §5 (versions).
- `DEC-069` §6 and `DEC-003`: the SDK and tier-B files stay local; tests skip by name.

### Constraints that apply

- `no-panics-on-untrusted-input`: checked arithmetic and `.get` reads in the gather, the strips and the scratch split; sizes from validated geometry.
- `oracle-must-be-shown-red`: AC9's red-proof (colour without the CA step at least 0.30 px off); AC3's composition is built from public primitives and the tests' own CA reference, never crate code.
- `test-before-implementation`: write the Failing Tests first and see each fail (or refuse) against `main`.
- `provenance-recorded-per-algorithm`: one ledger row (class 1: DNG's stage order and `WarpRectilinear`).

### Prior related work

- SPEC-099 (HANDOFF-210/211, PRs 373/374): the CA step, `CaDelay`, `ChromaticShift`, `tests/support/ca_reference.rs`, `tests/colour_sdk_stage3.rs`, `scripts/sdk-stage3.sh`. Its reflection: write the order mutants first; check that float mutants differ on Linux too.
- SPEC-093 (the developed mix, `tests/developed_mix.rs`'s rule-1 oracle and ranked faults), SPEC-050/051 (the colour scatter and threads), SPEC-064 (`warp_alpha`).

### Facts the design probe measured (2026-11-28, release build, this machine)

- **All numbers are in the spec's Implementation Context:** geometry (the Q3 and SL3 are Orientation 8, output 6336×9520), the exactness route, six colour and fifteen mix pre-registered digests, the rejected post-matrix order, the AC9 table.
- **Pre-registered digests** come from a probe built on public entries only: `srgb_linear_plane_into` and `mono_mix_plane_into` (SPEC-099's pins, reproduced), `ca_reference.rs`, `apply_warp_into`, `to_srgb_linear_into`, and the TIFF orientation table. If a digest misses, compare the gather with `apply_warp_into` term by term (Notes for the Implementer) and report; never edit a pin.
- **Exactness:** `srgb_linear_plane_into == to_srgb_linear_into(stripped, reference(demosaic_into(stripped, n)))` at every sample on all three files (the SL3 with DEC-086's fix applied by hand: 10,615 sites). So AC3 and AC7 can check the colour develop bit for bit.
- **The SDK references** already exist from SPEC-099's build, outside the repo: `/private/tmp/claude-501/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance--claude-worktrees-spec-099-build-ddb2b5/111b6c75-4198-4766-9cc9-425a4fe387af/scratchpad/sdk3` (`q2-full.tif`, `q2-green.tif`, `q2-green.dng`, and the same for `q3` and `sl3`). If that folder is gone, regenerate with `scripts/sdk-stage3.sh ~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate ~/PSeven/experiments/crustimg_redo_plus/images <dir outside the repo>`.
- **Probe sources** (not in the repo), in `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b`: `spec-100-design-probe.rs.txt` (the composition, the mix digests, the SDK oracle, the matrix measured from `to_srgb_linear_into`), `spec-100-design-probe-postmatrix.rs.txt` (the rejected order) and `spec-100-design-probe-output.txt`.
- **Corpus:** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. **Never copy any `-PB` file, SDK output or derived pixel into the repo, a fixture, a seed or CI.**
- **Cost baseline:** the probe's composition took 2.4 s (CA plane) plus 2.7 to 2.9 s (three warps, matrix, crop) on the Q3, single-threaded. Time `develop_color_into` and the default developed mix on the Q3 and record them (not a gate).

### Watch-outs found at design

- `ChromaticShift` holds its tables in `Vec`s and `CaDelay` owns a `Vec` ring. Under the mix's scratch form that breaks the zero-allocation tests. Move the tables to fixed arrays (`4097` `f64` each, the nodes likewise) and let `CaDelay` borrow its ring; the sensor-geometry pins must not move.
- `tests/developed_mix.rs::ac6_the_mix_sizes_an_optional_multi_plane_warp_as_it_develops` encodes DEC-086 item 6 (optional `N = 3` skipped, mandatory refused). It must be rewritten, not deleted (AC6).
- `tests/leica_colour.rs` asserts the developed outputs refuse `{ list: 3, id: 1 }`; those asserts go, replaced by AC7.
- With an optional warp, or no warp, the colour develop must stay on today's code path, untouched; the same for the mix without a CA step.

## Expected Deliverables

- Code and doc changes implementing SPEC-100's Acceptance Criteria (Design §1 to §7); the colour entries', threaded forms', `DevelopOptions::mono_mix`, `scratch_len` and both `DevelopPath` variants' docs.
- All Failing Tests written first, seen failing against `main`, then passing.
- The rule-clause map's mutants, each with the test that turned red (order mutants first).
- AC9 run locally with `IRRADIANCE_SDK_STAGE3_DIR` set; its per-file, per-plane numbers in the handback.
- CHANGELOG `[Unreleased]`: the two developed outputs develop the Q2, Q3 and SL3; the colour develop applies a mandatory warp (DNG order) and still skips optional ones; the developed mix applies the first warp whatever its flags with `N` 1 or 3 (an optional `N ≠ 1` warp, skipped before, is now applied or refused as malformed).
- `docs/provenance-ledger.md`: one row.
- A PR against `main` from branch `feat/spec-100-developed-warp`, referencing HANDOFF-212, SPEC-100, STAGE-034, PROJ-006, DEC-088, DEC-087, DEC-084, DEC-086 and DEC-059.

## Out of Scope

- `warp_alpha`, `Framing` or `apply_orientation` on the colour develop; any new public item.
- Applying an optional warp on the colour develop.
- A ring in place of the colour develop's `3A` camera buffer; threading the warped colour develop (follow-ups).
- The CA step's arithmetic, the accessor, the rim rule (SPEC-099).
- The mono develop of a colour file (`mono_mix: None`), `parse_warp_rectilinear`, `apply_warp_into`'s behaviour.
- `DEVELOP_VERSION`, `MONO_MIX_VERSION`.
- Committing, copying or redistributing any `-PB` corpus file or SDK output.

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
