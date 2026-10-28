---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-022
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: low                    # critical | high | medium | low
                                   #   Low: no current defect — HIT=0 on every
                                   #   decodable Q2M frame makes fixed_plane
                                   #   byte-identical to src, so the unasserted
                                   #   warp branch cannot yet mis-develop. This
                                   #   is harness completeness, not a live bug.
  complexity: XS                   # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: XS            # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-005
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # build delegated via HANDOFF-054
  created_at: 2026-10-27

references:
  decisions: [DEC-024]             # [DEC-NNN, DEC-MMM]
  constraints: [test-before-implementation, oracle-must-be-shown-red]
  related_specs: [SPEC-017, SPEC-018, SPEC-016]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-005's <capability>". Optional; null is acceptable.
value_link: "STAGE-005: the suite stops asserting `develop_into` uses the fixed plane on only one of its two branches — the warp branch every real Q2M frame takes gets its own assertion"

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
  tokens_estimate: 25000000        # build + verify; XS test-only, no punch-list expected
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-10-27
      notes: "Design in the orchestrator main loop (un-metered; null-with-note per AGENTS.md §4). A read-only Explore subagent gathered the opcode/fixture recipe (140,152 tokens, 24 tool-uses) — the subagent's cost, not transcribed into this spec total."
    - cycle: build
      agent: claude-opus-4-8
      interface: other
      tokens_total: 7430645
      estimated_usd: null
      duration_minutes: 20
      recorded_at: 2026-10-27
      notes: "Test-only, zero src/ change. tokens_total is a transcript sum DEDUPED BY message.id (43 distinct ids from 126 raw usage objects = 2.93x; 96.8% cache-read) — a FLOOR, written before the session closed. Worktree was cut from main (1cf848d), not from the design commit 4bb57a0 as the handoff assumed; cherry-picked 4bb57a0 (clean, byte-identical parents) to bring SPEC-022 + this handoff into the branch."
    - cycle: verify
      agent: claude-opus-4-8
      interface: other
      tokens_total: 3663255
      estimated_usd: null
      duration_minutes: 15
      recorded_at: 2026-10-27
      notes: "✅ APPROVED at f412ac3. Independent red-proof reproduced the build's md5 triple (ab3d77→3f0689→ab3d77) and the 243/0/3→241/2/3 suite delta exactly; AC9 stayed GREEN under the sever. tokens_total is a transcript sum DEDUPED BY message.id (32 ids from 82 raw usage objects; 97.1% cache-read) — a FLOOR, written before session close. No per-token USD meter (Max, extra-usage disabled)."
  totals:
    tokens_total: 11093900
    estimated_usd: 0.00
    session_count: 3
shipped_at: 2026-10-27
---

# SPEC-022: develop warp branch must develop the fixed plane not raw src

## Context

Raised as `SPEC-017/FU-10` at SPEC-017's verify round 2 (2026-09-08) and
dispositioned here at SPEC-017 ship. This is one of the "separate items"
`SPEC-016`'s L→S rescope pushed onto STAGE-005's backlog — a surface
reporting a result it has not fully established, the stage's chartered
class.

`develop_into` (`src/develop.rs`) resolves `effective_src` — the
`FixBadPixelsConstant`-repaired plane when `OpcodeList1` carries the fix,
else the raw `src` (`src/develop.rs:816`) — then consumes it at **two**
call sites, one per branch:

- **no-warp** (`src/develop.rs:826`): `crop_orient_normalize_into(…, effective_src, dst)`
- **warp** (`src/develop.rs:842`): `normalize_active_area_into(…, effective_src, …)`, then `apply_warp_into`, then crop/orient into `dst`

SPEC-017's `AC9` (`develop_output_is_bit_identical_across_two_runs`)
asserts the fixed plane reaches `dst`, but its fixture carries no
`OpcodeList3`, so it exercises **only** the no-warp branch (`:826`).
Verify measured (SPEC-017 round 2) that severing **only** the warp-branch
use site (`:842`, `effective_src → src`; md5 `b3b922d0 → 607c546a`,
compiles) leaves the entire tier-A suite green at 231/0/3: **no tier-A
test sets `opcode_list_3`**, so nothing drives `develop_into` with a fix
opcode and a non-identity warp together, and tier-B is blind because
HIT=0 makes `fixed_plane` byte-identical to `src` on all three Q2M
frames. The sting: every real Q2M frame carries a non-identity
`WarpRectilinear`, so **the branch real files actually take is the
unasserted one**.

