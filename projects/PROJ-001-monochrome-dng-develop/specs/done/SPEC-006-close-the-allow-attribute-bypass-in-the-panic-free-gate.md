---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-006
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-001
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
depends_on: [SPEC-001]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "makes the panic-free constraint true rather than nearly-true"

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
      agent: claude-opus-5
      interface: other
      tokens_total: 5121192
      estimated_usd: null
      duration_minutes: 15
      recorded_at: 2026-08-20
      notes: "All six acceptance criteria met; both red-proof directions measured and pasted. The headline: with the spec's #[allow] planted on a pub fn in src/lib.rs (before the #[cfg(test)] module), BUILD 0 CLIPPY 0 FMT 0 TEST 0 MSRV 0 DENY 0 REDPROOF 0 and the new NO-ALLOW gate 101 with two E0453s at src/lib.rs:88 -- the hole reproduced, one gate seeing it. Honest tree: all eight 0. Also proved the inner #![allow] form (101) and, on the honest tree, that --all-targets goes 101 on the test module's legitimate allow, which is why the scope is --lib. Mechanism transcribed verbatim from the spec; no text search; scripts/lint-red-proof.sh and src/lib.rs both untouched. constraints.yaml:33 rewritten to name both jobs, state SCOPE: --lib only, and say plainly that neither job proves any code is panic-free -- only that the policy is intact and inescapable on the library; constraints-view.sh output byte-identical before and after. Three deviations, all recorded in the handback: (1) the branch already pointed at 412cb1b (SPEC-002's design commit) rather than main, so it was reset onto main dd4eb42 -- nothing lost, 412cb1b is still the tip of feat/spec-002-corpus-manifest-reader; (2) AGENTS.md §6 gained the command block, because §6 makes recipe<->block correspondence SPEC-001 acceptance criterion 8; (3) CI inlines the cargo invocation rather than calling just, because just is not on ubuntu-latest -- caught by executing the YAML run: block before commit, not by reading it. Ran in an isolated git worktree: another session was moving HEAD in the shared checkout during this cycle. tokens_total is REAL but not from /cost (a client-side slash command the assistant cannot execute): summed 53 deduplicated usage objects in this session's own transcript (~/.claude/projects/-Users-...-irradiance/e8f27d72-....jsonl). Composition: input 106 + output 39,205 + cache-write 106,729 + cache-read 4,975,152 (97.1% cache-read). FLOOR -- written before the session ends. Same method as SPEC-001's verify-1/verify-2 and build-2/build-3/build-4; NOT comparable to build-1's 197,940 (token-counts-not-comparable). Follow-ups, none blocking: src/lib.rs's module doc now reads as if the gap is still open (one sentence would close it, deliberately left to avoid touching the file the gate protects); --all-features was NOT added to the gate on purpose, but DEC-002's std-behind-a-feature proposal will make that a real question; and toolchain-brief.md's +stable = 1.97.0 has drifted to 1.98.0 on this host."
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 4814757
      estimated_usd: null
      duration_minutes: 30
      recorded_at: 2026-08-20
      notes: "APPROVED at e4a7087 (implementation 618fd6f). All six acceptance criteria met and re-measured independently in a fresh worktree; eight follow-ups, ZERO ship-blocking. Ran check #9 myself: attack planted before the #[cfg(test)] module -> BUILD 0 CLIPPY 0 FMT 0 TEST 0 MSRV 0 DENY 0 REDPROOF 0, NO-ALLOW 101 with both E0453s at src/lib.rs:88, and `just lint-no-allow` 101 on the same tree; honest tree all eight 0. Then tried thirteen more bypasses and every one was caught: inner #![allow] in a module, crate-root #![allow] under the #![deny], #[cfg_attr(all(), allow(...))], #[cfg_attr(not(test), allow(...))], #[expect(...)], #[warn(...)], renamed aliases (clippy::option_unwrap_used / clippy::integer_arithmetic -> E0453 after rename), macro_rules!-generated #[allow], #[allow(clippy::restriction)] (five E0453s), a lib module pulled in via #[path] from tests/, Cargo.toml [lints.clippy] allow, and both group forms (#[allow(clippy::all)] / #[allow(warnings)]) which emit no E0453 but cannot silence the lints either - all still 101, even with the crate-root #![deny] deleted. Two measured results the constraint text does not yet carry: (a) the gate ALONE re-imposes all five lints at forbid level on --lib - with the crate-root #![deny] block deleted and a plain panicking pub fn with no attribute at all, it still exits 101, so for the library it is not dependent on job (1); (b) the gate is scoped by TARGET but not by FEATURE configuration - it runs default features, a no-op at zero features today but live the day DEC-002 puts std behind one. The largest follow-up: the gate's own flags are unpinned. Measured - swap all five -F to -D and the planted attack goes GREEN (0); drop -F clippy::expect_used and plant #[allow(clippy::expect_used)] on a pub fn that expects, and BOTH the no-allow gate and the full clippy gate exit 0, a panic on the public API with all eight gates green again. Nothing in CI notices either edit. That is DEC-009's own thesis one level up - SPEC-001's gate self-tests in CI, this one was proved red once by hand - and it wants its own spec, not another round here (the criteria as written are met and the gate is sound at this SHA). Three disclosed deviations all confirmed accurate and confined: the branch reset (412cb1b is contained in feat/spec-002-corpus-manifest-reader, whose tip has since moved to 112bd80 - nothing lost), the AGENTS.md §6 addition (two hunks, both inside §6, recipe text matches app.just), and CI inlining cargo (single additive hunk at ci.yml:120-165; every other job inlines the same way). constraints-view.sh output re-diffed byte-identical against main. Placement question answered: appending the attack AFTER the test module does trip clippy::items_after_test_module and turns CLIPPY 101 for an unrelated reason - reproduced - so any future automated red-proof for this gate must pin the site. tokens_total is REAL but not from /cost (a client-side slash command the assistant cannot execute): summed 42 deduplicated usage objects in this session's own transcript (~/.claude/projects/-Users-...-irradiance-verify-spec-006/42350191-....jsonl). Composition: input 84 + output 44,391 + cache-write 137,798 + cache-read 4,632,484 (96.2% cache-read). FLOOR - written before the session ends. Same method as SPEC-001's verify-1/2/3 and build-2/3/4; NOT comparable to build-1's 197,940 (token-counts-not-comparable). Cost transcribed by the tool, never by hand: ran `just handback-sync SPEC-006`, which stamped both HANDOFF-007 (build, 5,121,192) and HANDOFF-009 into cost.sessions and set synced_at - hand-appending double-counts (settled on SPEC-001, HANDOFF-004)."
  totals:
    tokens_total: 9935949
    estimated_usd: 0.00
    session_count: 2
