# AGENTS.md — Claude + Implementer Variant

Instructions for any AI agent working in this repository. Read this file first, every session.

> This file contains conventions only. For rules/constraints, see `/guidance/constraints.yaml` — **that file wins**; this one explains it. For architectural rationale, see `/decisions/`. For waves of work against this app, see `/projects/`.

---

## 1. Repo Overview

- **Repo (the app):** `irradiance`
- **Purpose:** A permissively-licensed, pure-Rust library that reads camera RAW
  files and develops them into images — bytes in, pixels and metadata out.
- **Primary stakeholders:** the maintainer (jysf); `crustyimg` as the first
  consumer; Rust imaging projects currently forced onto copyleft RAW crates.
- **Active project:** `PROJ-001` — Monochrome DNG develop, end to end.

See `.repo-context.yaml` for structured metadata.

### What this repo is, and is not

`irradiance` is a **library**. It takes a byte slice and returns pixels plus
typed metadata. It performs **no I/O**, ships **no CLI**, depends on **no
`image` crate**, and runs **no async runtime**. The consumer opens the file,
picks the allocator, and owns the encode.

`irr` is an **internal dev/oracle binary** — a bin target inside this crate used
to drive oracle comparisons and dump intermediates during development. It is
never a product surface, never documented for end users, and never the thing a
feature is designed around. If a capability only makes sense through `irr`, it
is a development affordance, not a library feature.

The API boundary is the point. A library named after its consumer accumulates
that consumer's types; this one must not. Constraint: `library-not-application`.

### The four disciplines this repo actually rests on

These are the blocking constraints in `guidance/constraints.yaml`. They are not
style preferences — each one is why this library can exist at all.

1. **Panic-free on untrusted input** (`no-panics-on-untrusted-input`). RAW is
   attacker-influenced binary: vendor-supplied offsets, tile tables, Huffman
   tables. A panic in a library is a denial of service in every consumer, and
   consumers include servers accepting uploads from strangers.
2. **The oracle must be shown red** (`oracle-must-be-shown-red`). Every oracle
   ships with a deliberate-fault test proving it FAILS on a broken input. A
   green oracle that cannot fail manufactures confidence. See §12.
3. **Provenance recorded per algorithm** (`provenance-recorded-per-algorithm`).
   Every algorithm and decoder gets a row in `docs/provenance-ledger.md`.
   **Reading a copyleft implementation is not permitted for anything in the
   default build.** See §10 and §15.
4. **Permissive dependencies only** (`no-copyleft-dependencies`). Permissive
   *is* the differentiator; every mature RAW decoder in every language is
   copyleft or C++. `rawler`/`rawloader`/`zenraw`/`quickraw`/`imagepipe` are
   out, including as dev-dependencies. `dnglab` is used as a **tool** — run,
   never linked — which imposes nothing.

---

## 2. Work Hierarchy

```
REPO (the app — persists across all projects)
 └─ PROJECT (a wave of work: "MVP", "improvements", "v2 redesign")
     └─ STAGE (a coherent chunk within a project)
         └─ SPEC (an individual task)
              └─ HANDOFF (architect → implementer delegation record)
```

Key distinctions:

- The **repo** is the app. It persists. `AGENTS.md`, `/docs/`, `/guidance/`,
  `/decisions/` live at repo level because they accumulate across all
  projects.
- A **project** (`/projects/PROJ-*/`) is a bounded wave of work. Project
  artifacts (brief, stages, specs, handoffs) live inside the project
  folder.
- A **stage** is an epic-sized chunk within a project. A project typically
  has 2–5 stages.
- A **spec** is a single implementable task. It belongs to exactly one
  stage within one project.
- A **handoff** is an architect-to-implementer delegation document.

**Decisions persist at repo level**, even though they're often made
during a specific project. A decision like "the bit unpacker is
row-major, MSB-first" was made during PROJ-001 but binds PROJ-002 and
PROJ-003 too. This is intentional.

**Specs do not cross project boundaries.** If a task isn't finished
when a project ships, either finish it first or defer it explicitly into
the next project's brief.

**IDs are globally unique and continuous across the repo.** `STAGE-*` and
`SPEC-*` numbers keep counting up across projects — they do **not** restart at
001 per project. If PROJ-001 ends at `STAGE-006` / `SPEC-037`, PROJ-002 begins
at `STAGE-007` / `SPEC-038`. `just new-stage` / `just new-spec` assign the next
number repo-wide, so an ID unambiguously identifies one artifact anywhere.

**There is no `just new-project`.** Projects are created by hand: copy
`projects/_templates/project-brief.md` to
`projects/PROJ-NNN-<slug>/brief.md` and fill it in. `just new-stage` /
`just new-spec` create the surrounding directories on demand
(`scripts/new-stage.sh:37`), so nothing else is needed. Recorded in DEC-000.

---

## 3. Business Value

Value structure exists at project and stage levels; specs link lightly.

**Project `value:` block** states the thesis — a testable claim about
what this wave of work delivers. Beneficiaries, success signals, and
risks to the thesis make it falsifiable, not marketing copy.

**Stage `value_contribution:` block** states what this coherent chunk
of work advances, what capabilities it delivers, and what it
explicitly doesn't try to do. Helps avoid stages that seem valuable
but don't contribute to the project thesis.

**Spec `value_link:`** is a one-sentence reference back to the
stage's value. Infrastructure specs may have
`value_link: "infrastructure enabling X"`. Optional but encouraged —
it surfaces specs that don't trace back to the thesis.

Reports (`just report-daily`, `just report-weekly`) aggregate these
signals: which stages advanced the thesis, which specs most directly
delivered it, and where value traceability broke down.

---

## 4. Cost Tracking Discipline

Every cycle on a spec appends a session entry to the spec's
`cost.sessions` list, with a **real** `tokens_total` for metered cycles —
so reports aggregate actual AI spend, not zeros. Documentation alone is
skippable, and cost tracking silently goes empty (all-null numerics) the
moment a prompt says "leave it null"; the rule below + `just cost-audit`
make it stick. Full reference: `docs/cost-tracking.md`.

- **Schema:** a single combined `tokens_total` per session (most harnesses
  report one number — `/cost` in Claude Code, the `usage` object from an
  API call, `subagent_tokens` in an `Agent` result). Do NOT split
  input/output; there is no reliable split.
- **build / verify cycles** are the metered ones and must NOT be left
  null. The agent that runs the cycle records the real `tokens_total` /
  `duration_minutes` / `estimated_usd` from its own interface — the
  implementer for **build** (Claude Code `/cost`, the API `usage` object,
  or whatever its tool reports), the reviewer for **verify**. Carry the
  build number across in the handoff if the implementer can't write the
  spec directly; whoever ships confirms the numbers are present.
- **design / ship cycles** are main-loop work with no clean per-cycle
  metering — leave numerics `null` with a "main-loop, not separately
  metered" note.
- **`estimated_usd`** = `tokens_total` × your model's published list rate,
  no cache discount — an order-of-magnitude estimate; say so in the note.
- **Interfaces:** set `interface:` to `claude-code` | `claude-ai`
  (estimate by length) | `api` (the `usage` object) | `ollama` | `other`.
  Only genuinely un-metered cycles may be null-with-note.

The cycle-prompt wording lives in
`projects/_templates/prompts/cost-snippet.md` — use it so prompts don't
re-introduce the "null numerics" loophole. **Ship computes `cost.totals`**
(sum of non-null sessions; `tokens_total` uses `0`, never `null`) and runs
`just cost-audit`, which **fails if any shipped spec lacks build/verify
cost** (constraint `cost-captured-per-cycle`; CI job `cost-data`; surfaced
in `just status` and `report-weekly`). Pre-process specs can be
grandfathered via `COST_AUDIT_GRANDFATHERED` in `scripts/_lib.sh` (empty
by default).

Reports aggregate cost by cycle, by interface, by spec, and by stage.

⚠ **This repo's estimates are borrowed, not measured.** PROJ-001's effort
figures were derived from crustyimg's cost data on a mature repo
(`projects/PROJ-001-monochrome-dng-develop/brief.md`, `risks_to_thesis`).
Greenfield may differ in either direction. Treat the first stage's actuals
as the first real datapoint, and let `just calibration` replace the
borrowed band as soon as it has one.

---

## 5. Tech Stack

- **Language:** Rust, **edition 2021**, **stable** toolchain. No nightly
  features in the library itself.
- **Runtime:** none. A native library — **no async runtime, no I/O, no CLI**
  (constraint `library-not-application`). `#![forbid(unsafe_code)]` unless a
  specific `DEC-*` says otherwise.
