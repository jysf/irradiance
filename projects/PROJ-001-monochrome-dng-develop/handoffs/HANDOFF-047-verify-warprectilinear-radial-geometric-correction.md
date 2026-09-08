---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.

handoff:
  id: HANDOFF-047
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session
                                    # (correction from tier_map.design's
                                    # claude-opus-5 prediction, per DEC-004
                                    # rule 3).
  to_agent: claude-opus-5           # ✓ CONFIRMED, not a prediction any more:
                                    # this session's own `message.model` is
                                    # `claude-opus-5` on all 129 metered
                                    # messages (DEC-004 rule 3). The
                                    # tier_map.verify hint was right this
                                    # cycle — unlike the build hint's 0-for-14.
  from_role: architect
  to_role: verifier                # implementer | verifier
  created_at: 2026-09-07
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-018

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. Required.
# `notes:` MUST be ONE PHYSICAL LINE — handback-sync truncates multi-line
# YAML scalars.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 22631969           # deduped by message.id, own transcript identified by scratchpad UUID ebb940e0-a34a-4440-ba39-2f3925a4b23b; message.model reports claude-opus-5, so tier_map.verify's prediction was CORRECT this cycle
  estimated_usd: 51.98             # per-component (in 258, out 69883, cache-write 245115, cache-read 22316713) at published Opus-tier rates ($15/$75/$18.75/$1.50 per Mtok) = $43.32, + 20% handback uplift
  duration_minutes: 22
  branch: feat/spec-018-warprectilinear-radial-geometric-correction
  pr: 16                           # verify did NOT open it; PR 16 was already open on arrival (orchestrator, 2026-09-07T07:54:45Z) — recorded as observed
  completed_at: 2026-09-07
  notes: PUNCH LIST on 40f5d45 (CI green there and on tip 823a7fc, 10 jobs each). Finding 1 is spec-correct but UNTESTED - reverting develop.rs to crop-then-warp compiles, changes real output, and leaves the whole suite green at 205/0/2, so the warp branch of develop_into has zero live coverage (SB-1). Finding 2 CONFIRMED behaviourally without reading dnglab source (corner-tile NCC vs dnglab - 0.989 to 0.995 for our UNWARPED render, minus 0.31 to plus 0.23 for our warped one, on two frames) and judged FU-10, ship with the ignore-marked tests. AC8 minus 60.169 and AC9 minus 60.193 / minus 55.075 reproduced exactly. SB-2 is two false claims in shipped rustdoc. Nine further follow-ups FU-1..FU-9.
  synced_at: 2026-09-07
  verdict: punch-list              # approved | punch-list | rejected — mirrors
                                   #   your review's banner; copies into
                                   #   spec.task.verify_verdict at ship.
---

# HANDOFF-047: Verify SPEC-018 — WarpRectilinear radial geometric correction

## Delegation Summary

Verify the SPEC-018 build shipped by HANDOFF-046 (build handback:
`status: completed`, `tokens_total: 116,480,125`, 82 min, three
findings raised — two of them substantial). Branch tip is at
`40f5d45`, three CI runs all green including the new `fuzz-warp`
smoke.

Your job: reconcile the build's claims against actual git+disk state
(DEC-004 rule 1), run the 14 acceptance criteria's named tests
yourself, run the 12 verify checks (`AGENTS.md §15` — 8 generic + 4
repo-specific bars), and return `✅ APPROVED` / `⚠ PUNCH LIST` /
`❌ REJECTED` with per-finding `SB-N` / `FU-N` labels.

**This spec's two findings are unusually load-bearing.** They question
SPEC-020's oracle at a structural level. Read HANDOFF-046's `##
Completion` and `decisions/DEC-024-*.md` in full before you form your
own opinion.

## ⚠ The two findings you must judge — the crux of this verify

### Finding 1: Pipeline-order correction (fixed in build)

**Build's claim:** SPEC-018's design assumed the warp runs AFTER
DefaultCrop; DNG 1.7 § 6.4.1 makes clear it runs over the full
ActiveArea, BEFORE DefaultCrop extracts the final display rectangle.
Corrected in `src/develop.rs`.

**Your check:**
1. Read DNG 1.7.0.0 § 6.4.1 yourself — the clause the build cites,
   with its exact text. `unrun-docs-carry-errors` (N=6) is why this
   matters. A citation you did not run against the actual spec is
   worth nothing.
