---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-056
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
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-002
  stage: STAGE-014
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-4-8  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: 2026-11-11

references:
  decisions: [DEC-053, DEC-049, DEC-012, DEC-051, DEC-009]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, library-not-application, one-spec-per-pr]
  related_specs: [SPEC-054, SPEC-042]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-014's <capability>". Optional; null is acceptable.
value_link: "closes the two 0.2.0 required-processing caveats and the 0.2.0 verify's hygiene nits, so 0.2.1 ships with fewer known limitations"

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
  tokens_estimate: 20000000
  sessions:
    - cycle: build
      handoff: HANDOFF-125
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 20494603
      estimated_usd: 14.01
      duration_minutes: 30
      recorded_at: 2026-11-11
      notes: "RE-SUMMED at ship 2026-11-11 by the orchestrator from the same transcript after the session ended: 101 unique message.ids, all claude-opus-5-5, no subagents; input 205 + cache-creation 220,017 + cache-read 20,206,354 + output 68,027 = 20,494,603; USD at the same rates. The handback's live floor was 19,174,151 / $13.09. Original note: FLOOR taken live just before the handback commit (docs/cost-tracking.md rule 6): session transcript a2bbac90-a1bd-4857-94f7-94b92579e8c9.jsonl, 96 unique message.ids, all claude-opus-5-5, no subagents; input 195 + cache-creation 212,456 + cache-read 18,900,990 + output 60,510 (max per message). USD at opus-5-5 list rates in 5/out 25 per MTok, cache read x0.10, cache writes x2.0 (1-hour TTL). The last few wrap-up messages are not in it; re-sum the same file after the session ends."
    - cycle: verify
      handoff: HANDOFF-126
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 10841767
      estimated_usd: 7.64
      duration_minutes: 25
      recorded_at: 2026-11-11
      notes: "RE-SUMMED at ship 2026-11-11 by the orchestrator from the same transcript after the session ended: 65 unique message.ids, all claude-opus-5-5, no subagents; input 159 + cache-creation 149,001 + cache-read 10,659,638 + output 32,969 = 10,841,767. The handback's live floor was 8,053,981 / $5.72. Original note: FLOOR taken live before the handback commit (docs/cost-tracking.md rule 6): transcript 8b5e64da-637a-4eba-9162-422f40773bbf.jsonl in the spec-056-verify-4123a1 worktree project dir, 51 unique message.ids, all claude-opus-5-5, no subagents; input 127 + cache-creation 130,095 + cache-read 7,905,121 + output 18,638 (max per message). USD at opus-5-5 list rates in 5/out 25 per MTok, cache read x0.10, cache writes x2.0. Re-sum the same file after the session ends. Verdict is on code commit 1973abf; PR 193 was already squash-merged to main as dd70dea (identical src/tests) before this verify ran."
  totals:
    tokens_total: 31336370
    estimated_usd: 21.65
    session_count: 2
shipped_at: 2026-11-11
---

# SPEC-056: Refuse unreadable opcode lists and guard the stage primitives (0.2.1 housekeeping)

## Context

This is the only spec in **STAGE-014**, PROJ-002's post-0.2.0 maintenance stage. It
releases as **0.2.1**. Tagging and `cargo publish` are the maintainer's step.

0.2.0 shipped with two required-processing caveats in its CHANGELOG's Known
limitations. Both were raised by the SPEC-054 verify (HANDOFF-115) and deferred by
SPEC-042:

1. **A malformed `OpcodeList` tag is read as absent.** When a list tag's entry is
   present but its payload can't be read, parsing tolerates it (DEC-012):
   `opcode_list_n = None`, and the tag goes into `malformed_tags` (`src/ifd.rs:1729-1767`).
   The required-processing check (`check_sensor_required_processing`,
   `src/opcode.rs:549`) only sees `None`, so the develop goes ahead without
   whatever that list required.
2. **The stage primitives skip the check.** `demosaic_into` (`src/demosaic.rs:234`),
   `demosaic_into_threaded` (`:432`) and `to_srgb_linear_into` (`src/color.rs:210`)
   take a `&Sensor` but never call it, while every `develop_*` does.