shipped_at: 2026-08-20
---

# SPEC-006: Close the allow-attribute bypass in the panic-free gate

## Context

SPEC-001 shipped a panic-free lint policy enforced by a red-proof that is itself
proven red (`DEC-009`). Its verify round 3 found — and the orchestrator
independently reproduced — that **the policy is exited by a single attribute**:

```rust
#[allow(clippy::panic, clippy::expect_used)]
pub fn boom(v: &[u8]) -> u8 { if v.is_empty() { panic!("empty") } *v.first().expect("x") }
```

`BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0` — **seven
green gates, two panics on the public API, no module involved.**

`DEC-009`'s red-proof **structurally cannot** close this: it mutates the crate
root and asserts the root's `#![deny(...)]` rejects the injection. No `#![deny]`
mutation test can observe an `#[allow]` beneath it. That is why this is a separate
spec rather than another round on that script — a fourth iteration of the same
mechanism was explicitly rejected in `DEC-009`.

Split out of SPEC-002 at SPEC-001's ship: the two share **no files** (that spec
touches `tests/**`; this one touches `scripts/`, `.github/workflows/` and
`guidance/`), and this hole is live *today* with no module, so it never depended
on SPEC-002's work.

## Goal

Make `no-panics-on-untrusted-input` mechanically true rather than nearly-true: an
`#[allow]` of any policy lint, anywhere outside the sanctioned exceptions
(`#[cfg(test)]` modules and `src/bin/`), must fail CI.

Then correct `guidance/constraints.yaml:33`, whose `enforcement:` field currently
reads as a stronger guarantee than holds (verify round 3, F-4).

## Inputs

What the implementer will read or consume.

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth requirements>
- **Related code paths:** `src/some/module/`

## Outputs

What the implementer will produce.

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New endpoints / functions / components:** <names and signatures>
- **New flags / options:** each flag's accepted values **and its default** — an
  unstated default makes the implementer guess.
- **Database changes:** <migrations, if any>

## Acceptance Criteria

1. A CI job runs the **forbid check** below and fails on any `#[allow]`/`#![allow]`
   of a policy lint in the library.
2. The sanctioned exceptions still pass untouched — `#[cfg(test)]` modules and
   `src/bin/irr.rs` keep their existing allows with no per-site special-casing.
3. **Shown RED:** planting `#[allow(clippy::panic)]` on a `pub fn` in `src/lib.rs`
   turns the job red, and that proof ships with the change.
4. **Shown GREEN on the honest tree:** the same command exits 0 on unmodified
   `main`. (The lesson DEC-009 cost three rounds — "fails on X" is meaningless
   without "passes without X".)
5. `guidance/constraints.yaml:33`'s `enforcement:` states what is now actually
   enforced — no more, no less.
6. All existing gates stay green.

## Failing Tests

Red today, green after build. **`-F` is `--forbid`: a forbidden lint cannot be
re-allowed in source, and attempting it is compiler error `E0453`.**

```bash
# THE GATE — must exit 0 on the honest tree, non-zero with any #[allow] planted
cargo clippy --lib --quiet -- \
  -F clippy::unwrap_used -F clippy::expect_used -F clippy::indexing_slicing \
  -F clippy::panic -F clippy::arithmetic_side_effects
```

**The red-proof** — plant this in `src/lib.rs` and the gate must fail:

