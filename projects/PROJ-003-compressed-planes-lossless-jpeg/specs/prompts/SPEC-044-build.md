# SPEC-044 BUILD — prompt for the implementer (cycle: build)

You're the implementer for a spec-driven workflow. Cycle: **build**. SPEC-044 —
**prove both held Compression-7 DNGs decode to a `u16` plane whose MD5 equals
`dnglab analyze --raw-checksum`, exactly**, and land the one decoder change the real
Pentax K-3 III needs to decode at all: `ljpeg::parse_headers` must **skip APPn / COM
header segments by length** (DEC-039). The de-interleave mapping (SPEC-046) is already
confirmed correct on both real files by the design §12 probe — you lock it with the
oracle, you do not re-derive it.

Read files in order, before writing code:

1. `/AGENTS.md`.
2. `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-095-build-bit-exact-corpus-oracle-both-held-compression-7.md`
   — your handoff; follow its "Context the Receiving Agent Needs" exactly. It carries the
   §12 probe numbers (both pinned MD5s, both marker maps) you transcribe rather than
   rediscover.
3. `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md` + the stage
   `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
4. `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-044-bit-exact-corpus-oracle-both-held-compression-7.md`
   — the spec; `## Inputs` has the probe + exact code anchors, `## Outputs` /
   `## Acceptance Criteria` / `## Failing Tests` / `## Notes` are the contract.
5. `DEC-039` (the APPn/COM skip + mapping-confirmed — read first), `DEC-037`
   (two-component scope), `DEC-030` (exact oracle), `DEC-003` (corpus never committed),
   `DEC-035` (clean-room), `DEC-016` (caller-owned buffer), `DEC-018` (u16 modular).
6. `/guidance/constraints.yaml` — `oracle-must-be-shown-red`,
   `no-panics-on-untrusted-input`, `no-new-top-level-deps-without-decision`.
7. `./guidance/toolchain-brief.md`; the shipped `src/ljpeg.rs` — `parse_headers` (the
   marker loop + the `other => LjpegUnexpectedMarker` arm you insert before), `parse_dht`
   (the length-read idiom to mirror), `read_u16_be` / `read_u8`; `tests/plane_oracle.rs`
   (the `DECODABLE` / `SKIPPED_COMPRESSED` lists, `plane_md5`, `decode_and_hash`,
   `plane_md5_matches_the_pinned_raw_checksum`, `compressed_files_are_skipped_by_name`,
   and the staged-copy red-proof machinery); `tests/develop_oracle.rs` (its corpus
   develop helper).

Before coding, mark the build cycle `[~]` in the SPEC-044 timeline. If the spec is
ambiguous or needs architect judgment, change it to `[?]` with a one-line reason and
**stop** — ask; don't guess.

Implement (per `## Outputs`):

- `src/ljpeg.rs`: add ONE arm to `parse_headers`' marker loop — APPn (`0xFFE0..=0xFFEF`)
  and COM (`0xFFFE`) are skipped by their 2-byte declared length (read length,
  `checked_sub(2)`, bounds-check against `data.len()`, advance `pos`; `Error::Truncated`
  on overflow/short), placed **before** the `other => LjpegUnexpectedMarker` catch-all. No
  new `Error` variant, no signature change, no new dependency. `ljpeg` stays `pub(crate)`.
- `tests/plane_oracle.rs`: introduce `DECODABLE_COMPRESSED: [&str; 2]` (the two held
  Compression-7 files), hash it in `plane_md5_matches_the_pinned_raw_checksum` alongside
  `DECODABLE`, and extend `compressed_files_are_skipped_by_name` so the three-list partition
  (`DECODABLE` + `DECODABLE_COMPRESSED` + `SKIPPED_COMPRESSED`) covers the manifest and the
  Compression-7 entries assert `require_uncompressed().is_err()` **and** that `unpack_into`
  succeeds. Do NOT drop them into `DECODABLE`.
- A tier-A APPn/COM fixture in `src/ljpeg.rs` (`mod tests`), using the SPEC-046
  two-component encoder: an APP0 / a COM between DHT and SOS decodes bit-exact to the
  segment-free plane; a truncated APPn length is a typed `Error`, no panic; the `Nf=1` path
  is unchanged.
- A real-file red-proof: a staged-copy subprocess that injects an `ljpeg.rs` two-component
  fault (de-interleave write-index, or a per-component predictor neighbour) and asserts a
  held Compression-7 oracle goes RED; negative control green. Mirror
  `inject_chunk_extraction_fault` (the existing one targets `src/plane.rs` and will NOT do).
- A develop smoke: a held Compression-7 file developed through `develop_into` — output
  `== output_dimensions(sensor)`, deterministic across two runs, no panic.
- Make every SPEC-044 `## Failing Test` pass.

Expected results to hit (already measured — a mismatch means a real regression, stop):
Typ 246 `c72e7d9579e2adaef9c6247841b8ac1d`, K3III `98bae1339c25eb6e04fbac6b5f643189`.

When done:

1. Fill `HANDOFF-095`'s `handback:` block + `## Handback` with a **real** `tokens_total`
   (run `/cost`; if your interface has no meter — the Claude Code desktop Code tab does not
   — report the `ccd_session_mgmt get_usage` occupancy as an explicit **FLOOR** with a note,
   as HANDOFF-089..094 did; do not invent). Reflection required.
2. Set `handoff.status` → completed, `handback.status` → completed.
3. Run: `just advance-cycle SPEC-044 verify`
4. Open a PR from `claude/spec-044-build-<suffix>` against `main`, referencing `PROJ-003`,
   `STAGE-011`, `SPEC-044`, `HANDOFF-095`, `DEC-039`, `DEC-037`.
5. Mark build `[x]` in the timeline with the PR number, cost, and date.

Running the gates (the tier-B lesson): the new oracle/red-proof/develop tests are
corpus-gated. Set `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
and run the **targeted** suites — `cargo test --lib ljpeg::`,
`cargo test --test plane_oracle`, `cargo test --test develop_oracle` — rather than a blind
full `just test` (its unrelated tier-B oracles buffer for minutes). Confirm
`just fmt-check` · `just lint-ci` · `just validate` green locally; lean on the PR's CI for
the broad tier-A / lint sweep (CI cannot see tier-B — no corpus — so the real-file oracle
runs only locally, by design).
