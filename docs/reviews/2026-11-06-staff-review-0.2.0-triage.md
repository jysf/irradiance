# External Staff review (0.2.0) — triage

- **Received:** 2026-11-06, relayed by the maintainer from an external "Staff Rust + Math/RAW + Product" review ("gemini argon").
- **Triaged by:** the orchestrator session (claude-opus-4-8), 2026-11-06.
- **Status:** assessed and prioritised. **No code or spec edits made from this triage yet** — folding the accepted items into specs/stages is gated on (a) the reviewer re-reviewing current `main` and (b) coordination with the parallel PROJ-002 colour session (see §3A).
- **⚠ Snapshot caveat (the headline):** the review reads as a tree taken **before** SPEC-043, SPEC-046, and DEC-038 landed. Its two sharpest code claims are already fixed in shipped code. Everything below is assessed against **current `main`** (tip after PR #121, SPEC-046 ship).

## Verdict table

| § | Finding | Verdict | Lands in | Priority |
|---|---------|---------|----------|----------|
| 1 | Finish `STAGE-011` before PROJ-002 `STAGE-010` (0.2.0) | **Already our plan** — STAGE-011 active, 0.2.0 wave parked. Rationale (SPEC-039 `plane::validate` must handle Compression-7 strips; SPEC-040 `Error::kind` classifies the final `Ljpeg*` surface) is sound. | — | confirm only |
| 2.1 | `take_table` `swap_remove` consumes the shared DC table | **STALE — fixed in SPEC-046.** `parse_headers` keeps `dc_tables: Vec<(u8, HuffTable)>` and resolves each component via `find_table(&dc_tables, td).clone()` (both held files share `Td=0`, table not consumed). *Sub-point:* "a later DHT with the same `Th` must replace the earlier" is a genuine latent gap — `find_table` returns the **first** match. | SPEC-045 hardening | low |
| 2.2 | Same-component predictor neighbours (`Ra=idx-2`, `Rb=idx-2X`, `Rc=idx-2X-2`) | **Already done** — SPEC-046 ships per-component predictor state; verify checked the two-output-columns-back stride by hand. | — | — |
| 2.3 | Restart `Ri` counts MCUs; both components reset at `RSTm` | **Latent** — neither held file has a `DRI`, so this is unexercised; valid T.81 conformance hardening for a restart-bearing two-component file. | SPEC-045 + fuzz | medium |
| 2.4 | Header hardening: `Pt < P`, `Tc == 0 && Th <= 3`, skip `APP0..APP15` / `COM` by length | **Valid** latent hardening (the held files trigger none; camera-embedded JPEG comments would hit the `APPn` gap). | SPEC-045 + fuzz | medium |
| 3A | ColorMatrix path `inv(CM)·D` double-applies white balance (magenta grey on Nikon D300S) | **ALREADY FIXED by DEC-038** — `cm_path_cam_to_xyz` does `CA(W→D50)·inv(CM)`, `W = inv(CM)·AsShotNeutral`, no WB diagonal; neutral-grey `R==G==B` red-proof in `tests/color_oracle.rs`; `DEVELOP_VERSION` 1→2. Authored by a **parallel `claude-sonnet-5-5` PROJ-002 session**. | — | — |
| 3B | Colour geometry: `develop_color_into` doesn't crop/orient (`output_dimensions`); `normalize_mosaic` 2×2 phase is buffer-relative, not `ActiveArea`-relative; honour `BlackLevelRepeatDim` | **Valid, likely still open** (DEC-038 was the matrix, not geometry). Pixel-output change → a new spec with a red-proof, before 0.2.0. **Coordinate with the sonnet PROJ-002 session.** | new PROJ-002 colour-geometry spec | **high** (pre-0.2.0) |
| 4 | opcode-list `MAX_TAG_VALUES` bound; fuzz Compression-7/`color` (drop the `require_uncompressed` early-return); `#[non_exhaustive]` on `Sensor`/`Cfa`/`WarpRect`/`Opcode`; `tag_name` in `Error` Display; tiled (`TAG_TILE_OFFSETS`) + `LinearizationTable` typed rejection; README/architecture docs | **Valid**, mostly cheap. `#[non_exhaustive]` is **semver-urgent before 0.2.0**. | SPEC-045 + PROJ-002 `STAGE-010` (SPEC-039/040/042) | high (`#[non_exhaustive]`), else medium |
| 5 | Perf/memory stage: RSS −34% (drop `fixed_plane` early; skip `src.to_vec()` when no pixel equals `constant`), warp invariant hoist (−47M `hypot`), chunked 12/14-bit unpack, kill per-pixel 64-bit divisions — all claimed bit-identical | **Valid headroom** (no optimisation done yet), but a **dedicated stage, not a 0.2.0 blocker** — the reviewer frames it that way too. Each change needs a `dst_after == dst_before` proof + before/after ms/RSS in `docs/provenance-ledger.md` + a `DEC-*`. | new dedicated perf stage, after correctness | medium, post-0.2.0 |

## The two stale claims, in detail (evidence for the pushback)

**2.1 — `src/ljpeg.rs` DC-table lookup is already shared-safe.** Current `main`:
```
let mut dc_tables: Vec<(u8, HuffTable)> = Vec::new();   // keyed by Th
...
for td in tds { component_tables.push(find_table(&dc_tables, td)?.clone()); }
fn find_table(tables, th) -> tables.iter().find(|(t,_)| *t == th).map(...)
```
No `swap_remove`; the table is cloned, not consumed, so both `Nf=2` components referencing `Td=0` decode. The `swap_remove` the reviewer cited was the pre-SPEC-046 single-component path. (Remaining gap: `find_table` returns the *first* same-`Th` table, so a mid-stream redefinition isn't honoured — folded into SPEC-045.)

**3A — the ColorMatrix path already Bradford-adapts (DEC-038).** `cm_path_cam_to_xyz(inv_cm, asn)` computes the scene white `W = inv(CM)·AsShotNeutral`, the chromatic-adaptation `CA = cone⁻¹ · diag(ρ_dst/ρ_src) · cone` (Bradford, D50 target), and returns `CA · inv(CM)` — the WB diagonal `D` is **not** applied here. DEC-038 bumped `DEVELOP_VERSION` 1→2 and added a neutral-grey red-proof. The reviewer saw DEC-033's `inv(CM)·D`.

## Pushback (to send to the reviewer)

> Thanks — high-signal review. Much of it predates our last three merges, so please **re-review against current `main`**:
> - **§2.1** `take_table`/`swap_remove` is already a `Td`-keyed, non-consuming `find_table(...).clone()` in the shipped two-component decoder (SPEC-046) — both held files share `Td=0` and decode.
> - **§3A** is already fixed — the ColorMatrix path Bradford-adapts the scene white (`CA(W→D50)·inv(CM)`, no WB diagonal) per DEC-038, with a neutral-grey `R==G==B` red-proof and `DEVELOP_VERSION` 1→2.
> - **§2.2** (per-component strides) is also shipped.
>
> Accepted: **§2.3/§2.4** (restart-MCU reset, header hardening — `Pt<P`, `Tc/Th`, `APPn`/`COM` skip, later-DHT-replaces) and **§4** fold into our hardening capstone + fuzz; **§3B** (colour crop/orient + `ActiveArea`-relative mosaic phase) is the one high-value correctness item we believe is still open — **can you confirm it reproduces on current `main`?** **§5** perf we'll take as its own bit-identical stage after correctness, not a 0.2.0 blocker.
>
> One ask: **which exact commit / tree did you review?** (so we can diff what's already resolved).

## Proposed folding (gated — not yet done)

On the maintainer's go, after the reviewer round + PROJ-002 coordination:
1. **SPEC-045** (STAGE-011 capstone) scope += §2.3, §2.4, the §2.1 DHT-redefinition sub-point, §4's opcode bound + fuzz-coverage (drop the `require_uncompressed` early-return in `fuzz/fuzz_targets/{plane,develop,demosaic}.rs`) + `tag_name` + tiled/`LinearizationTable` rejection.
2. **New PROJ-002 colour-geometry spec** for §3B (pixel output change → red-proof), coordinated with the parallel `claude-sonnet-5-5` session that owns the live colour work (DEC-038).
3. **`#[non_exhaustive]`** on the public structs — fold into the PROJ-002 0.2.0 wave (SPEC-042 release) since it is semver-urgent before the tag.
4. **New dedicated perf stage** for §5 — framed after the correctness/hardening specs; each optimisation gated on a `dst_after == dst_before` proof + before/after ms + peak-RSS in `docs/provenance-ledger.md` + a `DEC-*`.

## Sequencing (confirmed)

Review §1 matches the current plan: **PROJ-003 / STAGE-011 → PROJ-002 / STAGE-010 (0.2.0)**. STAGE-011 is 2 shipped (SPEC-043 routing, SPEC-046 two-component decode) / 2 active (SPEC-044 bit-exact `dnglab` oracle — next; SPEC-045 capstone). SPEC-044 is where the two-component **de-interleave mapping** is finally confirmed against the real files (the analytic round-trip can't — `shared-path-oracle-tautology`).
