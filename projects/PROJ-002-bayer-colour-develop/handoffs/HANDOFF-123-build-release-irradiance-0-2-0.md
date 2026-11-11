---
# Maps to ContextCore handoff.* semantic conventions.
# BUILD (Phase 1) of SPEC-042, the 0.2.0 release. Rationale + handback contract:
# docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-123
  cycle: build
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: implementer
  created_at: 2026-11-10
  status: completed

task:
  spec_id: SPEC-042

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: completed
  tokens_total: 13600000           # REAL combined count — what cost-audit reads (transcript sum, rounded up to cover the post-fill PR turns; see Cost self-report)
  estimated_usd: 9.00
  duration_minutes: 25
  branch: claude/spec-042-build-b5e5da
  pr: https://github.com/jysf/irradiance/pull/186
  completed_at: 2026-11-10
  notes: "Phase 1 prep-complete, cut-deferred. Cfa/WarpRect/Opcode sealed; self dev-dep makes bare cargo test work; 0.2.0 + CHANGELOG + release notes + pre-flight; tier-B 9/9 sha256 + 8/8 decodable bit-exact vs dnglab; just test 519/0/1, lint, deny, deny-fuzz, package-targets-check, publish dry-run all green. No tag, no publish."
  synced_at: null
---

# HANDOFF-123: Build the 0.2.0 release prep (SPEC-042, group F, Phase 1)

## Delegation Summary

Phase 1 of the 0.2.0 release: the **reversible prep PR** — seal the last public
parser-output types, bump the version, promote the CHANGELOG, write the release notes,
and attach the real-file corpus evidence. **You do NOT tag or publish** — that is Phase
2, human-gated. The whole decision is **DEC-052**; read it and SPEC-042's `## Build work`
first. Do not cut a partial release and do not claim colour accuracy.

## What to read first

- **Decision:** `./decisions/DEC-052-seal-the-remaining-public-parser-output-types-and-cut-0-2-0-as-semver.md` — the seal, semver/0.2.0, the deferred follow-ups (do NOT implement the two deferred opcode items here — they are SPEC-056), the no-colour-accuracy call.
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-042-release-irradiance-0-2-0.md` — `## Release Scope`, `## Build work`, the `## Release Pre-Flight` checklist (fill the instance commands + `[now]`/`[cut]`/`N/A` timing).
- **Decisions:** `DEC-028` (publish checks, metadata), `DEC-006` (CHANGELOG → version), `DEC-003` (tier-B never committed), `DEC-045`/`DEC-049` (the seal began with `Sensor`).
- **Code:** `src/ifd.rs` (`Cfa` `:616`), `src/opcode.rs` (`Opcode` `:105`, `WarpRect` `:155`), `Cargo.toml` (the `test-fixtures` feature + `[dev-dependencies]` + `[package]` version/metadata), `.repo-context.yaml` (`spec.version.scheme`), `tests/` (the `Cfa`/`WarpRect`/`Opcode` literal fixtures), `CHANGELOG.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md`; `docs/versioning.md`.

## Expected Deliverables (Phase 1)

- `#[non_exhaustive]` on `Cfa`, `WarpRect`, `Opcode`; in-crate matches still compile (no crate-wide wildcard added); the `tests/` fixtures for all three migrated onto the dev-only `test-fixtures` feature (extend it with constructors).
- A self dev-dependency (`irradiance = { path = ".", features = ["test-fixtures", "irr"] }`) so a bare `cargo test` compiles the integration tests.
- `Cargo.toml` `version = "0.2.0"` + publish-ready metadata (DEC-028); `.repo-context.yaml` `spec.version.scheme: semver`.
- `CHANGELOG.md`: `[Unreleased]` promoted to `## [0.2.0] - <date>` with a fresh empty `[Unreleased]`.
- Release notes (user-facing headline + **Known limitations** naming the two deferred opcode follow-ups and the absence of a colour-accuracy claim).
- The `## Release Pre-Flight` checklist filled with instance commands and honest `[now]`/`[cut]`/`N/A` timing; `cargo publish --dry-run` clean (`[now]`).
- **Tier-B corpus evidence run on the candidate SHA** recorded in the handback (`IRRADIANCE_CORPUS_DIR`, every held file bit-exact); `just package-targets-check`, `just deny`, `just deny-fuzz`, `just test` (`--all-features`), `just lint` all green.
- A PR against `main` from `claude/spec-042-build-<suffix>` referencing HANDOFF-123, SPEC-042, STAGE-010, PROJ-002, DEC-052 / DEC-028. The PR description says **"Phase 1 — prep-complete, cut-deferred"** and lists the Phase-2 steps for the human cutter.

