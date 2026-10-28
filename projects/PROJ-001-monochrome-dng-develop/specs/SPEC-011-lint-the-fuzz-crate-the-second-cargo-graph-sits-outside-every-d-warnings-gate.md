---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-011
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
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
  to_agent: claude-opus-5          # build delegated via HANDOFF-060
  created_at: 2026-10-27

references:
  decisions: [DEC-011, DEC-009, DEC-006]
  constraints: [oracle-must-be-shown-red, test-before-implementation]
  related_specs: [SPEC-003, SPEC-006, SPEC-016]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-002's <capability>". Optional; null is acceptable.
value_link: "STAGE-005: the fuzz crate — the second cargo graph the library's -D warnings gate never sees — gets its own clippy gate, closing the same uncovered-graph hole the licence gate had before deny-fuzz."

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
  tokens_estimate: 20000000        # build + verify; S, a lint gate + red-proof + a DEC, no code fixes
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-10-27
      notes: "Design in the orchestrator main loop (un-metered; null-with-note per AGENTS.md §4). A read-only Explore subagent gathered the inputs (112,053 tokens, 27 tool-uses): the lint-ci/deny-fuzz recipes + their CI jobs, the lint-red-proof mechanism (DEC-009), the fuzz crate state, and a LIVE clippy-on-fuzz run (clean, 0 warnings on 0.1.98 and 0.1.99, even under the five panic-free lints). That cost is the subagent's, not transcribed here."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-011: Lint the fuzz crate — the second cargo graph sits outside every -D warnings gate

## Context

> **Framed 2026-08-22, not designed.** Destination for `PATCH-001/FU-4`.
> ⚠ Numbered **011**, not 010: `just new-spec` assigned 010, which already
> belongs to a spec on the unmerged `feat/spec-005-metadata-oracle` branch —
> see the Notes at the bottom.

**`fuzz/` is linted by nothing.** Measured: `cargo clippy --all-targets
--all-features` at the repo root mentions `irradiance-fuzz` **0 times**, because
`DEC-011` deliberately keeps the fuzz crate outside the library's cargo graph so
`[dependencies]` stays empty.

This is **exactly** the shape `no-copyleft-dependencies` already had, and its
history is the argument for fixing it: `cargo deny` covered only the library
graph until `SPEC-003`'s verify found a hand-written table standing in for the
missing invocation — *and it was wrong on the one crate it existed to sanction*.
The fix was one flag, `--manifest-path fuzz/Cargo.toml`, and the rule written
into `constraints.yaml` was **"if a graph is not covered, add the invocation — do
not write down the answer."** The lint policy is now in the position the licence
policy was in then.

⚠ **It needs its own red-proof.** `oracle-must-be-shown-red` was widened by
`SPEC-003/FU-14` to cover **gates**, not just oracles, so a new lint gate ships
with a deliberate-fault proof and a negative control. That is why this is a spec
and not a line in `PATCH-001`.

Open question for design, not to be settled here: whether the fuzz crate should
carry the same five-lint panic-free policy as the library. It is dev-only and
never shipped, so the answer is plausibly *no* — but `DEC-011` put it outside the
graph for **dependency hygiene**, not to exempt it from review, and nobody has
decided.

## Goal

Close the gap where `fuzz/` — the separate cargo graph DEC-011 keeps outside the
library's `[dependencies]` — is seen by no `-D warnings` lint gate: add a
`just lint-fuzz` recipe (`cargo clippy --manifest-path fuzz/Cargo.toml
--all-targets -- -D warnings`) and a CI job, mirroring the `deny-fuzz` /
`licenses-fuzz` precedent, with a **red-proof** proving the new gate catches an
injected fault (`oracle-must-be-shown-red` covers gates). Record the lint
**scope** in a new decision: the fuzz crate gets clippy's default `-D warnings`,
**not** the library's five panic-free lints — it is dev-only tooling, and the
`irr` / `#[cfg(test)]` exemptions and the `no-panics-on-untrusted-input` `--lib`
scope are the precedent.

## Inputs

(All `file:line` cites are design-time; read the named item.)

- **`app.just`** (all lint recipes live here, not `justfile`): `deny-fuzz`
  (~`:97`, `cargo deny --manifest-path fuzz/Cargo.toml check licenses`) — the
  exact one-flag shape to mirror for clippy; `lint` (~`:62`) and `lint-ci`
  (~`:155`, the PINNED `~/.cargo/bin/cargo +stable clippy … --all-targets
  --all-features -- -D warnings` with the load-bearing `PATH` prefix);
  `lint-no-allow` (~`:134`) and `lint-red-proof` (~`:108`).
- **`.github/workflows/ci.yml`** — the jobs to mirror: `licenses-fuzz`
  (~`:106–114`, `cargo-deny-action` with `manifest-path: fuzz/Cargo.toml` — the
  structural template), `clippy` (~`:52–60`, the library's `clippy -D warnings`
  job — the command template), and `lint-policy-red-proof` (~`:166–200`, runs a
  red-proof script). A fuzz clippy job needs **no nightly** (fuzz is a plain
  cargo crate for clippy; only the `cargo-fuzz` smoke jobs need nightly).
