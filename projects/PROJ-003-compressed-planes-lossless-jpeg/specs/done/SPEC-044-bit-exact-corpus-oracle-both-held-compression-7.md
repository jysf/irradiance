---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-044
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   tier-B oracle harness extension (two files onto the
                                   #   manifest-driven path) + a bounded APPn/COM header-skip the
                                   #   real K3III needs (DEC-039) + a tier-A APPn fixture + an
                                   #   ljpeg staged-copy red-proof + a develop smoke. Each piece
                                   #   small; the breadth is what makes it M, not any one part.
  complexity_actual: M             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected M held: the oracle-harness extension + the one
                                   #   bounded APPn/COM header arm + a tier-A fixture + an ljpeg
                                   #   staged-copy red-proof + a develop smoke; built whole,
                                   #   APPROVED first pass. The design §12 probe pre-proved the
                                   #   decoder arm, so the build was transcription — but the breadth
                                   #   (five pieces across decoder + two oracle harnesses) is M.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-4-8      # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-095 (build) → tier_map.build
  created_at: 2026-11-06

references:
  decisions: [DEC-003, DEC-030, DEC-037, DEC-039, DEC-016, DEC-018]   # corpus never committed (003); exact oracle, no tolerance (030); two-component column-interleave (037); APPn/COM header-skip + mapping confirmed on both real files (039); caller-owned buffer (016); u16 modular arithmetic (018)
  constraints: [oracle-must-be-shown-red, no-panics-on-untrusted-input, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-043, SPEC-046, SPEC-037]   # the routing (043) + the two-component Nf=2 decode this verifies on real files (046, DEC-037) + the decoder (037)

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-046]             # the held files are two-component Nf=2 (DEC-037) and do not decode until SPEC-046 lands; SPEC-046 itself depends on the SPEC-043 routing

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-011's <capability>". Optional; null is acceptable.
value_link: "Proves the binding bit-exact — both held Compression-7 DNGs decode to a u16 plane matching dnglab --raw-checksum exactly — the verification half of STAGE-011's deliverable."

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
  tokens_estimate: 18000000        # ~18M — a verification spec with a small, de-risked decoder arm (the APPn/COM skip is already proven by the design probe); less than SPEC-046's build-from-scratch decode
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 55
      recorded_at: 2026-11-06
      notes: "main-session design (un-metered); §12 probe decoded BOTH held files through plane::unpack_into and compared the native-LE u16 plane MD5 to dnglab --raw-checksum (0.7.2) — Typ246 matched c72e…1d unmodified; K3III failed on an APP0(len 60) between DHT and SOS (LjpegUnexpectedMarker 0xe0), then matched 98ba…89 once a throwaway APPn/COM-skip arm was added to parse_headers. So the de-interleave mapping (SPEC-046) is confirmed on both real files and the only K3III gap is the header-skip. DEC-039 records both. Acceptance + failing tests + HANDOFF-095 + build prompt."
    - cycle: build
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 9196266
      estimated_usd: 19.00
      duration_minutes: 40
      recorded_at: 2026-11-06
      notes: "HANDOFF-095, PR #124. Built whole, APPROVED first pass. RECOVERED from the cold build transcript (session local_0a57d665, transcript 6415ecf4 in worktree slug spec-044-build-ab78fc), deduped by message.id at Opus list rates: 9,196,266 tok (97.9% cache-read) / $19.00, all claude-opus-5-5. The handback's 185,963 was a context-occupancy FLOOR; this cumulative supersedes it. Both pinned MD5s hit exactly on the real corpus; coverage went beyond spec (marker-lookalike APPn payloads, all sixteen APPn, five truncation shapes)."
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 3723032
      estimated_usd: 8.90
      duration_minutes: 30
      recorded_at: 2026-11-06
      notes: "HANDOFF-096, PR #125. Cold verify returned ✅ APPROVED first pass (no defect). RECOVERED from the verify transcript (session local_b8f4a5a7, transcript 4e8dbdc4 in worktree slug spec-044-verify-988ffd): 3,723,032 tok (96.5% cache-read) / $8.90, all claude-opus-5-5. The handback's 165,000 was a context-occupancy FLOOR; this cumulative supersedes it. PR #124 (build) was merged ahead of #125; the orchestrator merged main into the verify branch and resolved the spec/timeline conflicts to this superset before #125 merged."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 35
      recorded_at: 2026-11-06
      notes: "main-session ship (un-metered); reflection + cost recovery from both cold transcripts + resolved the #125 conflicts after #124 merged first + closed-recommendation for the stale SPEC-046 build PR #117 + archive + brag + CHANGELOG. STAGE-011 stays active (SPEC-045 remains, no stage-ship)."
  totals:
    tokens_total: 12919298
    estimated_usd: 27.90
    session_count: 4