### If `cycle: verify` — the verify contract

Review cold. Confirm the three types are sealed and in-crate matches compile; the fixtures
moved onto `test-fixtures` and a bare `cargo test` builds; `version = 0.2.0` + semver
scheme; CHANGELOG promoted; release notes name the known limitations and make no
colour-accuracy claim; `cargo publish --dry-run` is clean; the corpus evidence is attached
and bit-exact; no tag/publish happened (Phase 2 is deferred). `just decisions-audit
--changed`. Output exactly one of ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- **Phase 2** — the `v0.2.0` tag and `cargo publish` (human-gated). Do not create a tag or publish.
- The two deferred group-C opcode follow-ups (SPEC-056, 0.2.1).
- Any colour-accuracy claim (SPEC-035 is hardware-gated); any new capability; STAGE-008 work.

## Return Criteria

Fill the `handback:` front-matter (a real `tokens_total`) and the `## Handback` section
(with the corpus-evidence result and the filled pre-flight timing); set
`handoff.status`/`handback.status` → `completed`; open the Phase-1 PR. Report token usage
even if blocked.

## Handback

### Execution notes
- **Branch / PR:** `claude/spec-042-build-b5e5da` → https://github.com/jysf/irradiance/pull/186  **Completed at:** 2026-11-10  **All Phase-1 deliverables met?** yes
- **Candidate SHA:** `6843083` (code + CHANGELOG + README + spec notes/pre-flight); a follow-up docs-only commit fills this handback and the pre-flight SHA line (neither file is in the published package).
- **What landed:**
  - `#[non_exhaustive]` on `Cfa` (`src/ifd.rs`), `WarpRect` and the `Opcode` enum (`src/opcode.rs`); the old "`#[non_exhaustive]`-free by choice" note on `Opcode` rewritten to cite DEC-052. In-crate matches (`src/opcode.rs` `find_map` matches, `src/warp.rs`) compile unchanged — no wildcard added anywhere.
  - Dev-only, doc-hidden `Cfa::test_fixture()` (2×2 RGGB, plane colour `[0,1,2]`, layout 1) and `WarpRect::test_fixture()` (identity: `kr=[1,0,0,0]`, centred, 1 plane) behind the existing `test-fixtures` feature, the `Sensor::test_fixture` shape. Migrated every `Cfa {..}` / `WarpRect {..}` literal in `tests/` (color_oracle, colour_metadata_oracle, demosaic_oracle, threaded, warp). `tests/ifd_reader.rs`'s `Some(Cfa { repeat_dim, pattern, .. })` is a *pattern* with `..`, legal on a sealed struct — unchanged.
  - Self dev-dependency `irradiance = { path = ".", features = ["test-fixtures", "irr"] }`. Bare `cargo test --no-run` builds all 24 test targets with no warnings; a bare `cargo test` of demosaic_oracle/warp/threaded on a cleaned target ran 54/54 **including** `irr_demosaic_emits_a_valid_p6_ppm` (the self dev-dep turns `irr` on, so it is no longer ignored). Comment above that test updated.
  - `version = "0.2.0"`; license / description / keywords / categories / repository already publish-ready (DEC-028), unchanged. `.repo-context.yaml` `spec.version.scheme` was **already** `semver` — no edit needed.
  - CHANGELOG: `[Unreleased]` → `## [0.2.0] - 2026-11-10` with a summary paragraph, a fresh empty `[Unreleased]`, a `⚠` Changed entry for the seal, a **Known limitations** section, and a missing **Added** entry for SPEC-051's opt-in threading (it shipped in #157 without a CHANGELOG line).
  - README: `irradiance = "0.2"`, status "0.2.x".
  - SPEC-042: `## Release Notes` (seeded from `just release-notes PROJ-002` + `PROJ-003`, edited), `## Release Pre-Flight` filled, Phase-2 runbook under `## Notes for the Implementer`. Spec `cycle` left at `build` (verify/ship advance it).
