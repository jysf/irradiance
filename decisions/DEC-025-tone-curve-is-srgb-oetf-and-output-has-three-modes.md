---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-025
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-10-26
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - src/tone.rs
  - src/tone_table.rs
  - examples/gen-tone-table.rs
  - tests/tone_oracle.rs

tags:
  - develop
  - tone
  - output
  - srgb
  - determinism
  - spec-019
---

# DEC-025: the tone curve is the sRGB OETF; output has three modes, and `develop_into` stays scene-linear

## Decision

The STAGE-003 tone curve is the **sRGB OETF** (IEC 61966-2-1). The developed
image is encoded into one of **three caller-chosen output representations** by a
new, additive `src/tone.rs` layer:

- **scene-linear `f32`** in `[0.0, 1.0]` (`to_linear_f32_into`) — the tone curve
  is **not** applied; this is `develop_into`'s `u16` rescaled (`v / 65535.0`);
- **display-referred 16-bit sRGB `u16`** (`encode_srgb16_into`);
- **display-referred 8-bit sRGB `u8`** (`encode_srgb8_into`).

`develop::develop_into` is **not reopened**: it keeps producing scene-linear
`u16` full-scale per `DEC-018`. The sRGB mapping is a **committed, generated
const table** (`src/tone_table.rs`, from `examples/gen-tone-table.rs`); the
library runs **no** `powf`/`exp`/`ln` on the encode path (it does an integer
lookup), and `to_linear_f32_into` is a single IEEE division.

## Context

`develop_into` produces a scene-linear `u16` image (`DEC-018`); its own doc
(`src/develop.rs`) said the tone-curve stage `SPEC-019` would add was "for now, a
no-op". Two gaps remained before STAGE-004 (crustyimg) could consume the output:
no tone curve (a scene-linear image shown directly is far too dark — the
reference render `dnglab analyze --srgb` is display-referred 16-bit sRGB,
`docs/oracle-contract.md:22`), and one output representation where STAGE-003's
charter named three (linear f32, u16, 8-bit).

**Which curve is grounded in the real file, not assumed.** A design-time probe
(`exiftool -s -G1` on `LEICA-Q2-MONO/L1026192.DNG`, 2026-10-26) found the Q2M
carries **no** profile tone curve: no `ProfileToneCurve`, `LinearizationTable`,
`TransferFunction` or `RenderingIntent`; `PhotometricInterpretation: Linear Raw`,
`SamplesPerPixel: 1`, `BlackLevel: 512`, `WhiteLevel: 16383`. With no per-file
curve to honour, the only transfer between the scene-linear plane and the
display-referred sRGB reference is the **output colour space's own transfer
function** — the sRGB OETF. Honouring a per-file `ProfileToneCurve` /
`LinearizationTable` is deliberately out of scope (there is none here; it is the
next wave — STAGE-003 excludes "LUT / film response curves").

The sRGB OETF, scene-linear `L ∈ [0,1]` → display-referred `V ∈ [0,1]`:

```text
V = 12.92·L                        for L ≤ 0.0031308
V = 1.055·L^(1/2.4) − 0.055        otherwise
```

Table index `i` is the scene-linear `u16`; `L = i / 65535.0`; `V` is scaled to
the output range (`×65535` / `×255`), **round-to-nearest** (consistent with
`DEC-018`), clamped.

## Alternatives Considered

- **Option A: apply the curve inside `develop_into`, changing its output to
  display-referred (or to `f32`).**
  - What it is: fold the tone curve into the develop core; one output type.
  - Why rejected: it reopens `DEC-018` and changes a shipped function's decoded
    pixels — which moves the levels/geometry/warp oracles (`SPEC-015`/`SPEC-020`)
    under us and trips AGENTS.md §8's "a fix that changes decoded pixel output is
    never a patch." It also forecloses the scene-linear representation a future
    colour pipeline (PROJ-002) will want. Keeping the curve in a separate layer
    costs nothing and keeps every existing oracle stable (0 lines changed in
    `src/develop.rs`/`src/plane.rs`/`src/ifd.rs`).

- **Option B: compute the curve at runtime with `powf`.**
  - What it is: call `1.055·L^(1/2.4) − 0.055` per sample in the library.
  - Why rejected: `powf` is not guaranteed bit-identical across platforms, which
    breaks the determinism STAGE-003 requires (`DEC-002`; crustyimg consumes this
    with `build --frozen`). A committed integer table is byte-identical by
    construction; `powf` runs once, in the generator, on the maintainer's
    machine.