shipped_at: 2026-11-06
---

# SPEC-044: Bit-exact corpus oracle — both held Compression-7 DNGs vs dnglab

## Context

STAGE-011's bit-exact contract: the decoded `u16` samples of both held
Compression-7 DNGs must be proven **exactly** correct against an independent
implementation — the real-file counterpart to STAGE-009's analytic round-trip
oracle. The oracle is tier-B (`dnglab analyze --raw-checksum` on the two held DNGs)
and chart-independent; it is a different, **independent** shape from STAGE-009's
round-trip (the stage-born lesson `shared-path-oracle-tautology`), which is exactly
why it is load-bearing here — it is the first thing that can confirm SPEC-046's
column de-interleave *mapping* on real files.

**The design §12 probe (2026-11-06) decoded both held files through the shipped
`plane::unpack_into` and hashed the native-LE plane, and it found two things (DEC-039):**

1. **The SPEC-046 even/odd de-interleave mapping is already correct on both real
   files** — Leica M Typ 246 (P=12, big-endian) matched `dnglab --raw-checksum`
   (`c72e7d9579e2adaef9c6247841b8ac1d`) **exactly, unmodified**. The mapping question
   SPEC-046 flagged as analytically unprovable is resolved; no write-index change.
2. **The Pentax K-3 III (P=14, little-endian) does not decode yet** — its stream is
   `SOI → SOF3 → DHT → APP0(len 60) → SOS`, and `parse_headers` rejects the APP0 with
   `LjpegUnexpectedMarker { marker: 0xe0 }` (Typ 246 has no APPn, hence the asymmetry).
   With a bounded APPn/COM header-skip added to `parse_headers`, K3III then matched its
   pinned checksum (`98bae1339c25eb6e04fbac6b5f643189`) **exactly**.

So this spec is **verification + one minimal decoder tolerance** (skip APPn/COM header
segments by length — DEC-039), both proven by the same real-file oracle. It is the
fourth spec of `STAGE-011` (`PROJ-003`), depending on SPEC-046.

## Goal

Both held Compression-7 DNGs decode to a `u16` plane whose MD5 **equals**
`dnglab analyze --raw-checksum`, exactly, with a red-proof (perturbing the `ljpeg`
decode turns the real-file oracle red); and the decoded plane flows through the existing
develop/tone path unchanged (a source swap, not a second pipeline). This requires the
one decoder change DEC-039 scoped: `parse_headers` skips APPn (`0xFFE0..=0xFFEF`) and
COM (`0xFFFE`) segments by their declared length, which the real K3III needs to decode
at all.

## Inputs

What the implementer will read or consume.

