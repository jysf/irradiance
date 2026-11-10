---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-055
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
  complexity_actual: S
  verify_verdict: punch-list

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5
  to_agent: null
  created_at: null

references:
  decisions: [DEC-045, DEC-048]
  constraints: [no-copyleft-dependencies, oracle-must-be-shown-red, library-not-application]
  related_specs: [SPEC-042]

depends_on: []                     # independent — A/B/D fan out in parallel (DEC-045)

claimed_by: null

value_link: "the permissive-licence claim is actually enforced, and the published crate ships no internal tool — pre-0.2.0 release hygiene"

cost:
  tokens_estimate: 18000000
  sessions:
    - cycle: build
      handoff: HANDOFF-111
      agent: claude-opus-5
      interface: other
      tokens_total: 9800000
      estimated_usd: 7.10
      duration_minutes: 45
      recorded_at: 2026-11-10
      notes: "tokens summed from this session's transcript (dc7e7e0e…jsonl, 58 assistant msgs, all claude-opus-5-5: 45.8k out, 137.8k cache-write, 8.81M cache-read, ~0.1k in = 9.00M at PR time, 9.60M after the handback commit) + ~0.2M for the final correction turn; /cost not callable from inside the session. Knob was [licenses] include-dev, not a [graph] setting."
    - cycle: verify
      handoff: HANDOFF-116
      agent: claude-opus-5-5
      interface: other
      tokens_total: 7400000
      estimated_usd: 4.96
      duration_minutes: null
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5 verify. Verdict PUNCH LIST at 0e975ae — the licence gate and irr gating were correct, but the build's new CI red-proof job was red from two script bugs (CI colour codes broke the grep; grep -q under pipefail killed printf mid-write). Both fixed (--color never; grep <<<here-string), shipped via the rebased #174. Independent mutation proof: GPL dev-dep rejected in root+fuzz; irr stays out of the package; real tree passes. tokens_total approximate, summed from the session transcript (/cost is not callable in a delegated session)."
  totals:
    tokens_total: 17200000
    estimated_usd: 12.06
    session_count: 2
---

# SPEC-055: Licence gate over the dev-graph + package surface (group D)

## Context

Pre-0.2.0 code-review findings **#4 (P1, reproduced locally)** and **#6 (P2)**,
triaged in `docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`; adopted by
DEC-045. The decision is **DEC-048**.

- **#4 — the licence gate checks almost nothing.** Probed 2026-11-10 (cargo-deny
  0.19.9): `cargo deny list` and `cargo deny check licenses` evaluate a graph of
  **only** `irradiance@0.1.1`. The dev-deps `toml` and `ssimulacra2` resolve
  (`cargo tree -e dev` shows both) but are absent from the graph — cargo-deny drops
  dev-deps of non-workspace crates by default, and this is a single-package repo.
  `docs/license-policy.md` says dev-deps are covered, so the gate is weaker than the
  policy; a GPL dev-dependency would pass. The `fuzz/` graph (`just deny-fuzz`) is a
  separate invocation that needs the same treatment.
- **#6 — the package ships the internal tool.** `Cargo.toml` declares
  `[[bin]] name = "irr"` unconditionally and `include` carries `src/**/*.rs`, so
  `cargo install irradiance` builds and installs the oracle tool, contradicting
  `library-not-application`. The tests invoke `irr` via `CARGO_BIN_EXE_irr`
  (`tests/demosaic_oracle.rs:824`), so it must stay buildable for the dev/oracle
  build.

## Design

**Per DEC-048.** Two independent sub-parts in one spec.

### 1. Licence gate over the dev-dependency graph (+ mutation red-proof)

- Configure cargo-deny so `cargo deny check licenses` evaluates the **dev-deps** of
  the root graph **and** the `fuzz/` graph. The exact cargo-deny 0.19.9 knob is a
  **build-cycle investigation** (DEC-048 records two probes that did NOT work:
  `[graph] exclude-dev = false`, and adding an empty `[workspace]` table — do not
  repeat them). Verify by `cargo deny list` showing `toml` / `ssimulacra2` and their
  transitive crates.
- **Mutation red-proof (the binding guarantee):** `scripts/deny-red-proof.sh`,
  following the existing red-proof pattern (`scripts/lint-red-proof.sh`,
  `gate-audit-red-proof.sh`): in a temp copy, add a synthetic **GPL** dev-dependency
  (a tiny local path crate declaring `license = "GPL-3.0"` is enough — no network),
  assert `cargo deny check licenses` **fails** naming it, with the unmutated control
  passing first (`oracle-must-be-shown-red`, DEC-009). Cover **both** the root and
  the `fuzz/` graphs. Add a `just deny-red-proof` recipe and a CI step beside the
  `licenses` / `licenses — fuzz graph` jobs (`.github/workflows/ci.yml:142`,
  `cargo-deny-action@v2`).
- Reconcile `docs/license-policy.md` so its dev-dep claim matches the now-real gate.

### 2. Gate the `irr` binary out of the default surface (+ target-list check)

