# PROJ-003 close — code-review scope

Scope for the capstone full review at the PROJ-003 close (the post-PROJ-003 sequence's "full code review at close"). The review is maintainer-run and billed; this records the surface, the emphasis, and what earlier review already dispositioned, so the review spends its budget on fresh ground.

## What to review (the PROJ-003 code surface)

Baseline (parent of the first SOF-3 commit): **`60f232f`**. Everything PROJ-003 added sits in `60f232f..main` and is ~4,650 insertions, concentrated in two files plus the harness:

| File | Size | What it is | Review weight |
|------|------|------------|---------------|
| `src/ljpeg.rs` | **3,283 lines** | the clean-room SOF-3 lossless-JPEG decoder (`parse_headers`, `HuffTable::build`, `ScanReader`, `decode_difference`, `restart`, `reconstruct`, `decode_into`, + `cfg(test)` `encode_lj92` / `encode_lj92_two_component`) | **primary — highest risk** |
| `src/plane.rs` | +402 | the DNG `Compression == 7` binding: recognition, strip/tile assembly reuse, both byte orders, the APPn/COM header-skip (DEC-039), the `require_uncompressed` partition | **secondary** |
| `tests/plane_oracle.rs` | +283 | the bit-exact `dnglab --raw-checksum` oracle over the 2 held files (`DECODABLE_COMPRESSED`) | oracle integrity |
| `tests/plane_unpack.rs` | +260 | unit decode tests | supporting |
| `tests/support/tiff.rs` | +167 | synthetic SOF-3 / Compression-7 fixture builder | supporting |
| `fuzz/fuzz_targets/{ljpeg,plane,develop,demosaic}.rs` + seeds | — | panic-freedom harness; SPEC-045 dropped the `require_uncompressed` early-return so the **public** route is fuzzed | harness |

`src/develop.rs` / `src/demosaic.rs` are touched **only** at the fuzz-target level (public-route coverage); their logic is PROJ-001/002, out of scope here. `src/ljpeg.rs` is `pub(crate)` — an internal module, so this is a review of implementation, not public-API design.

## Invariants the review must hold the code to

1. **Clean-room / patent posture (DEC-035).** The decoder is implemented from published ITU-T T.81 Annex H, not derived from rawler / dnglab / libjpeg source. Flag any structure that looks copied rather than spec-derived. (A read-only review can't *prove* clean-room, but it can flag resemblance.)
2. **Panic-freedom on hostile input.** `ljpeg.rs` parses attacker-controlled bytes. Every slice, index, shift, and cast must be bounds/overflow-safe and return a typed `Ljpeg*` error, never panic. 3.1M fuzz runs are clean — the review should hunt the paths fuzzing under-weights: **allocation sizing from attacker-controlled width × height × components** (overflow / OOM), dimension math, and any `unwrap`/`expect`/slice in the header and restart paths.
3. **Integer decode correctness.** Predictors 1–7, point transform `Pt`, precision 8–16 bit, two's-complement difference decode, the **Nf=2 column de-interleave index math**, and restart-interval MCU counting. This is the correctness core; a subtle off-by-one here is bit-exact-invisible on the two held files but wrong on a third.
4. **Oracle is not tautological.** `plane_oracle.rs` must compare against the independent `dnglab` MD5, not a shared code path (the `shared-path-oracle-tautology` signal). The analytic round-trip red-proof must genuinely go red on a perturbed Huffman table / predictor / point transform.
5. **Determinism.** Integer-only by construction; byte-identical plane across macOS and Linux.
6. **Fresh SPEC-045 guards — read closely.** `parse_scan_header` rejecting `Pt >= P`; `parse_dht` rejecting `Tc != 0` / `Th > 3`; `find_table` making a later same-`Th` DHT win. These landed last and have the least soak time.

## Already triaged — do NOT re-litigate (point the review away from these)

An external staff review ("gemini argon") was triaged 2026-11-06 (`docs/reviews/2026-11-06-staff-review-0.2.0-triage.md`). Current state:

- **Fixed in shipped code** (the review's snapshot predated them): §2.1 `swap_remove`/shared-DC-table (now `Td`-keyed non-consuming `find_table`), §2.2 per-component predictor strides, §3A ColorMatrix white-balance double-apply (DEC-038).
- **Fixed in SPEC-045** (this capstone): §2.3 restart-MCU reset, §2.4 header hardening (`Pt<P`, `Tc/Th`, APPn/COM skip), the §2.1 later-DHT-replaces sub-point. If the review re-flags these, they are addressed — confirm, don't reopen.
- **Deferred elsewhere** (tracked, not PROJ-003 gaps): §3B colour crop/orient + ActiveArea-relative mosaic phase → a new PROJ-002 colour-geometry spec; §4 `#[non_exhaustive]` → PROJ-002 0.2.0 / SPEC-042 (semver-urgent); §4 opcode `MAX_TAG_VALUES` → PROJ-001; §4 `LinearizationTable` rejection + `tag_name` in `Error` Display → their own follow-ups; §5 perf/RSS → a dedicated bit-identical stage after correctness.

A genuinely new, high-value finding in `ljpeg.rs` or `plane.rs` is the win here; the above are noise.

## How to launch it (maintainer-run; billed)

The code is already merged across 7 PRs (#100/#104/#108/#119/#121/#126/#131), so there is no single diff-PR to point at. Two options:

- **Whole-surface review (recommended for a capstone):** point `/code-review ultra` at the finished modules `src/ljpeg.rs` and `src/plane.rs` — a review reads the decoder as it stands, not reconstructed from 7 diffs.
- **Range review (if ultra wants a diff):** the full PROJ-003 code delta is `60f232f..main` restricted to `src/ljpeg.rs src/plane.rs tests/ fuzz/`.

Suggested level: **ultra** — the one capstone review the post-PROJ-003 sequence calls for.