- **Files to read:**
  - `src/ljpeg.rs` — `parse_headers` (≈ line 347): its marker loop matches
    `SOF3 | DHT | DRI | SOS | EOI`, routes a non-SOF3 SOF to `LjpegUnsupportedProcess`,
    and sends everything else to `other => LjpegUnexpectedMarker` (line 393) — the arm
    that currently rejects K3III's APP0. `parse_dht` (≈ 456) is the length-read idiom to
    mirror (`read_u16_be` → `checked_sub(2)` → bounds-check against `data.len()` →
    advance `pos`; `Error::Truncated { at, len }` on overflow/short). `read_u16_be`,
    `read_u8`, `next_marker`, the `MARKER_*` consts, `is_other_sof`.
  - `tests/plane_oracle.rs` — the tier-B oracle. `plane_md5` (≈ 547, native-LE u16),
    `decode_and_hash` (≈ 557), `assert_plane_matches` (≈ 574, localizes a mismatch via
    `dnglab --raw-pixel`), the `plane_md5_matches_the_pinned_raw_checksum` tier-B test
    (≈ 605) that loops `DECODABLE`, the `DECODABLE` / `SKIPPED_COMPRESSED` lists (≈ 94 /
    113) and the `compressed_files_are_skipped_by_name` partition test (≈ 128), and the
    staged-copy red-proof machinery (`TempDir`, `copy_dir_recursive`, `MD5_SOURCE`,
    `inject_chunk_extraction_fault` ≈ 676 — the pattern to mirror for an `ljpeg.rs`
    injection).
  - `tests/develop_oracle.rs` — its own `DECODABLE` (≈ 107) and the
    `require → unpack → develop_into` corpus helper (≈ 139); `irradiance::develop::{develop_into, output_dimensions}`.
  - `tests/support/corpus.rs` (manifest reader, `CorpusRoot`, `CorpusFile::require`),
    `tests/corpus/manifest.toml` (the two held entries' pinned `raw_checksum`).
  - `STAGE-011`, `DEC-037`, `DEC-039`, `brief.md`.
- **§12 probe (done at design — the numbers the build transcribes, do not re-derive):**
  - `dnglab 0.7.2 analyze --raw-checksum`:
    - `LEICA-M-MONOCHROM-TYP246/M2462362.DNG` → `c72e7d9579e2adaef9c6247841b8ac1d`
      (dims 5984×4000, 12-bit; head `[108, 97, 101, 91, 99, 93, 88, 93]`, min 0, max 3947)
    - `PENTAX-K3III-MONO/K3III.DNG` → `98bae1339c25eb6e04fbac6b5f643189`
      (dims 6304×4224, 14-bit; head `[63, 63, 63, 63, 67, 63, 67, 63]`, min 0, max 16382)
  - Both equal the irradiance native-LE `plane_md5` with the APPn/COM-skip in place
    (Typ 246 needs no skip). These are also the values already pinned in `manifest.toml`.
  - K3III marker map (strip @181248, len 31047287): `SOI @0 → SOF3 @2 (P=14 Y=4224
    X=3152 Nf=2, comps (0,1,1,0)(1,1,1,0)) → DHT @18 (len 34) → APP0 @54 (len 60) →
    SOS @116 (Ns=2, Td=0/0, pred 1, Pt 0) → entropy → EOI @31047285`. One scan, no DRI.
  - Typ 246 marker map: `SOI → SOF3 → DHT → SOS` — **no APPn**, no DRI.
- **Related code paths:** `src/ljpeg.rs` (decoder), `tests/plane_oracle.rs`,
  `tests/develop_oracle.rs`.

## Outputs

What the implementer will produce.

- **`src/ljpeg.rs` modified (DEC-039):** `parse_headers`' marker loop gains one arm —
  APPn (`0xFFE0..=0xFFEF`) and COM (`0xFFFE`) markers are **skipped by their 2-byte
  declared length** (read length, `checked_sub(2)`, bounds-check against `data.len()`,
  advance `pos`; a length that overflows or runs past the buffer is
  `Error::Truncated`, never a panic), placed **before** the `other => LjpegUnexpectedMarker`
  catch-all. No new `Error` variant, no new const beyond an optional `MARKER_COM`/range
  check, no signature change, no new runtime dependency; `ljpeg` stays `pub(crate)`. The
  `Nf=1` path and the Typ 246 decode stay **byte-for-byte unchanged** (the arm only fires
  on a segment currently rejected).
- **`tests/plane_oracle.rs` modified:** the two held Compression-7 files move out of
  `SKIPPED_COMPRESSED` into the decode-and-hash oracle. Because they are **not**
  uncompressed, do not just drop them into `DECODABLE` (that would break
  `compressed_files_are_skipped_by_name`, which asserts every `DECODABLE` entry satisfies
  `require_uncompressed().is_ok()`). Introduce a `DECODABLE_COMPRESSED: [&str; 2]` list,
  hash it in `plane_md5_matches_the_pinned_raw_checksum` alongside `DECODABLE`, and extend
  the partition test so `DECODABLE + DECODABLE_COMPRESSED + SKIPPED_COMPRESSED` still
  covers the manifest and the Compression-7 entries assert
  `require_uncompressed().is_err()` **and** that `unpack_into` succeeds (they decode).
  `SKIPPED_COMPRESSED` is left with the PEF only.
- **A tier-A APPn/COM fixture test** in `src/ljpeg.rs` (`mod tests`): a hand-built
  two-component SOF-3 with an APP0 (and a COM) segment between DHT and SOS decodes
  bit-exact to the same plane as the same frame without the segments; a truncated APPn
  length is a typed `Error`, no panic.
- **A real-file red-proof** (staged-copy subprocess, mirroring `inject_chunk_extraction_fault`):
  a fault injected into `src/ljpeg.rs`'s two-component decode (e.g. swap the de-interleave
  write-index, or perturb a per-component predictor neighbour) turns the held
  Compression-7 oracle **red**; a negative control (no injection) stays green.
- **A develop smoke:** a held Compression-7 file developed end-to-end through the existing
  `develop_into` path succeeds, produces `output_dimensions(sensor)`, and is deterministic
  (two runs byte-identical) — proving the decoded plane is a source swap, not a second
  pipeline.
- **No** change to the reconstruction algorithm, the routing (SPEC-043), the container, or
  any public API.

## Acceptance Criteria

Testable outcomes. Each maps to at least one test.

