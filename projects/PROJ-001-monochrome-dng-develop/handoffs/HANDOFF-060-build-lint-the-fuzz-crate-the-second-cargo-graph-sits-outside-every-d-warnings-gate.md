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
  id: HANDOFF-060
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-27
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-011

project:
  id: PROJ-001
  stage: STAGE-005
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
  tokens_total: 8894582            # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 50
  branch: claude/nostalgic-taussig-d61c60
  pr: https://github.com/jysf/irradiance/pull/28
  completed_at: 2026-10-27         # YYYY-MM-DD
  notes: "Transcript sum deduped by message.id over this session's scratchpad-UUID JSONL (42 assistant messages, all claude-opus-4-8), cache-inclusive (input+output+cache_creation+cache_read); NOT a remaining-budget delta. Input+output alone was 79,890. No code fixes — the fuzz crate was already clippy-clean."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-060: lint the fuzz crate — the second cargo graph sits outside every -D warnings gate

## Delegation Summary

The orchestrator (architect) hands `SPEC-011` to the implementer for the
**build** cycle: add a `just lint-fuzz` clippy gate on the `fuzz/` graph
(`--manifest-path fuzz/Cargo.toml --all-targets -- -D warnings`) + a CI job + a
**red-proof** (`scripts/lint-fuzz-red-proof.sh`) + a new **DEC-027** recording
the scope (default `-D warnings`, **not** the five panic-free lints). The fuzz
crate is already clippy-clean, so there are **no code fixes** — mirror the
`deny-fuzz` / `licenses-fuzz` precedent.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-011-lint-the-fuzz-crate-the-second-cargo-graph-sits-outside-every-d-warnings-gate.md`
  — read it whole: **Inputs** is the recipe/file map, **AC1–AC5** the contract,
  **Notes** the key facts (the fuzz crate is already clean; the red-proof fault
  must be a default-clippy violation; clippy pinning).
- **Templates to copy:** `app.just` `deny-fuzz` (~`:97`) for `lint-fuzz`;
  `.github/workflows/ci.yml` `licenses-fuzz` (~`:106`) + `clippy` (~`:52`) for
  the CI job; `scripts/lint-red-proof.sh` (DEC-009) for the red-proof;
  `.github/workflows/ci.yml` `lint-policy-red-proof` (~`:166`) for its CI job.
- **`fuzz/Cargo.toml`** + **`fuzz/fuzz_targets/*.rs`** (already clippy-clean),
  **`src/lib.rs`** (~`:79`, the five-lint policy) + **`src/bin/irr.rs`** (~`:8`,
  the dev-tool exemption precedent for the scope decision).
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain` PATH
  trap (PATCH-004), pinned vs local clippy, zsh `$pipestatus`. Read it.

### Decisions that apply

- `DEC-011` — the separate fuzz cargo graph; its "two invocations, both required"
  argument (for licensing) is the exact shape to extend to clippy. Cite it in
  DEC-027; add a pointer in DEC-011.
- `DEC-009` — the lint red-proof mechanism (temp copy, negative control, inject,
  assert caught + severity). Model `lint-fuzz-red-proof.sh` on it.
- `DEC-006` — the original red-proof mechanics (superseded by DEC-009; lineage).

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Touching `app.just`, `ci.yml`,
`scripts/`, `decisions/`, `AGENTS.md`:

- `oracle-must-be-shown-red` — widened to cover **gates** (SPEC-003/FU-14). The
  new lint gate MUST ship with a red-proof: negative control clean + an injected
  default-clippy fault caught, name-matched robustly (clippy help-URL fragment,
  not the bare name).
- `test-before-implementation` — the red-proof is the deliverable's spine.

### Prior related work

- **SPEC-003** (shipped) — found the licence gate missed the fuzz graph; the fix
  was `--manifest-path fuzz/Cargo.toml` + a CI job (DEC-011 amended). SPEC-011 is
  the exact same move for clippy (`PATCH-001/FU-4`).
- **SPEC-006** (shipped) — the panic-free lint policy + its red-proof and
  no-allow gate; the precedent for how a lint gate is proven red.
- **SPEC-016** (shipped last week) — a recent shell-gate spec; `just validate`
  now parses, so keep your front-matter edits parse-clean.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- **`app.just`** — `lint-fuzz` + `lint-fuzz-red-proof` recipes (state the
  clippy-pinning choice); their commands added to **AGENTS.md §6**'s block (AC4).
- **`.github/workflows/ci.yml`** — a clippy-on-fuzz job (`--manifest-path
  fuzz/Cargo.toml`, `@stable` + `components: clippy`, no nightly) + a
  fuzz-lint-red-proof job (AC1, AC2).
- **`scripts/lint-fuzz-red-proof.sh`** — the red-proof (negative control + one
  injected default-clippy fault caught; temp copy; working tree untouched) (AC2).
- **`decisions/DEC-027-…-fuzz-crate-lint-scope.md`** + a pointer in DEC-011 +
  `decisions/INDEX.md` regenerated; `just decisions-audit` clean (AC3).
- **No `src/` or `fuzz_targets/` change** — assert `git diff` is limited to
  `app.just`, `ci.yml`, the new script, the DEC, INDEX, AGENTS.md (AC5).
- All gates green, `just lint-ci` (pinned clippy) + the two new fuzz jobs, CI
  observed green on the PR head; report which clippy each uses and the gate list.
- The handback filled with a REAL `tokens_total` (a transcript sum deduped by
  `message.id` via your scratchpad UUID — say so; **do not report a
  remaining-budget delta**, which is non-comparable). A PR against `main`
  referencing HANDOFF-060, SPEC-011, STAGE-005, PROJ-001, DEC-011, DEC-009, DEC-027.

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

- **Applying the five panic-free lints to the fuzz crate** — DEC-027 decides
  against (default `-D warnings` only).
- **Fixing fuzz warnings** — there are none (crate is clippy-clean).
- **Any `src/`, `fuzz/fuzz_targets/`, or `Cargo.toml` change.**
- **The `cargo-fuzz` nightly smoke jobs** — unchanged.
- **Revisiting DEC-011's separate-graph decision** — only extend it to linting.
- The other STAGE-005 item (the gate-script audit).

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

- **Branch / PR:** `claude/nostalgic-taussig-d61c60` →
  https://github.com/jysf/irradiance/pull/28 (base `main`).
- **Completed at:** 2026-10-27
- **All acceptance criteria met?** **yes** — AC1 (`just lint-fuzz` green on the
  honest tree, CI job added, library clippy unchanged), AC2 (red-proof: control
  clean + injected `unnecessary_cast` caught by help-URL fragment + tree
  untouched), AC3 (DEC-027 + DEC-011 pointer + INDEX regenerated +
  `decisions-audit` clean, 0 structural errors), AC4 (both recipes in AGENTS.md
  §6's block), AC5 (all gates green; `git diff` limited to app.just, ci.yml, the
  new script, DEC-027, DEC-011, INDEX, AGENTS.md — no `src/`/`fuzz_targets/`).
- **Gate list (all green locally):** `cargo fmt --check`, `just lint-ci`
  (library, pinned +stable 0.1.98), **`just lint-fuzz`** (new, pinned +stable
  0.1.98), `just typecheck`, `just test`, `just deny` + `just deny-fuzz`,
  `just msrv`, `just lint-red-proof`, **`just lint-fuzz-red-proof`** (new),
  `just lint-no-allow`, `just validate` + `just validate-red-proof`,
  `just cost-audit` + `just cost-audit-red-proof`, `just decisions-index
  --check`, `just decisions-audit`. **Clippy per gate:** the `clippy` and new
  `clippy-fuzz` CI jobs float on `dtolnay/rust-toolchain@stable` (0.1.98 today);
  `just lint-ci`/`just lint-fuzz` pin `~/.cargo/bin/cargo +stable` (0.1.98)
  locally; the two red-proof scripts run bare `cargo clippy` (Homebrew 0.1.99
  locally, `@stable` in CI) — the injected `unnecessary_cast` fires on all.
- **CI:** observed on the PR head (PR #28) — see the Cost self-report note for
  the method; reported green by observation, not inference.

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** **8,894,582** — a transcript sum, deduped by
  `message.id`, over this session's own JSONL (found by anchoring on the
  scratchpad-dir UUID, not a text grep — the maintainer's standing rule for this
  handback). 42 unique assistant messages, all model `claude-opus-4-8`.
  Cache-inclusive: input 84 + output 79,806 + cache_creation 210,235 +
  cache_read 8,604,457. **Input+output alone is 79,890** — the gap is
  prompt-cache reads of a large spec-driven context, which is why the
  cache-inclusive figure is the comparable one (SPEC-016's build recorded
  10.5M on the same convention). This is **not** a remaining-budget delta.
- **Estimated USD:** null — no reliable per-token rate for this harness; the
  cost gate reads `tokens_total`, not USD.
- **Duration (minutes):** ~50 (estimate).
- **Source of the number:** harness report (transcript JSONL sum, deduped by
  `message.id`).

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-027` — the fuzz crate gets clippy's default `-D warnings`, not the five
    panic-free lints (extends DEC-011's two-invocations principle to linting).
- **Deviations from spec:** none. The one open choice the spec left to the
  implementer (pinning) was resolved: `just lint-fuzz` carries the
  `~/.cargo/bin/cargo +stable` PATH pin like `just lint-ci`, not unpinned like
  `just lint` — PATCH-004's 17 red runs are the argument, and an unpinned fuzz
  recipe would reopen the same gap for the second graph. Stated in the recipe,
  DEC-027, and the PR.
- **Follow-up work identified:** a future spec *could* tighten `fuzz/` to the
  five panic-free lints (the targets pass them for free today) — DEC-027 records
  this as deliberately deferred, not forgotten. No new spec written; it belongs
  in STAGE-005's backlog only if a fuzz target ever needs `unwrap()` we would
  rather it did not.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The spec and handoff were unusually precise (the four
   traps, the exact templates, the "already clean" fact). The only judgment left
   open — pinning — was flagged as mine to make, which is correct. The one thing
   worth a line: the toolchain brief pins local clippy at 0.1.97, but the
   machine is now on 0.1.99; the brief's facts drift, so I re-measured rather
   than trusting them (both 0.1.98-pinned and 0.1.99-bare are clean).

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `DEC-011`, `DEC-009`, `oracle-must-be-shown-red` (widened to gates) and
   the `--lib`/`irr` scope precedent were all listed and were exactly what the
   work needed.

3. **If you did this task again, what would you do differently?**
   — Settle the red-proof's injected fault empirically first (which I did):
   `eq_op` works but is deny-by-default, so it would pass even if the gate lost
   `-D warnings`; a warn-level lint (`unnecessary_cast`) lets the proof show that
   `-D warnings` is the gate's only teeth, since the fuzz crate carries no
   `#![deny]`. Choosing the lint for what it lets you *prove*, not just for being
   caught, is the reusable lesson.