- **Option C: one output mode (u16 sRGB only), add others later.**
  - Why rejected: STAGE-003's charter names three, and the "ship the reader with
    the field" rule (AGENTS.md §11) says a representation nothing reads should not
    land. All three ship with a reader now (`irr develop --format`), rather than a
    mode landing unread.

- **Option D (chosen): sRGB OETF, table-driven, three additive encoders over a
  scene-linear `develop_into`.**
  - Why selected: grounded in the file probe (no profile curve to honour);
    deterministic by construction (committed table, no runtime transcendental);
    `DEC-018` untouched; and `f32` = scene-linear / `u16`+`u8` = display-referred
    gives a consumer both the raw-linear value and the ready-to-show image from
    one develop.

## Consequences

- **Positive.** `develop_into` and every oracle it feeds are unchanged; the
  encode layer is purely additive. Determinism holds across macOS/Linux from the
  committed table (`AC7`), not from matching a runtime float. `powf` is confined
  to `examples/gen-tone-table.rs`.
- **Positive.** Each mode has a reader the day it lands: `irr develop
  --format linear-f32|srgb16|srgb8` (default `srgb8`) and the public encoders'
  own unit tests. No unread output mode.
- **Negative / cost.** The committed `src/tone_table.rs` is large generated
  source (~8k lines; `[u16; 65536]` + `[u8; 65536]`). It is `#[rustfmt::skip]`ed
  so `cargo fmt --check` (a CI gate) leaves it alone, and regenerated by `just
  gen-tone-table` (which asserts no diff). The 8-bit encode of a 47 MP frame is a
  caller-owned `u8` buffer; the `f32` mode is 4× the pixel count — the caller
  chooses and sizes it (`DEC-016`).
- **Neutral — the red-proof mechanism.** The analytic oracle's red-proof
  (`tests/tone_oracle.rs`, `AC6`) is `DEC-021`'s in-process shape (mutate a
  **copy** of the reconstructed table, assert the ±1 oracle rejects it, with the
  unmutated table as the green negative control), **not** `DEC-017`'s
  copy-the-crate-and-rebuild mechanism. The reason is the subject: `DEC-017`
  must rebuild because its subject is `unpack_into`'s compiled behaviour; this
  oracle's subject is pure table *data* already in memory, so an in-process
  mutated copy is a real test of the comparison, and the shipped table carries no
  fault switch.
- **Neutral — ±1 is in the test, not the table.** Cross-platform `powf` can
  differ in the last bit, so the analytic oracle tolerates ±1 (`AC5`); the
  committed table is the single byte-identical source of truth (`AC7`), with the
  exact anchors (`0`, `65535`, the linear-segment breakpoint) pinned. Same split
  `DEC-023`/`SPEC-020` used for the perceptual metric.

## Validation

**Right if** a consumer (crustyimg, STAGE-004) can encode the display image from
the committed table with byte-identical results on macOS and Linux, and the
scene-linear `f32` mode serves a future colour pipeline without a develop-core
change.

**Wrong if** a Q2M frame (or a later camera in scope) turns out to carry a
`ProfileToneCurve` / `LinearizationTable` that should override the sRGB OETF — at
which point honouring it is a new spec (STAGE-003 excludes it), and this decision
is the scene-linear→display default it layers on top of, not a thing it
contradicts. Also wrong if `DEC-002` lands in a way that makes a runtime curve
acceptable and the table's size an unjustified cost — reopen then.

## References

- Related specs: SPEC-019 (this), SPEC-020 (the develop oracle / `--srgb`
  reference), SPEC-014/SPEC-015 (the develop core and its analytic oracle)
- Related decisions: DEC-018 (develop output is scene-linear `u16` — not
  reopened), DEC-016 (caller-owned buffers), DEC-002 (determinism from STAGE-003
  on), DEC-021 / DEC-017 (red-proof shapes), DEC-023 (the "±1 in the test, exact
  bytes committed" split)
- External docs: IEC 61966-2-1 (sRGB); `docs/oracle-contract.md`;
  `docs/provenance-ledger.md` (the sRGB OETF row)
- Evidence: the 2026-10-26 `exiftool` probe recorded in SPEC-019's Context
