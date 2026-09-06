---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-021
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-05
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - tests/develop_oracle.rs

tags:
  - testing
  - oracle
  - red-proof
  - develop
  - spec-015
---

# DEC-021: the develop oracle's two red-proofs use DIFFERENT mechanisms, on purpose

## Decision

`tests/develop_oracle.rs`'s two `AC5`/`AC6` red-proofs do not share one
mechanism, because the two faults are different KINDS of fault:

- **The levels fault** (`the_oracle_is_red_on_a_levels_fault`, `BlackLevel +
  64`) is injected **in-process, with no source mutation at all**:
  `BlackLevel` is a public `Sensor` field that `develop_into` (unmodified,
  already linked into the test binary) reads directly, so calling it with a
  `Sensor` carrying the wrong value fully reproduces the fault's effect —
  regardless of what code path would have PRODUCED that wrong value in a
  real bug (a tag-parsing error, a hand-off mistake, anything). No rebuild is
  needed or possible to skip.
- **The orientation fault** (`the_oracle_is_red_on_an_orientation_fault`,
  identity at `crop_source_coords`' call site — `SPEC-014/FU-3`'s historical
  bug) is injected by **copying the crate to a temp dir, textually mutating
  the copy's `src/develop.rs`, and rebuilding a small synthesized probe
  binary** — following `DEC-017`'s precedent for exactly this reason: it is a
  call-site defect inside a private function, expressible by no public
  `Sensor` field, and Rust has no way to swap compiled behaviour at runtime.

Both keep the working tree's `src/develop.rs` untouched (`AC7`) and both run
with `IRRADIANCE_CORPUS_DIR` unset, over hand-built fixtures (`AC6`).

## Context

`SPEC-013`'s plane oracle (`DEC-017`) established the mutate-copy-rebuild-run
mechanism for a fault in `unpack_into`'s compiled behaviour on a raw byte
stream, and explicitly rejected "perturb an in-memory value" for that case
because the plane oracle's subject is compiled behaviour, not a value
already in memory to flip.

`develop_into`'s situation is split down the middle. Its PUBLIC API is
`(&Sensor, &[u16], &mut [u16])` — already-parsed metadata and an
already-unpacked plane, both plain data. A fault that is fully described by
ONE WRONG FIELD VALUE (the levels fault) is exactly `DEC-017`'s rejected
Option A's use case (`tests/metadata_oracle.rs`'s own red-proof shape: flip
one field, diff), because the "compiled behaviour" argument does not apply —
`develop_into`'s use of `black_level` is a straight field read, not internal
control flow. A fault that is a CALL-SITE defect (the orientation identity
bug) is squarely `DEC-017`'s kept rationale: no public field expresses "the
code ignored this value's meaning at one specific line", so proving the
oracle catches it needs the real, compiled, mutated function.

Treating both faults the same way — either forcing the levels fault through
an unnecessary rebuild, or trying to fake the orientation fault via a
hand-written "what would the buggy code have produced" model computed
in-process — would either waste the ~1-2s `cargo build` cost twice for no
reason, or produce a WEAKER proof (a model of the bug's output, not the
bug's actual compiled output). Matching the mechanism to the fault's own
shape gets the strongest available proof for each at the lowest cost each
allows.

A useful side effect: unlike `SPEC-013`'s red-proof probe (which must
hand-build actual TIFF/DNG bytes, since `plane::unpack_into` takes raw file
bytes), `develop_into`'s probe binary needs no fixture FILE at all — the
`Sensor` and the tiny plane are Rust literals in the probe's own `main()`,
so there is no file I/O anywhere in either red-proof.

## Alternatives Considered

