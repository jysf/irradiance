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
  id: HANDOFF-087
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-037

project:
  id: PROJ-003
  stage: STAGE-009
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
  tokens_total: 271544             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 45
  branch: claude/spec-037-build-233973
  pr: 102
  completed_at: 2026-11-05         # YYYY-MM-DD
  notes: "No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-085/086). 271544 is this build session's context-window token occupancy at handback, read via the app usage tool (ccd_session_mgmt get_usage: 271,544 of 1,000,000, 27%). It is a snapshot / lower bound: true cumulative input+output across the build's turns is higher (per-turn context resends not captured). Not invented, but not a /cost number — treat as a FLOOR in rollups, exactly as HANDOFF-085/086 reported. All four gates green first pass; no new DEC."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-087: Lossless reconstruction (predictors, point transform, restart)

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-037` to `claude-opus-5` (acting
as implementer) for the **`build`** cycle: implement the lossless-JPEG
reconstruction on top of SPEC-036's shipped entropy front-end — predictors 1–7,
the T.81 §H.2.1 boundary rules, the point transform, and restart resets, turning
the decoded differences into a caller-owned `u16` plane — plus the in-harness LJ92
encoder that drives the bit-exact round-trip oracle and its red-proof.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-037-lossless-reconstruction-predictors-point-transform-restart.md` — read all of it; `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).

### Decisions that apply

- `DEC-035` — SOF-3 is patent-clean; implement the reconstruction clean-room from
  T.81 Annex H and add the provenance-ledger row. Cross-ref crates read-for-structure only.
- `DEC-016` — caller-owned output buffer; no allocation on the reconstruction path.
- `DEC-018` — u16 sample path; prediction is modular `2^16`, done with explicit
  `wrapping_*` (so `arithmetic_side_effects` is satisfied by intent).

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `no-panics-on-untrusted-input` — every neighbour / restart / length path returns
  a typed `Error`, never a panic (SPEC-038 fuzzes it).
- `oracle-must-be-shown-red` — the encode→decode round-trip must include a
  perturbed-decoder red-proof that provably goes RED, not just green round-trips.
- `provenance-recorded-per-algorithm` — add a `docs/provenance-ledger.md` row for
  the lossless reconstruction (T.81 Annex H.1).
- `test-before-implementation` — the spec's `## Failing Tests` come first.

### Prior related work

- `SPEC-036` / `HANDOFF-085` / **PR #98** — the shipped entropy front-end this
  builds on: `src/ljpeg.rs` `Frame`, `HuffTable`, `parse_headers -> (Frame,
  HuffTable, usize)`, `ScanReader::decode_difference(&HuffTable) -> i32`,
  `ScanReader::restart()`. Use these EXACT names; do not re-decode.
- `src/plane.rs` — the `u16` plane is row-major, one sample/pixel (match it).
  `src/lib.rs` — the `Ljpeg*` and `*WrongLength` error conventions; add
  `LjpegPlaneWrongLength`.
- SPEC-030 / SPEC-031 (demosaic) — the analytic-oracle + perturbed-kernel red-proof
  pattern to mirror for the round-trip oracle.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- `src/ljpeg.rs`: `reconstruct(frame, table, reader, out)` + `decode_into(data,
  out) -> Result<Frame>` implementing SPEC-037's `## Outputs` (predictors 1–7,
  §H.2.1 boundaries, point transform, restart reset), plus the `encode_lj92` test
  harness, the bit-exact round-trip oracle, and the perturbed-decoder red-proof.
- `src/lib.rs`: the new `LjpegPlaneWrongLength` variant. `docs/provenance-ledger.md`:
  the reconstruction row (T.81 Annex H.1).
- All of SPEC-037's `## Failing Tests` passing; `just test`, `just lint-ci -D
  warnings`, `just fmt-check`, `just validate` green; `#![forbid(unsafe_code)]` +
  the five-lint deny policy intact.
- A PR against `main` from branch `claude/spec-037-build-<suffix>`, its
  description referencing this handoff (`HANDOFF-087`), `SPEC-037`, `STAGE-009`,
  `PROJ-003`, and `DEC-035` / `DEC-016` / `DEC-018` (plus any new `DEC-*`).

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- The marker/frame parse, bit reader, and DC Huffman decode — that is SPEC-036
  (shipped); **call** it, don't re-implement it.
- The fuzz / truncation target — SPEC-038 (the `encode_lj92` harness seeds it, but
  the libFuzzer target itself is SPEC-038).
- The DNG `Compression 7` container binding + real-file `dnglab --raw-checksum`
  verification — STAGE-010; do not touch `src/ifd.rs` or `plane.rs`'s unpacker.