Not a live defect today (hence `priority: low`): with HIT=0 the two
branches develop identical bytes whether they read the fixed plane or
raw `src`. It becomes a real correctness hole the moment a frame has a
bad pixel (HIT>0) *and* a warp — then the fixed plane MUST flow through
`:842`, and nothing checks it.

## Goal

Add a synthetic tier-A fixture that drives `develop_into` with BOTH a
`FixBadPixelsConstant` `OpcodeList1` (HIT>0) and a non-identity
`WarpRectilinear` `OpcodeList3`, and asserts the developed output
reflects the FIXED plane through the warp branch (`:842`) — with a
red-proof (§16 rule 2, three clauses): severing `:842`'s
`effective_src → src` turns the new test red while AC9's no-warp
assertion stays green, then reverts byte-identical.

## Inputs

What the implementer will read or consume. (All `file:line` cites are against
this worktree at design time; treat them as "within ±a few lines" — read the
named function, not the exact line.)

- **`src/develop.rs`** — `develop_into`'s `Some(warp)` arm (~`:836–864`): it
  reads `effective_src` into `normalize_active_area_into` (~`:850`), then
  `apply_warp_into`, then `crop_and_orient_from_active_into`. `effective_src`
  is resolved once (~`:824`) to the `FixBadPixelsConstant`-repaired plane when
  `OpcodeList1` carries the fix, else raw `src`. **The red-proof mutates the
  `~:850` use site** (`effective_src → src`). Also read
  `apply_fix_bad_pixels_constant` (~`:418–499`): a sample inside the active
  area whose value equals the `Constant` sentinel is replaced by the
  **lower-median of its valid 3×3 neighbours**; it returns the fix count
  (HIT), which `develop_into` discards.
- **`tests/develop.rs`** — SPEC-017's AC9 `develop_output_is_bit_identical_across_two_runs`
  (~`:562–615`): the **no-warp-branch** mirror of this spec. Its centre-pixel
  assertion `dst1[2*5+2] == 1000` (~`:594`) is exactly what SPEC-022 replays
  through the warp arm. Reuse `minimal_sensor(w,h)` (~`:60–84`).
- **`tests/warp.rs`** — `develop_into_crops_from_the_warped_active_area_not_the_warped_crop`
  (~`:402–540`) and `seam_sensor` (~`:318–353`): the only existing model that
  drives `develop_into` all the way through the `Some(warp)` arm on a synthetic
  plane. Copy its structure.
- **`tests/support/opcode.rs`** — `single_fix_bad_pixels_constant_list`
  (~`:90–95`) and `single_warp_rectilinear_list` (~`:74–84`). **Build the
  opcode blobs with these helpers; do NOT hand-encode opcode bytes.**
- **`src/warp.rs`** — `is_identity` (~`:107–109`) and `apply_warp_into`
  (~`:221–272`): the optical-centre fixed-point property (a dest pixel at the
  optical centre maps to that exact source pixel regardless of `kr`), bilinear
  clamp-to-edge sampling.
- **`decisions/DEC-024`** — why the SSIMULACRA2 oracle cannot see this stage
  end-to-end, so analytic tier-A assertions like this one are load-bearing.
- **`projects/PROJ-001-monochrome-dng-develop/specs/done/SPEC-017-fixbadpixelsconstant-opcode.md`**
  — SB-2 (round 2) is the no-warp half of this exact fix; its handback
  documents the red-proof shape (scratchpad-copy mutation, md5 pair,
  `--no-fail-fast` suite total) to replicate here.

## Outputs

What the implementer will produce. This is a **test-only** spec — one new
tier-A test plus its red-proof. No `src/` change.

- **Files modified: `tests/develop.rs`** — the new tier-A test(s) below, built
  on `minimal_sensor` + the `opcode_support` helpers. **Choice stated, not
  silent:** the test lives in `tests/develop.rs` (beside AC9, same helpers and
  the same `normalize`-is-identity fixture), **not** `tests/warp.rs`. The
  fixture is a hand-built in-code `Sensor`; **no new fixture file** (the opcode
  bytes come from the helpers, not a committed `.hex`).