- **Framework:** none. A minimal, permissive dependency set decided per-DEC.
- ⚠ **Target surface is OPEN — see `DEC-002` (`status: proposed`, 0.72).** It
  proposes `no_std` + `alloc` where possible with `std` behind a default-on
  feature, **no `rayon`** (parallelism is the caller's choice), and output
  determinism pinned within a `develop_version`. It is gated on SPIKE-001
  measuring the cost. Until it is accepted or rejected: do not add `rayon`, do
  not assume `std` on the algorithmic path, and do not introduce runtime SIMD
  dispatch. A spec that forces the question should stop and ask.
- **Database:** none.
- **Testing:** `cargo test` (built-in harness) + `cargo fuzz` (libFuzzer) for
  every parser. `cargo-insta` is installed locally if snapshot tests are wanted;
  adopting it needs a `DEC-*` like any other dependency.
- **Linter / Formatter:** `cargo clippy` (0.1.97) and `cargo fmt` (rustfmt 1.9.0).
  `cargo-deny` (0.19.9) enforces the licence policy.
- **Hosting:** crates.io eventually — **not during PROJ-001**
  (`STAGE-004`, out of scope: publishing waits for a second camera).
  crustyimg consumes it as a **path dependency** behind a `raw-develop`
  cargo feature until then.
- **CI:** GitHub Actions, `.github/workflows/ci.yml`. Carries the
  language-agnostic `cost-data` + `decisions-index` gates plus the Rust jobs
  `SPEC-001` wired: `fmt --check`, `clippy -D warnings`, `test`,
  `deny check licenses`, an MSRV (1.90.0) check, and the lint-policy red-proof
  (`oracle-must-be-shown-red` applied to the gate — `DEC-009`, which supersedes
  `DEC-007`, which superseded `DEC-006`). A short fuzz
  smoke run is **not yet wired** — per §12 bar 2 it lands with the first
  parser spec (`SPEC-003`), not retrofitted.

### Measured toolchain — 2026-08-15/16, this machine only

> The fuller version, including the oracle tooling and the corpus location, is
> `guidance/toolchain-brief.md` — inject **that** into build handoffs.

Verified by running the commands, not assumed. Re-verify on any other host;
these are host facts, not repo facts. This is the short version of what
`guidance/toolchain-brief.md` is for.

| Fact | Value |
|---|---|
| `rustc` / `cargo` on `PATH` | **Homebrew 1.97.1**, from `/opt/homebrew/bin` |
| Why | `/opt/homebrew/bin` precedes `~/.cargo/bin` on `PATH` |
| rustup default toolchain | **nightly** — `cargo 1.99.0-nightly` |
| rustup `+stable` | `cargo 1.97.0` (**not** the same build as Homebrew's 1.97.1) |
| `cargo +nightly …` | **FAILS**: `error: no such command: +nightly` |
| `dnglab` | 0.7.2 (Homebrew, bottled arm64) |
| `exiftool` | 13.55 |

⚠ **The `+toolchain` trap.** The `cargo` that resolves on `PATH` is Homebrew's
real cargo, not a rustup shim, so it does not understand `+nightly`.
`cargo fuzz` needs nightly. Invoke it through the shim explicitly:

```bash
~/.cargo/bin/cargo +nightly fuzz run <target>
```

**`Cargo.toml` and `src/` now exist** (`SPEC-001`, 2026-08-18): `edition =
"2021"`, `rust-version = "1.90"` (measured, not guessed — still the oldest
toolchain installed; the true floor remains unmeasured), and the Rust CI jobs
above are wired.

---

## 6. Commands (exact)

**These run.** `SPEC-001` (2026-08-18) filled `app.just`'s stubs to match the
block below — `just build` / `just test` / `just lint` / `just typecheck` /
`just deny` / `just lint-red-proof`, plus `just install` / `just dev`;
`SPEC-006` added `just lint-no-allow`; `SPEC-003` added `just fuzz` /
`just fuzz-seeds`, and its punch-list round added `just deny-fuzz` and
`just msrv`; `SPEC-012` added `just fuzz-plane` (the sensor-plane unpacker's
own fuzz target — `just fuzz-seeds` now regenerates both targets' seeds);
`SPEC-014` added `just fuzz-develop` (levels/geometry's own fuzz target —
`just fuzz-seeds` now regenerates all three targets' seeds).
Every recipe's commands appear in the block below and nothing in
the block is unrunnable: that correspondence is acceptance criterion 8, so a
recipe that gains a command gains a line here in the same change.

App commands belong in **`app.just`** (project-owned, imported by the
template-managed root `justfile`) so a template update never clobbers them. For
template/workflow commands (`status`, `new-spec`, …) see `justfile`.

```bash
# install    — nothing to install; the toolchain is the dependency
cargo fetch

# dev        — no dev server; this is a library. Watch-build instead:
cargo build

# test       — the whole suite (tier-B tests skip loudly when the corpus is absent)
cargo test --all-features

# test one   — a single test by name
cargo test --all-features <test_name> -- --exact --nocapture

# oracle-meta — the live metadata oracle only (SPEC-005): exiftool/dnglab
#              cross-checks plus the red-proof. Tier-B half needs
#              $IRRADIANCE_CORPUS_DIR and the tools on PATH; both skip loudly,
#              naming what's missing, when absent. Tier-A red-proof needs
#              neither and is the only half CI runs.
cargo test --all-features --test metadata_oracle

# lint       — BOTH halves; `just lint` runs them in this order
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check

# lint-red-proof — proves the panic-free lint policy actually rejects a
#              violating function injected into a copy of src/lib.rs, with the
#              UNMUTATED copy run first as a negative control (DEC-009)
./scripts/lint-red-proof.sh

# lint-no-allow  — closes what the red-proof structurally cannot see: an
#              #[allow] of a policy lint BENEATH the crate root. `-F` is
#              `--forbid`, so re-allowing one is compiler error E0453 rather
#              than a silenceable warning. Scope is `--lib` on purpose — it
#              excludes #[cfg(test)] and src/bin/irr.rs, the sanctioned
#              exceptions (SPEC-006).
cargo clippy --lib --quiet -- \
    -F clippy::unwrap_used -F clippy::expect_used -F clippy::indexing_slicing \
    -F clippy::panic -F clippy::arithmetic_side_effects

# lint-ci    — clippy AS CI SEES IT, and the only local command that can.
#              `cargo clippy` on PATH is Homebrew's 0.1.97; CI uses
#              dtolnay/rust-toolchain@stable, which FLOATS — 0.1.98 today.
#              Under `-D warnings` every lint clippy ADDS is a new CI failure
#              on code that never changed, and `just lint` cannot see it.
#              That gap accounts for 14 of the 17 CONSECUTIVE red CI runs
#              PATCH-001 found (2026-08-20 → 2026-08-22, six shipped specs),
#              while every verify in that window reported "ten gates green" —
#              locally. ⚠ The other three, and the OLDER defect, were a
#              red-proof that had never once run successfully in CI.
#              ⚠ The PATH= prefix is the FOURTH `+toolchain` trap: bare
#              `~/.cargo/bin/cargo +stable clippy` still reports 0.1.97,
#              because the OUTER command goes through the shim but
#              `clippy-driver` is then found on PATH. Measured 2026-08-22.
#              RUN THIS BEFORE EVERY PUSH.
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +stable clippy \
    --all-targets --all-features -- -D warnings

# typecheck  — Rust has no separate typecheck; `check` is the fast path
cargo check --all-targets --all-features

# build      — release artifact
cargo build --release

# licences   — the permissive-only gate (constraint no-copyleft-dependencies).
#              TWO invocations, and BOTH are required: this repo has two cargo
#              graphs and cargo-deny evaluates one manifest's graph per run.
#              The first covers the library; it does not see a single crate
#              under fuzz/, which DEC-011 keeps outside the library's graph on
#              purpose. From SPEC-003's build until its verify cycle the second
#              was believed impossible and a hand-written table in DEC-011 stood
#              in for it — the table was wrong, and the command was one flag
#              away. Running only the first and reporting "licences green" is a
#              green that checked nothing about fuzz/.
cargo deny check licenses
cargo deny --manifest-path fuzz/Cargo.toml check licenses

# msrv       — compile the whole target set against EXACTLY the pinned 1.90.0.
#              ⚠ The `~/.cargo/bin/` prefix is the THIRD instance of the
#              `+toolchain` trap: a bare `cargo +1.90.0 check` fails with
#              `error: no such command: +1.90.0`, because `cargo` on PATH is
#              Homebrew's real cargo and it does not understand `+toolchain`
#              syntax at all. Only the rustup shim does. Unlike the fuzz trap,
#              no PATH= prefix is needed — nothing shells out to an inner
#              `cargo` here. Measured 2026-08-20 by two agents in succession,
#              each losing a loop to it, because until then this was the one
#              gate with no `just` recipe hiding the fix.
~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features

# fuzz       — the rustup shim must be FIRST ON PATH, not merely invoked: see
#              guidance/toolchain-brief.md, "The SECOND `+toolchain` trap".
#              cargo fuzz shells out to a bare `"cargo" "build"`, and that
#              INNER call is what resolves to Homebrew's stable cargo and
#              rejects -Zsanitizer. `~/.cargo/bin/cargo +nightly fuzz run`
#              alone therefore still fails. Measured 2026-08-18 and again
#              2026-08-20 (SPEC-003).
#
#              `fuzz/seeds/ifd` is the committed hand-built seed set;
#              `fuzz/corpus/ifd` is libFuzzer's own, and is gitignored.
mkdir -p fuzz/corpus/ifd
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
    fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60

# fuzz-plane — the sensor-plane unpacker's fuzz target (SPEC-012). Same
#              +toolchain trap and seed/corpus split as `fuzz` above.
mkdir -p fuzz/corpus/plane
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run plane \
    fuzz/corpus/plane fuzz/seeds/plane -- -max_total_time=60

# fuzz-develop — levels normalization and ActiveArea -> DefaultCrop ->
#              Orientation's own fuzz target (SPEC-014): attacker-controlled
#              crop origin, crop size, ActiveArea and orientation. Same
#              +toolchain trap and seed/corpus split as `fuzz` above.
mkdir -p fuzz/corpus/develop
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run develop \
    fuzz/corpus/develop fuzz/seeds/develop -- -max_total_time=60

# fuzz-seeds — regenerate the committed seed corpus from tests/support/tiff.rs
#              (ifd target) and examples/fuzz-seeds.rs's own fixtures (plane
#              and develop targets)
cargo run --quiet --all-features --example fuzz-seeds
```

Oracle commands (run as **tools**, never linked) live in
`docs/oracle-contract.md`. The one to memorise:

```bash
dnglab analyze --raw-checksum <file>.DNG
```

---

## 7. Directory Structure

Actual layout as of 2026-08-15, updated 2026-08-18 for `SPEC-001` (`Cargo.toml`,
`src/`). `fuzz/` and the `tests/corpus/` tier subdirectories remain **planned,
not present** — marked below. `tests/` holds no `.rs` file: the lint-policy
red-proof injects into a temp-dir copy of `src/lib.rs` rather than shipping a
snippet (`DEC-009`).

```
/
├── AGENTS.md                          # This file
├── CLAUDE.md                          # Pointer to AGENTS.md
├── README.md                          # Human-facing readme
├── LICENSE-MIT / LICENSE-APACHE       # Dual-licensed: MIT OR Apache-2.0
├── GETTING_STARTED.md                 # First-project walkthrough
├── FIRST_SESSION_PROMPTS.md           # Phase prompts
├── .repo-context.yaml                 # Repo (app) metadata
├── .variant                           # "claude-plus-agents"
├── VERSION                            # TEMPLATE provenance (0.6.38), NOT the app version
├── justfile                           # Template-managed: just status, new-spec, … (imports app.just)
├── app.just                           # Project-owned: just build/test/lint/deny — see §6
├── scripts/                           # Shell scripts powering justfile
├── docs/
│   ├── oracle-contract.md             # ⚑ The three oracle layers + the VERIFIED plane contract
│   ├── measured-q2m-dng.md            # ⚑ One real Q2M file, read with exiftool
│   ├── conformance-matrix.md          # ⚑ Camera coverage + the two-tier corpus policy
│   ├── provenance-ledger.md           # ⚑ Every algorithm's source and licence
│   ├── license-policy.md              # cargo-deny wiring
│   ├── architecture.md                # ⚠ still a template stub (waits on SPIKE-001)
│   └── decisions/                     # the TEMPLATE's own DEC-001…013 (see §10)
├── guidance/                          # Repo-level rules (across all projects)
│   ├── constraints.yaml               # ⚑ five blocking constraints — this file wins
│   ├── questions.yaml
│   ├── toolchain-brief.md             # ⚑ measured facts for cold build agents (DEC-004 r5)
│   └── signals.yaml                   # Typed feedback ledger (see docs/signals.md)
├── decisions/                         # THIS repo's DEC-* (across all projects)
├── feedback/                          # Inbound feedback, incl. template-level findings (DEC-000)
├── reports/                           # Daily + weekly report outputs
├── spikes/                            # Repo-level bounded explorations
│   └── SPIKE-001-…-oracle-discriminate.md
├── projects/
│   ├── _templates/                    # Shared templates (spec, stage, handoff, spike, patch, …)
│   └── PROJ-001-monochrome-dng-develop/
│       ├── brief.md
│       ├── stages/                    # STAGE-001 … STAGE-004
│       ├── specs/                     # SPEC-001 … SPEC-005 (STAGE-001, framed)
│       │   └── done/
│       └── handoffs/
├── Cargo.toml                         # edition 2021, rust-version 1.90, [lib] + [[bin]] irr
├── deny.toml                          # cargo-deny permissive-only allow-list
├── src/
│   ├── lib.rs                         #   the public API — bytes in, pixels + metadata out (no decode yet)
│   └── bin/irr.rs                     #   internal dev/oracle binary; NOT a product surface
├── tests/
│   └── corpus/
│       ├── manifest.toml
│       ├── tier-a/                    #   ← PLANNED — committed, licence-clean, runs in CI
│       └── tier-b/                    #   ← PLANNED — NEVER committed — .gitignore'd; skip loudly when absent
└── fuzz/                              # ← PLANNED — one target per parser, from the first parser spec
```

⚑ = written by the planning session that produced this repo; treat its numbers
as evidence with provenance and re-verify before relying on them.

---

## 8. Cycle Model

Every spec moves through five cycles. **Cycles are tags, not gates** — edit any artifact anytime. The word "cycle" names what a spec goes through on its way to shipping.

| Cycle | Purpose | Who |
|---|---|---|
| **frame** | Go/no-go on the spec | Human + Claude (1 min) |
| **design** | Spec + failing tests + handoff | Claude (architect) |
| **build** | Make failing tests pass | Implementer agent |
| **verify** | Review + validation | Claude (reviewer) |
| **ship** | Merge, deploy, reflect, archive | Human + light agent |

Valid transitions:
```
frame → design → build → verify → ship
                   ↑       │
                   └───────┘ (verify sends back on punch list)
```

**`frame` is optional for a single spec — most start at `design`.** By the time a
task reaches `just new-spec` it has usually already passed go/no-go at the
stage/backlog level, so `frame` is redundant (across the dogfood it went unused —
0 of 100+ specs). Use it for one spec only when that spec's very existence is
genuinely in question; otherwise begin at `design`.

**`frame` earns its keep in BATCH, though — that's what `just frame-stage`
is for.** It promotes every `- [ ] (not yet written)` line in a stage's
`## Spec Backlog` into a real spec at `cycle: frame`, so each one has a **stable
ID** a sibling can point `depends_on:` at. That turns a planned stage into a
dependency-aware batch you can fan out (`just ready`) instead of a prose list.

The fidelity line matters: an **outline captures SCOPE and DEPENDENCIES, not
APPROACH**. Design stays just-in-time — a stage framed as ten pre-designed specs
is ten guesses that go stale before you reach spec four. Fill an outline's
`## Context` / `## Goal` / `depends_on:` and leave the rest scaffolded until it
advances to `design`. At stage close, record **how many outlines survived
unchanged** (Stage-Level Reflection) — that's how you learn whether framing this
far ahead pays for your work, rather than assuming it does.

> **Repo-specific: do not `just frame-stage STAGE-001` before SPIKE-001 lands.**
> Every stage in PROJ-001 is designed test-first against facts the spike is
> supposed to *establish* — the corpus, the verified oracle contract, measured
> LOC replacing the stage-file estimates. Framing first would be framing ahead of
> what is knowable, and the outlines would go stale before the first one reaches
> design. SPIKE-001 also lists 11 open questions whose answers change the spec
> breakdown.

Projects and stages have lighter lifecycles (not full cycles):

- **Project status:** `proposed | active | on_hold | shipped | cancelled` — the
  **coarse, machine-keyed** lifecycle state tooling branches on. Keep it coarse.
- **Project `activity`** (optional): a **human-facing** refinement of the work
  happening *within* an `active` project — `requirements | design | build | test |
  blocked` (a suggested **open** set; extend it, e.g. `spike`). It says *what kind
  of work is going on now* without abusing `status` or making the project look
  stalled. PROJ-001 currently sits at `status: proposed` /
  `activity: requirements`; it moves to `activity: spike` when SPIKE-001 starts.
  `validate` warns on an unrecognized value but never fails.
- **Stage status:** `proposed | active | shipped | cancelled | on_hold`

A stage is `active` when its first spec enters design. `shipped` when
its spec backlog is complete AND the stage-level reflection is written.

### The patch lane (lightweight fixes — DEC-003)

A **patch** is a bounded fix to *already-shipped* behavior (a bug or UX papercut)
that adds **no new feature/command** and doesn't warrant a full spec + stage. It
runs a collapsed **`patch → verify → ship`** cycle instead of a spec's five:

- **patch** — design + build fused into one test-first pass (write the failing
  test *and* the fix together).
- **verify** — **kept, and kept independent** (a separate agent from the patch
  author). This is the one discipline the dogfood retrospective proved catches
  real defects; it is non-negotiable.
- **ship** — CHANGELOG `[Unreleased] → Fixed` + `just archive-patch`. **No stage
  bookkeeping** — a patch attaches to the project, not a stage.

**Stays:** the full gate suite, a `DEC-*` when there's a real decision, and
index-verify-before-ship. **Sheds:** the separate frame + design cycles and the
stage backlog/`Count:` bookkeeping. **Guardrail:** if a change adds a
command/flag or needs its own design exploration, it's a **spec, not a patch**.

In this repo the guardrail has a sharper edge: **a fix that changes decoded pixel
output is never a patch.** It changes what every oracle compares against and
what every downstream consumer renders. That is a spec, with its own red-proof.

Mechanics: `just new-patch "title" [PROJ-NNN]` scaffolds
`projects/PROJ-*/patches/PATCH-NNN-<slug>.md` (its own repo-wide `PATCH-*`
sequence). Patches are first-class in `just validate`, `just cost-audit`
(metered on `patch`+`verify`), and `just status`. `just archive-patch PATCH-NNN`
files it under `patches/done/`.

### Delegated cycles: the handoff / handback contract

**One handoff per delegated CYCLE.** With build and verify on different agents
you get two per spec — `handoff.cycle` distinguishes them, and `to_agent` comes
from `.repo-context.yaml` → `spec.agent.tier_map.<cycle>` (DEC-005).

```bash
just new-handoff SPEC-042 build     # → HANDOFF-NNN, to_agent = tier_map.build
just new-handoff SPEC-042 verify    # → HANDOFF-MMM, to_agent = tier_map.verify
```

**The handback is the return path, and it is mandatory.** The executing agent
fills the `handback:` front-matter block before reporting done — **including a
real `tokens_total`**. This is not a courtesy: build and verify are the *metered*
cycles the cost gate requires, and the orchestrator has no meter for an agent it
doesn't host. The agent that ran the cycle is the only party that knows the
number.

```bash
just handback-sync SPEC-042         # transcribe reported cost → cost.sessions
```

`handback-sync` is idempotent (it stamps `synced_at`) and **exits 1 if any
handoff hasn't handed back cleanly**, naming which one and why. The orchestrator
**never estimates a delegated cycle's cost** — an invented number is worse than
no number, because it looks real in every downstream rollup.

If a platform genuinely exposes no token count, say so once in
`.repo-context.yaml` → `cost.metering_source: none` (which disables the gate,
DEC-005) rather than guessing per-spec. Full contract: DEC-013.

### The spike lane (bounded exploration — DEC-012)

A **spike** is the phase *before* you know the shape: a bounded exploration whose
job is to produce information, not shipped behavior. It runs a collapsed
**`spike → land`** cycle. Two modes, one discipline:

| `spike.mode` | Is | Code is | Lands as |
|---|---|---|---|
| **`question`** | A timeboxed investigation | Evidence | `answered` / `inconclusive` |
| **`build`** | A **vibe-coding session** | The deliverable | `graduated` / `discarded` |

- **spike** — explore. **No spec, no failing tests, no `DEC-*` required.** This is
  the one place in the repo with no conventions to follow; the speed *is* the
  value. `test-before-implementation` does **not** apply during a spike.
- **land** — **mandatory.** Answer the question, emit `DEC-*` for the choices the
  exploration already made, and decide the code's fate.

**There is deliberately no `verify` step.** A patch keeps its independent verify
because it fixes *known* behavior against a *known* expectation. A spike has
neither acceptance criteria nor a spec, so a verify here would have nothing to
check and would degrade into theater — which would erode the real verify in the
other two lanes. The **timebox** and the **mandatory land step** replace it.

**Required from creation:** `spike.question` (one sentence — a spike with no
question is just coding; loose is fine for `mode: build`, absent is not) and
`spike.timebox`. **Hitting the timebox without an answer is `inconclusive`, which
is a real result** — not a reason to extend. Extending twice means it isn't a
spike, it's an unframed project.

**Guardrails:** a spike may not ship user-facing behavior (that's a spec, or a
patch for shipped behavior); its code may not be built upon before it lands; and
it is not a way to skip the cycle on work you already understand — if you can
write acceptance criteria, you have a spec.

> **Repo-specific: SPIKE-001's code is never merged.** The spike states it
> explicitly, and `test-before-implementation` is why: retro-fitting tests to
> existing decoder code produces tests that cannot fail — the same failure mode
> as an oracle that cannot go red. **What lands from SPIKE-001 is the corpus, the
> oracle harness proven red, the measured answers, and the DECs** — not the
> decoder. Its `mode` is `question` for exactly that reason.

**Graduating a `build` spike** (the vibe-coding → real-work conversion) writes
five things: `.repo-context.yaml`, `AGENTS.md`, `guidance/toolchain-brief.md`
(most valuable here — a spike generates exactly this friction), retroactive
`DEC-*` for **load-bearing choices only** with honest confidence, and a project
brief framed around **what comes next** (the spike is prior art in
`Dependencies → Depends on`). **Do NOT retro-write specs for code that already
works** — a spec directs work that hasn't happened yet.

Mechanics: `just new-spike "the question" [TIMEBOX] [MODE] [PROJ-NNN]` scaffolds
`spikes/SPIKE-NNN-<slug>.md` at the **repo root** — not under a project, because
a spike may precede any project (its own repo-wide `SPIKE-*` sequence;
`project.id` is optional and back-linked at land). `just validate` requires a
question, a timebox, and — on a spike at `cycle: land` — a real `spike.outcome`;
`just archive-spike SPIKE-NNN` refuses an un-landed spike and files it under
`spikes/done/`. `just cost-audit` does **not** gate spikes (cost is advisory).
A project exploring before it frames anything can set `project.activity: spike`.

---

## 9. Instruction Timeline

Every spec has a timeline file at
`projects/*/specs/SPEC-NNN-<slug>-timeline.md` listing cycle
instructions in order with status markers.

Status markers:

- `[ ]` not started — no one has picked this up yet
- `[~]` in progress — an executor is currently running this
- `[x]` complete — cycle finished; see the prompt file for what was run
- `[?]` blocked — needs a human decision or external unblock before
  proceeding. Include a one-line reason after the marker.

Cycle prompts live at `projects/*/specs/prompts/SPEC-NNN-<cycle>.md`.
The architect writes them; executors (the implementer agent for
build, Claude again for verify) read and run them.

**Discipline for executors:**

- When you start a cycle, mark it `[~]`.
- When you finish, mark it `[x]` with a one-line result (PR number,
  cost, completion date).
- If you hit a real blocker — constraint ambiguous, dependency
  missing, verify surfaced something needing architect judgment —
  mark `[?]` with a one-line reason. Do NOT use `[?]` as a "I don't
  know what to do" dumping ground. Blocked means the next move
  requires someone else; everything else is in-progress or a
  question to resolve in the current session.

**In this repo, a missing tier-B corpus file is `[?]`, not a silent pass.**
A skipped test must be visible (`docs/conformance-matrix.md`). An executor that
finds the corpus absent marks the cycle blocked with that reason rather than
reporting green on a suite that never ran the thing it claims to verify.

This is a convention, not a mechanism. No tooling enforces it; the
discipline lives in the prompt set. Skip it and nothing breaks, but
you lose the history artifact and the next executor has to hunt for
the right prompt.

---

## 10. Cross-Reference Rules

Every spec has these relationships, encoded in front-matter:

- `project.id` → the project it belongs to (e.g., `PROJ-001`)
- `project.stage` → the stage within that project (e.g., `STAGE-002`)
- `references.decisions` → DEC-* it was designed against
- `references.constraints` → constraints that apply
- `handoff.from_agent` / `handoff.to_agent` → roles in the delegation

When a spec references a DEC, the DEC does not reciprocally list the
spec. DECs are stable repo-level records; specs come and go.

### Two DEC namespaces — do not confuse them

- **`/decisions/`** — **this repo's** decisions. `DEC-000` onward. This is the
  one you write to, the one `just decisions-audit` lints, and the one
  `references.decisions` points at.
- **`/docs/decisions/`** — the **template's own** DEC-001…DEC-013 (interface
  contract, cost convention, patch lane, sub-agent execution, spike lane, …).
  Read-only background that explains *why the process is shaped this way*. When
  this file cites `DEC-003`/`DEC-004`/`DEC-012`/`DEC-013` in the process
  sections above, it means **these**.

The two namespaces collide by ID string, which is a real hazard when a spec
writes `references.decisions: [DEC-001]`. **Always disambiguate by path**, and
prefer citing this repo's decisions by their full filename.

### Every algorithm carries a provenance row

A spec that adds a decoder, an algorithm, or a numeric kernel is **not done
until `docs/provenance-ledger.md` has its row** — module, source, source
licence, provenance class (1–5), notes. This is the blocking constraint
`provenance-recorded-per-algorithm`, and it is what makes this library's
permissive claim defensible rather than asserted. `cargo deny` cannot see
provenance; only the ledger can.

---

## 11. Coding Conventions

- **Naming:** standard Rust — `snake_case` items, `CamelCase` types,
  `SCREAMING_SNAKE_CASE` consts. Prefer domain names from the DNG/TIFF
  specification over invented ones (`active_area`, `default_crop_origin`,
  `black_level`), so a reader can grep the spec. See §14.
- **File organization:** one module per concept, mirroring the pipeline stages —
  container/IFD reading, tag model, plane decode, opcodes, output. A module that
  needs two sentences to describe is two modules.
- **Imports:** no glob imports (`use foo::*`) outside test modules. Group std /
  external / crate-local, in that order — rustfmt's default.
- **Error handling:** **every fallible path returns a typed error.** No
  `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`, slice
  indexing that can go out of range, or arithmetic that can overflow in release,
  on **any parse or decode path** — this is the blocking constraint
  `no-panics-on-untrusted-input`, and it is why `.get()`, `checked_*`, and
  `try_into()` are the defaults here. Recursion (SubIFD walks) and chaining are
  depth- and cycle-guarded. `unwrap()` in `#[cfg(test)]` and in `src/bin/irr.rs`
  is fine; those are not library paths.
  Deny it mechanically, not by review alone — `#![deny(clippy::unwrap_used,
  clippy::expect_used, clippy::indexing_slicing, clippy::panic,
  clippy::arithmetic_side_effects)]` on the library, allowed in test modules.
  **Verified 2026-08-15 on clippy 0.1.97**: all five names are valid (no
  unknown-lint warning), and each *fires as an error* on a violating function —
  `v[0]`, `v.first().unwrap()` and `a + b` were each rejected in a scratch crate.
  Shape-checked and behavior-checked (§12).
- **Logging:** **none.** A library that logs imposes a logging framework on every
  consumer. Errors carry their context in the error type; diagnostics that only
  matter during development belong in `src/bin/irr.rs`.
- **`unsafe`:** `#![forbid(unsafe_code)]`. Lifting it needs its own `DEC-*` with
  a measured justification, not a performance hunch.
- **Comments:** Explain *why*, not *what*. In this repo the highest-value comment
  is a **spec citation** — the DNG/TIFF section, tag number, or paper that
  licenses the line of code. That comment is also the provenance evidence.
- **No dead code.** Delete, don't comment out.
- **Ship the reader with the field.** A field, column, flag, event or
  namespace lands **in the same change as the thing that reads it** — or it
  doesn't land. No "we'll wire it up later."

  This is the most reliably recurring defect this template has produced
  (N=5 across the dogfood: a stage cost slot nothing summed, a defect-stage
  question nothing counted, a decision `status` nothing rendered, a
  provenance namespace with zero readers, and a `value_link` that was filled
  in everywhere but never actually linked to anything).

  The failure is quiet, which is why it keeps happening: **an unread field
  looks identical to a field whose data hasn't arrived yet.** Nobody fills in
  a field that nothing displays, so it stays empty; and because it is empty,
  nobody notices it is also unread. By the time you want the data, the window
  to have collected it has closed — and unlike most defects, you cannot fix
  this one retroactively. There is no back-filling a year of measurements.

  The cheap version of the reader counts as a reader: a line in an existing
  view, one number in a rollup, a `--json` key someone can grep. It does not
  have to be a dashboard. It has to *surface*.

  **Delegating makes this worse, not better:** a handoff that adds a field is
  the easiest place to lose the reader, because the implementer ships the
  schema and the orchestrator assumes the surfacing came with it. Name the
  reader in the handoff's acceptance criteria.

  The decoder form of this rule: **a parsed tag with no consumer is not
  parsed** — either something reads `ActiveArea`, or it doesn't get a field.
- **Diagrams:** author them as Mermaid fenced blocks in markdown
  (`/docs/`, `/decisions/`, specs) so they render on GitHub and you can
  keep them current as part of the work. Update the relevant diagram in
  the same change, not afterward. See `/guidance/recommended-tools.md`.

---

## 12. Testing Conventions

- Every new function gets at least one test.
- **Test file naming:** unit tests in a `#[cfg(test)] mod tests` at the bottom of
  the module they test; integration tests in `tests/<area>.rs` (e.g.
  `tests/ifd_reader.rs`, `tests/plane_oracle.rs`). Fuzz targets in
  `fuzz/fuzz_targets/<parser>.rs`.
- **Coverage expectations:** no percentage target — a coverage number on a
  decoder is noise, because the interesting inputs are malformed ones a line
  counter cannot see. The real bar is the four below.
- Must test: happy path, error cases, edge cases from acceptance criteria.
- Need not test: third-party internals, `std`, the Rust compiler.
- **TDD:** Tests live in the spec's `## Failing Tests` section, written
  during **design**, made to pass during **build**.
- **Exception — the spike lane.** `test-before-implementation` does NOT apply
  during a `cycle: spike` exploration (DEC-012): a spike has no acceptance
  criteria to test against, and the speed is the point. It applies again the
  moment anything graduates into a spec.

### The four bars that actually gate this repo

**1. Every oracle ships proven red.** Blocking constraint
`oracle-must-be-shown-red`. Each oracle layer ships with a deliberate-fault
test — a corrupted tag, an injected off-by-one in the bit unpacker, a wrong
black level — and that fault must turn the oracle **red**. If it doesn't, the
oracle is not wired to what it claims to check, and the green it produces is
manufactured confidence. The red-proof is part of the spec's `## Failing Tests`,
not a follow-up. The three layers and their exact commands are in
`docs/oracle-contract.md`; the plane contract there was **verified 2026-08-15,
not assumed**, and three wrong guesses preceded it — do not re-derive them.

**2. Fuzz targets arrive with the first parser spec, not retrofitted.** A parser
spec that adds a new input surface adds its fuzz target in the same change and
**runs it** before the spec ships. A retrofitted fuzz target tests the shape the
code already has; a designed-in one tests the shape the input can take. This is
how `no-panics-on-untrusted-input` is enforced mechanically rather than by
review. Corpus seeds come from tier A, including the deliberately truncated and
malformed fixtures. Remember the `+toolchain` trap in §5:
`~/.cargo/bin/cargo +nightly fuzz run <target>`.

**3. Layer-0 assertions are free — assert them.** The packing arithmetic must
reproduce `StripByteCounts` exactly (`docs/measured-q2m-dng.md`:
8424 × 5632 × 14 bits = 83,026,944 bytes). That check needs no oracle tooling,
no corpus, and no network. Any invariant with that property belongs in the code
as an assertion on a typed error path, not in a test that might not run.

**4. A skipped test must be loud.** Tier-B corpus files are never committed
(30–60 MB, and copyrighted by whoever shot them). Tests that need them **skip
with a clear message naming the missing file**, and run where the corpus
exists. A silent skip is a test that reports green for work it never did — the
same defect class as an oracle that cannot go red.

### Design-time disciplines (carried from the template, and load-bearing here)

- **Behavioral pre-flight (design-time).** When a spec's literal/artifact makes a
  claim about *runtime behavior* — a component registers, a hook fires, a binary
  resolves on PATH, a server answers, a config is actually loaded — exercise that
  behavior through the surface that **runs** it before declaring design done, not
  merely the surface that **validates its shape**. A manifest that passes
  `validate --strict` can still register nothing; a completion script that lints
  can still emit the wrong marker; a config that parses can still not take effect.
  Shape-check and behavior-check are *different checks* — neither substitutes for
  the other. The defect class that escapes design→build→verify is disproportionately
  operational/runtime, not spec-logic; this is where to catch it.
  **The `cargo +nightly` failure in §5 is exactly this defect, caught early:**
  the tool was installed, the command was plausible, and it did not run.
- **Design-time probe / measure-before-build (design-time).** When a spec's
  implementation depends on the *actual* behavior of a load-bearing external — a
  library's real API signature, a tool resolving on the **pinned** toolchain, the
  true version floor, a config field the engine actually reads — or when it
  **tunes toward a measurable target**, probe or measure the real thing **against
  the real pinned tree during design**, and record the verified facts (the exact
  calls, the baseline number) in the spec's `## Implementation Context` / the
  handoff (or the governing `DEC-*`). Two recurring moves: (1) **probe the real
  API/tool** — don't trust the model's prior; the pinned version's signature may
  differ (a wrong assumed call is then caught at design, not mid-build); (2)
  **measure the baseline now** so the target and the change are grounded in
  numbers, not guesses. When you do, build collapses to a near bit-for-bit
  *transcription* instead of a discovery loop — the strongest efficiency lesson
  from the dogfood (recurring across projects, highest single-lesson frequency).
  In this repo the probe target is usually a **byte layout**: read the real
  file's bytes and close the arithmetic before writing the unpacker.
  Complementary verify move — **adversarial mutation:** revert the change and
  confirm the guard *fails*; it both proves the test has teeth and surfaces
  dead/no-op config (a field the engine never reads). The oracle red-proof (bar 1
  above) is this move promoted to a blocking constraint.

---

## 13. Git and PR Conventions

- **Branch:** `feat/spec-NNN-<slug>`, `fix/spec-NNN-<slug>`, `chore/<slug>`.
  Spikes: `spike/NNN-<slug>` — **never merged** (see §8).
- **One spec per branch, one PR per branch.** Constraint `one-spec-per-pr`.
- **Commits:** [Conventional Commits](https://www.conventionalcommits.org/) —
  `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `perf:`, `chore:`. Scope with
  the module where it helps (`feat(ifd): …`). The existing history follows this
  (`chore: scaffold claude-plus-agents; frame PROJ-001 and SPIKE-001`).
  A commit that changes decoded pixel output says so in the subject.
- **Never commit a RAW file.** `.gitignore` blocks `tests/corpus/tier-b/` and
  every common RAW extension. If a fixture must be committed it goes in tier A
  and must be licence-clean — `dnglab makedng` output or a hand-built header.
  Check `git status` before `git add -A`; a 60 MB blob is not removable from
  history by deleting it in the next commit.
- **PR description must include:**
  - Project: `PROJ-NNN`
  - Stage: `STAGE-NNN`
  - Spec: `SPEC-NNN`
  - Handoff: `HANDOFF-NNN`
  - Decisions referenced: `DEC-NNN, DEC-MMM` (say which namespace — §10)
  - Constraints checked: `[list]`
  - New `DEC-*` files created during build
  - **Provenance:** the ledger rows added, or "none — no new algorithm"
  - **Oracle:** which layer this touches, and the red-proof that accompanies it

**One git worktree per concurrent session.** This variant routinely has
two agents in flight (architect and implementer). If more than one session
touches this repo at once, each MUST run in its own `git worktree`, not the
shared checkout — two agents writing one working tree corrupt each other
(a parallel build can clobber an uncommitted edit, or a commit can land on
the wrong branch). `git worktree add <path> <branch>`, work there, commit +
push, then `git worktree remove`. Always check `git branch --show-current`
before any commit.

### Delegated execution (sub-agents) — DEC-004

This variant delegates build/verify to a separate implementer/reviewer agent via
`HANDOFF-*`. Five rules keep that delegation honest:

1. **Reconcile over self-report — never flip `handoff.status` to `completed` (or
   advance `task.cycle`) on the sub-agent's word alone.** After it reports, verify
   the *claimed* result against actual **git + disk** state:
   - `git log <base>..HEAD` and `git ls-remote origin <branch>` — are the commits
     actually there (locally *and* pushed)?
   - the spec's `## Failing Tests` files exist on disk, and the gate actually ran?

   Trust git/disk over **any** agent self-report or timeline marker — both lie (a
   truncated report can claim "done" with the commit or push missing; agents have
   reported "pushed" while `origin` was still at the prior SHA). **If the sub-agent
   dies mid-cycle:** reconcile the partial output, finish the *mechanical remainder*
   in the coordinator loop (don't re-run the whole cycle), and attribute cost to
   the sub-agent's metered portion (`subagent_tokens`), recording the coordinator
   finish as a separate null-with-note cost session.

   **The general form: verify any claim you are about to act on, against the
   thing it describes.** Build output is only the most obvious case. A number in
   a roadmap, a count in a harvest, a "we already tried that" in a decision
   record — each is a self-report by a past session, and each rots silently.
   Prefer the source: run the count, read the corpus, check the remote. This is
   *cheap* and it is *mechanical* — it needs no judgement, only the habit of
   looking before acting. In this variant the surface is larger, because a
   delegated agent's handback is a claim too.

   **The docs in `/docs/` are self-reports too.** `measured-q2m-dng.md`,
   `oracle-contract.md` and `conformance-matrix.md` were written by a session
   that could not run this code, against **one file from one camera on one
   firmware**. Treat their numbers as evidence with provenance and re-verify
   before relying on them. If one is wrong, say so and fix it — that is the
   expected outcome, not a failure.
2. **One sub-agent at a time; no interleaved tree ops.** Launch exactly one
   build/verify sub-agent, then do **no** git/tree operations in the shared
   checkout — no `new-spec`, `checkout`, or commits, and don't design the next
   spec — until it reports complete and its branch is merged. The structural fix
   is per-agent `git worktree` isolation (the worktree habit above).
3. **Set the sub-agent's model explicitly** from `.repo-context.yaml`
   `spec.agent.tier_map` (design/build/verify) — don't rely on a default (a silent
   Opus default is a ~6× cost surprise). `new-spec`/`new-patch` stamp `agents.*` /
   `handoff.from_agent` from it (DEC-005).
4. **Sanction a trivial dev-dep + its DEC in one build pass.** The implementer
   can't stop-and-ask mid-run, so the `no-new-top-level-deps-without-decision`
   constraint carves out an exception: a build cycle MAY add a clearly-trivial
   **DEV-only** dependency and author its DEC in the same pass.
   **This repo narrows the exception:** the dev-dep must still pass
   `no-copyleft-dependencies` (permissive licence) **and** must not be a RAW
   decoder. `rawler`, `rawloader`, `zenraw`, `quickraw`, `imagepipe` and
   `demosaic` are never sanctioned by this rule — they need their own decision,
   and the answer is expected to be no. When in doubt, stop and ask; a wrong
   dependency here is not a workaround review can undo.
5. **Inject the toolchain brief into the handoff / implementer prompt.** A cold
   implementer re-imports generic tool-priors and wastes loops rediscovering this
   repo's specifics. Give it `/guidance/toolchain-brief.md` so it doesn't.
   ⚠ **That file is still the template stub.** Until it is filled, inject §5's
   *Measured toolchain* table instead — especially the `cargo +nightly` trap,
   which will otherwise cost every fuzz-touching build cycle a loop.

---

## 14. Domain Glossary

Terms are the DNG/TIFF specification's own wherever possible — prefer these over
invented names in code (§11).

- **RAW** — unprocessed sensor readout plus the metadata needed to interpret it.
- **DNG** — Adobe's Digital Negative: a TIFF/EP-derived container with a
  **public specification and a patent grant** for compliant implementations.
  That grant is why this repo can implement it from the spec rather than from
  anyone's code.
- **IFD / SubIFD** — Image File Directory: TIFF's tag table. A DNG's `IFD0` is
  typically the *preview*; the SubIFD with `SubfileType: Full-resolution image`
  holds the sensor data. Confusing the two is how a "RAW decoder" ends up
  reporting the embedded JPEG's properties.
- **CFA (Colour Filter Array)** — the colour mosaic over a sensor (usually
  Bayer). **A Leica Q2 Monochrom has none** — `PhotometricInterpretation: Linear
  Raw`, `SamplesPerPixel: 1`. That is why PROJ-001 has no demosaic, no white
  balance and no colour matrix: they are **absent, not deferred**.
- **Demosaic** — reconstructing three colour channels from a CFA mosaic.
  PROJ-002.
- **Sensor plane** — the 2-D array of per-photosite values. In this repo,
  canonically an uncropped `u16` buffer, native-endian, values zero-extended.
- **Packed bits** — 14-bit samples stored contiguously with no byte padding.
  `8424 × 5632 × 14 bits = 83,026,944 bytes`, which equals `StripByteCounts`
  exactly — the layer-0 oracle.
- **BlackLevel / WhiteLevel** — the sensor's zero and saturation points (512 /
  16383 on the measured Q2M). Normalization maps between them.
- **ActiveArea → DefaultCrop → Orientation** — the three-stage geometry pipeline.
  The bit-exact oracle attaches **before** all three (see below).
- **Opcode list** — DNG's embedded per-file processing instructions.
  `OpcodeList1: FixBadPixelsConstant` runs on the raw plane; `OpcodeList3:
  WarpRectilinear` is a radial polynomial geometric correction. Both are real on
  the measured Q2M, not hypothetical — the Q-series 28 mm lens is designed
  around software distortion correction.
- **Oracle** — an independent implementation used as ground truth. Here: `dnglab`,
  **run as a tool, never linked**. Three layers: metadata, structure, and the
  bit-exact sensor plane. Plus a develop layer scored with SSIMULACRA2.
- **`--raw-checksum`** — `dnglab`'s MD5 of the **uncropped `u16` plane, native
  little-endian, 14-bit values zero-extended, no black subtraction, no crop**.
  Verified 2026-08-15. Not `--full-pixel` (that's the preview) and not the
  big-endian PGM payload from `--raw-pixel`.
- **Red-proof** — the deliberate-fault test that shows an oracle failing. See §12.
- **Tier A / Tier B corpus** — committed licence-clean fixtures that run in CI /
  real camera files that are never committed. `docs/conformance-matrix.md`.
- **Provenance class** — 1 (published spec) … 5 (read a copyleft implementation,
  **not permitted** in the default build). `docs/provenance-ledger.md`.
- **`irradiance`** — the library. **`irr`** — the internal dev/oracle binary,
  never a product surface.
- **crustyimg** — the sibling consumer repo. It consumes `irradiance` behind a
  `raw-develop` cargo feature. It is **not** this repo, and the dependency runs
  one way only.

---

## 15. Cycle-Specific Agent Rules

### During **design**

Set the **expected size** in `task.complexity` on the t-shirt scale
`XS | S | M | L | XL | XXL`. This is a *prediction*, and the point of a
prediction is that it later gets checked: ship stamps `task.complexity_actual`,
and `just calibration` shows whether you systematically under- or
over-estimate. `XL`/`XXL` is itself a finding — a spec that size is almost
certainly a stage; split it.

Optionally record `cost.tokens_estimate` (predicted total tokens) too. Once
enough specs have shipped, `just calibration` prints the token band each
expected size *actually* landed in — at which point the size you assign doubles
as a token estimate, measured from this repo rather than guessed. None of this
gates anything; the feedback loop is the whole value.

**Additionally, in this repo, design is not done until:**
1. The **failing tests include the oracle's red-proof**, if the spec touches an
   oracle (§12 bar 1).
2. The **fuzz target is specified**, if the spec adds a parser or a new input
   surface (§12 bar 2) — named, seeded, and in the acceptance criteria.
3. The **provenance row is drafted** — module, source, source licence, class —
   if the spec adds an algorithm or decoder (§10).
4. The **byte-level facts were probed against a real file**, not assumed
   (§12, design-time probe). Record them in `## Implementation Context`.

### During **build** (implementer reads this)

Before writing code:
1. Read the `/projects/PROJ-*/handoffs/HANDOFF-*.md` for your spec.
2. Read the linked `SPEC-*.md`, `STAGE-*.md`, and the project's `brief.md`.
3. Read every `DEC-*` listed in the handoff's references — check which
   namespace (§10).
4. Read `/guidance/constraints.yaml`; check rules for paths you'll touch. The
   five blocking ones in §1 apply to essentially every code path here.
5. Read `/guidance/toolchain-brief.md` and §5's *Measured toolchain* table.
6. If anything is ambiguous, add to `/guidance/questions.yaml` and stop.

**Do not read a copyleft RAW implementation to solve a problem.** Not
LibRaw, dcraw, rawspeed, rawler, rawloader, or a GPL/LGPL port of any of them.
If the algorithm appears to be available only that way, **stop and ask** — that
is a decision, not a build step (`provenance-recorded-per-algorithm`).
`dnglab` may be *run* to produce reference output; its source is not a
reference.

When done:
1. Fill in the handoff's `## Completion` section (including reflection).
2. Update `handoff.status` → `completed`; update spec's `task.cycle` → `verify`.
3. Append a build cost session entry to the spec's `cost.sessions`.
4. Create `DEC-*` files for non-trivial implementer decisions. When a
   decision is tied to specific code, fill in its `affected_scope`
   with the path globs it governs. This is required for file-bound
   decisions — it's what lets `just decisions-audit --changed` surface the
   decision when those paths change later. Leave `affected_scope: []` only for
   decisions not tied to particular files (e.g. a process choice).
5. **Add the provenance-ledger row** for anything you implemented, with the
   honest class. "I read it years ago and reimplemented from memory" is class 5,
   not class 3.
6. Open PR following Section 13.

Shortcut: `just advance-cycle SPEC-NNN verify`.

### During **verify** (reviewer reads this)

Check:
1. Acceptance criteria all met and tested?
2. Failing tests from spec now pass?
3. No drift from referenced decisions?
4. No constraint violations?
5. Non-trivial implementer choices have accompanying `DEC-*`?
6. Implementer reflection answered (not mailed in)?
7. `cost.sessions` has entries for prior cycles? Flag if missing
   (don't block).
8. For any acceptance criterion claiming **runtime behavior** (a component
   registers, a hook fires, a binary resolves on PATH, a server answers, a
   config takes effect), was the *behavioral* surface actually exercised — not
   just the shape validated (§12 behavioral pre-flight)? This is the class that
   escapes.

**This repo's four extra checks — any one failing is a ❌, not a punch list:**

9. **Did the oracle go red?** Run the deliberate-fault test yourself and watch it
   fail. A red-proof you did not personally observe failing is a self-report
   (DEC-004 rule 1).
10. **Does the fuzz target exist and has it run?** For any spec touching a
    parser. Not "a target is committed" — it *ran*, and for how long.
11. **Is there a provenance row for every new algorithm, with an honest class?**
    And is the source actually permissive or a published spec?
12. **Is any new dependency permissive, and is it not a RAW decoder?**
    `cargo deny check licenses` passing is necessary, not sufficient — it sees
    declared licences, not provenance (`docs/provenance-ledger.md`).

For check 3, run `just decisions-audit --changed` — it flags which
`DEC-*` records govern the files the implementer touched, so you can
confirm the work stayed consistent with them. `just decisions-audit`
(no flag) lints the records themselves. See `/guidance/recommended-tools.md`
for optional, heavier verify tooling.

Append a verify cost session entry before returning the verdict.

Output: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

**Every finding is labelled ship-blocking or follow-up, and gets an id.** The
labels are the reviewer's core judgement — a defect that lets bad data or a panic
reach a consumer is ship-blocking; a sharp edge that fails loudly is a follow-up,
which gets filed and does not hold the spec.

- `SB-N` — ship-blocker. `FU-N` — follow-up.
- **Numbering is PER SPEC and restarts at 1.** `SPEC-003/FU-1` and
  `SPEC-004/FU-1` are different findings; there is no repo-wide counter and
  nothing enforces one.
- **Cite a finding from another spec with its spec prefix** — `SPEC-003/FU-11`,
  never a bare `FU-11`. Inside its own spec's documents the bare form is fine.
- Round 2 of a cycle continues its spec's sequence rather than restarting, so a
  finding keeps one id for the life of the spec.
- ⚠ **Per-spec restart is in force from `SPEC-007` onward, and the ids before it
  are not renumbered.** There are three eras, and a reader will meet all three:

  | Specs | Label | Numbering |
  |---|---|---|
  | `SPEC-001`, `SPEC-002`, `SPEC-006` | `F-N` | per spec — pre-dates the `FU-`/`SB-` split entirely |
  | `SPEC-003`, `SPEC-004` | `FU-N` | one continuous run: `FU-1`…`FU-15`, then `FU-16`…`FU-21` |
  | `SPEC-007` onward | `FU-N` / `SB-N` | per spec, restarting at 1 — the rule above |

  `SPEC-004/FU-20` is a real id cited in four artifacts; an id that moves is worse
  than an id that looks odd. Read the prefix, not the number.

The point of the id is that a finding **survives a handoff boundary**.
`SPEC-003/FU-11` was raised at verify, carried into the next build brief, deferred
with a stated reason, re-raised at the following verify, and finally became
`SPEC-007`'s Context — four artifacts and three sessions without anyone restating
it from scratch. Prose alone cannot do that.

> **Provenance, recorded honestly:** this convention was **not** in the template.
> It was invented by SPEC-003's verify session in its handback (`de7a598`),
> adopted without question by the orchestrator, and written into decision records
> and a framed spec before anyone asked what it meant. It is codified here because
> it earned its place — but it accumulated authority for several rounds first,
> which is the failure mode `guidance/signals.yaml` exists to catch.

### Where an unresolved follow-up goes

The id lets a finding survive a handoff boundary. It does **not** make anyone
decide it, and for eight cycles nothing did: 34 `FU-N`s accumulated across
handbacks, shipped-spec reflections and `DEC-012` with no index, no status and no
owner, so a finding's fate depended on whether the orchestrator happened to act on
it that session. `guidance/signals.yaml` had a forcing function; follow-ups had a
naming convention.

Reconciled 2026-08-21: **34 findings across four specs. 26 already had a findable
disposition** — a fix, an owning spec, a signal, or a stated answer — **and 8 had
none**, four of them because a shipped reflection said "yes, one spec" and no spec
was ever created. The ids did not lose findings; the ids were never the missing
half. The missing half was a **disposition point**.

**The rule: a follow-up is dispositioned at the ship cycle of the spec that raised
it, and never crosses that ship undecided.** That bounds an open follow-up's
lifetime to one spec — which is precisely why this convention needs **no tracker
and no new list**. A register of unresolved follow-ups would only ever hold one
spec's worth, for the length of a punch-list round, and the two lists we already
have are the two destinations.

**Four dispositions. Every follow-up gets exactly one:**

| Disposition | Means | Where it lands |
|---|---|---|
| `fixed` | Done, in this spec's own cycles or at ship | name the commit or `file:line` in the row |
| `spec: SPEC-NNN` | It is work someone must do | a **real spec** — `frame` is enough, `ready` is not required. Put the finding's id in that spec's `## Context` |
| `signal: <signal-id>` | It is a recurring pattern, or friction in the process itself | a `guidance/signals.yaml` entry — new, or evidence added to an existing one. That ledger's close ritual now owns it |
| `closed: <reason>` | Deliberately not doing it | one line of why, in the row. A close whose trigger is a *test that will fail* is a good close; a close whose trigger is someone remembering is not |

**"Carried into the next build brief" is not a disposition.** That is what was
happening, and it is why `SPEC-003/FU-11` needed four artifacts and three sessions
to reach an owner. Carrying is a fine *tactic* inside a spec's own punch-list
round; it is not an answer at ship.

**Spec or signal?** A follow-up that names one file and one fix is a spec (or is
`fixed`). A follow-up about a *class* — a rule that will recur, friction in the
process — is a signal. Do not route concrete defects into `signals.yaml`: it
carries ~13 entries and a ten-minute walk at each close, and thirty bug rows would
destroy the thing that makes it work. Do not route process friction into a spec:
it has no acceptance criteria and will sit in `frame` forever.

**The record lives in the spec, not the handback.** Ship appends a `## Follow-ups`
table to the spec — one row per follow-up id raised against it across every cycle,
with its disposition. The handback is where a finding is *raised*; the spec is
where it is *decided*, because the spec is archived under `specs/done/` and read by
`just status`, while a handback is (`SPEC-003/FU-14`'s own phrase for it) the least
durable place in this repo.

```markdown
## Follow-ups

| id | finding | disposition |
|---|---|---|
| `FU-1` | one line | `fixed` — `src/ifd.rs:188` |
| `FU-2` | one line | `spec: SPEC-009` |
| `FU-3` | one line | `signal: tier-map-predicts-what-it-should-record` |
| `FU-4` | one line | `closed` — reason |
```

⚠ **What enforces this today: nothing.** This repo has measured **twice** that a
documented step with no surface simply does not happen —
`brag-step-skipped-at-ship` (six ships, zero entries, caught by a human) and
`named-tests-can-pass-vacuously`. Until `just validate` asserts that every `FU-N`
raised in a spec's handoffs has a row in that spec's `## Follow-ups` table, this
rule rests on the ship cycle being run honestly, and it is the same shape as the
two steps that were not. Tracked as `follow-up-disposition-has-no-surface`.

### During **ship**

Append a `## Reflection` block to the spec with three answers:
1. What would I do differently next time?
2. Does any template, constraint, or decision need updating?
3. Is there a follow-up spec to write now?

Then:
- Update the spec's `task.cycle` → `ship`.
- **Append the `## Follow-ups` table** — one row per `FU-N` raised against this
  spec across every cycle, each with one of the four dispositions. No follow-up
  crosses this ship undecided; see *Where an unresolved follow-up goes* above.
- Append a ship cost session entry, then compute `cost.totals`.
- Stamp `task.complexity_actual` — what it actually took, on the same
  `XS|S|M|L|XL|XXL` scale as the expected `task.complexity`. Ship is the only
  moment that number is knowable.
- Run `just archive-spec SPEC-NNN` (moves to `done/`, updates stage).
- **Confirm `docs/provenance-ledger.md` and `docs/conformance-matrix.md` are
  current.** A camera or algorithm that gained support without gaining a row is
  the "unread field" defect (§11) in its most expensive form.
- If Q2 surfaces a template/constraint/decision change you're NOT making now,
  record it in `/guidance/signals.yaml` (`type: lesson` with its N-count for a
  recurring coding pattern; `type: process-debt` for tooling friction) so a
  close forces the decision. See `docs/signals.md`; browse `just dash signals`.
  **Template-level friction also goes to `/feedback/`** — see DEC-000.
- If stage backlog is complete, run the Stage Ship prompt.
- Log the win — **on by default** (DEC-010). Call the configured tool directly
  (default `brag`): `brag add -t "<what shipped>" -k shipped -i "<IMPACT>"` (CLI),
  or the `brag_add` tool over `brag mcp serve` (MCP). Seed it from the spec's
  `value_link` + `cost.totals`, and frame the **impact** (the outcome / who's
  better off), not the output. See `guidance/recommended-tools.md`; opt out via
  `spec.accomplishments.enabled: false`.
- Commit.

**Cutting a release?** A release is its own spec — scaffold it with
`just new-release-spec "<version>" STAGE-NNN` (or `just new-spec … --release`).
It carries a generic runtime **pre-flight checklist** (tag integrity, artifact
trust on a clean host, channel trust, data isolation, runtime smoke, rollback —
DEC-006); fill in the tool-specific command for each before you publish. Every
defect that escaped design→build→verify across the dogfood projects was
operational/runtime, so don't skip it. For the version to cut, run
`just next-version` — this app's `spec.version.scheme` is **`semver`**
(`.repo-context.yaml:60`), *not* the template's `calver` default, because a
library's version number has to signal compatibility to consumers (DEC-007;
DEC-000). That app version lives in git tags; the top-level `VERSION` file is
template provenance (0.6.38), not the app's version (see `docs/versioning.md`).

⚠ **Do not publish to crates.io during PROJ-001** — `STAGE-004` puts it
explicitly out of scope; it waits for a second camera.

---

## 16. Confidence Discipline

Decisions in `/decisions/` have an `insight.confidence` field (0.0–1.0).
Honest values matter — they drive these behaviors:

- **Design phase:** if Claude emits a decision at confidence < 0.7, it
  also adds an entry to `/guidance/questions.yaml` flagging it for
  further investigation.
- **Verify phase:** if a spec references any decision at confidence < 0.6,
  that's a yellow flag worth surfacing in the review.
- **Weekly review:** all decisions at confidence < 0.8 are listed with
  a note on whether recent work has strengthened or weakened them.

Use 1.0 only for decisions that are truly locked (tech stack choice
after it's been installed and working, for example). Most decisions
should land between 0.7 and 0.95.

### Three rules earned the hard way — codified at STAGE-001's close, 2026-08-22

These are not style preferences. Each is a **lesson at or past its bar** in
`guidance/signals.yaml`, and between them they account for a false green that
shipped a panic past seven gates, a decision record rejected on three counts, and
a blocking constraint's CI gate that was dark for **17 consecutive runs**.

**1. The writing rule — `measurement-over-generalised` (N=5).**

> When a probe result becomes a claim, the sentence names **the exact command and
> its scope**. Any word that generalises beyond the run — *"the day X happens"*,
> *"the boundary"*, *"the class"*, *"always"* — must either be **deleted** or be
> backed by a **second measured point in a different direction**.

Apply it while typing the sentence, not in review. Instance 5 was committed by an
agent who knew the rule, had just enforced it on someone else, and was writing
that very correction — so a checklist provably does not catch this; a rule you
apply as you write does. "Mutated `RowsPerStrip`" is not "the boundary is
mutation-tested". One point on a boundary is never a boundary.

**2. Assert the match count — `attribute-text-inside-doc-comments` (N=5).**

> Any tooling that pattern-matches source text or tool output must anchor
> deliberately, exclude `//` / `//!` / `/* */`, and **assert how many times it
> matched**. Never take the first hit.

`index()` on source text finds *documentation about* the code as readily as the
code. Every one of the five instances produced a **wrong answer** rather than an
error — twice a false negative, once a false green that shipped a panic.

**3. A gate must fail through its own `die` — `a-gate-that-fails-mutely-is-a-gate-that-never-ran` (N=4).**

> Every `grep` whose result a gate depends on is **guarded** (`|| true`) and its
> match count **asserted**. A gate never exits on a pipeline's own status; it
> exits through its own error message, with a reason.

Rules 2 and 3 are one rule from two sides — a match landing on the wrong text, and
a **non**-match becoming control flow. Two traps, both measured:
- **Guarding one `grep` in a pipeline is not enough, and looks like it is.** A
  zero-match emits nothing, so the *next* `grep` zero-matches too and aborts
  anyway — byte-for-byte the original behaviour.
- **The obvious test exercises the wrong path.** Forcing an out-of-range value
  makes the leading `grep` *match*. Only a genuine zero-match of the **leading**
  `grep` exposes the defect.

A proof that dies without a message is indistinguishable from a proof that never
ran — which is the exact thing these gates exist to prevent.

**In this repo, "measured once, on one file" is not 1.0.** The oracle contract
was verified against a single Leica Q2 Monochrom frame from one firmware. High
confidence for that file; lower for the claim as stated generally. Say which
you mean.

---

## 17. Pointers

**This repo's substance**

- Constraints (**this file wins**): `/guidance/constraints.yaml`
- The oracle contract — three layers + the verified plane contract: `/docs/oracle-contract.md`
- The measured Q2M file: `/docs/measured-q2m-dng.md`
- Camera coverage + corpus policy: `/docs/conformance-matrix.md`
- Provenance ledger (every algorithm's source + licence): `/docs/provenance-ledger.md`
- Licence policy + cargo-deny wiring: `/docs/license-policy.md`
- What we're building: `/projects/PROJ-001-monochrome-dng-develop/brief.md`
- Open exploration: `/spikes/`

**Process**

- Open questions: `/guidance/questions.yaml`
- Toolchain brief for cold build agents: `/guidance/toolchain-brief.md` — **filled 2026-08-16**; inject it into every build handoff (DEC-004 rule 5). Leads with the `cargo +nightly` trap.
- Signals (typed feedback ledger): `/guidance/signals.yaml` (browse `just dash signals`; ritual + bar in `docs/signals.md`)
- **This repo's** decisions: `/decisions/` (audit with `just decisions-audit`)
- **The template's** decisions (background only): `/docs/decisions/` — see §10
- Template-level findings: `/feedback/` — see `DEC-000`
- Recommended (optional) tools: `/guidance/recommended-tools.md`
- Versioning (`semver` here; `just next-version`): `/docs/versioning.md` (DEC-007)
- Projects: `/projects/` · Templates: `/projects/_templates/`
- Reports: `/reports/` (daily, weekly)
- Timelines: `/projects/*/specs/SPEC-NNN-*-timeline.md` (per-spec)
- Cycle prompts: `/projects/*/specs/prompts/`
- Phase prompts: `/FIRST_SESSION_PROMPTS.md` · First walkthrough: `/GETTING_STARTED.md`
- Daily commands: run `just --list`

**Known stubs and leftovers** (recorded in `DEC-000`, listed here so no agent
mistakes them for content):

- `/docs/architecture.md` — template stub; populate when the module layout is
  real (after SPIKE-001). The only stub left in `docs/`.
- **Deleted 2026-08-16** (DEC-000 follow-ups): `docs/api-contract.md` and
  `docs/data-model.md` (this library has no external API and no persistent data),
  and `decisions/DEC-001-example-structured-logging.md` (it said `repo: my-app`,
  cited a constraint that no longer exists, and was the only thing `just status`
  counted under "Total decisions"). `GETTING_STARTED.md` and
  `FIRST_SESSION_PROMPTS.md` still reference all three — those are **template
  onboarding docs describing a generic instance**, not instructions for this repo.
  Ignore those lines.
- `/docs/blog/`, `/docs/talks/`, `/docs/sessions/`, `/docs/harvests/`,
  `/docs/ROADMAP.md`, `/reports/daily/2026-04-21.md`,
  `/reports/weekly/2026-W17.md`, `/feedback/2026-0*.md`,
  `/.github/TEMPLATE_README.md` — ~4,800 lines describing the **template
  repo and other instances**, not this app. Useful as prior art; not this
  repo's history. Do not cite them as facts about `irradiance`.