- **`scripts/lint-red-proof.sh`** — the red-proof mechanism (DEC-009): copy the
  crate to a temp dir, a **negative control** (unmutated copy clean), inject a
  violation, assert it is caught and is `deny`-severity; working tree untouched.
  The template for the fuzz-lint red-proof (⚠ `scripts/validate-red-proof.sh`
  does **not** exist — do not cite it).
- **`fuzz/Cargo.toml`** (`irradiance-fuzz`; `libfuzzer-sys` + `irradiance` path
  dep; **no `[lints]` table**) and **`fuzz/fuzz_targets/*.rs`** (5 targets:
  `ifd`, `plane`, `develop`, `warp_opcode`, `opcode` — all written defensively,
  already clippy-clean).
- **`decisions/DEC-011-libfuzzer-sys-in-a-separate-fuzz-crate.md`** — the
  "two invocations, both required" argument (~`:158–213`); it is dependency-
  hygiene only and says nothing about linting, so SPEC-011 extends it.
- **`guidance/constraints.yaml`** — `no-panics-on-untrusted-input` (~`:29–33`,
  scope **`--lib` only**, explicitly excluding `#[cfg(test)]` and `src/bin/irr.rs`)
  and `oracle-must-be-shown-red` (covers gates, SPEC-003/FU-14). **`src/lib.rs`**
  (~`:79–86`, the five-lint `#![deny]`) and **`src/bin/irr.rs`** (~`:8–11`, the
  dev-tool exemption precedent — "unwrap is fine there").

## Outputs

A CI/lint gate spec — no `src/` or fuzz-target code change (the fuzz crate is
already clippy-clean).