2. Read `src/develop.rs` around the warp application point. Confirm
   the pipeline actually flows: unpack → normalise → **warp** →
   DefaultCrop → orientation. If it flows differently, the fix did
   not land.
3. Bar 9's red-proof intersects here: does mutating the pipeline
   order to `crop-then-warp` produce a different scored output?
   If yes, the fix has teeth; if no, the pipeline order does not
   actually matter to the observable output on any test we have,
   and that itself is a finding (an unread invariant).

### Finding 2: dnglab does not implement DNG opcodes — the oracle is broken for warp

**Build's claim:** Confirmed by reading `dnglab/dnglab`'s own source
that it does NOT apply WarpRectilinear or FixBadPixelsConstant to
its `--srgb` output — no application code anywhere, only tag
copying. This means SPEC-020's oracle CANNOT validate a warp in
either direction: a correct warp scores **−60.169** against
dnglab's uncorrected reference; doing nothing at all scores
**+83.145** (closer to dnglab because we then match dnglab's own
unwarped output).

**⚠ This questions SPEC-020 at a structural level, not just SPEC-018.**
SPEC-020 shipped last week with the reasoning "SPEC-018 depends on
SPEC-020's oracle so the warp has a check it cannot rewrite"
(SPEC-020's `depends_on:` motivation is now inverted).

**Your check:**
1. **Read dnglab's source yourself and confirm the claim.** Don't
   trust the build's assertion; go to `dnglab/dnglab` on GitHub (or
   wherever your Homebrew bottle was built from) and grep for
   `WarpRectilinear`, `FixBadPixelsConstant`, `apply_opcode`,
   `OpcodeList3`. §15 rule 8 (behavioral pre-flight) and §16 rule 4
   (unrun-docs) both apply here: the claim "dnglab does not
   implement opcodes" is a runtime-behavioural claim, and the way to
   verify it is to read the code that would implement it — or, more
   directly, run `dnglab analyze --srgb` on `L1021223.DNG` and check
   whether the output shows any radial distortion correction (a
   straight line at the corner will bend outward if the warp applied,
   stay straight if not).
2. **Reproduce the two measured scores** on your machine:
   - "correct warp scored against dnglab" = −60.169
   - "no warp scored against dnglab" = 83.145
   Use SPEC-020's develop-oracle wiring; `cargo test --all-features
   --test warp -- --ignored --nocapture` runs the ignored test
   without ignoring it. If the numbers do not reproduce ± 0.5, that
   is a finding.
3. **Judge the `#[ignore]` decision.** AC8 and AC9 are `#[ignore]`d
   with DEC-024's reasoning inline (`tests/warp.rs:356` and `:403`).
   Is that honest — i.e., is the ignore's stated reason the actual
   reason — or is it hiding a defect in the applier itself? The way
   to distinguish: read `tests/warp.rs`'s AC4 and AC10 (the
   analytic corner-displacement tests). AC4 verifies the geometry
   against the DNG spec math directly, oracle-free. If AC4 passes
   and AC8 fails, the applier is right and the oracle is broken.
   If AC4 also has cracks, the applier is wrong.
4. **Judge whether the `#[ignore]` is the right disposition, or
   whether SPEC-020/DEC-005's oracle scope should be formally
   narrowed** — mirror SPEC-015's substitution of an analytic check
   where no comparison oracle covered the surface. This is
   ORCHESTRATOR-LEVEL judgement, not build-level — flag it as
   `SB-N` if you think ship must not proceed until it is decided,
   or as `FU-N` if ship can proceed with the current `#[ignore]` and
   a follow-up spec addresses the oracle scope.

## The four repo-specific verify bars (§15 bars 9–12)

**Bar 9 — did the oracle go red?** Two red-proofs are on the table:
   AC9 (tier-B, `kr1 = 0.0` mutation) and AC10 (tier-A, same
   mutation, synthetic input). Bar 9's original intent — that the
   perceptual oracle turns red on a broken warp — is now known to
   be structurally impossible on the current oracle (finding 2). So
   the FUNCTIONAL bar shifts to AC10: the tier-A analytic red-proof
   MUST fail cleanly when `kr1` is zeroed. Run it yourself; watch
   the peak location move ≥ 20 px. If it does not, the analytic
   check has no teeth either, and finding 2 becomes a ship-blocker.

