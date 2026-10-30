# Changelog

All notable changes to this app. Newest at top.

Keep an `## [Unreleased]` section on top and add entries under it as you work —
the patch lane files fixes here (`[Unreleased] → Fixed`), and a release spec
(DEC-006) promotes the section to a version heading when it cuts the tag.

## [Unreleased]

### Added

- **Above-WhiteLevel samples accepted** (`irradiance::plane`, SPEC-029, PROJ-002) —
  `plane::unpack_into` no longer rejects a sample above `WhiteLevel` (specular
  highlights are legal); raw values are preserved, not clamped. Amends **DEC-008**
  (via DEC-029) and removes `Error::SampleExceedsWhiteLevel`. With SPEC-028, both
  corpus Bayer DNGs (Canon single-strip, Nikon 12-strip) now decode fully —
  unblocking the bit-exact colour-plane oracle.
- **Multi-strip sensor planes** (`irradiance::plane`, SPEC-028, PROJ-002) —
  `plane::unpack_into` now assembles a plane stored as **N strips** (each strip
  unpacked into its own rows; sub-byte depths padded per-strip), not just a single
  strip — opening the common multi-strip layout (e.g. a Nikon D300S Bayer DNG, 12
  strips). Single-strip planes stay byte-for-byte unchanged; panic-free and fuzzed
  (20M+ runs, 0 crashes).
- **CFA (Bayer) sensor recognition** (`irradiance::ifd`, SPEC-024, PROJ-002) — the
  sensor gate now accepts a colour-mosaic DNG (`PhotometricInterpretation == 32803
  (CFA)`, `SamplesPerPixel == 1`) alongside the monochrome `LinearRaw` plane, and
  `Sensor` exposes a structured `Cfa` model (`CFAPattern` / `CFARepeatPatternDim` /
  `CFAPlaneColor` / `CFALayout`), `None` for a monochrome sensor. Parsed as an
  interpretation-class field — a malformed CFA tag costs that field, never the file
  (DEC-012) — and the mosaic plane still reads bit-exact. No demosaic or colour
  yet; the monochrome path is byte-for-byte unchanged.
- The crate exists: `irradiance` library + `irr` dev binary, edition 2021,
  measured MSRV 1.90, **zero dependencies**, `MIT OR Apache-2.0` (SPEC-001).
- Panic-free lint policy (`unwrap_used`, `expect_used`, `indexing_slicing`,
  `panic`, `arithmetic_side_effects` at `deny`; `#![forbid(unsafe_code)]`),
  enforced by a **red-proof that is itself proven red** — it injects violations
  into a copy of the real library and requires a clean negative control, so it
  fails if the policy is deleted, weakened to `allow`, downgraded to `warn`, or
  if clippy never ran (DEC-009).
- **The TIFF/IFD container reader** (`irradiance::ifd`, SPEC-003) — the first
  code in this library to read an actual RAW file. Walks IFD0's chain, recurses
  `SubIFDs` (tag 330), and reads entry tags, types, counts and payloads, with
  **every** offset and length bounds-checked into a typed `Error`. Depth-,
  cycle- and count-guarded: a `SubIFD` that points at itself terminates with
  `Error::CyclicIfd` rather than recursing forever.
  - Sensor-IFD selection keys on `NewSubfileType == 0 &&
    PhotometricInterpretation == 34892 && SamplesPerPixel == 1` — **never on
    largest dimensions**, because a Q2M's `SubIFD2` is a full-resolution JPEG
    preview only 56 px narrower than the plane.
  - Verified against `exiftool 13.55` on all **7** corpus files — **6 `II` / 1
    `MM`** by byte order, and **6 DNG / 1 PEF** by container; the two are
    independent axes and the PEF is `II` as well. Four bodies: Leica Q2
    Monochrom ×3, Leica M Monochrom, Leica M Monochrom (Typ 246, the only `MM`),
    Pentax K-3 III Monochrome ×2. Includes the Pentax's malformed
    `BlackLevelRepeatDim`, which costs the tag and not the file.
  - Compressed planes (2 JPEG, 1 Pentax PEF) are **rejected cleanly** with
    `Error::UnsupportedCompression`; their tags stay readable.
- **A fuzz target, shipped in the same change** (`fuzz/fuzz_targets/ifd.rs`,
  AGENTS §12 bar 2) with a committed hand-built seed corpus, and **shown to
  work**: a deliberately unchecked index was caught by libFuzzer in under a
  second (exit 77 + crash artifact), then removed. 13.0 M executions clean on
  the restored code.
- `irr ifd [--entries] <file>` — dumps the container walk and the sensor
  plane's tags, including the free layer-0 packing check
  (`width x height x bits == StripByteCounts x 8`).
