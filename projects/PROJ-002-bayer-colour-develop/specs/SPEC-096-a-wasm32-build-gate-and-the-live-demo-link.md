---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-096
  type: chore                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-002
  stage: STAGE-033
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: 2026-11-25

references:
  decisions: [DEC-002, DEC-009]   # [DEC-NNN, DEC-MMM]
  constraints: [oracle-must-be-shown-red, library-not-application, one-spec-per-pr, cost-captured-per-cycle]
  related_specs: [SPEC-056]        # doc-check: the gate + red-proof shape this copies

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-033's <capability>". Optional; null is acceptable.
value_link: "the gate and the README link that make 'irradiance builds for the browser' a CI-checked, discoverable fact (STAGE-033)"

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
  tokens_estimate: 8000000
  sessions:
    - cycle: build
      handoff: HANDOFF-200
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 8799858
      estimated_usd: 6.79
      duration_minutes: 60
      recorded_at: 2026-11-25
      notes: "FLOOR taken before the handback commit, from transcript bc4383e8-214e-44a7-a105-8d69a9586e17.jsonl (52 unique message.ids, all claude-opus-5-5, no subagents): input 103 + cache-creation 161,883 (1-hour) + cache-read 8,602,932 + output 34,940, priced at 5 / 10 / 0.5 / 25 USD per million. INCLUDES the design cycle: one side session designed and built this spec, so design is not separately metered. Re-sum at ship."
    - cycle: verify
      handoff: HANDOFF-201
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 1212540
      estimated_usd: 1.68
      duration_minutes: 4
      recorded_at: 2026-11-25
      notes: "Cold verify by a separate subagent; self-measured from its transcript agent-aa7bc8d73c353b98b.jsonl (14 unique message.ids): input 28 + cache-creation 94,265 + cache-read 1,110,937 + output 7,310, at 5 / 10 / 0.5 / 25 USD per million. APPROVED at cb00c38."
  totals:
    tokens_total: 10012398
    estimated_usd: 8.47
    session_count: 2
---

# SPEC-096: a wasm32 build gate and the live demo link

## Context

STAGE-033 (PROJ-002), its only spec. `jysf/irradiance-demo` deploys "Anatomy of a RAW" to
GitHub Pages: the published crate (0.2.6) compiled to WebAssembly, showing each pipeline
stage on a real DNG, with output SHA-256 equal to the native build's. DEC-002 (still
`proposed`) kept the browser target open; its Validation section asks that "a `wasm32`
build of the plane path compiles without contortions". It does, today, with no changes,
but nothing in this repo checks it.

**Design-time probe (2026-11-25, rustc 1.98.0 stable via rustup, aarch64-apple-darwin, on
`3f9df0b`):**

- `cargo build --release --target wasm32-unknown-unknown --lib` and the same with
  `--features write` both finish clean (about 2.5 s each, warm).
- The handoff's suggested faults do **not** go red. On `wasm32-unknown-unknown`,
  `std::time::Instant::now()` and `std::fs::read` compile (std ships them as stubs that
  panic or return `Unsupported` at run time), so a build gate cannot see them. Measured by
  appending each to a temp copy of `src/lib.rs`: both wasm builds finished.
- Faults that compile natively and fail on wasm32, measured the same way:
  - a unix-only std API (`use std::os::unix::ffi::OsStrExt; s.as_bytes()` on an `OsStr`):
    `error[E0433]: cannot find \`unix\` in \`os\``, plus E0599 for `as_bytes`;
  - a constant that assumes a 64-bit `usize` (`pub const X: usize = 1 << 32;`):
    `error[E0080]: attempt to shift left by \`32_i32\`, which would overflow`.
- Local gotcha: Homebrew's `cargo`/`rustc` come first on `PATH` and have no wasm32 std
  (`error[E0463]: can't find crate for \`std\``). `rustup run stable cargo` alone still hit
  it here; prefixing `PATH="$HOME/.cargo/bin:$PATH"` (as `just lint-ci` does) works.

## Goal

CI fails when the library stops compiling for `wasm32-unknown-unknown` (default features or
`write`), a red-proof shows that gate going red on wasm-only faults and green on the honest
tree, and the README links the live demo.

## Inputs

