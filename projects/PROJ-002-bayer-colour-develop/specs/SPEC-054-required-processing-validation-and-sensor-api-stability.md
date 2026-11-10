---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-054
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
  complexity_actual: null
  verify_verdict: null

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-4-8
  to_agent: null
  created_at: null

references:
  decisions: [DEC-045, DEC-049]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, library-not-application]
  related_specs: [SPEC-039, SPEC-042]

depends_on: []                     # independent to start; gates SPEC-039 (group E) per DEC-045

claimed_by: null

value_link: "a DNG that requires an unsupported correction is told so, not handed a plausible-looking wrong image — pre-0.2.0 correctness"

cost:
  tokens_estimate: 24000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-054: Required-processing validation + Sensor API stability (group C)

## Context

Pre-0.2.0 code-review finding **#3 (P1)**, triaged in
`docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`; adopted by DEC-045. The
decision is **DEC-049**.

`Sensor.opcode_lists: [bool; 3]` (`src/ifd.rs:685-687`, assembled at `:1708-1714`) is
**presence-only**; only `opcode_list_1` and `opcode_list_3` keep their bytes, so
`OpcodeList2`'s payload is **dropped on read**. Worse, `parse_opcode_list`
(`src/opcode.rs:299`) **discards the `Flags` word for every recognized opcode** — it
retains the optional bit only on `Opcode::Unknown` (`:354-364`) — and the colour path
(`develop_color_into`, `src/color.rs:288`) calls it **not at all** (`src/color.rs:255`
documents that opcode lists "are not applied on this path"). The mono helpers
(`parse_warp_rectilinear` / `parse_fix_bad_pixels_constant`) resolve a list with
`find_map`, i.e. first-match-only.

So a DNG that *requires* a correction the pipeline does not apply — a mandatory opcode
in `OpcodeList2`, any mandatory opcode on the colour path, a recognized-but-wrong-list
mandatory opcode, or a second mandatory instance beyond the first — develops to a
plausible-looking wrong image and returns `Ok(())`. This is the
`parser-accepts-what-pipeline-cannot-honor` signal.

