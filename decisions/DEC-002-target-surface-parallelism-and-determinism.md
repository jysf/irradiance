---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-002
  type: decision
  confidence: 0.72
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-15
supersedes: null
superseded_by: null

status: proposed
deciders: []

tags:
  - architecture
  - api
  - determinism
  - wasm
---

# DEC-002: target surface, parallelism, and determinism — decided together

> ⚠ **A shipped spec now depends on this being unresolved.** `SPEC-012`'s
> `DEC-016` chose a caller-owned buffer (`unpack_into(&mut [u16])`, no allocation
> in the library) **specifically because this decision is still `proposed`** — a
> `Vec`-returning primitive would commit the library to allocating 95 MB on the
> caller's behalf before the `no_std`/`alloc` question is settled.
>
> ⚠ **And `SPEC-012`'s verify measured a second constraint this decision must
> account for** (`SPEC-012/FU-4`): peak RSS for a 47 MP decode is **182,435,840
> bytes**, reproduced to the byte, because `unpack_into` indexes the whole file
> at **absolute offsets** — so the caller must hold the entire file addressable
> alongside the 94.9 MB plane. `mmap` is the escape and is currently
> undocumented. Whichever way this decision lands, that contract is part of it.

## Decision

**Proposed, not accepted.** Three questions that look separate are one question,
and answering any of them alone bakes in an answer to the other two. This DEC
states the coupling and proposes an answer; SPIKE-001 measures the cost before it
is accepted.

**1. `irradiance` is `no_std` + `alloc` where it can be, and never assumes threads.**
The decode and develop path takes a byte slice and returns buffers. It performs no
I/O. `std` is permitted only behind a default-on `std` feature, for error
ergonomics — never on the algorithmic path.

**2. Parallelism is the caller's choice, not the library's.** No `rayon` dependency.
The library exposes work in units a caller can parallelise (per-row, per-tile) and
provides a `parallel` feature *later* if measurement justifies it. `crustyimg`
already owns a rayon batch layer; a second, inner parallelism is not obviously a
win and is definitely a determinism hazard.

**3. Output is deterministic within a declared `develop_version`.** Table-driven
tone curves rather than `powf`, pinned reduction order for anything that
accumulates, no runtime SIMD dispatch. Algorithm improvements ship as a *new*
version; old versions keep rendering as they did.

## Context

### Why these are one decision

They collide pairwise, so deciding any one in isolation silently decides the others:

- **wasm × parallelism.** `wasm32-unknown-unknown` has no threads by default. A
  `rayon` dependency on the algorithmic path forecloses the browser target, which
  is the project's most distinctive potential artifact — nobody has a client-side
  RAW developer, and every web RAW tool uploads your files.
- **parallelism × determinism.** Stencil operations (unpack, demosaic, per-pixel
  colour) are deterministic under any parallelism. **Reductions are not** —
  histogram-driven auto-tone, any accumulate. So "add rayon" is safe for some
  future ops and unsafe for others, and the distinction has to be designed in
  rather than discovered.
- **determinism × wasm.** `powf`/`exp` differ in their last bits across libm
  implementations *and* across targets. A tone curve built on `powf` cannot be
  byte-identical between macOS, Linux and wasm. Table-driven curves are the fix,
  and they are cheap only if adopted before the curve exists.

### Why determinism at all, when the library makes no such promise

The library deliberately does **not** promise byte-stability — that would freeze
the algorithm, which is fatal for a developer whose whole job is to get better.
But `crustyimg` consumes this behind `raw-develop`, and its `build --frozen` is a
shipped guarantee. A non-deterministic develop flowing through it produces
spurious lockfile drift on someone's CI.

Process versions resolve the tension, and every mature developer in this space
reached the same answer (Lightroom, Capture One, darktable). Deterministic
*within* a declared version; improvements are a new version; the recipe and
lockfile record which one produced the bytes.

### Why `no_std` is proposed rather than assumed

`demosaic` 0.3.0 is `no_std` with zero dependencies, which is evidence the
algorithmic layer of this domain does not need `std`. The parser is a different
question: bounded slice reads want no allocation beyond output buffers, but error
types are more ergonomic with `std::error::Error`. Hence `std` as a default-on
feature rather than an outright ban.

## Alternatives Considered

- **Take `rayon` now and accept the loss of wasm.** Rejected for now. A 47 MP
  develop is embarrassingly parallel and the temptation is real, but the caller
  can parallelise across *images* — which is what crustyimg already does — and
  that captures most of the benefit for batch work without foreclosing anything.
  Revisit if single-image latency measures badly in SPIKE-001.
- **Promise byte-stability outright.** Rejected. It would freeze the demosaic, the
  tone curve, and every future improvement. Recorded in the brief as an explicit
  non-goal.
- **Ignore determinism and let crustyimg deal with it.** Rejected. crustyimg
  cannot fix a non-deterministic upstream; the cheapest place to hold the property
  is where the floating-point work happens.
- **`std` everywhere; skip wasm.** Tenable, and it may be where this lands if
  SPIKE-001 shows `no_std` costs real ergonomics. But the browser artifact is the
  project's most differentiated marketing surface and the cost of keeping the door
  open now is close to zero.

## Consequences

- **Positive:** the browser target stays open at near-zero cost; `build --frozen`
  stays meaningful; the reduction-versus-stencil distinction gets designed in
  rather than discovered when a test goes flaky.
- **Negative:** no in-library parallelism means single-image develop is slower than
  it could be; `no_std` costs some error ergonomics; table-driven curves cost a
  little accuracy versus `powf` and some setup code.
- **Neutral:** `develop_version` is one field, cheap on day one and an ugly
  retrofit later.

## Validation

**Right if** SPIKE-001 measures single-image develop latency as acceptable without
in-library parallelism; the same DNG produces byte-identical output on macOS and
Linux; and a `wasm32` build of the plane path compiles without contortions.

**Wrong if** `no_std` forces awkward error handling through the whole public API,
or if single-image latency is bad enough that callers would rather have inner
parallelism than the browser target. Either finding should reopen this before
STAGE-002 rather than after.

**Also wrong if** nobody ever builds the browser artifact — then `no_std` bought
nothing and cost ergonomics. That is a judgement to revisit at PROJ-002, honestly,
not to defend out of consistency.

## References

- SPIKE-001 questions 6, 7, 12, 13 — the measurements that accept or reject this
- `docs/oracle-contract.md` — byte-identity is one of the oracle's checks
- `projects/PROJ-001-*/brief.md` — determinism is listed as an explicit non-goal
  for the library and a shipped guarantee for crustyimg
- crustyimg `DEC-006` (`no-async-runtime`, rayon for batch) — the consumer's
  existing parallelism layer