- **Corpus evidence (candidate tree, `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`):** `corpus-status` 9/9 present, no tier-B skip; `dnglab` + `exiftool` on PATH. **Every held file bit-exact:** all 9 match their pinned SHA-256 (`corpus_files_match_their_pinned_sha256`); all 8 decodable files — Leica Q2 Monochrom ×3, Leica M Monochrom, Canon 5D III (Bayer), Nikon D300S (Bayer, 12-strip), Leica M Typ 246 + Pentax K-3 III DNG (Compression 7) — match `dnglab --raw-checksum` (`plane_md5_matches_the_pinned_raw_checksum`, `corpus_comp7_files_still_bit_exact`, `compression7_plane_is_byte_identical_across_two_decodes`); the Pentax PEF is refused by name as designed (`compressed_files_are_skipped_by_name`). Metadata oracles vs exiftool/dnglab, `plane::validate` on the corpus, the Q2M develop/perceptual oracles all green. Never committed (DEC-003).
- **Gates on the candidate:** `just test` (corpus on) 24 runs, **519 passed / 0 failed / 1 ignored** (the long-standing SB-1 `q2m_frames_have_at_least_one_replaced_pixel`); `just lint`, `just lint-ci`, `just lint-no-allow`, `just lint-fuzz` green; `just deny` + `just deny-fuzz` "licenses ok"; `just package-targets-check` ✓ (packaged `irradiance-0.2.0` builds no executable); `cargo publish --dry-run` on the clean commit: packaged 19 files, verify-compile ok, upload aborted (dry run) — only the expected "ignoring test … not included" warnings from the `include` allow-list; the packaged manifest has **no** self dev-dependency (path-only, stripped) and `.cargo_vcs_info.json` sha1 = `6843083…`. `just validate` ✓. `git tag -l 'v0.2*'` empty.
- **Pre-flight timing:** `[now]` — 1 (version 0.2.0, no stray tag, vcs sha1 provenance in the dry-run), 3-proxy (dry-run + package-targets-check), 5 on the candidate (tests + corpus). `[cut]` — 1 (create/push `v0.2.0`, `--points-at`, published `.cargo_vcs_info.json` sha1 = tag commit), 3 (clean-host `cargo add irradiance@0.2.0`, docs.rs green), 5 (README example on a clean host), 6 (`cargo yank` path, exercised on 0.1.0). `N/A` — 2 (no binary to sign), 4 (no data store).
- **For verify:** n/a (build cycle).

### Cost self-report
- **Tokens (total):** 13,600,000 (transcript sum at handback fill: 12,978,292 — in 153 / cache-write 166,275 / cache-read 12,767,105 / out 44,759 — rounded up to cover the PR/push turns after the fill; the exact final figure is in the session's closing report)  **Estimated USD:** ~9.00  **Duration (min):** ~25  **Source:** transcript (`5f7124cf-d5bc-469e-9f2a-93ced8bd9bbd.jsonl`, claude-opus-5-5)

### Drift and new artifacts
- **New decisions:** none.
- **Deviations:** (1) No `Opcode` constructor added — `#[non_exhaustive]` on an enum seals the variant *set*, not variant construction, so every `Opcode::… { .. }` literal in `tests/` still compiles and there was nothing to migrate; an unused constructor would be dead API. (2) `.repo-context.yaml` untouched — `scheme: semver` was already set. (3) Two CHANGELOG/notes additions beyond the brief, both accuracy fixes: the missing SPEC-051 threading entry, and a third known limitation (the colour develop does not run the `OpcodeList` stages; required ones are refused) that the README already states. (4) README bumped to `"0.2"` so the published README matches the published version.
- **Follow-ups:** `just id-audit` reports three pre-existing id collisions (DEC-028, DEC-041, …) between *other* worktree branches (e.g. `claude/irradiance-roadmap-reconcile-dfbd8a`, `dng-writer-88c7b8`) and main — not introduced here, worth a cleanup pass. SPEC-056 (the two deferred opcode follow-ups) stays as planned for 0.2.1.

### Reflection
1. What was unclear? — Whether `Opcode` needed a constructor: DEC-052 lists it, but enum-level `#[non_exhaustive]` leaves variants constructible, so the only external breakage is exhaustive matching (none in tests/fuzz/bin).
2. A constraint/decision that should have been listed? — That `[Unreleased]` should be audited against the shipped specs at the cut: SPEC-051 had shipped without a CHANGELOG line, and `just release-notes` is what surfaced it.
3. What would you do differently? — Run the slow `just test` first thing (it is ~15 min with the corpus on) and do docs work while it runs — which is what happened, and it paid off.