Fixing it adds a `Sensor` field, which collides with the semver-urgent
`#[non_exhaustive]` question (`docs/reviews/2026-11-06-staff-review-0.2.0-triage.md`
§4): `Sensor` has public fields, so a field add breaks consumers unless the type is
sealed first. DEC-049 seals `Sensor` here (it is a parser-output view, not a
consumer-built type — triage open decision #3) and schedules the other public structs'
sweep into SPEC-042.

**Measured (corpus probe 2026-11-10, DEC-049):** no corpus file carries `OpcodeList2`;
the Bayer/colour files carry no opcode lists at all; the Q2-Mono's mandatory list-1 /
list-3 opcodes are both applied on the mono path. So the whole of this spec is
hostile/unsupported-input hardening — every shipped corpus file must stay `Ok` and
bit-exact.

## Design

**Per DEC-049.** Two coupled parts.

### 1. Keep the `OpcodeList2` bytes

Add `opcode_list_2: Option<Vec<u8>>` to `Sensor`, read from `TAG_OPCODE_LIST_2`
(`51009`, already defined `src/ifd.rs:132`) exactly as `opcode_list_1` /
`opcode_list_3` are read and assembled (`:1708-1714`), with the same `DEC-012`
interpretation-class tolerance: a structurally-unreadable tag is `None` and recorded
in `malformed_tags`, never fatal. Keep `opcode_lists: [bool; 3]` as the presence
summary.

### 2. Required-processing validation on every develop path

Add a `pub(crate)` **flags-aware support scan** — walk the raw opcode framing
(`count:u32be`, then `id, version, flags, size` per opcode, the same reader
`parse_opcode_list` uses; always big-endian) over each present list, taking the path's
**applied set**, and return `Ok(())` or
`Error::UnsupportedRequiredOpcode { list: u8, id: u32 }`. It must still surface genuine
framing errors (`Error::Truncated`, `Error::MalformedOpcodeParams`) and the existing
`Error::UnsupportedMandatoryOpcode` for an unknown mandatory id.

Rule per opcode in list `n`:
- `Flags` bit 0 set (**optional**) → may skip;
- bit 0 unset (**mandatory**) → must be *exactly* the instance the path applies;
  otherwise `Error::UnsupportedRequiredOpcode { list: n, id }`.

Applied set:
- **mono** (`develop_into`): the **first** `FixBadPixelsConstant` (id 4) in list 1 and
  the **first** `WarpRectilinear` (id 1) in list 3;
- **colour** (`develop_color_into`): **empty**;
- **`OpcodeList2`**: empty for both paths.

This makes the error fire for: a mandatory opcode in list 2; any mandatory opcode on
the colour path; a mandatory recognized opcode in the wrong list (e.g. a mandatory
`WarpRectilinear` in list 2); and a second mandatory instance beyond the one the
applier consumes (the `find_map` gap). Optional instances are skipped in every case.

Call the scan at the **top of each public develop entry**, before any allocation or
decode: `develop_into` and its `f32` / threaded variants; `develop_color_into`,
`develop_color_f32_into`, and the threaded colour entry. (§12: the test must exercise
each entry, not just one — the variants share a kernel but are separate public doors.)

**No `DEVELOP_VERSION` bump** (`src/lib.rs`): this only converts some former-`Ok` into
`Err` and never changes a still-succeeding develop's bytes.

### 3. Seal `Sensor`

Add `#[non_exhaustive]` to `Sensor`. This forbids external struct-literal / `..rest`
construction, which breaks the 7 `Sensor { .. }` fixtures in the `tests/` crate
(`tests/develop_oracle.rs:151,204,538,649,924`, `tests/colour_metadata_oracle.rs:662`,
`tests/color_oracle.rs:306`). Migrate them onto an **in-crate, dev-only constructor or
builder** gated behind a cargo feature (mirror `fuzzing`, `DEC-036`): `just test` runs
`--all-features`, so the fixtures compile; the default / published surface gains
nothing. Give the builder sane defaults so each fixture sets only what it exercises.
(The `Cfa` / `WarpRect` / `Opcode` seal is **not** here — SPEC-042.)

## Inputs

- **`src/ifd.rs`** — `Sensor` struct (`:642`), the opcode-list field docs (`:685-704`),
  the assembly block (`:1708-1714`), `TAG_OPCODE_LIST_2` (`:132`), the `DEC-012`
  interpret-phase helper (`:1246`) and `malformed_tags` (`:767`).
- **`src/opcode.rs`** — `parse_opcode_list` (`:299`), `FLAG_OPTIONAL` (`:208`), the
  framing reader, the `find_map` entry points (`:381`, `:417`).
- **`src/develop.rs`** — `develop_into` and its opcode dispatch (`:831-872`), the `f32`
  and threaded variants.
- **`src/color.rs`** — `develop_color_into` (`:288`), `develop_color_f32_into`, the
  threaded entry (`:383`), and the `:255` "not applied" note to reconcile.
- **`src/lib.rs`** — the `#[non_exhaustive]` `Error` enum (`:143`),
  `UnsupportedMandatoryOpcode` (`:512`) and its `Display` (`:913`) as the shape to
  mirror; `DEVELOP_VERSION`.
- **`tests/develop_oracle.rs`, `tests/colour_metadata_oracle.rs`,
  `tests/color_oracle.rs`** — the `Sensor` fixtures to migrate.
- **`Cargo.toml`** — `[features]` (the `fuzzing` precedent).
- **`fuzz/fuzz_targets/`** — the opcode / develop targets to extend over the scan.
- **`docs/provenance-ledger.md`**, **`CHANGELOG.md`**.

## Outputs

- **Modified:** `src/ifd.rs` (`opcode_list_2`, `#[non_exhaustive] Sensor`);
  `src/opcode.rs` or a preflight helper (the scan); `src/lib.rs`
  (`UnsupportedRequiredOpcode` + `Display`); `src/develop.rs` + `src/color.rs` (call
  the scan at each entry; reconcile the `:255` note); the 3 test files (fixture
  migration); `Cargo.toml` (dev feature); `CHANGELOG.md`; `docs/provenance-ledger.md`.
- **Created:** the dev-only `Sensor` fixture constructor/builder; red-proof
  script(s)/tests per `oracle-must-be-shown-red`.
- **No `DEVELOP_VERSION` change** and **no new runtime/published dependency.**

## Acceptance Criteria

- [ ] **List-2 bytes kept:** `Sensor.opcode_list_2` carries `OpcodeList2`'s bytes when
      present and readable; `None` + `malformed_tags` when the tag is malformed
      (`DEC-012` parity with lists 1/3).
- [ ] **Mono required-processing:** a fixture with a **mandatory** opcode in
      `OpcodeList2`, and one with a **second** mandatory `FixBadPixelsConstant` in
      `OpcodeList1`, each return `Error::UnsupportedRequiredOpcode`; the Q2-Mono's
      single mandatory list-1 + list-3 opcodes still apply and the output is bit-exact.
- [ ] **Colour required-processing:** a Bayer fixture with any **mandatory** opcode in
      any list returns `Error::UnsupportedRequiredOpcode`; a fixture whose only opcodes
      are **optional** still returns `Ok`.
- [ ] **Each public entry validates:** the error fires from `develop_into`,
      `develop_color_into`, and their `f32` / threaded variants (one test per entry).
- [ ] **Corpus unchanged:** every shipped corpus file (mono + colour) still develops
      `Ok` and byte-identically (local tier-B run, `DEC-003`); recorded in the handback.
- [ ] **`Sensor` sealed:** `Sensor` is `#[non_exhaustive]`; the crate builds and
      `just test` passes with the migrated dev-feature fixtures; a default
      `cargo build` is unaffected.
- [ ] **No determinism change:** `DEVELOP_VERSION` is unchanged; no still-succeeding
      develop's bytes move.
- [ ] **Panic-free / gates:** full gate suite green (`just test`, `lint-ci`,
      `fmt-check`, `validate`, `cost-audit`, `deny`, `deny-fuzz`); the scan runs on
      hostile bytes with checked arithmetic.

## Failing Tests

- **Opcode-validation red-proof** (primary oracle) — a mono fixture requiring a
  mandatory `OpcodeList2` op and a colour fixture requiring a mandatory op each show
  **`Ok` (silently wrong) on the pre-fix tree** and a typed
  `Error::UnsupportedRequiredOpcode` after; control (optional-only) green first
  (`oracle-must-be-shown-red`, DEC-009).
- **Duplicate-mandatory** — two mandatory `FixBadPixelsConstant` in list 1 → error
  (the `find_map` gap).
- **Per-entry behavioural tests** — each public develop entry rejects the required op
  (§12; not just one representative).
- **Corpus regression guard** — the tier-B corpus develops bit-exact (local run).
- **Fuzz** — the opcode/develop target(s) feed arbitrary opcode-list bytes through the
  scan; `just fuzz` clean for the configured window.

## Non-Goals

- Not *implementing* any new opcode, nor the colour-path opcode pipeline — this
  refuses unsupported required processing, it does not honour it.
- No change to a DNG that needs no unsupported op (today's corpus develops unchanged).
- The `#[non_exhaustive]` sweep of `Cfa` / `WarpRect` / `Opcode` is SPEC-042's; this
  spec seals only `Sensor`, because it adds a `Sensor` field.
- Not the parser guard (SPEC-052), the Compression-7 shape binding (SPEC-053), or the
  licence/package work (SPEC-055).

## Notes for the Implementer

- Read `## Design` and **DEC-049** first; the corpus-safety measurement is in DEC-049
  (no corpus file carries `OpcodeList2`; the Bayer files carry no opcode lists), so the
  validation is pure hardening — if a corpus develop changes, something is wrong.
- `parse_opcode_list` drops `flags` for recognized opcodes, so you **cannot** validate
  off its typed output — use a flags-aware scan over the raw framing (DEC-049's
  rejected-alternatives explain why).
- `#[non_exhaustive]` on a struct still allows construction **inside** the crate, so
  `src/`'s own test `Sensor { .. }` literals (e.g. `src/develop.rs:944`) are fine; only
  the separate `tests/` crate needs the dev-feature constructor.
- The scan is a natural seed for group E's (SPEC-039) unified preflight — keep it a
  standalone `pub(crate)` helper E can call, not inlined per entry.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
4. **Where was the worst defect caught?** — `design` | `build` | `verify` | `ship` | `escaped` | `none`. — <one word>
5. **What can a user do now that they couldn't before?** — <answer | none>