```rust
#[allow(clippy::panic, clippy::expect_used)]
pub fn boom(v: &[u8]) -> u8 { if v.is_empty() { panic!("e") } *v.first().expect("x") }
```

Expected, measured at design:

```
error[E0453]: allow(clippy::panic) incompatible with previous forbid
error[E0453]: allow(clippy::expect_used) incompatible with previous forbid
exit 101
```

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- ...

## Notes for the Implementer

### The mechanism is decided and measured — transcribe it

**`cargo clippy --lib -- -F <each policy lint>`.** All three properties verified
at design on the real crate:

| run | result |
|---|---|
| `#[allow]` planted on a `pub fn` | **exit 101**, `E0453: allow(clippy::panic) incompatible with previous forbid` |
| honest tree | **exit 0** — the negative control holds |
| `--all-targets` instead of `--lib` | exit 101 — because tests legitimately allow |

That last row is *why* the scope is `--lib`: it excludes `#[cfg(test)]` modules
(not compiled without `cfg(test)`) and `src/bin/irr.rs` (a different target), so
criterion 2 needs no per-site special-casing at all.

### Why this beats the text search everyone reaches for first

**There is no text matching, so the `attribute-text-inside-doc-comments` lesson
(N=5) cannot bite.** `src/lib.rs` contains attribute text inside its own module
docs; a grep would have to exclude `//`, `//!`, `/* */` and anchor at column 0,
and five separate attempts on SPEC-001 got that wrong — each producing a wrong
*answer* rather than an error.

`E0453` is also strictly stronger than detecting a violation: it forbids the
**escape hatch itself**, firing on the attribute whether or not the code beneath
it actually panics.

### Two things worth knowing

- **`--force-warn` is the wrong tool here**, though it looks right. It *does*
  override `#[allow]` (measured: 2 warnings at the planted line) but `-D warnings`
  **cannot promote a force-warn diagnostic to an error**, so the exit code stays
  **0**. A gate built on it would need output parsing — reintroducing exactly the
  text-matching fragility this design avoids.
- **Scope honestly.** This covers the `--lib` target. It is not a claim about
  every future target, and `constraints.yaml` (criterion 5) should say so rather
  than overstate again — F-4 was raised because the last wording did.

### Scope discipline

One gate, one CI job, one corrected sentence. **Do not** touch
`scripts/lint-red-proof.sh` — `DEC-009`'s red-proof is sound for what it pins,
and this gate covers the part it structurally cannot. They are complementary.

## Reflection

**1. What would I do differently next time?**

Nothing about the mechanism — the design-time probe settled `-F` before build and
build was a transcription. That is the pattern working exactly as AGENTS.md §12
describes, and it is the contrast with SPEC-001, where three rounds went into a
mechanism nobody had measured first.

What I would do differently is **specify the red-proof's placement**. My spec said
what to plant and not *where*; planting after the `#[cfg(test)]` module trips
`clippy::items_after_test_module` and reddens CLIPPY for an unrelated reason,
which would have *understated* the hole. Both the builder and the reviewer hit it
independently (F-6). Placement is part of a reproduction, not an implementation
detail.

**2. Does any template, constraint, or decision need updating?**

Three, all filed rather than fixed here:

- **F-1 is the real one:** the gate's own flags are unpinned. Swapping all five
  `-F` → `-D`, or dropping a single `-F`, restores the original hole with **eight
  green gates and nothing in CI noticing**. That is `DEC-009`'s thesis one level
  up — the thing that checks the policy can itself be silently weakened. See §3.
- **F-5 — two owners, one field.** `AGENTS.md` §15 tells the agent to append the
  cost session; `DEC-013` says `handback-sync` transcribes it. Both cannot be
  authoritative, and this cycle had to discover which by finding `cost.sessions`
  empty. A template-level contradiction.
- **F-2/F-3 — `constraints.yaml:33`** omits feature configuration (no-op today,
  live when `DEC-002`'s `std` feature lands) and *understates*: with the crate-root
  `#![deny]` deleted and no attribute anywhere, the gate still exits 101, because
  the `-F` flags re-impose the whole policy on `--lib`. Understating is the
  conservative direction, so: follow-up, not a defect.

**3. Is there a follow-up spec to write now?**

**Deliberately not yet — and that judgement is the point.** F-1 is real, but each
gate we have built closes the previous one's hole, and I am not going to spawn
SPEC-007 reflexively. The threats are not equivalent:

- `#[allow]` on a `pub fn` (SPEC-006) is a **one-line source change that looks
  innocuous** in review. Worth a mechanism.
- Editing the gate's own `-F` flags (F-1) means **changing the gate's definition
  in `ci.yml`/`app.just`** — visible in any diff, and the sort of thing review is
  actually good at.

So F-1 is recorded as a signal with that reasoning, not converted into a spec.
The open question — *where does this recursion stop?* — belongs to the maintainer
at project close, not to another round of me deciding alone. My current position:
these gates protect against **accident and drift**, not against an adversary with
commit rights, and `constraints.yaml` should eventually say so in those words.


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