- [ ] With the APPn/COM-skip, **both** held Compression-7 DNGs decode to a `u16` plane
  whose native-LE MD5 **equals** the pinned `raw_checksum`
  (`plane_md5_matches_the_pinned_raw_checksum` over `DECODABLE_COMPRESSED`): Typ 246
  `c72e7d9579e2adaef9c6247841b8ac1d`, K3III `98bae1339c25eb6e04fbac6b5f643189`. Tier-B,
  corpus-gated; skips loudly by name when a file is absent.
- [ ] `compressed_files_are_skipped_by_name` still accounts for **every** manifest entry
  across the three lists, and (corpus-present) asserts each `DECODABLE_COMPRESSED` entry
  is `require_uncompressed().is_err()` yet decodes via `unpack_into`, while the remaining
  `SKIPPED_COMPRESSED` entry (the PEF) does not.
- [ ] `parse_headers` **skips** an APPn / COM segment between DHT and SOS and decodes the
  frame bit-exact (tier-A, hand-built, runs in CI without the corpus); a **truncated**
  APPn length is a typed `Error`, never a panic.
- [ ] The **Nf=1 path and the Typ 246 decode are byte-for-byte unchanged** — every shipped
  `ljpeg`/plane test stays green; Typ 246's checksum is identical with and without the new
  arm (it has no APPn).
- [ ] The **red-proof** goes red: a perturbed `ljpeg` two-component decode turns a held
  Compression-7 oracle red (staged-copy subprocess), and the negative control stays green
  (`oracle-must-be-shown-red`).
- [ ] A held Compression-7 file **develops end-to-end** through the existing `develop_into`
  path — correct `output_dimensions`, deterministic, no panic.
- [ ] `#![forbid(unsafe_code)]` + the five-lint deny policy hold; `just test`,
  `just lint-ci`, `just fmt-check`, `just validate` green; no new runtime dependency.

## Failing Tests

Written during **design**, BEFORE handoff. The implementer's job in **build** is to make
these pass. Tier-A runs everywhere; tier-B is corpus-gated (skips loudly by name).

- **`src/ljpeg.rs` (`mod tests`)** — tier A, hand-built via the SPEC-046 two-component encoder:
  - `"appn_segment_between_dht_and_sos_is_skipped"` — a two-component SOF-3 with an APP0
    (length-delimited, arbitrary payload) inserted between DHT and SOS decodes bit-exact to
    the same plane as the segment-free frame.
  - `"com_segment_is_skipped"` — same, with a COM (`0xFFFE`) segment.
  - `"truncated_appn_length_is_a_typed_error_not_a_panic"` — an APPn whose declared length
    runs past the buffer returns `Error::Truncated`, no panic.
  - `"single_component_decode_unchanged_by_appn_arm"` — a segment-free `Nf=1` scan decodes
    identically to the shipped behaviour (characterization; the new arm must not perturb it).
- **`tests/plane_oracle.rs`** — tier B (corpus) + tier-A partition:
  - `plane_md5_matches_the_pinned_raw_checksum` (extended) — now also hashes
    `DECODABLE_COMPRESSED`; asserts both held Compression-7 planes equal their pinned
    `raw_checksum`.
  - `compressed_files_are_skipped_by_name` (extended) — three-list partition covers the
    manifest; Compression-7 entries are compressed-but-decodable, the PEF is not.
  - `"ljpeg_decode_fault_turns_compression7_oracle_red"` — staged-copy subprocess injects
    an `ljpeg.rs` two-component fault and asserts a held Compression-7 oracle goes red;
    negative control green. Corpus-gated.
- **`tests/develop_oracle.rs`** (or `plane_oracle.rs`) — tier B:
  - `"compression7_file_develops_through_the_existing_pipeline"` — a held Compression-7
    file decodes and runs through `develop_into`; output dimensions `== output_dimensions`,
    deterministic across two runs.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to happen, create a new