- `just fuzz` / `just fuzz-seeds`, which encode the `cargo fuzz` PATH trap so
  nobody rediscovers it.
- CI: `fmt --check`, `clippy -D warnings`, `test`, **two** `cargo deny check
  licenses` jobs — one for the library's graph and one for `fuzz/`'s, which is a
  separate package and a separate graph (DEC-011) — an MSRV check, and the lint
  red-proof.
  ⚠ CI **cannot** verify decode correctness — tier-B corpus files are never
  committed (DEC-003), so a green badge does not mean the decoder is bit-exact.
- **The live metadata oracle** (`tests/metadata_oracle.rs`, `tests/support/tools.rs`,
  SPEC-005) — `Sensor` is now diffed against `exiftool 13.55` and
  `dnglab analyze --meta --json`, shelled out to fresh every run, replacing a
  frozen hand-transcribed table. No new dependency: `exiftool -T` needs no
  parser, and `dnglab`'s JSON is read by asserting the handful of keys this
  oracle needs are unique in the document before trusting a match.
  - `exiftool` cross-checks all eleven tag-level fields on all **7** corpus
    files, absence included (an M Monochrom's missing `ActiveArea`, a Pentax
    PEF's five absent DNG tags).
  - `dnglab` cross-checks six scalars on the **6 DNG** files (excludes the PEF —
    its values come from rawler's camera database, not the file, evidenced by
    a `bitDepth` of 16 against the file's own `BitsPerSample` 14) and asserts,
    rather than ignores, that its `cropArea.p` is sensor-absolute
    (`ActiveArea` origin + `DefaultCropOrigin`) where ours and exiftool's are
    DNG-relative.
  - The Pentax DNG's malformed `BlackLevelRepeatDim` (`DEC-012`) is asserted
    three ways in one test: exiftool reads a bare `1`, dnglab warns on stderr
    and substitutes, and our reader reports `None` with `50713` recorded in
    `malformed_tags`.
  - **Proven red, both directions** (`oracle-must-be-shown-red`): a tier-A
    pair replays a committed `exiftool` line through the real parsing code
    with no tool and no corpus (CI's only reachable half) — clean on an
    honest reading, exactly one named `Mismatch` on one perturbed field. A
    tier-B pair patches `ActiveArea`'s payload bytes in an in-memory copy of a
    real file (mutation asserted to have actually changed the buffer first),
    diffs the patched reader against the tool reading of the *original* file,
    and confirms re-running on the unpatched bytes is clean again.
  - `just oracle-meta` runs this file alone; both tool-absence and
    corpus-absence skip loudly, naming what is missing.
- **Tone curve and the three output modes** (`src/tone.rs`, SPEC-019). The
  scene-linear `u16` develop result is encoded for display through a
  **table-driven sRGB OETF** (IEC 61966-2-1): the library runs no `powf` — the
  curve is two committed, regenerable lookup tables (`just gen-tone-table`),
  byte-identical across platforms by construction. Three caller-owned encoders:
  `to_linear_f32_into` (scene-linear, no curve), `encode_srgb16_into` (16-bit
  sRGB) and `encode_srgb8_into` (8-bit sRGB); `irr develop --format` selects
  among them. `develop_into` is unchanged — `DEC-018`'s scene-linear output
  stands; the encoders are a composable layer on top (`DEC-025`).
  - **Analytic tone oracle, proven red** (`oracle-must-be-shown-red`,
    `tests/tone_oracle.rs`): every table entry is checked against the sRGB
    formula recomputed in `f64` to within ±1 code value — independent of rawler
    — and a deliberately perturbed entry is confirmed to turn the oracle red,
    then reverted byte-identical.
- **The develop warp branch is now pinned to the fixed plane** (`tests/develop.rs`,
  SPEC-022). `develop_into`'s `WarpRectilinear` branch already developed the
  `FixBadPixelsConstant`-repaired plane, but nothing asserted it — SPEC-017's
  coverage reached only the no-warp branch, and every real Q2M frame takes the
  warp branch. A synthetic tier-A fixture (a bad pixel at the warp's
  optical-centre fixed point plus a non-identity warp) now proves the fixed
  value reaches the developed output through the warp branch, **shown red**
  (`oracle-must-be-shown-red`): severing the branch to the raw source turns the
  two new tests red (suite 243/0/3 → 241/2/3) while SPEC-017's no-warp assertion
  stays green. No `src/` change — latent hardening that closes the hole before a
  bad-pixel-bearing frame arrives.

### Known gaps

- No pixel decode or unpack yet — `StripOffsets`/`StripByteCounts` are read as
  **tags** only. The unpacker and DEC-008's two-path (`bits % 8`) rule are
  STAGE-002.
- No typed tag model: `irradiance::ifd` widens `BYTE`/`SHORT`/`LONG` to `u32`
  and returns `Error::UnexpectedFieldType` for `RATIONAL` and the signed types.
  SPEC-004 owns that.
- No multi-strip corpus file. All seven held planes are single-strip, and the
  tests **assert** that (`tests/ifd_reader.rs:352`, `:443`, `:448`) rather than
  merely not exercising the alternative — so the day a multi-strip file arrives
  it fails those assertions loudly instead of silently taking an untested path.
  That is the right way round, but it is a test to update, not a reader bug.

### Changed

### Fixed

- **Gate scripts can no longer report green without their check running**
  (SPEC-023, STAGE-005's capstone — the generalisation of SPEC-016/021/011).
  Every gate here is a `set -euo pipefail` shell script, and in that context an
  unguarded `grep` aborts the whole script on no-match while a process- or
  command-substitution's non-zero exit is invisible to `set -e` — either way a
  gate can print a green verdict (or die) with its check never having run. A new
  `just gate-audit` (`scripts/gate-audit.sh`) + a CI job scan all **8**
  verdict-bearing `scripts/*.sh` for that shape, name each offender and which
  shape, exclude comments, and — because the audit is itself source-matching
  tooling — **assert their own scan count**, so a glob matching nothing is a loud
  failure, not a silent pass. Shipped with a red-proof
  (`oracle-must-be-shown-red`, which covers gates): a shape injected into a
  **copied** gate is caught by name with a clean negative control and isolation,
  the working tree untouched. The three live offenders are fixed and each proven:
  `validate.sh` (a non-`SyntaxError` ruby fault now turns the gate RED instead of
  a green *"0 … parseable"*), `handback-sync.sh` (a `superseded` handoff is
  skipped, not treated as `pending` — closing the
  `handback-sync-trips-on-superseded-handoffs` signal and retiring the manual
  `synced_at` kludge SPEC-016's ship needed for `HANDOFF-038`), and
  `decisions-audit.sh` (a DEC missing its `# DEC-` H1 no longer aborts the lint
  silently). No `src/` change.
- **The fuzz crate is now linted** (SPEC-011). `fuzz/` is a second cargo graph
  DEC-011 keeps outside the library's `[dependencies]`, so `cargo clippy
  --all-targets -- -D warnings` at the repo root mentioned `irradiance-fuzz`
  **zero** times — a clippy regression in a fuzz target would have shipped
  silently. A new `just lint-fuzz` (`cargo clippy --manifest-path fuzz/Cargo.toml
  --all-targets -- -D warnings`) and a CI job close it, exactly as `deny-fuzz`
  did for the licence gate: the gate is now **two invocations, both required**.
  Scope is clippy's default `-D warnings`, **not** the library's five panic-free
  lints (the fuzz harness is dev-only glue, not an attacker-surface — `DEC-027`,
  extending DEC-011). Shipped with a red-proof (`oracle-must-be-shown-red`, which
  covers gates): an injected `unnecessary_cast` is caught only because
  `-D warnings` promotes it, with a clean negative control. No `src/` or
  fuzz-target change — the crate was already clippy-clean.
- **The develop oracle stops claiming warp-bearing pixels it cannot see**
  (SPEC-021). SPEC-020's SSIMULACRA2-vs-`dnglab --srgb` tolerance (≥85) was
  advertised over all pixels, but `dnglab`/`rawler` implement **no** DNG
  OpcodeList processing and `dnglab` ignores EXIF `Orientation`, so for
  warp-bearing and rotated frames the comparison is structurally meaningless (a
  *correct* `WarpRectilinear` scores −60.169 against dnglab's uncorrected
  reference, vs a +83.145 no-warp baseline). The oracle's scope is now formally
  narrowed to **warp-free, Orientation-1 pixels** (new `DEC-026` amending
  DEC-005; a carve-out in `docs/oracle-contract.md`), and SPEC-018's two dead
  `#[ignore]`'d oracle tests are **retired, not hidden**: replaced by a passing
  characterization asserting the blindness is real (−60.169 / −40.281 < +83.145,
  measured) plus an explicit `Orientation ≠ 1` skip — so the oracle's silence on
  warp can't be mistaken for a passing verdict. Warp geometry stays verified
  analytically (`tests/warp.rs` AC4/AC10). No `src/` logic change; repo-wide
  `#[ignore]` drops 3 → 1.
- **`just validate` now PARSES every artifact's front matter** instead of only
  grepping for required keys. The old gate reported *"valid required
  front-matter"* without ever running a YAML parser, so an artifact no parser
  could read was reported valid — one shipped, was archived, and survived three
  later specs. Now it parses every `*.md` under `projects/` + `decisions/`
  (**121** artifacts, a superset of the old specs+patches+spikes scope) with
  `ruby`'s `YAML.parse`, fails loudly naming the file and the parser's own
  message, and — closing the mute-gate class — **refuses to report green if the
  parser itself is missing** rather than silently skipping the check. Proven
  red (`oracle-must-be-shown-red`): the old script passes a parse-broken fixture
  the new one rejects, with the parser's message shown. Fixed the one live
  offender (`HANDOFF-039`) that broadening the scope to `handoffs/` revealed
  (SPEC-016; ships the `validate` half of the
  `handback-sync-truncates-multi-line-scalars` signal).
- **`just cost-audit` no longer scans patch/spec bodies as front matter** and no
  longer requires build+verify totals on patches whose verify handoff has not
  handed back — DEC-013 says `handback-sync` is the ledger, not the audit
  (PATCH-003, closing PATCH-002's two ship-blockers).
- **Both `just lint` and `just lint-ci` now print the clippy version they
  invoked**, with a note that `lint` is unpinned (PATH-resolved Homebrew) and
  `lint-ci` is what CI sees. A gate that does not state what produced its
  result is the defect that cost this repo 17 consecutive red CI runs (PATCH-004).
- **`just cost-audit` now enforces the stage-level `orchestration_cost:` block**
  at ship — a stage template's own `orchestration_cost` slot was rejected
  because nothing summed it, so ~31% of PROJ-001's real spend was
  documented-but-invisible in every rollup (PATCH-002, DEC-022 amending
  DEC-013 §5).

- **The licence gate now actually covers `fuzz/`.** `DEC-011` recorded that
  `cargo deny` could not reach the fuzz package and hand-wrote its licence table
  instead. Both halves were wrong: `cargo deny --manifest-path fuzz/Cargo.toml
  check licenses` runs fine, and it **failed** — `libfuzzer-sys` declares
  `(MIT OR Apache-2.0) AND NCSA` (conjunctive, so NCSA is not optional) where the
  table said `MIT OR Apache-2.0`, and `irradiance-fuzz` itself carried no
  `license` field, which is `unlicensed` and an error. NCSA is permissive
  (OSI-approved, FSF Free/Libre) so nothing copyleft was ever linked — but the
  record was wrong on a `blocking` constraint, in the document standing in for
  the gate. Now: a named per-crate exception in `deny.toml` (not a widened
  `allow`), a `license` field on `fuzz/Cargo.toml`, a corrected and re-measured
  table in DEC-011 covering all ten crates, and `just deny-fuzz` + a CI job so it
  is a gate rather than a paragraph.
- `just msrv` — the MSRV gate was the only one of the ten with no recipe, so it
  was the only one handing out a raw `cargo +1.90.0 …` that fails with
  `no such command` under the default PATH (the third instance of the
  `+toolchain` trap; `guidance/toolchain-brief.md`).
- Corpus facts in `SPEC-003` and this file: the byte-order count (**6 `II` / 1
  `MM`**, not 5 + a PEF — container and byte order are different axes), the
  compression count (**2 JPEG**, code 7; `K3III.PEF` is code **65535**,
  vendor-private, not JPEG), and "the full-resolution SubIFD" (the PEF has none —
  its plane is in `IFD0`).
- `docs/conformance-matrix.md` gained the three bodies that were held, read
  end-to-end and unlisted, against that file's own opening rule.
- The malformed-tag rule is now stated rather than implied (**DEC-012**):
  **strict on structure, tolerant on shape.** A malformedness that changes *what
  exists* — the header, an entry table, the chain's `next`, or `SubIFDs` — is
  fatal to the container; one that changes only what a known-optional fixed-length
  tag *says* costs that tag and is reported in `Sensor::malformed_tags`. No
  behaviour changed; `SPEC-004` widens the type model on top of this boundary and
  should not have had to guess it.
- **`tests/ifd_reader.rs`'s hand-transcribed tag-value table is gone** (SPEC-005).
  `EXPECTED` now carries only this reader's own structure claims — byte order,
  IFD count, sensor-IFD index, opcode-list presence, malformed tags — none of
  which any external tool reports. Every tag *value* (dimensions, levels,
  geometry, orientation) is cross-checked live by `tests/metadata_oracle.rs`
  instead of trusted from a table one past session typed by hand; the layer-0
  packing check now compares against the file's own live `StripByteCounts`
  rather than a second hand-typed copy of it.