- **No `src/` change** — the warp branch already develops `effective_src`
  (Non-Goal below). Touching `src/develop.rs` beyond the red-proof's
  throwaway mutation fails this spec.
- **No new dependencies, no new `DEC-*`, no new provenance-ledger row** — the
  `FixBadPixelsConstant` and `WarpRectilinear` algorithms this test composes
  are already recorded (SPEC-017, SPEC-018). No new fuzz target.
- **The red-proof is discharged and recorded in the build handback** (md5
  before/after the `effective_src → src` sever, suite delta under mutation),
  **not** a committed script — it is a one-shot mutation proof like SPEC-017's
  SB-2, not a standing gate like `lint-red-proof.sh`.

## Acceptance Criteria

The fixture (reused by AC1–AC4) — all values measured, not assumed:
`minimal_sensor(5, 5)` with `black_level = Some(0)`, `white_level =
Some(65535)` (⇒ `normalize` is the identity map), `default_crop_size =
Some(5×5)`, no `ActiveArea` (⇒ whole 5×5 plane, optical centre at the integer
pixel `(2, 2)`), no orientation (⇒ 1); `opcode_list_1 =
single_fix_bad_pixels_constant_list(0, 2, 0)` (sentinel `Constant = 0`);
`opcode_list_3 = single_warp_rectilinear_list([1.0, -0.1, 0.0, 0.0], [0.0,
0.0], 0.5, 0.5, 1, 0)` (non-identity ⇒ the `Some(warp)` arm runs); raw plane
`vec![1000u16; 25]` with `src[2*5+2] = 0` (one bad pixel at the optical centre).