- `Cargo.toml`: add `irr = []` under `[features]`, and
  `required-features = ["irr"]` to the `[[bin]] name = "irr"`. A default
  `cargo build` / `cargo install` / `cargo package` then does not build the `irr`
  executable; `just test` (`cargo test --all-features`) enables `irr`, so
  `CARGO_BIN_EXE_irr` still resolves and `tests/demosaic_oracle.rs` passes. (Mirrors
  the existing `fuzzing` feature.)
- **Package-target-list check:** a check that the **default-feature** target set has
  no `irr` binary — e.g. a test over `cargo metadata` / a default
  `cargo build --message-format=json`, or a `scripts/package-targets-check.sh` — run
  in CI, so a future unconditional bin fails loudly.

Chosen over moving `irr` to `examples/` (breaks `CARGO_BIN_EXE_irr`) and over
`exclude` (leaves a `[[bin]]` pointing at an excluded file — `cargo install` fails);
see DEC-048. **Confirm the `irr` disposition at PR review (open maintainer decision #2).**

## Inputs

- **`deny.toml`** — the `[graph]` / `[licenses]` config to extend; the `exceptions`
  (libfuzzer-sys NCSA) to preserve.
- **`Cargo.toml`** — `[features]` (l. `fuzzing = []`), the `[[bin]] name = "irr"`
  (l.42-ish), the `include` allow-list.
- **`.github/workflows/ci.yml`** — the `licenses` job (l.142, `cargo-deny-action@v2`),
  the `licenses — fuzz graph` job, and the existing red-proof steps (l.33/58/76) as
  the CI-wiring pattern.
- **`app.just`** — `deny` (l.89), `deny-fuzz` (l.106), and the `*-red-proof` recipes
  as the pattern.
- **`scripts/`** — `lint-red-proof.sh` / `gate-audit-red-proof.sh` as the red-proof
  template.
- **`docs/license-policy.md`** — the dev-dep claim to reconcile.
- **`tests/demosaic_oracle.rs`** (l.824, `CARGO_BIN_EXE_irr`) — the invocation that
  must keep working under `--all-features`.

## Outputs

- **Files modified:** `deny.toml` (dev-graph evaluation); `Cargo.toml` (`irr` feature
  + `required-features`); `.github/workflows/ci.yml` (red-proof + target-list steps);
  `app.just` (`deny-red-proof`, maybe `package-targets-check` recipes);
  `docs/license-policy.md`; `CHANGELOG.md` (`[Unreleased] → Changed`/`Security`).
- **Files created:** `scripts/deny-red-proof.sh`; a package-target-list check
  (script or test).
- **No library code change** and **no new dependency** (the red-proof's GPL crate is a
  throwaway in a temp dir, never committed to the real graph).

## Acceptance Criteria

- [ ] **Dev-graph evaluated:** after the fix, `cargo deny list` (root) includes the
      dev-deps `toml` and `ssimulacra2` and their transitive crates; `just deny` and
      `just deny-fuzz` both still pass on the real tree.
- [ ] **Licence red-proof bites:** `scripts/deny-red-proof.sh` shows the gate green on
      the unmodified tree and **red** when a synthetic GPL dev-dependency is injected,
      for both the root and the `fuzz/` graph; it is wired into CI.
- [ ] **`irr` excluded from the default surface:** a default `cargo build` (no
      `--all-features`) produces no `irr` binary, and the package-target-list check
      asserts it; `cargo package` succeeds (manifest self-consistent).
- [ ] **Dev/oracle build intact:** `just test` (`--all-features`) builds `irr` and
      `tests/demosaic_oracle.rs` passes unchanged.
- [ ] **Policy reconciled:** `docs/license-policy.md`'s dev-dep statement matches the
      gate's actual behaviour.
- [ ] **Panic-free / gates:** full gate suite green (`just test`, `lint-ci`,
      `fmt-check`, `validate`, `cost-audit`, both `deny` invocations).

## Failing Tests

- **`scripts/deny-red-proof.sh`** (the primary oracle) — control green, GPL-dev-dep
  mutation red, for root and `fuzz/`. Asserted by name/shape like the other
  red-proof scripts; run in CI.
- **Package target list** — a test (e.g. `tests/package_surface.rs`) or
  `scripts/package-targets-check.sh` asserting the default-feature build exposes no
  `irr` bin, and that `--features irr` does.
- **Oracle regression guard** — `tests/demosaic_oracle.rs` continues to pass under
  `--all-features` (no new test; it must not regress).

## Non-Goals

- No change to the licence **policy** allow-list or `exceptions`; this closes the gap
  between the policy and the gate.
- No new runtime or dev dependency in the committed tree.
- Not the `fuzzing` feature, the parser guard (SPEC-052), or the Compression-7 shape
  binding (SPEC-053).

## Notes for the Implementer

- Read `## Design` and **DEC-048** first; DEC-048 lists two cargo-deny knobs that
  already failed — start from `cargo deny list` and the cargo-deny 0.19.9 graph docs.
- The two sub-parts are independent; do them in either order. The licence red-proof
  is the load-bearing deliverable for #4.
- `just test` is `--all-features`, so it will build `irr` — confirm the oracle test
  still finds `CARGO_BIN_EXE_irr`. Default `cargo build` must NOT build it.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary:
   `design` | `build` | `verify` | `ship` | `escaped` | `none`.
   — <one word>

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after. Write `none` if this spec has no user-visible outcome.
   — <answer | none>
