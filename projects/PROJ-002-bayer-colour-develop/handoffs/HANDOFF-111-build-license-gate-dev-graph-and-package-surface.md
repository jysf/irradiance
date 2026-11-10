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
  id: HANDOFF-111
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-10
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-055

project:
  id: PROJ-002
  stage: STAGE-010
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
  tokens_total: 9800000            # REAL combined count — what cost-audit reads
  estimated_usd: 7.10              # tokens_total × your rate, or your harness's number
  duration_minutes: 45
  branch: claude/spec-055-build-37bb23
  pr: https://github.com/jysf/irradiance/pull/168
  completed_at: 2026-11-10         # YYYY-MM-DD
  notes: "tokens summed from this session's transcript (dc7e7e0e…jsonl, 58 assistant msgs, all claude-opus-5-5: 45.8k out, 137.8k cache-write, 8.81M cache-read, ~0.1k in = 9.00M at PR time, 9.60M after the handback commit) + ~0.2M for the final correction turn; /cost not callable from inside the session. Knob was [licenses] include-dev, not a [graph] setting." 
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-111: Licence gate over the dev-graph + package surface (SPEC-055, group D)

## Delegation Summary

The architect hands `SPEC-055` to the implementer for the **build** cycle: make the
licence gate actually evaluate the dev-dependency graph (root + `fuzz/`), proven by a
mutation red-proof, and gate the internal `irr` binary out of the default
build/package/install (review findings #4 and #6). The whole decision is **DEC-048**.
**Read SPEC-055 in full and DEC-048 first; DEC-048 records two cargo-deny knobs that
already FAILED in probing — start from `cargo deny list`, not from those.**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-055-license-gate-dev-graph-and-package-surface.md` — all of it, especially `## Design` and `## Failing Tests`.
- **Decision:** `./decisions/DEC-048-license-gate-over-the-dev-graph-and-gate-the-irr-binary.md` — the mechanism, the two dead-end probes, and the `irr` disposition rationale.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md` (re-planned by DEC-045).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (findings #4, #6).
- **Files:** `deny.toml`, `Cargo.toml` (the `[[bin]] irr` + `[features]`), `.github/workflows/ci.yml` (l.142 `cargo-deny-action@v2`, and the red-proof steps l.33/58/76 as the pattern), `app.just` (`deny` l.89, `deny-fuzz` l.106), `scripts/lint-red-proof.sh` + `gate-audit-red-proof.sh` (the red-proof template), `docs/license-policy.md`, `tests/demosaic_oracle.rs` (l.824 `CARGO_BIN_EXE_irr`).
- **Toolchain brief:** `./guidance/toolchain-brief.md` (DEC-004 rule 5).

> ⚠ Line numbers read 2026-11-10 against the current tree; re-confirm before editing.
> The exact cargo-deny dev-graph knob is yours to find (see DEC-048) — the red-proof
> is what proves you found it.

### Decisions that apply

- `DEC-048` — the dev-graph licence evaluation + the `irr` feature-gate this spec builds.
- `DEC-011` — `fuzz/` is a second cargo graph outside the library's; both must be denied (`just deny` + `just deny-fuzz`).
- `DEC-036` — the existing `fuzzing` feature precedent for a dev-only empty cargo feature.
- `DEC-045` — the re-plan. `DEC-009` — control run before any red-proof. `DEC-023`/`DEC-010` — why `ssimulacra2`/`toml` are dev-deps.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text.

- `no-copyleft-dependencies` — the gate must reject a GPL dep; the red-proof proves it.
- `oracle-must-be-shown-red` — the licence red-proof is the load-bearing deliverable.
- `library-not-application` — `irr` is a dev/oracle tool, not a shipped surface.

### Prior related work

- `SPEC-011` / `DEC-027` — the `clippy — fuzz graph` job: the same "one policy, two
  cargo graphs" move this applies to the licence gate.
- `SPEC-052` / `SPEC-053` (groups A / B) — sibling pre-0.2.0 hardening specs.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- The cargo-deny dev-graph fix + `scripts/deny-red-proof.sh` (control green, GPL-dev-dep
  mutation red) for root and `fuzz/`, wired into CI with a `just deny-red-proof` recipe.
- The `irr` feature-gate (`required-features = ["irr"]`) + a package-target-list check,
  run in CI; `just test` (`--all-features`) still builds `irr` and the oracle test passes.
- `docs/license-policy.md` reconciled; CHANGELOG `[Unreleased]` entry.
- A PR against `main` from branch `claude/spec-055-build-<suffix>`.
- PR description referencing: HANDOFF-111, SPEC-055, STAGE-010, PROJ-002, DEC-045 / DEC-048.
- Confirmation in the handback that a default `cargo build` produces no `irr` binary and
  `just deny`/`just deny-fuzz` both still pass on the real tree.

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

- Any change to the licence policy allow-list or `exceptions` (the NCSA/libfuzzer-sys
  exception stays); this closes the gap between the policy and the gate, nothing more.
- Any new runtime or dev dependency committed to the real tree (the red-proof's GPL
  crate is a throwaway in a temp dir only).
- The `fuzzing` feature, the parser guard (SPEC-052), the Compression-7 shape binding
  (SPEC-053).
- Deciding a DIFFERENT `irr` disposition than DEC-048's `required-features` gate — if
  the maintainer wants `examples/` or `exclude`, that is a new decision at PR review,
  not a build-time change.

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

### Execution notes

- **Branch / PR:** `claude/spec-055-build-37bb23` → https://github.com/jysf/irradiance/pull/168
- **Completed at:** 2026-11-10
- **All acceptance criteria met?** yes, with one deviation in how AC1 is verified (see Deviations).
  - **Dev-graph evaluated:** fix is `[licenses] include-dev = true` in `deny.toml`. Root `cargo deny check licenses` now evaluates 48 crates (was 1), including `toml v0.8.23`, `ssimulacra2 v0.5.1` and all their transitives. Fuzz graph evaluates 12 crates (build-dep `cc` was already covered by default). **`just deny` and `just deny-fuzz` both pass on the real tree** (confirmed after the change, and again in the final gate run).
  - **Licence red-proof bites:** `scripts/deny-red-proof.sh` → control green on both graphs (with `toml`/`ssimulacra2`/`libfuzzer-sys` named in the accepted list), a GPL-3.0-only dev-dep rejected by name (`error[rejected]`) in root and fuzz, the same root mutation green with `include-dev` removed, isolation green. Shown red: removing `include-dev` from the real `deny.toml` makes the script fail at its control. Measured green on cargo-deny 0.19.9 (local) and 0.20.2 (the `cargo-deny-action@v2` image). Wired as `just deny-red-proof` and CI job `licenses-red-proof` (pinned 0.20.2).
  - **Pre-fix confirmation:** before the change, a GPL-3.0-only dev-dep passed both `just deny` and `just deny-fuzz` (measured on temp copies).
  - **`irr` excluded:** `irr = []` + `required-features = ["irr"]`. **A default `cargo build` produces no `irr` binary** (no executable in cargo's JSON artifact stream; confirmed in a clean target dir too). `cargo package` succeeds, and the unpacked `.crate` built with default features (what `cargo install` builds) produces no executable either. `scripts/package-targets-check.sh` asserts all of that plus a positive control (`--features irr` produces `irr`) and a red half (temp copy with `required-features` stripped surfaces `irr`). Wired as `just package-targets-check` and CI job `package-surface`.
  - **Dev/oracle build intact:** `just test` green; `tests/demosaic_oracle.rs` 26/26 under `--all-features`, including on a clean target dir.
  - **Policy reconciled:** `docs/license-policy.md` gains a dev-dep section, a coverage table per graph and the knob in the worked example; also `no-copyleft-dependencies` enforcement text, AGENTS.md command list, `src/bin/irr.rs` header, CHANGELOG `[Unreleased]` (Security + Changed).
  - **Gates:** `just test`, `lint-ci`, `fmt-check`, `validate`, `cost-audit`, `deny`, `deny-fuzz`, `deny-red-proof`, `package-targets-check`, `gate-audit` all green locally.

### Cost self-report

- **Tokens (total):** 9,800,000 (summed from the transcript: 9.00M at PR time, 9.60M after the handback commit, + ~0.2M for the final correction turn)
- **Estimated USD:** 7.10 (Opus rates: $25/M out, $6.25/M cache-write, $0.50/M cache-read, $5/M in)
- **Duration (minutes):** ~45
- **Source of the number:** harness transcript — this session's `dc7e7e0e-1485-49ec-b98c-0097915b9d1b.jsonl`, de-duplicated by message id, all `claude-opus-5-5`. `/cost` is not callable by the agent from inside the session.

### Drift and new artifacts

- **New decisions emitted:**
  - none. DEC-048 left the exact knob to this cycle; it turned out to be `[licenses] include-dev`. Recorded in `deny.toml` and `docs/license-policy.md`; the verifier may want a one-line addendum on DEC-048.
- **Deviations from spec:**
  - **AC1 verified with `check licenses`, not `cargo deny list`.** `list` ignores `include-dev` and still prints only `irradiance@0.1.1` (on both 0.19.9 and with the knob set). Verification goes through `cargo deny -L debug check licenses`, whose accepted list names every evaluated crate; the red-proof's control asserts on that list.
  - **`tests/demosaic_oracle.rs` changed (one attribute).** Cargo still sets `CARGO_BIN_EXE_irr` without the feature but does not build the bin, so plain `cargo test` on a clean target failed `irr_demosaic_emits_a_valid_p6_ppm` (spawn error; measured). It is now `cfg_attr(not(feature = "irr"), ignore = "...")` — a visible skip by default, runs under `--all-features`. The spec said the file should pass "unchanged"; this was needed to keep plain `cargo test` working.
  - Also touched outside the spec's Outputs list: `guidance/constraints.yaml` (enforcement text only), `AGENTS.md` (command list), `src/bin/irr.rs` (doc header only).
- **Follow-up work identified:**
  - SPEC-042 (release, group F): the release checklist can now cite `just deny-red-proof` and `just package-targets-check` as evidence items.
  - The `licenses-red-proof` job pins cargo-deny 0.20.2 to match the action image; if the image moves, move the pin.
  - Unchanged: `just build` (`cargo build --release`) no longer builds `irr`; provenance-ledger RSS commands that use `target/release/irr` now need `--features irr` (noted in the irr.rs header, not rewritten in the ledger).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The "dev-deps not in the graph" framing pointed at graph gathering, but debug logs showed 48 crates gathered and 1 evaluated, which pointed straight at the licence check. AC1's "`cargo deny list` includes the dev-deps" is not achievable as written, since `list` ignores the knob.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `CARGO_BIN_EXE_*` + `required-features` interaction: cargo sets the variable without building the bin, so `cargo test` without features breaks at runtime, not compile time. Also worth listing that `Cargo.lock` is gitignored, so a red-proof can't rely on an offline lockfile in CI.

3. **If you did this task again, what would you do differently?**
   — Run `-L debug` on the first probe (it shows gathered vs evaluated right away), and run plain `cargo test` on a clean target dir before trusting a green test run after any `required-features` change. A stale `target/debug/irr` hid the break at first.