The 0.2.0 verify (HANDOFF-124) also left two non-blocking hygiene items:

3. **`cargo doc` with warnings-as-errors fails.** Measured 2026-11-11:
   `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --lib` reports **19**
   `rustdoc::private_intra_doc_links` errors across 8 files: `color.rs` 5,
   `demosaic.rs` 4, `lib.rs` 3, `tone.rs` 3, and 1 each in `develop.rs`, `ifd.rs`,
   `plane.rs` and `warp.rs`. Examples: public docs linking `ljpeg`,
   `ljpeg::reconstruct`, `lower_median`, `is_identity`, `BRADFORD_D50_TO_D65`,
   `SRGB_FROM_XYZ_D65`, `interpolation_weight` and `colour_difference`. On docs.rs
   these render as plain text. No CI job builds docs.
4. **The nested-cargo red-proof tests break under `CARGO_TARGET_DIR`.** The verify
   named `tests/develop.rs`, but the design probe found the **same defect in all 7**
   nested-cargo tests. Each one runs `cargo build --release` in a staged temp crate,
   then runs `dir.join("target/release/<probe>")`. When `CARGO_TARGET_DIR` is set,
   the inner build writes elsewhere, so the binary isn't found. Worse, the
   **mutated** staged crate then builds into the caller's shared target dir. Sites:
   `tests/develop.rs:985`, `tests/develop_oracle.rs:695`,
   `tests/ifd_resource_guard.rs:374`, `tests/ljpeg_shape_binding.rs:462`,
   `tests/plane_oracle.rs:839`, `tests/plane_validate.rs:1078` and
   `tests/required_processing.rs:567`. None of these tests sets or removes the
   variable today.

**Corpus probe (2026-11-11, `irr ifd` over all 8 held tier-B files).** No file has
an opcode-list tag in `malformed_tags`; the only malformed tag in the corpus is the
K-3 III's `50713`. The Bayer files (Canon 5D III and Nikon D300S) carry no opcode
lists. The Q2-Mono files carry lists 1 and 3, both readable. So items 1 and 2 are
pure hardening: every corpus develop stays `Ok` and byte-identical.

## Goal

Every public entry that takes a `Sensor` and yields developed or camera-referred
pixels refuses with a typed error when the file declares processing the path won't
apply, **or** when the file has an opcode list the library can't read. `cargo doc`
is warning-free and gated in CI. The nested-cargo tests work with any
`CARGO_TARGET_DIR`.

## Design

**Follow DEC-053.** Do not re-litigate its rejected alternatives: no sentinel-id
reuse, no parse-time failure, no `&Sensor` on `apply_warp_into`, and no escape hatch.

### 1. Refuse an unreadable list (DEC-053 §1)

In `check_sensor_required_processing` (`src/opcode.rs:549`), handle each list in
order 1, 2, 3:

- If that list's tag (`TAG_OPCODE_LIST_1/2/3`, `src/ifd.rs:130-134`) is in
  `sensor.malformed_tags`, return
  `Err(Error::UnreadableOpcodeList { list: n })`.
- Otherwise, scan the list exactly as today.

Do the unreadable check list by list, before that list's scan, so the reported
error is deterministic.

Add the variant to `Error` (`src/lib.rs:274`, which is `#[non_exhaustive]`):

- a doc comment that cites SPEC-056 and DEC-053;
- a `Display` arm next to `UnsupportedRequiredOpcode` (`:1084`), e.g.
  `"OpcodeList{list} is present but unreadable (malformed tag entry); its required
  processing cannot be confirmed"`;
- an `Error::kind()` arm returning **`ErrorKind::Corrupt`**. That match has no
  wildcard (DEC-051), so the build won't compile until the arm exists.

The condition is `malformed_tags` **membership**, not `opcode_lists[n] && field.is_none()`.
`malformed_tags` is the documented record of "present but unreadable" (DEC-012). If
you choose to add a `debug_assert!`, it must not be reachable from hostile input.

### 2. Guard the Sensor-taking primitives (DEC-053 §2)

- Make `DevelopPath` (`src/opcode.rs:480`) `pub` and `#[non_exhaustive]`, and document
  both variants for consumers.
