---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-016
  type: decision
  confidence: 0.85
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

created_at: 2026-09-04
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/plane.rs

tags:
  - decode
  - unpack
  - allocation
  - spec-012
---

# DEC-016: `unpack_into` writes a caller-owned buffer; it does not allocate

## Decision

The sensor-plane unpacker's public entry point is
`unpack_into(sensor: &Sensor, byte_order: ByteOrder, file: &[u8], dst: &mut
[u16]) -> Result<(), Error>`. It performs **no allocation** — `dst`'s length is
checked against `width × height` and rejected if it does not match; the
function never grows, shrinks, or replaces the buffer. A convenience that
returns an owned `Vec<u16>` is not provided by this spec and can be added
later as a thin wrapper without changing this signature.

## Context

`SPEC-012` is the first spec that produces pixels. A Q2 Monochrom plane is
`8424 × 5632 × 2 = 94,887,936` bytes — real memory, on top of an ~86 MB input
already held by the caller. `library-not-application` (`guidance/constraints.yaml`)
says the consumer opens the file and picks the allocator; `DEC-002` (`status:
proposed`, confidence 0.72) proposes `no_std` + `alloc` with `std` behind a
default-on feature, and is not yet decided.

So the API shape is a real commitment, not a detail: whichever primitive ships
first is the one every caller (including `crustyimg`) will be built against.

## Alternatives Considered

- **Option A: `unpack(&self) -> Result<Vec<u16>, Error>`.**
  - What it is: the unpacker allocates and returns an owned plane.
  - Why rejected: convenient, but it commits the library to allocating ~95 MB
    on the caller's behalf on every call, with no way to opt out. It also
    requires an allocator unconditionally, which forecloses `DEC-002`'s
    `no_std` option before that decision is made — the wrong layer to settle
    an open question.

- **Option B (chosen): `unpack_into(&self, dst: &mut [u16]) -> Result<(), Error>`.**
  - What it is: the caller supplies the buffer; the function only checks its
    length and fills it.
  - Why selected: needs no allocator at all, so it survives `DEC-002` however
    that lands. A caller who cannot afford (or does not want) a fresh
    allocation — reusing a buffer across frames, an arena, a `no_std` target —
    has a way to say so; a caller who wants the `Vec`-returning convenience
    can trivially build it on top (`let mut v = vec![0u16; w*h]; unpack_into(&s,
    order, &file, &mut v)?;`). The reverse is not true: `unpack_into` cannot be
    recovered from `unpack` without an extra copy.

- **Option C: accept `&Container` instead of `(byte_order, file)` separately.**
  - What it is: thread the whole `ifd::Container` through instead of its two
    relevant parts.
  - Why rejected: `Container`'s `data` field is private by design (SPEC-003),
    and `plane` has no reason to depend on `ifd::Container`'s internals when
    `byte_order()` (already public) and the original byte slice (already held
    by whoever called `Container::parse`) are the only two things this module
    needs. Keeping the dependency to `Sensor`/`ByteOrder` only, both already
    public, keeps `plane` decoupled from `ifd`'s container-walk concerns.

## Consequences

- **Positive.** No allocator dependency anywhere in `src/plane.rs`; the
  module compiles and runs identically whichever way `DEC-002` resolves.
- **Positive.** A caller reusing one buffer across many frames (the expected
  `crustyimg` shape — develop a burst, not one frame) pays one allocation for
  the whole burst instead of one per frame.
- **Negative.** Every caller must compute `width × height` and allocate
  before calling — one more step than `unpack() -> Vec<u16>` would have been.
  A `Vec`-returning convenience can be added later without breaking this
  signature (`AC` follow-up, not a blocker).
- **Neutral.** `AC8`'s peak-RSS measurement is therefore mostly the caller's
  buffer, not anything `unpack_into` itself allocates — the function's own
  additional working memory is O(1) (a `BitReader` cursor), not O(pixels).

## Validation

Right if a caller can decode a full 47 MP Q2M frame through `unpack_into`
with the caller owning the only allocation involved, measured via `irr unpack`
(`AC8`). Revisit if `DEC-002` lands on a shape where a caller-supplied `&mut
[u16]` is not the natural buffer type (e.g. a fallible-allocation API), or if
a `Vec`-returning convenience is added and its own shape needs its own record.

## References

- Related specs: SPEC-012
- Related decisions: DEC-002 (proposed, unresolved), DEC-008
- Constraint: `library-not-application`