spec (in this stage's backlog), not expand this one.

- The Compression-7 recognition + routing itself — **SPEC-043** (this spec verifies it).
- The provenance-ledger entry, determinism assertion (macOS/Linux byte-identity), and the
  broader container→decoder truncation / hostile-strip hardening — **SPEC-045**. (This spec
  adds only the one APPn/COM-skip the real K3III needs to decode at all — DEC-039 — plus its
  own truncated-length guard; it does not take on SPEC-045's full hardening sweep.)
- The §2.1 later-DHT-replaces sub-point, restart-MCU reset (§2.3), `Pt<P` / `Tc/Th`
  validation (§2.4) from the external review — **SPEC-045** (no held file exercises them;
  K3III's APPn is the only review item a held file actually triggers).
- Multi-component / Bayer-in-LJ92 colour interleave, and OpcodeList2 GainMap — deferred
  (DEC-037; no held file).
- Any change to the SOF-3 **reconstruction algorithm** (predictors, de-interleave, entropy
  decode) — STAGE-009 / SPEC-046 (shipped). The APPn/COM skip is header hygiene, not a
  reconstruction change.

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities. Keep short — the full context graph lives
in the handoff file.

- **The APPn/COM skip is already proven** (design §12 probe, DEC-039). It is a single
  bounded match arm before the `other =>` catch-all in `parse_headers`, using the exact
  length-read idiom `parse_dht` already uses. Do not over-build it: no APPn *content*
  parsing, no mid-scan APPn handling (K3III's is header-only; none observed in the scan).
- **Don't drop the Compression-7 files into `DECODABLE`** — `compressed_files_are_skipped_by_name`
  asserts `DECODABLE ⇒ require_uncompressed().is_ok()`. Use a separate
  `DECODABLE_COMPRESSED` list and extend the partition + oracle loops.
- **The oracle digest shape is fixed:** native-LE `u16`, uncropped plane, streamed
  sample-by-sample (`plane_md5`). Both pinned checksums are already in `manifest.toml` —
  read them from `entry.oracle.raw_checksum`, never hard-code.
- **Red-proof = the ljpeg decode, on a real file.** The existing `inject_chunk_extraction_fault`
  targets `src/plane.rs`'s `BitReader` and would not perturb the JPEG path, so it cannot be
  reused as-is — add a sibling injection into `src/ljpeg.rs` (de-interleave write-index or a
  per-component predictor neighbour) and run the staged-copy probe against a held
  Compression-7 file. The analytic round-trip red-proof (SPEC-046) cannot stand in: it
  shares our own convention (`shared-path-oracle-tautology`), which is the whole reason this
  real-file oracle exists.
- **Patent/clean-room:** still DEC-035 (T.81 Annex H, clean-room); skipping APPn/COM is
  Annex B marker structure, no new algorithm family.
- **Both byte orders confirmed:** Typ 246 big-endian + K3III little-endian both match; the
  SOF-3 entropy bitstream is MSB-first regardless of container byte order.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Little. Running the §12 probe as the **real decode on the real files** (not
   just the container) was the decisive move: it confirmed SPEC-046's de-interleave
   mapping on both bytes orders AND caught the K3III APP0 gap at design, so the build
   was transcription and landed APPROVED first pass. The one lesson for next time is
   to distrust a triage's "latent — no held file triggers it" call until the real
   decode has run: the external review filed the APPn/COM-skip (§2.4) as latent, and
   the probe showed a held file (K3III) actually triggers it.

2. **Does any template, constraint, or decision need updating?**
   — The signal `probe-the-decode-input-not-just-the-container` reached **N=2
   paired-opposing** this spec (N=1 the SPEC-043 miss; N=2 this probe catching APP0 at
   design) — bar met, `status: open`, dispositioned at STAGE-011 close into the §12
   step ("run the real decode on the real input, not just the container"). DEC-039
   records both the APPn/COM-skip fold and the mapping confirmation; no new DEC needed.
   The "a verification spec makes no decoder change" expectation was narrowed by DEC-039
   (header hygiene ≠ a reconstruction-algorithm change) — a recorded, bounded exception,
   not a template change.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new one. **SPEC-045** (the STAGE-011 capstone: provenance-ledger entry,
   macOS/Linux determinism assertion, container→decoder hardening) is the last spec and
   already framed; it should also fold the external review's §2.1 later-DHT-replaces,
   §2.3 restart-MCU reset, and §2.4 `Pt<P` / `Tc/Th` header validation — none of which a
   held file exercises, but the APPn precedent shows header hardening pays off on real
   files. Then PROJ-003 closes.

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — design
   *(The K3III APP0 gap was caught by the design §12 probe, before any build code; the
   build itself was clean and APPROVED first pass with no punch list.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not.
   — Both held DNG `Compression 7` files — the Leica M Monochrom Typ 246 (big-endian,
   P=12) and the Pentax K-3 III Mono (little-endian, P=14) — now decode through the
   public `plane::unpack_into` to a `u16` plane that is **bit-exact** against
   `dnglab analyze --raw-checksum` (`c72e7d9579e2adaef9c6247841b8ac1d` and
   `98bae1339c25eb6e04fbac6b5f643189`) and develop end-to-end through the existing
   pipeline, where before SPEC-044 the Pentax file did not decode at all (its APP0
   header segment was rejected) — so the compressed monochrome files are now fully
   readable, not just the uncompressed planes.