- Rename the existing `pub(crate) fn check_required_processing(lists, path)`
  (`:535`) to an internal name, e.g. `check_lists`.
- Expose `pub fn check_required_processing(sensor: &Sensor, path: DevelopPath) ->
  Result<(), Error>`. It is the old `check_sensor_required_processing` made public,
  so either keep one name and update the call sites, or have the old name delegate.
  Give it a rustdoc that explains when a composer needs it, with `# Errors`.
- Call it **first**, before any allocation, with `DevelopPath::Colour` in
  `demosaic_into`, `demosaic_into_threaded` and `to_srgb_linear_into`. If
  `develop_color_*` calls one of these internally, a second check is cheap and fine.
  Don't restructure to avoid it.
- In the docs of `warp::apply_warp_into` (`src/warp.rs:225`) and
  `develop::apply_fix_bad_pixels_constant` (`src/develop.rs:460`), say that they are
  the opcode appliers, take no `Sensor`, and that the caller validates with
  `opcode::check_required_processing`.
- Update the stale `Sensor::opcode_lists` doc ("decoding the opcode streams is
  STAGE-003", `src/ifd.rs:721`).

### 3. Rustdoc private links and a doc gate

- Fix all 19 private intra-doc links. For each one, either reword it to plain code
  formatting (`` `name` `` with no link), or link a public item that says the same
  thing. Do **not** silence the lint with an `#[allow]` or with
  `--document-private-items`.
- Add `just doc-check`: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --lib
  --all-features`. Also check `--no-default-features` if the default surface differs.
  Print which `rustdoc` answered, following the PATCH-004 convention.
- Add a CI job `docs` that runs it on `@stable`.
- **The gate needs a red-proof** (`oracle-must-be-shown-red` covers gates): add
  `just doc-check-red-proof` (script under `scripts/`) and a CI job next to the gate.
  Follow DEC-009's mechanism:
  1. Copy the crate to a temp dir and run `doc-check` unmutated. This is the negative
     control and must pass.
  2. Inject one public doc comment linking a private item into the copy. The gate
     must fail, and the failure must be caught by the lint name.
  3. Leave the working tree untouched.
- The new script must pass `just gate-audit`: no swallowed exit, every grep guarded,
  and a count asserted.

### 4. Nested-cargo tests honour `CARGO_TARGET_DIR`

In each of the 7 staged builds, put the staged crate's build in its own target dir.
Add `.env_remove("CARGO_TARGET_DIR")` to the inner `cargo build` Command. Setting
`.env("CARGO_TARGET_DIR", dir.join("target"))` is an equivalent choice; pick one and
use it everywhere. The probe path `dir.join("target/release/…")` then stays correct.

Explicit isolation is the point: a mutated staged crate must never build into the
caller's target dir. Cover all 7 sites, not just `tests/develop.rs`. If the shared
staging helper is copied across files, keep the fix identical in each.

### Out of the build: release bookkeeping

The build adds `CHANGELOG.md` `[Unreleased]` entries:

- **Fixed:** both required-processing caveats;
- **Added:** `opcode::check_required_processing` and `DevelopPath`;
- **Changed:** the stage primitives now refuse; rustdoc links.

It also **removes the two Known-limitations bullets** this spec closes. The `0.2.1`
version bump and the dated CHANGELOG section belong to the ship PR (orchestrator).
Tagging and publishing are the maintainer's.

## Inputs

- **DEC-053** (this decision); **DEC-049** (the contract and the scan this extends);
  **DEC-012** (read-tolerance); **DEC-051** (`ErrorKind`); **DEC-009** (control
  first).
- `src/opcode.rs`:
  - `DevelopPath` `:480`
  - `check_required_processing` `:535`
  - `check_sensor_required_processing` `:549`
  - `check_list` `:565`
- `src/ifd.rs`:
  - the opcode-list tags `:130-134`
  - the field docs `:721-749`
  - the reads `:1729-1767`
  - `malformed_tags` `:812`
- `src/lib.rs`:
  - `Error` `:274`
  - the opcode variants `:609-629`
  - `Display` `:1078-1090`
  - `ErrorKind` / `kind()` `:1246-1275`
- `src/demosaic.rs` `:234`, `:432`; `src/color.rs` `:210`; `src/warp.rs` `:225`;
  `src/develop.rs` `:460`.
- `tests/required_processing.rs`: the SPEC-054 fixture builder (`fixture_bytes`,
  `:89`), the per-entry helpers, and the nested-cargo red-proof
  (`required_opcode_probe`).
- The 7 nested-cargo test files listed in Context item 4.
- `justfile`, `.github/workflows/ci.yml` (the doc gate and its red-proof),
  `scripts/` (the red-proof pattern, e.g. `lint-red-proof.sh`), `CHANGELOG.md`.

## Outputs

- **Modified:**
  - `src/opcode.rs` (the unreadable check, the public check and `DevelopPath`);
  - `src/lib.rs` (the variant, `Display` and `kind()`);
  - `src/demosaic.rs` and `src/color.rs` (the guard calls);
  - `src/warp.rs` and `src/develop.rs` (applier docs);
  - every file carrying a private doc link;
  - the 7 nested-cargo test files;
  - `tests/required_processing.rs` (new cases);
  - `justfile`, `ci.yml`, `CHANGELOG.md`.
- **Created:** `scripts/doc-check-red-proof.sh`.
- **No** `DEVELOP_VERSION` change. **No** new dependency of any kind. **No**
  provenance row: this adds no new algorithm. Say so in the PR.

## Acceptance Criteria

- [ ] **Unreadable list refused.** For each list n in 1, 2 and 3, a fixture whose
      list-n tag entry is present but unreadable (payload offset past EOF):
      - parses `Ok`;
      - has the tag in `malformed_tags`;
      - returns `Error::UnreadableOpcodeList { list: n }` from `develop_into`;
      - returns it from every colour develop entry (Bayer fixture).
- [ ] **Error shape.** `UnreadableOpcodeList` exists, its `Display` is non-empty and
      names the list, and `kind()` returns `ErrorKind::Corrupt`.
- [ ] **Primitives guarded.** `demosaic_into`, `demosaic_into_threaded` and
      `to_srgb_linear_into` each refuse a Bayer fixture that has:
      - a mandatory opcode → `UnsupportedRequiredOpcode`;
      - an unreadable list → `UnreadableOpcodeList`.
      Each one still returns `Ok` on an optional-only fixture and on an opcode-free
      fixture. That is one test per primitive per case.
- [ ] **Public check.** `irradiance::opcode::check_required_processing(&sensor,
      DevelopPath::Mono|Colour)` is callable from the `tests/` crate. It returns the
      same result the matching `develop_*` would, for the existing SPEC-054 fixtures.
      `DevelopPath` is `#[non_exhaustive]`.
- [ ] **Appliers documented.** The docs of `apply_warp_into` and
      `apply_fix_bad_pixels_constant` state that the caller owns validation and link
      the public check.
- [ ] **Docs clean and gated.** `just doc-check` passes on the final tree with 0
      errors. CI job `docs` exists and is **observed green** on the PR head (`gh run
      view`, not a laptop run). `just doc-check-red-proof` passes, with its control
      green and its injected private link caught, and runs in CI.
- [ ] **CARGO_TARGET_DIR-proof.** With `CARGO_TARGET_DIR` set to a temp dir, the 7
      nested-cargo tests pass. Run them with `CARGO_TARGET_DIR=$(mktemp -d) just
      test` or an equivalent filtered run, and record the command in the handback. No
      staged build writes into the outer target dir.
- [ ] **Corpus unchanged.** Every held tier-B file (8) develops `Ok` and
      byte-identically to `main` on its path, mono or colour. This is a local run
      (DEC-003); record the hashes or the probe in the handback.
      `DEVELOP_VERSION` is unchanged.
- [ ] **Gates.** All green: `just test`, `lint-ci`, `lint-no-allow`, `fmt-check`,
      `validate`, `cost-audit`, `gate-audit`, `deny`, `deny-fuzz`, `doc-check`,
      `msrv`.

## Failing Tests

Write these first. Each one must be **observed red on the pre-change tree**, for the
right reason, before the fix goes in.

- **Unreadable-list red-proof (primary).** The list-2 unreadable fixture on
  `develop_into`, and the list-1 unreadable fixture on `develop_color_into`, each
  return `Ok(())` on `main` (the silent-wrong case) and the typed error after the
  fix. The SPEC-054 optional-only and opcode-free controls stay green throughout
  (DEC-009). Extend the existing nested-cargo `required_opcode_probe` red-proof if it
  can express this. Otherwise, the before/after unit assertions plus the recorded
  `main` run are the proof.
- **Per-primitive tests.** One test per primitive per case (mandatory, unreadable,
  optional-only, opcode-free). The first two are red on `main`.
- **Per-list tests.** Unreadable list 1, 2 and 3 on the mono path.
- **`kind()` classification.** `UnreadableOpcodeList` maps to `Corrupt`.
- **Doc gate red-proof.** `scripts/doc-check-red-proof.sh`: the control passes and
  the injected private link fails by lint name.
- **CARGO_TARGET_DIR.** The 7 nested-cargo tests fail on `main` when the variable is
  set (observe one, `tests/develop.rs`, and record it) and pass after the fix.
- **Fuzz.** No new input surface: the check reads only `malformed_tags`, which is
  already fuzzed through the container. Run the existing opcode/develop fuzz target
  for the configured window and confirm it is clean.

## Non-Goals

- No new opcode, and no opcode application on the colour path.
- No develop-anyway escape hatch (DEC-053 defers it).
- No signature change to `apply_warp_into` / `apply_fix_bad_pixels_constant`.
- No change to parsing or `info` behaviour for any file.
- Not the version bump, not the tag, not publishing.
- Not STAGE-005's XS harness bullets.

## Notes for the Implementer

- Read **DEC-053**, then SPEC-054 and DEC-049 for the scan you are extending. The
  corpus probe above means a changed corpus develop indicates a bug.
- The fixture builder in `tests/required_processing.rs` writes a tiny DNG from
  bytes. To make an unreadable list, give the opcode-list entry a payload
  offset/count that runs past EOF, so `payload()` fails and the tag lands in
  `malformed_tags`. **Assert that it's in `malformed_tags`** in the test, so the
  fixture can't quietly turn into an absent tag.
- Bayer primitive fixtures need a CFA sensor. Reuse the existing `fixture(cfa: true,
  …)` builder.
- Run `just lint-ci` before every push (AGENTS.md §6). Commit-message bodies must be
  plain prose with no code tokens: the push ruleset rejects `->`, `#[…]` and `--flags`.

---

## Reflection

*Appended during **ship**, 2026-11-11.*

1. **What would I do differently next time?** Hold the build PR until the cold
   verify reports. #193 merged before HANDOFF-126 finished. The verify still
   approved the identical code at `1973abf`, but in a punch-list case the fix would
   have landed as a second PR against an already-changed main.
2. **Does any template, constraint, or decision need updating?** No. Both handbacks
   again reported live floors that ran low: build 19.17M against a re-summed
   20.49M, verify 8.05M against 10.84M. The existing ship practice of re-summing
   the transcript after the session ends caught both. It is recorded in the cost
   notes, and no new rule is needed.
3. **Is there a follow-up spec I should write now before I forget?** No new one.
   Two follow-ups already have homes:
   - STAGE-014's four [XS] harness outlines (from PROJ-001 STAGE-005);
   - the develop-anyway escape hatch that DEC-053 deferred until a consumer asks.
4. **Where was the worst defect caught?** — design. The design probe found the
   `CARGO_TARGET_DIR` defect in all 7 nested-cargo tests, not just the one the
   0.2.0 verify named.
5. **What can a user do now that they couldn't before?** Before, a DNG with an
   unreadable opcode list, or a file with mandatory opcodes passed straight to
   `demosaic_into` or `to_srgb_linear_into`, could produce a silently uncorrected
   image. Now it returns a typed error, and composers can call
   `opcode::check_required_processing` themselves. In addition, `cargo doc` builds
   warning-free under a CI gate.