**Bar 10 — fuzz target exists and ran?** YES — the build added
   `fuzz/fuzz_targets/warp_opcode.rs` per SPEC-018's spec, and
   `just fuzz-warp` runs the 60 s smoke. Confirm:
   1. `fuzz/fuzz_targets/warp_opcode.rs` exists.
   2. `app.just` has the `fuzz-warp` recipe with the `PATH=` prefix
      per §5's `+toolchain` trap.
   3. AGENTS.md §6's code block was updated with the new recipe (§6
      rule 8).
   4. CI on `40f5d45` shows the `fuzz smoke — warp_opcode` job green.
   5. `fuzz/seeds/warp_opcode/` has at least the real-Q2M-bytes seed
      plus the truncated / unknown-mandatory / length-overflow
      hand-crafted variants HANDOFF-046's `## References` names.

**Bar 11 — provenance rows for both new algorithms?** Check
   `docs/provenance-ledger.md` — expected TWO rows: `src/opcode.rs`
   (DNG 1.7.0.0 § Chapter 6, class 1) and `src/warp.rs` (transform
   from DNG 1.7.0.0 § 6.4.1 class 1; kernel — bilinear was the
   pre-registered first choice — class 1 if implemented from
   spec-side geometry, class 2 if from a permissive-crate reading).
   Read the rows and judge whether the classes are honest.

**Bar 12 — no new dependencies?** `Cargo.toml` under
   `[dependencies]` MUST have NO new entries. `library-not-
   application` is why. Run `cargo tree -e normal` — if anything
   other than `irradiance v0.1.0` alone appears, the whole spec is
   in question. Also confirm no `rayon`, no `is_x86_feature_
   detected`, no runtime SIMD dispatch (DEC-002 / AC12).

## The eight generic verify checks (§15)

1. All 14 acceptance criteria met and tested? (AC1..AC14 — the spec
   numbers them.) Two are `#[ignore]d` (AC8, AC9); judge whether
   that constitutes "met" per finding 2's reasoning above.
2. Failing tests from the spec now pass? Twelve named in `##
   Failing Tests`; run each with `cargo test --all-features
   <name> -- --exact --nocapture` and confirm each RAN (the
   `named-tests-can-pass-vacuously` trap fires on partial-name
   matches).
3. No drift from referenced decisions? Run `just decisions-audit
   --changed main`. DEC-002/004/005/011/016/018 were the original
   references; DEC-023 (SPEC-020's ssimulacra2 sanction) is
   preserved; DEC-024 is NEW and its scope is `src/warp.rs` +
   `src/opcode.rs` + `src/develop.rs`.
4. No constraint violations? The five blocking constraints all
   apply. `no-panics-on-untrusted-input` on the new parser is the
   sharpest.
5. Non-trivial implementer choices have accompanying DEC-\*?
   DEC-024 covers three: kernel (bilinear), pipeline order
   (unwrapped-then-warped-then-crop), and the oracle scope
   narrowing. Judge whether DEC-024 is a genuine record or a
   catch-all.
6. Implementer reflection answered (not mailed in)? Read
   HANDOFF-046's `## Reflection` and confirm substance.
7. `cost.sessions[build]` present after handback-sync (orchestrator
   already ran it — 116,480,125 tokens on claude-sonnet-5). Confirm.
8. Runtime-behavioural surfaces exercised, not shape-validated?
   The develop pipeline (the warp applies on real bytes to produce
   real pixels), the parser (real Q2M bytes round-trip), and the
   fuzz target (real 60s smoke) all pass this. **The oracle scope
   claim** does NOT — that is finding 2's crux, and your check is
   above.

## The four codified lessons + a-fix-inherits

1. **`measurement-over-generalised`** — the two scores in
   finding 2 (−60.169 and 83.145) are claims from ONE probe on ONE
   frame. Reproduce them on the other two decodable frames if you
   can; if the pattern holds across all three, the finding
   generalises; if it does not, dig deeper.
2. **`attribute-text-inside-doc-comments`** — the panic-free lint
   scanner already trips this. Assert match counts on any grep.
3. **`a-gate-that-fails-mutely-is-a-gate-that-never-ran`** — every
   new gate dies through its own error, not via `set -o pipefail`.
4. **`unrun-docs-carry-errors` N=6** — SPEC-018 shipped its own
   Context correction at SPEC-020's ship (instance 6 was the
   per-frame coefficient mis-cite). Do NOT re-introduce a claim
   about DNG § 6.4.1 or dnglab's source you did not personally
   verify.