- Multi-component / Bayer-in-LJ92, lossy DCT, arithmetic coding, Nikon
  NEF-lossless, Pentax PEF.

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

- **Branch / PR:** `claude/spec-037-build-233973` → PR #102
- **Completed at:** 2026-11-05
- **All acceptance criteria met?** yes. `reconstruct` + `decode_into` implement
  predictors 1–7 (T.81 Table H.1), the §H.2.1 boundary rules (first sample /
  first line Ra / first column Rb / post-restart = `2^(P−Pt−1)`), the point
  transform (prediction in the pre-shift domain, stored `<< Pt`), and the
  per-restart predictor reset; caller-owned, zero-allocation (`DEC-016`),
  modular `2^16` via explicit `wrapping_*` + a shift-free `floor_div2`
  (`DEC-018`); `Error::LjpegPlaneWrongLength { expected, got }` added. The seven
  `## Failing Tests` all pass, including the full round-trip matrix (predictors
  `1..=7` × precision `{8,12,16}` × `Pt {0,2}` × restart on/off) and the
  perturbed-predictor red-proof. `docs/provenance-ledger.md` carries the
  reconstruction row. `just test` · `just lint-ci` (clippy 0.1.98, the floating
  CI version) · `just fmt-check` · `just validate` all green first pass.
- **For `verify`:** n/a (this is the build cycle).

### Cost self-report

- **Tokens (total):** 271544 — a **FLOOR**, not a `/cost` cumulative.
- **Estimated USD:** null (no rate applied to a floor).
- **Duration (minutes):** ~45.
- **Source of the number:** none available (no interactive `/cost` in the
  Claude Code desktop app's Code tab). 271,544 is this session's context-window
  occupancy at handback (`ccd_session_mgmt get_usage`: 271,544 / 1,000,000,
  27%). It is a lower bound — per-turn context resends are not captured — and is
  reported as a floor exactly as HANDOFF-085/086 did; the orchestrator
  supersedes it from the session transcript at ship.

### Drift and new artifacts

- **New decisions emitted:**
  - None. Reconstruction is straight T.81 Annex H.1 governed by the existing
    `DEC-035` (patent/clean-room/provenance), `DEC-016` (caller-owned buffer)
    and `DEC-018` (u16 modular arithmetic) — no judgment call needed a new DEC.
- **Deviations from spec:**
  - Field name: the new variant is `LjpegPlaneWrongLength { expected, got }`
    (as the spec's `## Outputs` and the build prompt specify). The crate's other
    `*WrongLength` variants use `actual` rather than `got`; I followed the spec's
    explicit `got`. Flagging the convention is not uniform — the architect may
    want to settle it (a trivial rename either way) but it is not blocking.
  - Nothing else: the spec's `## Outputs` was a complete arithmetic contract
    (predictor table, boundary-rule precedence, modular-`2^16` + point-transform
    domain, restart semantics) and was implemented verbatim.
- **Follow-up work identified:**
  - None beyond the already-planned STAGE-009 backlog: SPEC-038 (fuzz /
    truncation-hardening — the `encode_lj92` harness is kept clean and
    parameterised to seed it) and STAGE-010 (the DNG `Compression 7` container
    binding + the real-file `dnglab --raw-checksum` cross-check, which is where
    `Pt > 0` and multi-strip LJ92 meet real bytes).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the content; the one friction was mechanical. The handoff and
   build prompt (and the spec's reconstruction contract) had just landed on
   `main` via PR #101, so on my freshly-cut worktree the two files 404'd on first
   read and I had to fast-forward to `origin/main` before starting. The handoff
   does say "sync if you don't see the design commit," but the 404 arrived first.
   A build branch cut *after* its design PR merges would avoid the one-commit lag.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The genuinely-underspecified corner in lossless JPEG — the prediction
   after a *mid-interval* restart, and whether the post-restart "use the initial
   prediction" rule outranks the first-line/first-column positional rules — the
   spec's `## Outputs` pinned explicitly (it lists the initial-prediction case
   first, "AND the first sample after each restart"), so I gave it precedence and
   the round-trip oracle (encoder mirrors the exact same rule) proves
   self-consistency. Nothing was left to guess.

3. **If you did this task again, what would you do differently?**
   — Fast-forward and confirm the `design(spec-037)` commit is present *before*
   reading the handoff, rather than after a 404. And I'd reach for the shared
   production `predict` in the oracle's encoder from the start: I briefly weighed
   duplicating the predictor formulas in the encoder for "independence," but the
   real independent checks are the hand-computed direct tests (which never call
   `predict`) and the perturbed-decoder red-proof — so sharing `predict` makes the
   encoder a trivially-correct inverse with no loss of rigour.
