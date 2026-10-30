---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-025
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-006
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: []                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-024, SPEC-028, SPEC-029]  # gate (024) + the plane must unpack first (028 multi-strip, 029 WhiteLevel)

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: "Advances STAGE-006's bit-exact-plane guarantee — proves the Bayer mosaic reads correctly before any colour is interpreted."

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
      agent: claude-opus-4-8
      interface: other
      tokens_total: 15781793
      estimated_usd: 46.15
      duration_minutes: 18
      recorded_at: 2026-10-30
      notes: "test-only — no src/ change, no manifest change, no new DEC. Both corpus Bayer planes (Canon single-strip, Nikon 12-strip) MD5-match dnglab analyze --raw-checksum; the byte-aligned red-proof (byte-order swap in unpack_byte_aligned) turns the Nikon plane red with a green control — the guard DEC-029 moved onto this oracle. Added to tests/plane_oracle.rs reusing SPEC-013's apparatus; stage_probe_crate generalised from a bool to an Option<fn(&Path)> injector so the mono (sub-byte) and Bayer (byte-aligned) faults share it, mono behaviour unchanged. tokens summed from this session's own Claude Code transcript JSONL (in+out+cache); estimated_usd at Opus list rates, cache-read-dominated."
    - cycle: verify
      agent: claude-opus-4-8
      interface: other
      tokens_total: 8224450
      estimated_usd: 17.73
      duration_minutes: 25
      recorded_at: 2026-10-30
      notes: "✅ APPROVED at commit 53f3803 (PR #48, already MERGED to main before this verify ran — a retrospective cold independent review; no build branch remained to push to, hence a small verify-bookkeeping PR). PR #48 confirmed TEST-ONLY + bookkeeping: tests/plane_oracle.rs + the SPEC-025 spec + HANDOFF-070 — no src/ change, no tests/corpus/manifest.toml change. All 5 ACs reproduced cold with the corpus present (7/7). Independent ground truth: dnglab 0.7.2 analyze --raw-checksum recomputed on both Bayer DNGs == the pinned constants EXACTLY (Canon a18235d6abced4df9948b6fbe91aba7c, Nikon b8e8025d370cc2e97f675e63a0e6bbdb) — the oracle compares irradiance's unpacked-plane MD5 to dnglab's number, not to itself (no tautology). Both Bayer planes decode BIT-EXACT (Canon single-strip 5920x3950, Nikon 12-strip 4352x2868), confirmed live via --nocapture (BAYER ORACLE lines, not SKIP). Byte-aligned red-proof reddens: honest b8e8025d == pinned Nikon, mutant fdcb897fbae2bd9d180e05f9217f5011 != honest, control green — the exact byte-swap guard DEC-029 moved onto this oracle, and (Nikon reference) it also exercised SPEC-028 multi-strip assembly inside the staged mutant. Mono regression GREEN: plane_oracle 15/15 (was 12; +3 Bayer), the sub-byte BitReader red-proof still reddens live (cb653b5b -> 59b032fe). No-corpus path verified: Bayer tests print loud SKIP and pass without asserting (0.00s) while tier-A RFC-1321 still runs. Gates: just test (full suite, 0 failed across all binaries; src/lib.rs 95/0, plane_unpack 18/0, develop 28/0 1 documented ignore), just lint-ci clippy -D warnings pinned +stable 0.1.98 exit 0, cargo fmt --check exit 0, just validate exit 0 (33 valid / 148 parseable), just ready clean, just decisions-audit 0 structural errors (11 pre-existing advisory), plane fuzz 13,407,764 runs/61s 0 crashes 0 artifacts tree clean. No new DEC; verify emitted none. tokens_total summed from this verify session's own transcript JSONL (scratchpad-UUID anchored 1390d94e, deduped by message.id; all 51 usage records claude-opus-4-8; input+output+cache_creation+cache_read), cache-read-dominated (8.03M of 8.22M, 97.6%); estimated_usd at per-category Opus list rates. Floor-by-id snapshot, not a budget delta."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-10-30
      notes: "un-metered main-loop cycle (design/ship run in the orchestrator session, DEC-005) — null-with-note per cost-audit. Ship ritual: Reflection, cost totals, timeline, archive, brag. No CHANGELOG entry — test-only, no library-behaviour or public-API change (the Bayer-decode behaviour is logged under SPEC-028/029). Build merged as PR #48, verify-bookkeeping as PR #49; this ship PR closes the cycle. verify_verdict approved; worst-defect none."
  totals:
    tokens_total: 24006243
    estimated_usd: 63.88
    session_count: 3
shipped_at: 2026-10-30
---

# SPEC-025: Bit-exact mosaic plane oracle: the Bayer file's unpacked plane MD5 equals `dnglab analyze --raw-checksum`, with its red-proof