**`a-fix-inherits-the-precondition-of-the-thing-it-fixes`** — the
build's `#[ignore]` for AC8/AC9 is a fix that inherits SPEC-020's
oracle assumption. The way to judge it: does the fix state the
assumption, or just work around it? DEC-024 states the assumption
explicitly (dnglab does not apply opcodes → the oracle scores
divergence, not correctness), which is the right shape.

## Return Criteria

1. **Verdict at the top:** `✅ APPROVED` / `⚠ PUNCH LIST` /
   `❌ REJECTED`, with the branch SHA. Note explicitly whether
   finding 2 is `SB-N` (block ship until DEC-024's oracle narrowing
   is confirmed or replaced) or `FU-N` (ship with the current
   `#[ignore]` and address at a follow-up spec).

2. **Every finding labelled `SB-N` / `FU-N` — numbering restarts at
   1 for SPEC-018** (per-spec convention, §15).

3. **The four repo-specific bars (9–12) each explicitly answered**
   with the command you ran and its observation.

4. **Finding 2's dnglab-doesn't-implement-opcodes claim verified
   IN CODE.** Cite the file path in dnglab's source that would
   apply the opcode and does not, or (equivalent) show the
   `dnglab analyze --srgb` output visually not applying the warp
   correction.

5. **The 12 named tests run and each observed passing** — sum
   across all targets, quote the counts. AC8's and AC9's `#[ignore]`
   status confirmed explicitly.

6. **`just lint-ci` run locally, clippy version asserted** (0.1.98
   at CI floating tip; local 0.1.97 diverges — PATCH-004 made
   both recipes print their version).

7. **CI observed green on `40f5d45`** — run id and job count. The
   `fuzz-warp` job green.

8. **`cargo deny check licenses`** and `cargo deny
   --manifest-path fuzz/Cargo.toml check licenses` both quoted.

9. **`cost.sessions[build]` present** after handback-sync. Report
   what tokens_total you observed.

10. **PR NOT opened by verify** — orchestrator opens it after your
    verdict lands. Do not push to the branch (verify edits nothing)
    unless you have a punch-list fix so small it is faster to apply
    than to file.

## Cost

Fill the `handback:` block above with real numbers from your
interface. `notes:` is ONE PHYSICAL LINE. Identify your own
transcript by scratchpad UUID, not text-matching. Price
per-component; +20% uplift for the turns writing the handback.
Fill the new `verdict:` field so it copies into
spec.task.verify_verdict at ship.

## References

- `projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction.md`
  — the spec.
- `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-046-build-warprectilinear-radial-geometric-correction.md`
  — the build handoff and its `## Completion` block detailing the
  two findings.
- `decisions/DEC-024-warprectilinear-kernel-pipeline-order-and-the-broken-oracle.md`
  — the new decision record. Read in full.
- `decisions/DEC-005-develop-oracle-mechanics.md` — the pre-registered
  ≥ 85 tolerance and its calibration; finding 2 questions its scope.
- `projects/PROJ-001-monochrome-dng-develop/specs/done/SPEC-020-develop-oracle-vs-dnglab-srgb.md`
  — the develop oracle SPEC-018 depends_on; its assumptions are what
  finding 2 questions.
- `src/opcode.rs`, `src/warp.rs`, `src/develop.rs` — new / modified;
  the pipeline-order fix (finding 1) lives in `develop.rs`.
- `tests/warp.rs` — the new integration tests, including the two
  `#[ignore]`d ones.
- `tests/perceptual_oracle.rs` — SPEC-020's oracle wiring.
- `docs/oracle-contract.md` — the three-layer oracle contract; §
  "This oracle is single-sourced" is finding 2's precedent.
- `docs/provenance-ledger.md` — bar 11 targets.
- **DNG 1.7.0.0 specification § 6.4.1 "WarpRectilinear"** — the
  authoritative source for the pipeline order (finding 1) and
  the transform.
- **dnglab source code** — `dnglab/dnglab` on GitHub or wherever
  Homebrew's bottle was built from. Finding 2's IN-CODE verification
  target.

## What this verify does NOT ask for

- **Re-designing SPEC-018 or SPEC-020.** Findings become `SB-N` /
  `FU-N`, dispositioned at ship.
- **Fixing the oracle scope.** That is a decision the ship cycle or
  a follow-up spec makes; verify judges whether the current state
  ships or not.
- **Running the ignored tests without `--ignored`.** They are
  documented as broken by finding 2; running them normally is
  vacuous evidence. Run them WITH `--ignored` to reproduce
  finding 2's numbers.
- **Opening the PR.** Orchestrator's step after verdict.
