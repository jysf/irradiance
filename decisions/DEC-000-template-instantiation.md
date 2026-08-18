---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-000
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.90                   # see "Confidence" below — the observations are
                                     # facts; the dispositions are choices; the
                                     # cleanups are prescribed but not yet done.
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5                  # NB: .repo-context.yaml tier_map still names
                                     # claude-opus-4-7 — see "What still needs doing"
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-15
supersedes: null
superseded_by: null
status: accepted                     # proposed | accepted | rejected | deprecated | superseded
deciders: [jysf, claude]

# This decision governs the repo's process surface — the files that say how work
# is done here, as opposed to any code.
affected_scope:
  - AGENTS.md
  - .repo-context.yaml
  - guidance/constraints.yaml
  - decisions/**
  - feedback/**

# NB: `just decisions-audit` warns that the two bare globs below "likely match
# nothing". That warning is a FALSE POSITIVE — verified with
# `just decisions-audit --changed`, which matches `AGENTS.md` correctly, because
# the matcher compares globs against repo-relative paths where a root-level file
# IS a bare name. Do NOT "fix" it by writing `**/AGENTS.md`: that glob is strictly
# worse (it would also match a nested one). See finding 9 in
# feedback/2026-08-15-irradiance-scaffold.md.

tags:
  - process
  - template
  - instantiation
  - provenance
---

# DEC-000: How the spec-driven template was instantiated as `irradiance`

## Decision

`irradiance` runs the spec-driven template (v0.6.38, `claude-plus-agents`
variant) **with its work hierarchy, cycle model, lane structure and cost
discipline intact**, and with five deliberate deviations recorded here:
`semver` instead of the default `calver`; five domain constraints replacing the
template's app-shaped seeds; the spike lane used **before** any stage is framed;
`MIT OR Apache-2.0` dual licensing; and `AGENTS.md` rewritten so that every
statement in it is true of this repo. Template-level findings from the
instantiation are captured separately in
[`feedback/2026-08-15-irradiance-scaffold.md`](../feedback/2026-08-15-irradiance-scaffold.md),
which is the template's own established channel for them.

This DEC is numbered **000** because it precedes every decision about the
library itself. It is the record of the ground the rest stands on.

## Context

One of the template's stated purposes is to find out whether it generalises.
Every full-tier instance in `docs/harvests/instances.md` to date is an **app or a
CLI**, and the only prior `claude-plus-agents` instance (`uw`) is recorded there
as *"dead — abandoned, nothing to harvest."* `irradiance` is therefore the first
instantiation that tests two things at once: the template against a **library**,
and the plus-agents variant against a live project.

That makes the instantiation itself a deliverable. Instantiation friction is
observable exactly once and is then invisible forever — by the time a project has
shipped a stage, nobody remembers which defaults were wrong on day one. This is
written at scaffold time for that reason.

**Why the findings are split across two files.** `docs/harvests/instances.md`
shows the established pattern: its *"Insights captured in"* column points at
`feedback/<date>-<instance>-<project>.md` files inside the instances themselves,
which the template maintainer then harvests. `feedback/` is therefore the
intended home for the **generalisable** half, and this DEC holds the half that
binds only `irradiance`.

## What travelled unchanged

These needed no adaptation and are load-bearing here:

- **The work hierarchy** — repo → project → stage → spec → handoff, with
  repo-wide continuous IDs. Correct for a library without modification.
- **The five-cycle model and its three lanes** — spec (`frame → design → build →
  verify → ship`), patch (`patch → verify → ship`), spike (`spike → land`).
- **The independent verify cycle.** Named across the dogfood as the single best
  quality lever, and it is doubly right here: a decoder's failure mode is output
  that looks plausible, which is precisely what an author cannot see in their own
  work.
- **Cost tracking discipline** and the handoff/handback contract (DEC-013). The
  plus-agents variant is metered on delegated cycles, and the gate is enabled
  (`.repo-context.yaml` → `cost.metering_source: subagent_tokens`).
- **Design-time probe / measure-before-build** (AGENTS.md §12). It arrived from
  crustyimg's experience with library APIs; in this repo it becomes *probe the
  real bytes* — read the actual file and close the arithmetic before writing an
  unpacker. Better suited to this domain than to the one it came from.
- **"Ship the reader with the field."** Generalises cleanly and sharply: a parsed
  tag with no consumer is not parsed.
- **The DEC log itself**, and supersession rather than deletion.

## What did not travel, and what replaced it

### 1. `version.scheme`: `calver` → `semver`

The template defaults to `calver`
(`variants/claude-plus-agents/.repo-context.yaml:60` as shipped) and DEC-007
justifies it as needing *zero judgment*. Sound for an app; inverted for a
library. A library's version number is a **compatibility claim consumers depend
on mechanically** — `^0.3` resolves, `v2026.08.0` does not. Changed to `semver`
at `.repo-context.yaml:60`.

### 2. Constraints: app-shaped seeds → five domain constraints

The template shipped `use-project-logger` and `no-auth-changes-without-approval`
— meaningless in a library with no logging framework and no auth. Both were
removed. `guidance/constraints.yaml` now carries five blocking constraints
written for this repo, and **that file wins** over `AGENTS.md`, which explains it:

1. `no-panics-on-untrusted-input`
2. `provenance-recorded-per-algorithm`
3. `no-copyleft-dependencies`
4. `library-not-application`
5. `oracle-must-be-shown-red`

The generic three (`no-secrets-in-code`, `test-before-implementation`,
`one-spec-per-pr`) were kept as-is.

**Consequence to watch:** `use-project-logger` had a referrer — the seeded
`decisions/DEC-001-example-structured-logging.md` cites it at lines 81 and 103.
Removing the constraint without removing its referrer left a dangling reference
that `just decisions-audit` reports as **clean**, because it lints structure, not
referents. Seeded artifacts that cite each other must be removed together.

### 3. The spike lane runs before any stage is framed

The template's spike lane (DEC-012) is used exactly as designed and **before**
`just frame-stage` is ever run. `SPIKE-001` asks whether a Q2M DNG can be decoded
bit-exact *and whether the oracle discriminates*, and carries 11 open questions
whose answers change PROJ-001's spec breakdown — including measured LOC that is
meant to replace the stage files' estimates.

Framing STAGE-001 before that spike lands would be framing ahead of what is
knowable. Recorded in `AGENTS.md` §8 as a standing rule for this project.

This is also the template's clearest **win** at instantiation, and it is written
up in full in the feedback capture: an external plan had independently designed a
"STAGE-000 spike stage" from scratch to hold exactly this exploration, and the
template's existing primitive is strictly better — a spike attaches to the repo
rather than a project, so it can precede the project it informs;
`test-before-implementation` is principled-suspended rather than violated; and
`inconclusive` is a valid outcome, so an expired timebox produces a result rather
than pressure to extend.

### 4. Licence: `LICENSE` → `LICENSE-MIT` + `LICENSE-APACHE`

`scripts/scaffold-clean.sh` deliberately never auto-deletes `LICENSE` (removing
it could leave a repo unlicensed) and flags it for review instead. Reviewed:
this library is `MIT OR Apache-2.0`, the Rust ecosystem convention and
crustyimg's. Apache-2.0 alone would be more restrictive than the norm and would
undercut the entire pitch — **permissive is the differentiator**, and the whole
reason this library exists is that every mature alternative in every language is
copyleft or C++. Recorded in `docs/provenance-ledger.md`.

### 5. `AGENTS.md` rewritten (this session)

As scaffolded it was 739 lines carrying **29 `[REPLACE]` markers** across nine
sections, describing a generic app: a database, a dev server, a `src/` tree, a
logging convention. Rewritten to 1,142 lines with zero markers, preserving the
§1–§17 numbering because external files reference it by section number
(`.github/workflows/ci.yml`, `scripts/_lib.sh`, `scripts/cost-audit.sh`,
`docs/cost-tracking.md`, `projects/_templates/*`, `guidance/*` all cite
`AGENTS.md §N`).

What was added that the template had no slot for: the library/binary split
(`irradiance` vs `irr`), the oracle red-proof discipline, the provenance rules,
fuzz-from-the-first-parser-spec, the two-tier corpus, the measured toolchain
table, and a DNG/RAW glossary.

## Alternatives Considered

- **Option A: fork or hand-roll the process instead of instantiating the
  template.**
  - Why rejected: the four disciplines this repo rests on (panic-free parsing,
    red-provable oracles, per-algorithm provenance, permissive-only deps) all
    need a **decision log and an independent verify cycle** to survive contact
    with a year of work. Both already exist here and are proven across three
    prior projects. Rebuilding them to avoid ~4,800 lines of leftover
    maintainer docs is a bad trade.

- **Option B: instantiate and adapt nothing — treat the defaults as the
  convention.**
  - Why rejected: it would ship `calver` on a library, a constraint file
    demanding a project logger in a repo whose coding convention is *no logging
    at all*, and an `AGENTS.md` that tells a build agent to run `npm test`. The
    first delegated build cycle would burn its loops on that, and the fifth rule
    of DEC-004 exists precisely because cold agents trust what they are given.

- **Option C: instantiate, adapt the defaults, and record the friction in
  `feedback/` (chosen).**
  - Why selected: it keeps every mechanism that earned its place, fixes the
    defaults that are wrong for a library, and returns the findings through the
    channel `docs/harvests/instances.md` already points at. The template's stated
    purpose is to discover whether it generalises; an instance that adapts
    silently answers nothing.

## Consequences

- **Positive.** The process surface is now true rather than aspirational: an
  agent reading `AGENTS.md` cold gets this repo's real toolchain, real
  constraints and real commands. `guidance/constraints.yaml` is the single
  authority and `AGENTS.md` explains it, so the two cannot drift into rival rule
  sets. The plus-agents blind spot in the template's own registry starts closing.

- **Negative.** This repo is now a **deviated instance**: `semver`, a rewritten
  `AGENTS.md`, and replaced constraints all mean a future template update cannot
  be applied wholesale. Diffing an upstream `AGENTS.md` against this one will be
  manual. That cost is accepted — the alternative was an `AGENTS.md` that lies.

- **Negative.** ~4,825 lines / ~296 KB of template-maintainer content
  (`docs/blog/`, `docs/talks/`, `docs/sessions/`, `docs/harvests/`,
  `docs/ROADMAP.md`, other instances' `feedback/`, two stale `reports/`) are
  committed here permanently. They are prior art, not this repo's history.
  `AGENTS.md` §17 lists them so no agent mistakes them for facts about
  `irradiance`.

- **Neutral.** Two DEC namespaces now coexist — `/decisions/` (this repo's) and
  `/docs/decisions/` (the template's `DEC-001`…`DEC-013`, which `AGENTS.md` cites
  as live process rationale). They collide by ID string, and `scripts/_lib.sh:56-64`
  makes only `/decisions/` visible to tooling in an instance. `AGENTS.md` §10
  requires disambiguation by path. The proper fix is upstream, and is in the
  feedback capture.

- **Neutral.** This DEC takes number **000**, leaving `DEC-001` free — which
  matters because the seeded example decision currently squats on it.

- **Neutral.** `just decisions-audit` emits two scope warnings against this file.
  They are false positives, verified against `--changed`; the note in the
  front-matter says why, so a later session does not "fix" a correct glob into a
  worse one.

## What still needs doing

Four of the six were completed **2026-08-16**; the remaining two are gated on
artifacts that do not exist yet, as noted below.

1. ✅ **Done 2026-08-16** — deleted `decisions/DEC-001-example-structured-logging.md`
   (`repo: my-app`, cited the deleted `use-project-logger` constraint, and squatted
   on `DEC-001`).
2. ✅ **Done 2026-08-16** — deleted `docs/api-contract.md` and `docs/data-model.md`,
   template stubs for an external API and persistent data this library has neither
   of.
3. ✅ **Done 2026-08-16** — `guidance/toolchain-brief.md` populated with measured
   facts, leading with the `cargo +nightly` trap (DEC-004 rule 5 injects it into
   every build handoff).
4. ⏳ **Gated** — `docs/architecture.md` waits until the module layout is real
   (after SPIKE-001, with STAGE-001's first spec). Writing it now would be
   inventing a structure the spike is meant to inform.
5. ⏳ **Gated** — the Rust CI jobs (`fmt --check`, `clippy -D warnings`, `test`,
   `cargo deny check licenses`, fuzz smoke) land with the first `Cargo.toml`.
   `.github/workflows/ci.yml` carries only the language-agnostic gates today.
6. ✅ **Done 2026-08-16** — `.repo-context.yaml` → `spec.agent.tier_map` reconciled
   to `claude-opus-5` (design/verify) and `claude-sonnet-5` (build). The tier
   *intent* is unchanged; only the ids are now real. DEC-004 rule 3 depends on that
   map being accurate — a stale map is the silent cost surprise the rule exists to
   prevent, because the orchestrator stamps it into every handoff.

Added since, and also done 2026-08-16:

7. ✅ **`AGENTS.md` §5 no longer contradicts `DEC-002`.** DEC-002 (`proposed`, 0.72)
   proposes `no_std` + `alloc`, no `rayon`, and pinned determinism; §5 had flatly
   said "standard library". It now records the target surface as **open** and names
   what not to assume until the DEC resolves.

## Validation

This decision was right if, at PROJ-001's close:

- No spec had to be re-litigated because `AGENTS.md` and
  `guidance/constraints.yaml` disagreed.
- No delegated build cycle was lost to a false statement in `AGENTS.md` — the
  measurable version: **zero** build handbacks reporting a toolchain or command
  that did not exist.
- The five domain constraints were each cited by at least one real spec. A
  constraint no spec ever references is decoration, and should be deleted rather
  than admired.
- The spike-before-frame ordering paid: STAGE-001's spec breakdown differs from
  what would have been framed before SPIKE-001 ran. If it does **not** differ,
  that is a finding — the spike was less informative than assumed, and the next
  project should frame earlier.

Revisit if:

- The template ships a fix for any item in the feedback capture and the deviation
  here becomes unnecessary (notably `semver` at init, and the DEC namespace
  collision).
- A second project in this repo finds the constraints app-shaped in the other
  direction — too decoder-specific to govern, say, a colour pipeline.

## Confidence

**0.90.** The observations are verified facts, each cited to a file and line and
each checked by running the tooling rather than reading about it. The
dispositions (semver, five constraints, spike-first, dual licence) are considered
choices with stated reasoning. The 0.10 is the six items under "What still needs
doing": until they are done, this record describes an intent as well as a state,
and the gap between those is exactly where process records rot.

## References

- Template-level half of these findings:
  [`feedback/2026-08-15-irradiance-scaffold.md`](../feedback/2026-08-15-irradiance-scaffold.md)
- Rewritten conventions: [`AGENTS.md`](../AGENTS.md) (§1, §5, §8, §10, §12, §13, §17)
- Rules this explains, and which override it: [`guidance/constraints.yaml`](../guidance/constraints.yaml)
- Template decisions relied on (note the namespace — `/docs/decisions/`):
  DEC-003 (patch lane), DEC-004 (sub-agent execution), DEC-005 (agent
  portability), DEC-007 (versioning default), DEC-012 (spike lane), DEC-013
  (delegated cost handback)
- Instance registry that names the plus-agents blind spot:
  [`docs/harvests/instances.md`](../docs/harvests/instances.md)
- This repo's own licence rationale: [`docs/provenance-ledger.md`](../docs/provenance-ledger.md)
- The exploration this defers framing to:
  [`spikes/SPIKE-001-…`](../spikes/SPIKE-001-can-we-decode-a-leica-q2-monochrom-dng-bit-exact-and-does-the-oracle-discriminate.md)