- **`app.just`** — a new **`lint-fuzz`** recipe mirroring `deny-fuzz`:
  `cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings`,
  plus a **`lint-fuzz-red-proof`** recipe (`./scripts/lint-fuzz-red-proof.sh`).
  ⚠ The clippy-version divergence (local Homebrew 0.1.97/0.1.99 vs CI's pinned
  stable) cost this repo 17 red runs (PATCH-004): state whether `lint-fuzz` is
  unpinned-local (like `lint`) with the CI job as the authoritative gate, or
  carries the `~/.cargo/bin/cargo +stable` PATH pin (like `lint-ci`). Add both
  recipes' commands to **AGENTS.md §6**'s code block (the "every recipe's
  commands appear in the block" rule).
- **`.github/workflows/ci.yml`** — a new clippy-on-fuzz job (mirror
  `licenses-fuzz` structurally + `clippy` for the command; `--manifest-path
  fuzz/Cargo.toml`, `dtolnay/rust-toolchain@stable` + `components: clippy`, no
  nightly), and a job running `scripts/lint-fuzz-red-proof.sh` (mirror
  `lint-policy-red-proof`).
- **`scripts/lint-fuzz-red-proof.sh`** (new) — the red-proof, modeled on
  `lint-red-proof.sh` (DEC-009) but simpler: copy `fuzz/` (+ the library it path-
  depends on) to a temp dir, **negative control** (unmutated copy passes
  `-D warnings`), inject **one default-clippy violation** into a copied fuzz
  target, assert the gate catches it (non-zero, named), revert/clean. The fault
  must be something **default `-D warnings` catches** (the chosen scope) — NOT a
  panic-free-lint-only violation, since those are not in scope and the crate
  passes them anyway.
- **`decisions/DEC-027-…-fuzz-crate-lint-scope.md`** (new) — records: (a) the
  lint gate is a **second clippy invocation** on `fuzz/Cargo.toml`, extending
  DEC-011's "two invocations, both required" principle from licensing to
  linting; (b) **scope = clippy default `-D warnings`, NOT the five panic-free
  lints** — rationale (fuzz is dev-only glue; a harness panic is a test bug
  libFuzzer already catches, not a product vulnerability; precedent: the
  `irr`/`#[cfg(test)]` exemptions and `no-panics-on-untrusted-input`'s `--lib`
  scope), with the counter-argument weighed (the targets pass the five for free
  today, so a future spec could tighten). Cites DEC-011. `just decisions-index`
  regenerated; a pointer added in DEC-011.
- **No `src/` or `fuzz/fuzz_targets/` change** — the crate is already clean. No
  new provenance row.

## Acceptance Criteria

- [ ] **AC1 — the fuzz graph is linted.** `just lint-fuzz` runs
      `cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings`
      and exits 0 on the honest tree (the crate is already clean — measured 0
      warnings on CI-pinned clippy 0.1.98 and on 0.1.99). A CI job runs it. The
      library's root `clippy` job is unchanged and still does **not** see `fuzz/`
      (two invocations, both required — DEC-011's shape). State in the handback
      which clippy the recipe and CI job use.

- [ ] **AC2 — the gate is shown red (`oracle-must-be-shown-red`, covers gates).**
      `scripts/lint-fuzz-red-proof.sh`, run from CI and locally: (i) a **negative
      control** — the unmutated `fuzz/` copy passes `-D warnings` (exit 0); (ii)
      a **default-clippy** violation injected into a COPIED fuzz target is
      **caught** — the mutated copy exits non-zero and the offending lint is
      named (matched robustly, not by a string that also appears in the source);
      (iii) the working tree is never mutated. The injected fault is a
      default-lint violation (the chosen scope), not a panic-free-only one.

- [ ] **AC3 — the lint scope is decided and recorded.** `DEC-027` exists: the
      fuzz crate gets clippy default `-D warnings`, **not** the five panic-free
      lints, with the rationale and the counter-argument weighed; it extends
      DEC-011's two-invocations principle to clippy and cites it. DEC-011 carries
      a pointer. `just decisions-index` regenerated; `just decisions-audit` clean.

- [ ] **AC4 — the recipes are discoverable.** `lint-fuzz` and
      `lint-fuzz-red-proof` appear with their commands in **AGENTS.md §6**'s code
      block (the repo's "every recipe's commands appear in the block" rule).

- [ ] **AC5 — gates green, CI observed on the shipping SHA.** `just lint-ci`
      (pinned clippy), the existing suite, and the two new fuzz jobs. Name the
      gate list you ran; report CI green by observation, not inference. **No
      `src/` or `fuzz_targets/` change** (assert `git diff` is limited to
      `app.just`, `ci.yml`, the new script, the DEC, INDEX, and AGENTS.md).

## Failing Tests

The deliverable's "test" is the red-proof shell gate (AC2), not a cargo test —
and as in SPEC-016, `lint-fuzz` itself PASSES on the honest tree (the crate is
clean). What makes the gate load-bearing is the red-proof.

- **`scripts/lint-fuzz-red-proof.sh`** (new)
  - negative control: the unmutated `fuzz/` copy passes `cargo clippy
    --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings` (exit 0).
  - injected fault: one default-clippy violation in a COPIED fuzz target is
    caught (exit non-zero, lint named robustly); working tree untouched. State
    its exact form (which lint, which target) in the handback.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- **Applying the five panic-free lints to the fuzz crate** — DEC-027 decides
  against (default `-D warnings` only). A future spec may tighten; the targets
  pass the five for free today.
- **Fixing fuzz warnings** — there are none; the crate is already clippy-clean.
- **Changing DEC-011's dependency-hygiene / separate-graph decision** — SPEC-011
  extends it to linting, it does not revisit it.
- **Any `src/`, `fuzz/fuzz_targets/`, or `Cargo.toml` change.**
- **The `cargo-fuzz` nightly smoke jobs** (`fuzz-opcode` / `fuzz-warp`) — unchanged.
- The other STAGE-005 item (the gate-script audit).

## Notes for the Implementer

⚠ **This spec's own ID is a finding.** `just new-spec` assigned it **010** —
already taken by `SPEC-010-distinguish-an-unparseable-tool-reading-…`, which
exists only on the unmerged `feat/spec-005-metadata-oracle` branch. The script
scans the working tree, so an ID minted while a branch is in flight can collide
with one minted on that branch, and `AGENTS.md` §2 says IDs are *globally unique
and continuous across the repo*. Renamed to **011** by hand before it landed.
Recorded as a signal (`spec-ids-collide-across-unmerged-branches`); the fix
belongs upstream in the template, not here.

- **The fuzz crate is ALREADY clippy-clean** (0 warnings on 0.1.98 and 0.1.99,
  even under all five panic-free lints) — the gate ships green with no code
  fixes. So the red-proof's injected fault must be a **default-clippy** violation
  the `-D warnings` scope catches (an unused variable / import, or a clippy
  default like `needless_return`), NOT a panic-free-only one.
- **Model**: `lint-fuzz` on `deny-fuzz` (`app.just:~97`); the CI job on
  `licenses-fuzz` (`ci.yml:~106`) + `clippy` (`ci.yml:~52`); the red-proof on
  `scripts/lint-red-proof.sh` (DEC-009 two-run shape, but one injected fault, not
  five). No nightly — `fuzz/` is a plain cargo crate for clippy. ⚠
  `scripts/validate-red-proof.sh` does not exist; don't cite it.
- **Clippy pinning** (PATCH-004, 17 red runs): the CI job uses
  `dtolnay/rust-toolchain@stable`; a local `lint-fuzz` can be unpinned, but say
  so and make the CI job the authoritative gate — or add the `~/.cargo/bin/cargo
  +stable` PATH pin like `lint-ci`.
- **The red-proof's lint-name match must be robust** — `lint-red-proof.sh`
  matches clippy's help-URL fragment, not the bare name, so the proof can't pass
  on a string coincidence. Borrow that discipline.
- zsh: `$pipestatus`, not `PIPESTATUS`. Working tree stays clean (the red-proof
  mutates a temp copy only).

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
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
   — <answer | none>