- `scripts/doc-check.sh`, `scripts/doc-check-red-proof.sh` — the shape to copy (optional
  crate-dir argument, temp-copy red-proof that runs the exact gate script, PATCH-004 "say
  which tool answered", counted surfaces, attributed red).
- `app.just` (`doc-check`, `lint-ci`), `.github/workflows/ci.yml` (`docs`, `docs-red-proof`).
- `README.md` (status block, "Building and testing"), `AGENTS.md` §6.

## Outputs

- **Created:** `scripts/wasm-check.sh` — the gate. `wasm-check.sh [crate-dir]`, default the
  repo root. Prints the `cargo` and `rustc` that answered. Fails with a named reason if the
  active toolchain has no `wasm32-unknown-unknown` std (sysroot
  `lib/rustlib/wasm32-unknown-unknown/lib` absent), before building. Builds two surfaces,
  in order: `cargo build --release --target wasm32-unknown-unknown --lib` (default
  features), then the same with `--features write`. Asserts 2 surfaces built and the rlib
  exists under the target dir. Prints a ✓ line with the count.
- **Created:** `scripts/wasm-check-red-proof.sh` — copies `Cargo.toml`, `Cargo.lock`,
  `README.md`, `src/` to a temp dir with its own `CARGO_TARGET_DIR`; runs the gate on the
  unmutated copy (control, must pass); then two isolated mutations of the copied
  `src/lib.rs`, each restored from a pristine copy before the next:
  - **A, unix-only API, unconditional:** native `cargo build --release --lib` of the
    mutated copy must pass; the gate must fail with E0433 naming `unix`.
  - **B, 64-bit `usize` constant behind `#[cfg(feature = "write")]`:** native
    `cargo build --release --lib --features write` must pass; the gate must fail with E0080
    naming the overflow, and its output must show the default surface finished first (so
    B proves the `write` surface is really built).
- **Modified:** `app.just` — recipes `wasm-check` and `wasm-check-red-proof`, each running
  its script with `PATH="$HOME/.cargo/bin:$PATH"` so a local rustup toolchain wins over
  Homebrew's (harmless where that directory does not exist).
- **Modified:** `.github/workflows/ci.yml` — jobs `wasm32` ("rust / wasm32 build (browser
  target)") and `wasm32-red-proof`, each `dtolnay/rust-toolchain@stable` with
  `targets: wasm32-unknown-unknown`, running the script.
- **Modified:** `README.md` — one or two sentences linking https://jysf.github.io/irradiance-demo/
  and https://github.com/jysf/irradiance-demo; a `just wasm-check` line in "Building and
  testing".
- **Modified:** `AGENTS.md` §6 — the two recipes, documented like `doc-check`.
- No `src/` change, no `Cargo.toml` change, no new dependency, no DEC (the shape is
  SPEC-056's; DEC-002 is cited, not amended).

## Acceptance Criteria

- [ ] AC1: `just wasm-check` on the honest tree exits 0 and reports 2 surfaces built (default,
      `write`).
- [ ] AC2: `just wasm-check-red-proof` exits 0: control green; mutation A native-green and
      gate-red with E0433 naming `unix`; mutation B native-green (with `write`) and gate-red
      with E0080, after the default surface finished.
- [ ] AC3: the gate fails with a named reason, not a compiler error, when the toolchain has no
      wasm32 std (checked by hand once at build with Homebrew's toolchain first on PATH;
      recorded in the handoff).
- [ ] AC4: CI jobs `wasm32` and `wasm32-red-proof` exist and are green on the PR.
- [ ] AC5: `README.md` links the live demo and its source repo; `AGENTS.md` §6 lists both
      recipes.
- [ ] AC6: `just gate-audit` passes with the two new scripts in its scan (they are
      `scripts/*.sh` gates and must not have the mute-death shape).
- [ ] AC7: no change under `src/` or to `Cargo.toml`.

No numeric tolerance is involved.

## Failing Tests

The gate's tests are its red-proof (oracle-must-be-shown-red covers gates; DEC-009):

- **`scripts/wasm-check-red-proof.sh`** (does not exist at design, so `just
  wasm-check-red-proof` fails):
  - control — the gate passes the unmutated copy, 2 surfaces;
  - mutation A — native green, gate red, E0433 naming `unix`;
  - mutation B — native green with `write`, gate red, E0080, default surface finished first.

**Pre-registered mutants of the gate** (verify runs each against the red-proof):

- M1: the gate drops `--target wasm32-unknown-unknown` (builds for the host). Caught: A and
  B both stay green.
- M2: the gate builds only the default surface. Caught by B (its fault is behind `write`).
  Distinguishing fixture: B; A cannot distinguish (it fails on the default surface).
- M3: the gate swallows cargo's exit (`|| true` on a build). Caught: A stays green.
- M4: the red-proof's native control is removed. Not caught by the red-proof itself (it
  is the red-proof); verify checks by reading that each mutation's native build runs and
  is asserted.

**Rule-clause map / moved assertions:** no DEC rule implemented and no behaviour changed;
none.

## Non-Goals

- Running tests or the develop under a wasm runtime. Run-time-only failures on wasm32
  (`Instant::now`, `std::fs`, `std::thread::spawn`, a 4 GiB allocation) are not seen by a
  build gate; filed as a follow-up below.
- `wasm32-wasip1` or other wasm targets; `--no-default-features` (default features are
  empty today, so it is the same surface).
- Any wasm-bindgen glue, JS, or app surface in this crate (`library-not-application`).
- Changing DEC-002's status (the maintainer's call; the demo is new evidence for its
  Validation section).

## Notes for the Implementer

- Copy `doc-check`'s structure closely; `gate-audit` scans `scripts/*.sh` and asserts
  counts, so count everything you claim.
- rustc's wording varies by version: match the error codes (`E0433`, `E0080`) and the
  token `unix` / `overflow`, not the whole sentence.
- Name the injected items so they cannot collide (`__wasm_check_red_proof_*`).
- Follow-up to file at ship: a run-time wasm smoke (for example `cargo test` under
  `wasm32-wasip1` with wasmtime, or a clippy `disallowed_methods` list for
  `Instant::now`/`SystemTime::now`/`std::fs`/`std::thread::spawn` in library code).

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?**
   — <one word>

5. **What can a user do now that they couldn't before?**
   — <answer | none>