- [x] **AC1 — the warp branch develops the FIXED plane.** `develop_into` on the
      fixture (which routes through `src/develop.rs`'s `Some(warp)` arm) yields
      `developed[2*5+2] == 1000`: the centre bad pixel reaches `dst` as its
      neighbours' lower-median (`1000`), **not** the raw marker `0`. This holds
      because `(2,2)` is the warp's optical-centre fixed point, so it samples
      `active_normalized[2][2]` exactly (no interpolation).
      **Test:** `develop_warp_branch_develops_the_fixed_plane` (tier A).
- [x] **AC2 — the fix is non-vacuous (HIT > 0).** A direct
      `apply_fix_bad_pixels_constant` call on a copy of the raw plane returns a
      count of exactly `1`, so AC1 cannot pass merely because no pixel was ever
      bad. Mirrors AC9's `count` check.
      **Test:** same test as AC1 (companion assertion).
- [x] **AC3 — the warp arm did real resampling (branch non-vacuous).** With the
      SAME fixture but a non-uniform ramp background (`value = 1000 + 100*x`,
      bad pixel still `0` at `(2,2)`; its neighbours' lower-median is `1200`),
      `develop_into` with `opcode_list_3 = Some(...)` differs from the same
      sensor with `opcode_list_3 = None` at off-centre pixels (proving the warp
      ran), while the centre still equals the fixed value (`1200`) under both.
      **Test:** `develop_warp_branch_actually_warps` (tier A).
- [x] **AC4 — red-proof (`oracle-must-be-shown-red`, §16 rule 2, three
      clauses).** In a scratchpad copy (working tree untouched): severing the
      warp arm's `effective_src → src` at `~src/develop.rs:850` (① file changed)
      **compiles** (② builds), turns `develop_warp_branch_develops_the_fixed_plane`
      **RED** at `developed[12]: left 0, right 1000` while SPEC-017's AC9
      (no-warp branch) stays **GREEN** (③ output changed on the asserted path
      only), then reverting is **byte-identical** and the test is green again.
      Suite total measured with `--no-fail-fast` (plain `cargo test`
      fail-stops at the develop binary). The build handback records its own
      md5 pair — spellings of the sever vary, so the md5 need not match
      SPEC-017's; the RED is the proof.

## Failing Tests

Specified here at **design**; the implementer writes them in **build** and
discharges the red-proof. Note the inversion for a completeness spec: on the
honest tree both tests are **GREEN** (the wiring is already correct). What
makes them load-bearing is AC4's red-proof — they go **RED** only when the warp
arm is severed to read raw `src`. "Failing" here means "fails under the sever,"
not "fails on the current code."

- **`tests/develop.rs`**
  - `develop_warp_branch_develops_the_fixed_plane` — asserts
    `developed[2*5+2] == 1000` from `develop_into` on the AC1 fixture (the
    `Some(warp)` arm), and that a direct `apply_fix_bad_pixels_constant` on a
    copy returns `count == 1` (AC1 + AC2). RED under the `~:850` sever
    (`left 0, right 1000`); AC9 stays green.
  - `develop_warp_branch_actually_warps` — asserts the ramp-background develop
    differs between `opcode_list_3 = Some(...)` and `= None` at off-centre
    pixels, while the centre equals `1200` under both (AC3). Proves the
    `Some(warp)` arm did real resampling, not an identity pass-through.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- Changing `src/develop.rs` behaviour — the wiring is correct; this spec
  only adds the assertion that proves it for the warp branch.
- The oracle-scope narrowing for warp-bearing pixels — that is `SPEC-021`.
- Sourcing or fabricating a *real* Q2M frame with a bad pixel; the fixture
  is synthetic (a hand-built `Sensor` with both opcode lists set).

## Notes for the Implementer

- **This is the exact mirror of SPEC-017's SB-2 fix** (`tests/develop.rs:594`,
  the `dst1[2*5+2] == 1000` assertion), routed through the `Some(warp)` arm
  instead of the `None` arm. Read that test first.
- **Reuse the helpers** (`single_fix_bad_pixels_constant_list`,
  `single_warp_rectilinear_list`); do not hand-encode opcode bytes. The stream
  is always big-endian regardless of TIFF byte order.
- **Optical-centre fixed-point** is why the assertion is exact: on a 5×5
  whole-plane sensor the dest pixel `(2,2)` maps to source `(2,2)` for any
  `kr`, so `developed[12]` is `1000` (fixed) or `0` (raw) with no interpolation
  blur. Keep the bad pixel at the centre; don't move it.
- The warp coeffs `kr = [1.0, -0.1, 0.0, 0.0]`, `kt = [0.0, 0.0]` are chosen so
  `f(r) = 1 − 0.1·r² ≤ 1` (source stays in-extent, no clamp surprises) and
  `kt = [0,0]` dodges `Error::UnsupportedWarpTangentialTerms`. `kr1 ≠ 0` is
  what makes `is_identity` false.
- **Red-proof hygiene:** mutate a scratchpad COPY of the tree (`rsync` minus
  `target/` and `.git/`), never the working tree; record the honest→severed→
  reverted md5 triple for `src/develop.rs` and read exit codes directly (this
  shell is zsh — `PIPESTATUS` is a bashism; use `$pipestatus` or capture `$?`).
  Use `cargo test --no-fail-fast` for the true suite total.
- Don't touch `src/develop.rs` (Non-Goal) except as the throwaway red-proof
  mutation, which must be fully reverted. `git status` ends clean apart from
  `tests/develop.rs`.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Land the design on `main` (or pass the branch at start) before delegating:
   the proposal **card** dropped `source_branch`, so the build worktree was cut
   from `main` and had to cherry-pick the design commit `4bb57a0` to see the
   spec + handoff. It recovered cleanly, but a card can't inherit an
   unmerged branch — worth knowing for the next delegated spec.

2. **Does any template, constraint, or decision need updating?**
   — No. The test-completeness + one-shot red-proof pattern (mutation recorded
   in the handback, not a standing script) worked cleanly — a faithful mirror of
   SPEC-017's SB-2. It's the second instance of "assert a develop branch uses
   the fixed plane" (no-warp was SPEC-017, warp is this); if a third branch ever
   appears, that's a `lesson` worth recording, but two is not yet a pattern.

3. **Is there a follow-up spec I should write now before I forget?**
   — No. This spec spawned none; verify found no blockers and no follow-ups.
   STAGE-005 still carries SPEC-011, SPEC-016 and SPEC-021. (The lossless-JPEG
   decoder gap seen in the develop demo is out of STAGE-005's scope — a future
   stage, not a follow-up here.)

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — none
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
   — Nothing user-visible today — this is latent hardening. Before: the warp
   branch developed the fixed plane but nothing proved it, and severing it to
   raw `src` left the whole suite green (SPEC-017's verify measured exactly
   that). After: the guarantee is pinned by a tier-A test proven red under that
   sever (suite 243/0/3 → 241/2/3), so once a frame carries a bad pixel (HIT>0)
   *and* a warp, `develop_into` cannot silently develop from unrepaired pixels.