> **DESIGN — `cycle: design`.** Approach + failing tests below; a build session
> makes the Failing Tests pass. STAGE-006's capstone — the first bit-exact proof
> the Bayer mosaic decodes.

## Context

The mosaic is still a single sensor plane, so PROJ-001's layer-0 contract — the
unpacked plane's MD5 equals `dnglab analyze --raw-checksum` — transfers to a Bayer
file unchanged (the checksum is over the raw mosaic bytes, independent of
demosaic). Once the Bayer plane actually unpacks — SPEC-024 accepts CFA, SPEC-028
adds multi-strip assembly (the Nikon is 12 strips), SPEC-029 relaxes the WhiteLevel
assertion (the Canon clips one) — assert the checksum on the corpus Bayer DNGs via
`plane::unpack_into` (`src/plane.rs`), with the mandatory red-proof. It is the proof
the input is correct before STAGE-007 interprets a single colour. The oracle method
(SPEC-013) needs no change — only the plane, and a second corpus entry.

## Goal

Prove the unpacked mosaic plane of an uncompressed Bayer DNG is **bit-exact**:
its MD5 equals `dnglab analyze --raw-checksum` for both corpus files, and the
oracle demonstrably goes **red** on a deliberately corrupted read. No colour, no
demosaic.

## Inputs