- **Option A: mutate-and-rebuild for both faults, for mechanical symmetry.**
  - Why rejected: the levels fault needs no rebuild to be fully and
    faithfully reproduced, so doing one anyway adds a `cargo build --release`
    (~1-2s, `DEC-017`'s own measurement) for a fault a plain field
    assignment already proves, with no gain in rigor — arguably a LOSS, since
    an in-process test is decoupled from any one hypothesized source of the
    wrong value.
- **Option B: an in-process "fault model" for the orientation bug — compute
  what the buggy code WOULD have produced by hand, in the test file, without
  touching `src/`.**
  - Why rejected: this tests the model, not the actual compiled
    `develop_into`. A future refactor of the call site that preserves the
    BUG in some structurally different form could leave a hand-written model
    green while the real code stays broken — the same "reimplements the
    subject" trap `DEC-017`'s Option A rejection already names.
- **Option C (chosen): match the mechanism to the fault's own shape.**
  - Why selected: each fault gets the mechanism its own nature requires, at
    the minimum cost that mechanism allows — no rebuild where none is
    needed, a real rebuild where nothing else would be faithful.

## Consequences

- **Positive.** Both red-proofs are as strong as `SPEC-013`'s (a real fault,
  in the real compiled function or the real linked one, never a model of
  either), and the levels one is nearly free (no subprocess at all).
- **Positive.** The orientation red-proof's probe needs no fixture file,
  unlike `SPEC-013`'s — one less thing that can drift from the in-process
  comparison it is checked against.
- **Negative.** A reader has to understand TWO mechanisms instead of one to
  follow `tests/develop_oracle.rs`'s red-proof section, rather than one
  mechanism applied twice. This record and the module doc's cross-references
  are the mitigation.
- **Negative — `SPEC-015/FU-7`.** The orientation red-proof's fault (identity
  at `crop_source_coords`' call site) reads outside the crop window on its
  6-pixel fixture, so the mutant is a DIFFERENT multiset (three zeros the
  honest tree never produces), not merely a WRONG permutation of the honest
  tree's own multiset. `AC3`'s permutation property therefore goes red here on
  degeneracy, never on the narrower claim its name suggests — "the wrong
  permutation was applied." That narrower claim cannot be proven by any
  rank/frequency check at all (`DEC-020`'s own inherent limit,
  `SPEC-015/FU-6`); this red-proof is sound for the fault it actually injects,
  and the scope gap is a naming/documentation issue, not a weaker proof than
  advertised for what it tests.
- **Neutral.** The mutate-copy-rebuild-run helpers (`TempDir`,
  `copy_dir_recursive`, `stage_probe_crate`, `build_and_run_probe`) are
  duplicated from `tests/plane_oracle.rs` rather than factored into a shared
  `tests/support/` module, matching that file's own choice not to share this
  machinery with anything — `tests/plane_oracle.rs` is left untouched
  (`SPEC-013`'s tests keep passing unmodified), and a shared abstraction for
  a mechanism used by exactly two call sites, one of which needed to stay
  frozen, was judged not worth the coupling.

## Validation

**Right if** a future spec adding a THIRD red-proof for `src/develop.rs` (or
elsewhere) can look at both mechanisms here, correctly classify its own
fault as "a wrong field value" or "a call-site/control-flow defect", and
reuse the matching one without re-deriving the distinction.

**Wrong if** a fault shows up that is neither — expressible by no public
field AND not a code-path defect (for instance, a fault only reachable
through some interaction between two structs neither of which alone carries
it) — revisit which of the two mechanisms actually fits, or whether a third
is needed, rather than forcing it into the nearer of the two.

## References

- Related specs: `SPEC-013`, `SPEC-014` (the orientation fault's origin,
  `FU-3`)
- Related decisions: `DEC-017` (the plane oracle's own mechanism decision,
  and the origin of the mutate-copy-rebuild-run apparatus this reuses),
  `DEC-009` (a red-proof needs a negative control — `the_orientation_fixture_oracle_control_is_green`
  is that control for the mutation half; the levels fault's own honest-tree
  assertion is its control), `DEC-020` (the oracle's own property-set
  mechanism, a separate concern from how each fault is INJECTED)