- **Files to read:**
  - `tests/plane_oracle.rs` — the SPEC-013 oracle: `decode_and_hash` + `plane_md5`
    (reuse for the Bayer plane), the manifest-driven `DECODABLE` mono oracle, and the
    **red-proof apparatus** (`TempDir`, `stage_probe_crate`, `build_and_run_probe`,
    `PROBE_MAIN`, `inject_chunk_extraction_fault`) — reuse the stage/build/run machinery.
  - `tests/plane_unpack.rs:895` (`the_nikon_d300s_twelve_strip_plane_...`) — the
    direct-path, skip-when-absent pattern for a Bayer corpus file (the Bayer DNGs are
    **not in `tests/corpus/manifest.toml` yet** — SPEC-027's job — so reference them by
    `CorpusRoot` + relative path, like this test).
  - `src/plane.rs:187` `unpack_byte_aligned` (byte-order branch `:210–212`,
    `from_le_bytes` / `from_be_bytes`) — the Bayer red-proof's injection target.
  - `docs/oracle-contract.md` (`--raw-checksum` = MD5 of the uncropped u16 plane,
    native-LE); `dnglab` (to compute each Bayer file's pinned checksum).
- **Corpus:** `CANON-EOS-5D-MARK-III/5G4A9394.DNG` (single-strip, 16-bit, 5920×3950)
  and `NIKON-D300S/_DSC4615.DNG` (12-strip, 16-bit, 4352×2868).

## Outputs

- **Files modified:** `tests/plane_oracle.rs` (or a sibling `tests/plane_oracle_bayer.rs`).
  **No `src/` change; no manifest change** (Bayer stays direct-path until SPEC-027).
- **Bayer plane oracle (Tier B, corpus-gated):** `decode_and_hash` each Bayer DNG and
  assert the plane MD5 equals its pinned `--raw-checksum`, held as named constants
  computed at build via `dnglab analyze --raw-checksum` (one per file). Skip-when-absent,
  loudly, like `plane_unpack.rs`.
- **Bayer red-proof (byte-aligned path):** a new inject fn swapping `unpack_byte_aligned`'s
  byte order (`from_le_bytes` ↔ `from_be_bytes`, `src/plane.rs:210–212`) in a staged copy
  via the existing `stage_probe_crate` / `build_and_run_probe`; assert the mutant's Bayer
  MD5 ≠ honest, with an unmutated control that reproduces the pinned checksum. **This is
  the guard DEC-029 moved here** — the removed `SampleExceedsWhiteLevel` assertion used to
  catch exactly this byte-swap; now the oracle must. (The mono red-proof covers only the
  sub-byte `BitReader`, which a 16-bit Bayer plane never hits.)
- **Reference file:** prefer the Nikon (12-strip) so the red-proof also exercises the
  multi-strip assembly; a byte-order swap corrupts every sample, so either file proves it.

## Acceptance Criteria

- [ ] Our unpacked plane MD5 equals `dnglab analyze --raw-checksum` for BOTH Bayer
  DNGs — Canon single-strip 5920×3950 and Nikon 12-strip 4352×2868. The first
  bit-exact proof the Bayer mosaic decodes.
- [ ] A byte-order swap injected into `unpack_byte_aligned` turns the Bayer plane's
  MD5 **red** (mutant ≠ honest); the unmutated control reproduces the pinned checksum.
- [ ] The mono oracle (4 `DECODABLE` files) and its sub-byte red-proof stay green
  (regression — SPEC-013's behaviour untouched).
- [ ] No corpus → the Bayer oracle + red-proof skip loudly; the tier-A tests
  (no corpus, no tools) still run.
- [ ] No `src/` change and no manifest change — the oracle is test-only.

## Failing Tests

- **`tests/plane_oracle.rs` (or `tests/plane_oracle_bayer.rs`)**
  - `bayer_plane_md5_matches_dnglab_raw_checksum` — Canon + Nikon; each decoded
    plane's MD5 == its pinned `--raw-checksum` constant. Corpus-gated (skip loud).
  - `a_byte_order_swap_turns_the_bayer_oracle_red` — the byte-aligned red-proof:
    mutant Bayer MD5 ≠ honest. Corpus-gated.
  - `the_bayer_red_proof_control_is_green` — the unmutated staged copy reproduces the
    pinned Bayer checksum (apparatus negative control).
- **Pinned constants (filled at build):** `CANON_RAW_CHECKSUM`, `NIKON_RAW_CHECKSUM`
  = `dnglab analyze --raw-checksum <file>`.
- **Regression (stay green):** `plane_md5_matches_the_pinned_raw_checksum` (mono) and
  `an_injected_unpacker_fault_turns_the_oracle_red` (sub-byte).

## Non-Goals

- Demosaic or any mosaic → RGB interpolation — STAGE-007
- Parsing or checking colour metadata — SPEC-026
- Changing the mono plane oracle (SPEC-013) beyond adding a corpus entry
- Tolerance-based comparison — this layer stays exact

## Notes for the Implementer

- **Direct-path, not the manifest.** The Bayer DNGs are not in `tests/corpus/manifest.toml`
  yet (SPEC-027 registers them), so reference them by `CorpusRoot` + relative path with a
  loud skip, like `tests/plane_unpack.rs`'s Nikon test. Do NOT add them to `DECODABLE` or
  the manifest — that would trip `compressed_files_are_skipped_by_name`'s count. SPEC-027
  later migrates the Bayer corpus into the manifest and may fold these tests into the
  manifest-driven oracle; keep that migration easy.
- **The red-proof must target the BYTE-ALIGNED path.** The mono red-proof injects a
  sub-byte `BitReader` fault a 16-bit Bayer plane never hits. Swap the byte order in
  `unpack_byte_aligned` instead — it corrupts values without changing length (same
  discipline as SPEC-013's chosen fault) and is the exact byte-swap DEC-029 says the
  oracle now guards. Assert `mutant ≠ honest` directly, and keep the control.
- Reuse `decode_and_hash`, `plane_md5`, `stage_probe_crate`, `build_and_run_probe` — only
  the inject fn and the reference file are new.
- **Capstone:** with this green, the Bayer mosaic decode is bit-exact-verified end to end
  (CFA identity + multi-strip + WhiteLevel). STAGE-007 (demosaic) then builds the first
  colour image on a plane proven correct.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Spike the actual unpack *before* designing the capstone oracle. SPEC-025 was
   designed assuming the Bayer plane already unpacked; mid-design it turned out it
   didn't — `unpack_into` was single-strip-only (the Nikon is 12 strips) and the
   WhiteLevel assertion rejected the Canon's one clipped sample — which forced the
   SPEC-028 (multi-strip) + SPEC-029 (WhiteLevel) precursor split. A ten-minute
   "does `unpack_into` even return a plane for these two files?" spike at frame
   time would have surfaced both precursors before the capstone was specced,
   instead of during it.

2. **Does any template, constraint, or decision need updating?**
   — No. Verify emitted no new DEC; DEC-029 — the byte-swap guard this oracle now
   carries — was already written at SPEC-029. One small bookkeeping drift recurred:
   the design-time `handoff.to_agent` tier_map hint (`claude-opus-5`) didn't match
   the agent that actually ran the cycle (`claude-opus-4-8`). The handback
   self-corrects it (HANDOFF-071 stamped the real agent), so it is working as
   designed, not process-debt — not worth a signal.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new one — it is already **SPEC-027**. The two Bayer DNGs are referenced by
   direct path (`CorpusRoot` + relative), not the manifest; SPEC-027 registers them
   in `tests/corpus/manifest.toml` and may fold these direct-path tests into the
   manifest-driven oracle. The Notes-for-Implementer already flags "keep that
   migration easy," so the follow-up is captured.

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — `none` — clean build; verify reproduced all five ACs cold with no punch-list.

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — A downstream consumer can now trust that irradiance's uncompressed-Bayer
   mosaic read is byte-for-byte correct: both corpus DNGs' unpacked-plane MD5 equal
   `dnglab analyze --raw-checksum` (Canon 5920×3950 single-strip `a18235d6…`, Nikon
   4352×2868 12-strip `b8e8025d…`), and a byte-order-swap red-proof fails if that
   ever regresses — the proof the sensor input is correct before STAGE-007
   interprets any colour. (No new public API; the gain is the guarantee, not a call.)
